use candid::{candid_method, Principal};
use ic_canisters_http_types::{
    HttpRequest as AssetHttpRequest, HttpResponse as AssetHttpResponse, HttpResponseBuilder,
};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    query, update,
};
use ic_solana::{
    metrics::{encode_metrics, read_metrics, Metrics},
    request::RpcRequest,
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{
        AbciInfo, BlockComplete, BlockResults, Blockchain, CandidValue, CommitResult, ConsensusParamsResult,
        ConsensusState, DumpConsensusState, HeaderResult, NetInfo, NumUnconfirmedTransactionsResult, Status, Tx,
    },
};
use ic_solana_rpc::{
    auth::{do_authorize, do_deauthorize, require_manage_or_controller, require_register_provider, Auth},
    constants::NODES_IN_SUBNET,
    http::{get_http_request_cost, rpc_client, serve_logs, serve_metrics},
    providers::{do_register_provider, do_unregister_provider, do_update_provider},
    state::{read_state, replace_state, InitArgs},
    types::{RegisterProviderArgs, UpdateProviderArgs},
    utils::{parse_pubkey, parse_pubkeys, parse_signature, parse_signatures},
};

/// Returns the current health of the node.
/// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of
/// the latest cluster-confirmed slot.
#[update(name = "sol_getHealth")]
#[candid_method(rename = "sol_getHealth")]
pub async fn sol_get_health(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<bool> {
    let client = rpc_client(source, config);
    Ok(client.get_health().await?)
}

#[update(name = "sol_getStatus")]
#[candid_method(rename = "sol_getStatus")]
pub async fn sol_get_status(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<Status> {
    let client = rpc_client(source, config);
    Ok(client.get_status().await?)
}

#[update(name = "sol_getAbciInfo")]
#[candid_method(rename = "sol_getAbciInfo")]
pub async fn sol_get_abci_info(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<AbciInfo> {
    let client = rpc_client(source, config);
    Ok(client.get_abci_info().await?)
}

#[update(name = "sol_getConsensusState")]
#[candid_method(rename = "sol_getConsensusState")]
pub async fn sol_get_consensus_state(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<ConsensusState> {
    let client = rpc_client(source, config);
    Ok(client.get_consensus_state().await?)
}

#[update(name = "sol_getDumpConsensusState")]
#[candid_method(rename = "sol_getDumpConsensusState")]
pub async fn sol_get_dump_consensus_state(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<DumpConsensusState> {
    let client = rpc_client(source, config);
    Ok(client.get_dump_consensus_state().await?)
}

#[update(name = "sol_getNetInfo")]
#[candid_method(rename = "sol_getNetInfo")]
pub async fn sol_get_net_info(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<NetInfo> {
    let client = rpc_client(source, config);
    Ok(client.get_net_info().await?)
}

#[update(name = "sol_getBlock")]
#[candid_method(rename = "sol_getBlock")]
pub async fn sol_get_block(source: RpcServices, config: Option<RpcConfig>, height: String) -> RpcResult<BlockComplete> {
    let client = rpc_client(source, config);
    Ok(client.get_block(height).await?)
}

#[update(name = "sol_getBlockByHash")]
#[candid_method(rename = "sol_getBlockByHash")]
pub async fn sol_get_block_by_hash(
    source: RpcServices,
    config: Option<RpcConfig>,
    hash: String,
) -> RpcResult<BlockComplete> {
    let client = rpc_client(source, config);
    Ok(client.get_block_by_hash(hash).await?)
}

#[update(name = "sol_getBlockResults")]
#[candid_method(rename = "sol_getBlockResults")]
pub async fn sol_get_block_results(
    source: RpcServices,
    config: Option<RpcConfig>,
    height: String,
) -> RpcResult<BlockResults> {
    let client = rpc_client(source, config);
    Ok(client.get_block_results(height).await?)
}

#[update(name = "sol_getBlockchain")]
#[candid_method(rename = "sol_getBlockchain")]
pub async fn sol_get_blockchain(
    source: RpcServices,
    config: Option<RpcConfig>,
    min_height: String,
    max_height: String,
) -> RpcResult<Blockchain> {
    let client = rpc_client(source, config);
    Ok(client.get_blockchain(min_height, max_height).await?)
}

#[update(name = "sol_getCommit")]
#[candid_method(rename = "sol_getCommit")]
pub async fn sol_get_commit(source: RpcServices, config: Option<RpcConfig>, height: String) -> RpcResult<CommitResult> {
    let client = rpc_client(source, config);
    Ok(client.get_commit(height).await?)
}

#[update(name = "sol_getConsensusParams")]
#[candid_method(rename = "sol_getConsensusParams")]
pub async fn sol_get_consensus_params(
    source: RpcServices,
    config: Option<RpcConfig>,
    height: String,
) -> RpcResult<ConsensusParamsResult> {
    let client = rpc_client(source, config);
    Ok(client.get_consensus_params(height).await?)
}

#[update(name = "sol_getHeader")]
#[candid_method(rename = "sol_getHeader")]
pub async fn sol_get_header(source: RpcServices, config: Option<RpcConfig>, height: String) -> RpcResult<HeaderResult> {
    let client = rpc_client(source, config);
    Ok(client.get_header(height).await?)
}

#[update(name = "sol_getHeaderByHash")]
#[candid_method(rename = "sol_getHeaderByHash")]
pub async fn sol_get_header_by_hash(
    source: RpcServices,
    config: Option<RpcConfig>,
    hash: String,
) -> RpcResult<HeaderResult> {
    let client = rpc_client(source, config);
    Ok(client.get_header_by_hash(hash).await?)
}

#[update(name = "sol_getNumUnconfirmedTxs")]
#[candid_method(rename = "sol_getNumUnconfirmedTxs")]
pub async fn sol_get_num_unconfirmed_txs(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<NumUnconfirmedTransactionsResult> {
    let client = rpc_client(source, config);
    Ok(client.get_num_unconfirmed_txs().await?)
}

#[update(name = "sol_getTx")]
#[candid_method(rename = "sol_getTx")]
pub async fn sol_get_tx(source: RpcServices, config: Option<RpcConfig>, hash: String, proof: bool) -> RpcResult<Tx> {
    let client = rpc_client(source, config);
    Ok(client.get_tx(hash, proof).await?)
}
/// Sends a JSON-RPC request to a specified Solana node provider,
/// supporting custom RPC methods.
#[update]
#[candid_method]
pub async fn request(
    source: RpcServices,
    method: String,
    params: CandidValue,
    max_response_bytes: Option<u64>,
) -> RpcResult<String> {
    let client = rpc_client(source, None);
    let res = client
        .call::<_, serde_json::Value>(RpcRequest::Custom { method }, params, max_response_bytes)
        .await?;
    Ok(serde_json::to_string(&res)?)
}

/// Calculates the cost of an RPC request.
#[query(name = "requestCost")]
#[candid_method(query, rename = "requestCost")]
pub fn request_cost(json_rpc_payload: String, max_response_bytes: u64) -> u128 {
    if read_state(|s| s.is_demo_active) {
        0
    } else {
        get_http_request_cost(json_rpc_payload.len() as u64, max_response_bytes)
    }
}

#[query(name = "getNodesInSubnet")]
#[candid_method(query, rename = "getNodesInSubnet")]
fn get_nodes_in_subnet() -> u32 {
    NODES_IN_SUBNET
}

#[query(name = "getProviders")]
#[candid_method(query, rename = "getProviders")]
fn get_providers() -> Vec<String> {
    read_state(|s| s.rpc_providers.iter().map(|(k, _)| k.0).collect())
}

#[update(name = "registerProvider", guard = "require_register_provider")]
#[candid_method(rename = "registerProvider")]
fn register_provider(args: RegisterProviderArgs) {
    do_register_provider(ic_cdk::caller(), args)
}

#[update(name = "unregisterProvider")]
#[candid_method(rename = "unregisterProvider")]
fn unregister_provider(provider_id: String) -> bool {
    do_unregister_provider(ic_cdk::caller(), &provider_id)
}

#[update(name = "updateProvider")]
#[candid_method(rename = "updateProvider")]
fn update_provider(args: UpdateProviderArgs) {
    do_update_provider(ic_cdk::caller(), args)
}

#[query(name = "getAuthorized")]
#[candid_method(query, rename = "getAuthorized")]
fn get_authorized(auth: Auth) -> Vec<Principal> {
    read_state(|s| {
        let mut result = Vec::new();
        for (k, v) in s.auth.iter() {
            if v.is_authorized(auth) {
                result.push(k.0);
            }
        }
        result
    })
}

#[update(guard = "require_manage_or_controller")]
#[candid_method]
fn authorize(principal: Principal, auth: Auth) -> bool {
    do_authorize(principal, auth)
}

#[update(guard = "require_manage_or_controller")]
#[candid_method]
fn deauthorize(principal: Principal, auth: Auth) -> bool {
    do_deauthorize(principal, auth)
}

#[query]
fn http_request(request: AssetHttpRequest) -> AssetHttpResponse {
    match request.path() {
        "/metrics" => serve_metrics(encode_metrics),
        "/logs" => serve_logs(request),
        _ => HttpResponseBuilder::not_found().build(),
    }
}

#[query(name = "getMetrics")]
#[candid_method(query, rename = "getMetrics")]
fn get_metrics() -> Metrics {
    read_metrics(|m| m.to_owned())
}

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
#[query(hidden = true)]
fn __transform_json_rpc(mut args: TransformArgs) -> HttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach
    // consensus! Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.
    args.response.headers.clear();
    args.response
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    post_upgrade(args)
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: InitArgs) {
    replace_state(args.into());
}

fn main() {}

// Order dependent: do not move above any exposed canister method!
ic_cdk::export_candid!();
