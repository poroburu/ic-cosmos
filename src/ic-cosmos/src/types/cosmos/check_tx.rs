use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos::common::Event;

/// Represents the response from the /check_tx endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CheckTxResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The error message if any
    pub error: String,
    /// The check transaction result
    pub result: CheckTxResult,
}

/// Represents the check transaction result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CheckTxResult {
    /// The result code
    pub code: i32,
    /// The result data
    pub data: String,
    /// The result log
    pub log: String,
    /// Additional information
    pub info: String,
    /// The gas wanted
    pub gas_wanted: String,
    /// The gas used
    pub gas_used: String,
    /// The events
    pub events: Option<Vec<CheckTxEvent>>,
    /// The codespace
    pub codespace: String,
}

/// Represents a check transaction event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct CheckTxEvent {
    /// The event type
    #[serde(rename = "type")]
    pub check_tx_type: String,
    /// The event attributes
    pub attributes: Vec<Event>,
}
