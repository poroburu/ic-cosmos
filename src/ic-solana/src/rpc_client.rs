use std::{cell::RefCell, collections::BTreeSet};

use hex;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    add_metric_entry,
    constants::*,
    request::RpcRequest,
    rpc_client::multi_call::{MultiCallError, MultiCallResults},
    types::{
        ABCIQueryResult, AbciInfo, BlockComplete, BlockResults, Blockchain, BroadcastTxResult, CheckTxResult,
        CommitResult, ConsensusParamsResult, ConsensusState, DumpConsensusState, HeaderResult, NetInfo,
        NumUnconfirmedTransactionsResult, Status, Tx,
    },
};

mod compression;
mod multi_call;
mod types;

pub use types::*;

use crate::{
    logs::DEBUG,
    metrics::{MetricRpcHost, MetricRpcMethod},
    rpc_client::compression::decompress_if_needed,
};

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RpcClientConfig {
    pub response_consensus: Option<ConsensusStrategy>,
    pub response_size_estimate: Option<u64>,
    pub request_cost_calculator: Option<RequestCostCalculator>,
    pub host_validator: Option<HostValidator>,
    pub transform_context: Option<TransformContext>,
    pub use_compression: bool,
    pub is_demo_active: bool,
}

#[derive(Clone, Debug)]
pub struct RpcClient {
    pub providers: BTreeSet<RpcApi>,
    pub config: RpcClientConfig,
}

impl RpcClient {
    pub fn new<T: Into<Vec<RpcApi>>>(providers: T, config: Option<RpcClientConfig>) -> Self {
        Self {
            providers: providers.into().into_iter().collect(),
            config: config.unwrap_or_default(),
        }
    }

    fn response_size_estimate(&self, estimate: u64) -> u64 {
        self.config
            .response_size_estimate
            .unwrap_or(estimate + HEADER_SIZE_LIMIT)
    }

    fn consensus_strategy(&self) -> ConsensusStrategy {
        self.config.response_consensus.as_ref().copied().unwrap_or_default()
    }

