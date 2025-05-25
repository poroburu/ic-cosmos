use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos_common::{PubKey, ValidatorPriority};

/// Represents the dump consensus state response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct DumpConsensusResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The dump consensus state result
    pub result: DumpConsensusState,
}

/// Represents the complete consensus state information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct DumpConsensusState {
    /// The round state information
    pub round_state: DumpRoundState,
    /// The peer states
    pub peers: Vec<PeerState>,
}

/// Represents the detailed round state information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct DumpRoundState {
    /// The current height
    pub height: String,
    /// The current round
    pub round: i32,
    /// The current step
    pub step: i32,
    /// The start time of the round
    pub start_time: String,
    /// The commit time
    pub commit_time: String,
    /// The validators information
    pub validators: ValidatorSet,
    /// The proposal information
    pub proposal: Option<Proposal>,
    /// The proposal block information
    pub proposal_block: Option<Block>,
    /// The proposal block parts information
    pub proposal_block_parts: Option<BlockParts>,
    /// The locked round
    pub locked_round: i32,
    /// The locked block information
    pub locked_block: Option<Block>,
    /// The locked block parts information
    pub locked_block_parts: Option<BlockParts>,
    /// The valid round
    pub valid_round: i32,
    /// The valid block information
    pub valid_block: Option<Block>,
    /// The valid block parts information
    pub valid_block_parts: Option<BlockParts>,
    /// The votes information
    pub votes: Vec<VoteSet>,
    /// The commit round
    pub commit_round: i32,
    /// The last commit information
    pub last_commit: Option<LastCommit>,
    /// The last validators information
    pub last_validators: ValidatorSet,
    /// Whether timeout precommit was triggered
    pub triggered_timeout_precommit: bool,
}

/// Represents a proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Proposal {
    /// The proposal type
    #[serde(rename = "type")]
    pub proposal_type: bool,
    /// The proposal height
    pub height: String,
    /// The proposal round
    pub round: i32,
    /// The proposal pol round
    pub pol_round: i32,
    /// The proposal block ID
    pub block_id: BlockID,
    /// The proposal timestamp
    pub timestamp: String,
    /// The proposal signature
    pub signature: String,
}

/// Represents a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Block {
    /// The block header
    pub header: BlockHeader,
    /// The block data
    pub data: Vec<String>,
    /// The block evidence
    pub evidence: Vec<Evidence>,
    /// The last commit
    pub last_commit: Option<LastCommit>,
}

/// Represents block parts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockParts {
    /// The block parts header
    pub header: BlockPartsHeader,
    /// The block parts
    pub parts: Vec<String>,
}

/// Represents a block ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockID {
    /// The block hash
    pub hash: String,
    /// The block parts
    pub parts: BlockPartsHeader,
}

/// Represents a block header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeader {
    /// The version
    pub version: Version,
    /// The chain ID
    pub chain_id: String,
    /// The height
    pub height: String,
    /// The time
    pub time: String,
    /// The last block ID
    pub last_block_id: BlockID,
    /// The last commit hash
    pub last_commit_hash: String,
    /// The data hash
    pub data_hash: String,
    /// The validators hash
    pub validators_hash: String,
    /// The next validators hash
    pub next_validators_hash: String,
    /// The consensus hash
    pub consensus_hash: String,
    /// The app hash
    pub app_hash: String,
    /// The last results hash
    pub last_results_hash: String,
    /// The evidence hash
    pub evidence_hash: String,
    /// The proposer address
    pub proposer_address: String,
}

/// Represents a version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Version {
    /// The block version
    pub block: String,
    /// The app version
    pub app: String,
}

/// Represents evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Evidence {
    /// The evidence type
    #[serde(rename = "type")]
    pub evidence_type: String,
    /// The evidence height
    pub height: i32,
    /// The evidence time
    pub time: i32,
    /// The total voting power
    pub total_voting_power: i32,
    /// The validator
    pub validator: Validator,
}

/// Represents a validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Validator {
    /// The validator's public key
    pub pub_key: PubKey,
    /// The validator's voting power
    pub voting_power: String,
    /// The validator's address
    pub address: String,
}

/// Represents a set of validators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorSet {
    /// The list of validators
    pub validators: Vec<ValidatorPriority>,
    /// The proposer
    pub proposer: ValidatorPriority,
}

/// Represents a set of votes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct VoteSet {
    /// The round number
    pub round: i32,
    /// The prevotes
    pub prevotes: Option<Vec<String>>,
    /// The prevotes bit array
    pub prevotes_bit_array: String,
    /// The precommits
    pub precommits: Option<Vec<String>>,
    /// The precommits bit array
    pub precommits_bit_array: String,
}

/// Represents the last commit information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct LastCommit {
    /// The votes
    pub votes: Vec<String>,
    /// The votes bit array
    pub votes_bit_array: String,
    /// The peer majority information
    pub peer_maj_23s: PeerMajority,
}

/// Represents peer majority information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PeerMajority {
    // This is an empty object in the OpenAPI spec
}

/// Represents a peer's state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PeerState {
    /// The node address
    pub node_address: String,
    /// The peer state information
    pub peer_state: PeerStateInfo,
}

/// Represents detailed peer state information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PeerStateInfo {
    /// The round state information
    pub round_state: PeerRoundState,
    /// The peer statistics
    pub stats: PeerStats,
}

/// Represents a peer's round state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PeerRoundState {
    /// The current height
    pub height: String,
    /// The current round
    pub round: i32,
    /// The current step
    pub step: i32,
    /// The start time
    pub start_time: String,
    /// Whether there is a proposal
    pub proposal: bool,
    /// The proposal block parts header
    pub proposal_block_parts_header: Option<BlockPartsHeader>,
    /// The proposal block parts
    pub proposal_block_parts: Option<String>,
    /// The proposal pol round
    pub proposal_pol_round: Option<i32>,
    /// The proposal pol
    pub proposal_pol: Option<String>,
    /// The prevotes
    pub prevotes: Option<String>,
    /// The precommits
    pub precommits: Option<String>,
    /// The last commit round
    pub last_commit_round: Option<i32>,
    /// The last commit
    pub last_commit: Option<String>,
    /// The catchup commit round
    pub catchup_commit_round: Option<i32>,
    /// The catchup commit
    pub catchup_commit: Option<String>,
}

/// Represents block parts header information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockPartsHeader {
    /// The total number of parts
    pub total: i32,
    /// The hash of the parts
    pub hash: String,
}

/// Represents peer statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct PeerStats {
    /// The number of votes
    pub votes: String,
    /// The number of block parts
    pub block_parts: String,
}
