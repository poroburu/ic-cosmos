use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos_common::PubKey;

/// Represents the response from the /validators endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorsResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The validators result
    pub result: ValidatorsResult,
}

/// Represents the validators result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorsResult {
    /// The block height
    pub block_height: String,
    /// The validators
    pub validators: Vec<ValidatorWithPriority>,
    /// The total count
    pub count: String,
    /// The total number of validators
    pub total: String,
}

/// Represents a validator with priority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorWithPriority {
    /// The validator address
    pub address: String,
    /// The validator public key
    pub pub_key: PubKey,
    /// The voting power
    pub voting_power: String,
    /// The proposer priority
    pub proposer_priority: String,
}
