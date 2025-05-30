use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the consensus state response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ConsensusStateResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The consensus state result
    pub result: ConsensusState,
}

/// Represents the consensus state information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ConsensusState {
    /// The round state information
    pub round_state: RoundState,
}

/// Represents the round state information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct RoundState {
    /// Height/round/step information in format "height/round/step"
    #[serde(rename = "height/round/step")]
    pub height_round_step: String,
    /// Start time of the round
    pub start_time: String,
    /// Hash of the proposal block
    pub proposal_block_hash: String,
    /// Hash of the locked block
    pub locked_block_hash: String,
    /// Hash of the valid block
    pub valid_block_hash: String,
    /// Height vote set information
    pub height_vote_set: Vec<HeightVoteSet>,
    /// Proposer information
    pub proposer: Proposer,
}

/// Represents the height vote set information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct HeightVoteSet {
    /// The round number
    pub round: i32,
    /// Array of prevotes
    pub prevotes: Vec<String>,
    /// Bit array representation of prevotes
    pub prevotes_bit_array: String,
    /// Array of precommits
    pub precommits: Vec<String>,
    /// Bit array representation of precommits
    pub precommits_bit_array: String,
}

/// Represents the proposer information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Proposer {
    /// The proposer's address
    pub address: String,
    /// The proposer's index
    pub index: i32,
}
