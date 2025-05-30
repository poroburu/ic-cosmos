use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos_common::Event;

/// Represents the response from the /tx endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct TxResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The transaction result
    pub result: Tx,
}

/// Represents the transaction result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Tx {
    /// The transaction hash
    pub hash: String,
    /// The block height
    pub height: String,
    /// The transaction index
    pub index: i32,
    /// The transaction result
    pub tx_result: TxResultData,
    /// The transaction data
    pub tx: String,
}

/// Represents the transaction result data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct TxResultData {
    /// The result code
    pub code: i32,
    /// The result data
    pub data: String,
    /// The result log
    pub log: String,
    /// The gas wanted
    pub gas_wanted: String,
    /// The gas used
    pub gas_used: String,
    /// The transaction tags
    pub tags: Option<Vec<Event>>,
}