    /// Generate the next request id.
    pub fn next_request_id(&self) -> u64 {
        NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            id
        })
    }

    /// Asynchronously sends an HTTP POST request to the specified URL with the given payload and
    /// maximum response bytes and returns the response as a string.
    /// This function calculates the required cycles for the HTTP request and logs the request
    /// details and response status. It uses a transformation named "cleanup_response" for the
    /// response body.
    ///
    /// # Arguments
    ///
    /// * `provider` - RPC API provider.
    /// * `payload` - JSON payload to be sent in the HTTP request.
    /// * `max_response_bytes` - The maximal size of the response in bytes. If None, 2MiB will be
    ///   the limit.
    ///
    /// # Returns
    ///
    /// * `RpcResult<Vec<u8>>` - The response body as a vector of bytes.
    async fn call_internal(
        &self,
        provider: &RpcApi,
        payload: &Value,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Vec<u8>> {
        let cluster = provider.cluster();
        let url = cluster.url();

        // Ensure "Content-Type: application/json" is present
        let mut headers = provider.headers.clone().unwrap_or_default();
        if !headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("Content-Type"))
        {
            headers.push(HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            });
        }

        if self.config.use_compression {
            headers.push(HttpHeader {
                name: "Accept-Encoding".to_string(),
                value: "gzip, deflate".to_string(),
            });
        }

        let body = serde_json::to_vec(payload).map_err(|e| RpcError::ParseError(e.to_string()))?;

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            max_response_bytes,
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            transform: self.config.transform_context.clone(),
        };

        // Calculate cycles if a calculator is provided
        let (cycles_cost, cycles_cost_with_collateral) = self
            .config
            .request_cost_calculator
            .as_ref()
            .map_or((0, 0), |calc| calc(&request));

        let parsed_url = url::Url::parse(url).map_err(|_| RpcError::ParseError(format!("Invalid URL: {}", url)))?;

        let host = parsed_url
            .host_str()
            .ok_or_else(|| RpcError::ParseError(format!("Error parsing hostname from URL: {}", url)))?;

        let rpc_host = MetricRpcHost(host.to_string());
        let rpc_method = MetricRpcMethod(Self::find_rpc_method_name(payload).to_string());

        if let Some(is_allowed) = self.config.host_validator {
            if !is_allowed(host) {
                add_metric_entry!(err_host_not_allowed, rpc_host.clone(), 1);
                return Err(RpcError::Text(format!("Disallowed RPC service host: {}", host)));
            }
        }

        // Handle cycle accounting if not in demo mode
        if !self.config.is_demo_active {
            let cycles_available = ic_cdk::api::call::msg_cycles_available128();
            if cycles_available < cycles_cost_with_collateral {
                return Err(RpcError::Text(format!(
                    "Insufficient cycles: available {}, required {} (with collateral).",
                    cycles_available, cycles_cost_with_collateral
                )));
            }
            ic_cdk::api::call::msg_cycles_accept128(cycles_cost);
            add_metric_entry!(cycles_charged, (rpc_method.clone(), rpc_host.clone()), cycles_cost);
        }

        log!(
            DEBUG,
            "Calling url: {url} with payload: {payload}. Cycles: {cycles_cost}"
        );

        add_metric_entry!(requests, (rpc_method.clone(), rpc_host.clone()), 1);

        match http_request(request, cycles_cost).await {
            Ok((response,)) => {
                let bytes = if self.config.use_compression {
                    decompress_if_needed(response.body)?
                } else {
                    response.body
                };
                let body = std::str::from_utf8(&bytes).map_err(|e| RpcError::ParseError(e.to_string()))?;

                log!(
                    DEBUG,
                    "Got response (with {} bytes): {} from url: {} with status: {}",
                    body.len(),
                    body,
                    url,
                    response.status
                );

                // JSON-RPC responses over HTTP should have a 2xx status code,
                // even if the contained JsonRpcResult is an error.
                // If the server is not available, it will sometimes (wrongly) return HTML that will
                // fail to parse as JSON.
                let http_status: u16 = response.status.0.try_into().expect("Invalid http status code");
                // TODO: investigate
                // if !is_successful_http_code(&status) {
                //     return Err(RpcError::JsonRpcError { status, body }.into());
                // }

                add_metric_entry!(responses, (rpc_method, rpc_host, http_status.into()), 1);

                Ok(bytes)
            }
            Err(error) => {
                add_metric_entry!(err_http_outcall, (rpc_method, rpc_host), 1);
                Err(error.into())
            }
        }
    }

    /// Calls multiple providers in parallel and returns the results.
    async fn parallel_call(&self, payload: &Value, max_response_bytes: Option<u64>) -> Vec<RpcResult<Vec<u8>>> {
        futures::future::join_all(self.providers.iter().map(|provider| {
            log!(DEBUG, "[parallel_call]: will call provider: {:?}", provider);
            async { self.call_internal(provider, payload, max_response_bytes).await }
        }))
        .await
    }

    /// Makes a single JSON-RPC call.
    pub async fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: RpcRequest,
        params: P,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<JsonRpcResponse<R>> {
        let payload = method.build_json(self.next_request_id(), params);
        let results = self
            .parallel_call(
                &payload,
                max_response_bytes.map(|estimate| self.response_size_estimate(estimate)),
            )
            .await;
        let bytes = Self::process_result(
            method,
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Makes multiple JSON-RPC calls in a single batch request.
    pub async fn batch_call<P: Serialize, R: DeserializeOwned>(
        &self,
        requests: &[(RpcRequest, P)],
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Vec<JsonRpcResponse<R>>> {
        let payload = RpcRequest::batch(
            requests
                .iter()
                .map(|(method, params)| (method.to_owned(), params, self.next_request_id()))
                .collect(),
        );

        let results = self
            .parallel_call(
                &payload,
                max_response_bytes.map(|estimate| self.response_size_estimate(estimate)),
            )
            .await;

        let bytes = Self::process_result(
            Self::find_rpc_method_name(&payload),
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Returns the current health of the node.
    /// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of the latest
    /// cluster-confirmed slot.
    ///
    /// Method relies on the `getHealth` RPC call to get the health status:
    ///   https://solana.com/docs/rpc/http/getHealth
    pub async fn get_health(&self) -> RpcResult<bool> {
        let response: crate::rpc_client::types::JsonRpcResponse<serde_json::Value> =
            self.call(RpcRequest::GetHealth, (), Some(128)).await?;
        match (response.result, response.error) {
            (Some(val), _) if val.is_object() && val.as_object().unwrap().is_empty() => Ok(true), // healthy
            (_, Some(_err)) => Ok(false), // unhealthy (or propagate error if you prefer)
            _ => Err(RpcError::ParseError("Unexpected health response".to_string())),
        }
    }
    pub async fn get_status(&self) -> RpcResult<Status> {
        let response: JsonRpcResponse<Status> = self.call(RpcRequest::GetStatus, (), Some(128)).await?;
        response.into_rpc_result()
    }

    /// Returns the ABCI info of the node.
    /// This includes the application name, version, and last block information.
    pub async fn get_abci_info(&self) -> RpcResult<AbciInfo> {
        let response: JsonRpcResponse<AbciInfo> = self.call(RpcRequest::GetAbciInfo, (), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_consensus_state(&self) -> RpcResult<ConsensusState> {
        let response: JsonRpcResponse<ConsensusState> = self
            .call(
                RpcRequest::GetConsensusState,
                (),
                Some(COSMOS_CONSENSUS_STATE_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_dump_consensus_state(&self) -> RpcResult<DumpConsensusState> {
        let response: JsonRpcResponse<DumpConsensusState> = self
            .call(
                RpcRequest::GetDumpConsensusState,
                (),
                Some(COSMOS_DUMP_CONSENSUS_STATE_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_net_info(&self) -> RpcResult<NetInfo> {
        let response: JsonRpcResponse<NetInfo> = self
            .call(RpcRequest::GetNetInfo, (), Some(COSMOS_NET_INFO_SIZE_ESTIMATE))
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_block(&self, height: String) -> RpcResult<BlockComplete> {
        let response: JsonRpcResponse<BlockComplete> = self
            .call(RpcRequest::GetBlock, (height,), Some(GET_BLOCK_SIZE_ESTIMATE))
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_block_by_hash(&self, hash: String) -> RpcResult<BlockComplete> {
        let base64_hash = convert_hex_to_base64(hash)?;

        let response: JsonRpcResponse<BlockComplete> = self
            .call(
                RpcRequest::GetBlockByHash,
                (base64_hash,),
                Some(GET_BLOCK_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_block_results(&self, height: String) -> RpcResult<BlockResults> {
        let response: JsonRpcResponse<BlockResults> = self
            .call(
                RpcRequest::GetBlockResults,
                (height,),
                Some(COSMOS_BLOCK_RESULTS_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_blockchain(&self, min_height: String, max_height: String) -> RpcResult<Blockchain> {
        let response: JsonRpcResponse<Blockchain> = self
            .call(
                RpcRequest::GetBlockchain,
                (min_height, max_height),
                Some(COSMOS_BLOCKCHAIN_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_commit(&self, height: String) -> RpcResult<CommitResult> {
        let response: JsonRpcResponse<CommitResult> = self
            .call(RpcRequest::GetCommit, (height,), Some(COSMOS_COMMIT_SIZE_ESTIMATE))
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_consensus_params(&self, height: String) -> RpcResult<ConsensusParamsResult> {
        let response: JsonRpcResponse<ConsensusParamsResult> =
            self.call(RpcRequest::GetConsensusParams, (height,), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_header(&self, height: String) -> RpcResult<HeaderResult> {
        let response: JsonRpcResponse<HeaderResult> = self.call(RpcRequest::GetHeader, (height,), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_header_by_hash(&self, hash: String) -> RpcResult<HeaderResult> {
        let hash = remove_0x_prefix(hash);

        let response: JsonRpcResponse<HeaderResult> =
            self.call(RpcRequest::GetHeaderByHash, (hash,), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_num_unconfirmed_txs(&self) -> RpcResult<NumUnconfirmedTransactionsResult> {
        let response: JsonRpcResponse<NumUnconfirmedTransactionsResult> =
            self.call(RpcRequest::GetNumUnconfirmedTxs, (), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_tx(&self, hash: String, proof: bool) -> RpcResult<Tx> {
        let base64_hash = convert_hex_to_base64(hash)?;
        let response: JsonRpcResponse<Tx> = self
            .call(RpcRequest::GetTx, (base64_hash, proof), Some(COSMOS_TX_SIZE_ESTIMATE))
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_abci_query(
        &self,
        path: String,
        data: String,
        height: String,
        prove: bool,
    ) -> RpcResult<ABCIQueryResult> {
        let response: JsonRpcResponse<ABCIQueryResult> = self
            .call(
                RpcRequest::GetAbciQuery,
                (path, data, height, prove),
                Some(COSMOS_ABCI_QUERY_SIZE_ESTIMATE),
            )
            .await?;
        response.into_rpc_result()
    }

    pub async fn get_check_tx(&self, tx: String) -> RpcResult<CheckTxResult> {
        let response: JsonRpcResponse<CheckTxResult> = self.call(RpcRequest::GetCheckTx, (tx,), Some(128)).await?;
        response.into_rpc_result()
    }

    pub async fn get_broadcast_tx_async(&self, tx: String) -> RpcResult<BroadcastTxResult> {
        let response: JsonRpcResponse<BroadcastTxResult> =
            self.call(RpcRequest::GetBroadcastTxAsync, (tx,), Some(128)).await?;
        response.into_rpc_result()
    }

    /// Processes the result of an RPC method call by handling consistent and inconsistent responses
    /// from multiple providers.
    fn process_result<T: Serialize>(method: impl ToString, result: Result<T, MultiCallError<T>>) -> RpcResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(MultiCallError::ConsistentError(err)) => Err(err),
            Err(MultiCallError::InconsistentResults(multi_call_results)) => {
                let results = multi_call_results
                    .into_vec()
                    .into_iter()
                    .map(|(provider, result)| {
                        let cluster = provider.cluster();
                        add_metric_entry!(
                            inconsistent_responses,
                            (
                                MetricRpcMethod(method.to_string()),
                                MetricRpcHost(cluster.host_str().unwrap_or_else(|| "(unknown)".to_string()))
                            ),
                            1
                        );
                        Ok((provider, serde_json::to_string(&result?)?))
                    })
                    .collect::<Result<Vec<(RpcApi, String)>, RpcError>>()?;

                Err(RpcError::InconsistentResponse(results))
            }
        }
    }

    /// Calculate the max response bytes for the provided block range.
    fn get_block_range_max_response_bytes(start_slot: u64, limit: u64) -> u64 {
        let end_slot = start_slot.saturating_add(limit);
        let max_slot_str_len = end_slot.to_string().len() as u64;
        let commas_size = if limit > 0 { limit - 1 } else { 0 };
        36 + (max_slot_str_len * limit) + commas_size
    }

    /// Extracts the JSON-RPC `method` name from the request payload.
    ///
    /// This function searches for the `method` field within the provided JSON-RPC
    /// request payload. It handles both single and batch requests:
    ///
    /// - **Single Request**: Retrieves the `method` directly from the payload.
    /// - **Batch Request**: Retrieves the `method` from the first request in the batch.
    ///
    /// If the `method` field is not found, in either case returns `"unknown"`.
    fn find_rpc_method_name(payload: &Value) -> &str {
        payload
            .pointer("/method")
            .or_else(|| payload.pointer("/0/method"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    }
}

fn remove_0x_prefix(hash: String) -> String {
    if hash.starts_with("0x") {
        hash[2..].to_string()
    } else {
        hash
    }
}

fn convert_hex_to_base64(hash: String) -> Result<String, RpcError> {
    let hash = remove_0x_prefix(hash);
    let bytes = hex::decode(hash).map_err(|e| RpcError::ParseError(format!("Invalid hex hash: {}", e)))?;
    let base64_hash = base64::encode(bytes);
    Ok(base64_hash)
}

// TODO:
// pub fn is_response_too_large(code: &RejectionCode, message: &str) -> bool {
//     code == &RejectionCode::SysFatal && (message.contains("size limit") ||
// message.contains("length limit")) }
//
// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct ResponseSizeEstimate(u64);
//
// impl ResponseSizeEstimate {
//     pub fn new(num_bytes: u64) -> Self {
//         assert!(num_bytes > 0);
//         assert!(num_bytes <= MAX_PAYLOAD_SIZE);
//         Self(num_bytes)
//     }
//
//     /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
//     /// This number should be lower than `MAX_PAYLOAD_SIZE`.
//     pub fn get(self) -> u64 {
//         self.0
//     }
//
//     /// Returns a higher estimate for the payload size.
//     pub fn adjust(self) -> Self {
//         Self(self.0.max(1024).saturating_mul(2).min(MAX_PAYLOAD_SIZE))
//     }
// }
//
// impl std::fmt::Display for ResponseSizeEstimate {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
