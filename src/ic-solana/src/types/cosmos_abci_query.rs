use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the response from the /abci_query endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ABCIQueryResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The error message if any
    pub error: String,
    /// The ABCI query result
    pub result: ABCIQueryResult,
}

/// Represents the ABCI query result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ABCIQueryResult {
    /// The ABCI response
    pub response: ABCIResponse,
}

/// Represents the ABCI response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ABCIResponse {
    /// The response code
    pub code: i32,
    /// The response log
    pub log: String,
    /// The response index
    pub index: String,
    /// The response key
    pub key: Option<String>,
    /// The response value
    pub value: String,
    /// The response proof
    pub proof: Option<String>,
    /// The response height
    pub height: String,
    /// The response codespace
    pub codespace: String,
}
