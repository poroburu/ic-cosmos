use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the complete ABCI info response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciInfoResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The ABCI info result
    pub result: AbciInfo,
}

/// The ABCI info result containing node information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciInfo {
    /// The ABCI response containing node information
    pub response: AbciResponse,
}

/// The ABCI response containing detailed node information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciResponse {
    /// The application data, typically a JSON string containing application-specific information
    pub data: String,
    /// The version of the ABCI implementation
    pub version: String,
    /// The version of the application (optional)
    #[serde(rename = "app_version")]
    pub app_version: Option<String>,
    /// The height of the last committed block
    #[serde(rename = "last_block_height")]
    pub last_block_height: String,
    /// The application hash of the last committed block
    #[serde(rename = "last_block_app_hash")]
    pub last_block_app_hash: String,
}

/// Represents the complete ABCI query response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciQueryResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The error message if any
    pub error: String,
    /// The query result
    pub result: AbciQueryResult,
}

/// The ABCI query result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct AbciQueryResult {
    /// The query response
    pub response: QueryResponse,
}

/// The query response containing the result of an ABCI query
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct QueryResponse {
    /// The response log
    pub log: String,
    /// The block height at which the query was executed
    pub height: String,
    /// The proof of the query result
    pub proof: String,
    /// The query result value
    pub value: String,
    /// The query key
    pub key: String,
    /// The query index
    pub index: String,
    /// The response code
    pub code: String,
}
