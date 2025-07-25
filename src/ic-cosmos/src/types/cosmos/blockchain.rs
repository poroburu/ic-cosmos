use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::cosmos::common::{BlockHeader, BlockID};

/// Represents the response from the /blockchain endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockchainResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The blockchain result
    pub result: Blockchain,
}

/// Represents blockchain information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Blockchain {
    /// The last block height
    pub last_height: String,
    /// The block metadata
    pub block_metas: Vec<BlockMeta>,
}

/// Represents block metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct BlockMeta {
    /// The block ID
    pub block_id: BlockID,
    /// The block size
    pub block_size: String,
    /// The block header
    pub header: BlockHeader,
    /// The number of transactions
    pub num_txs: String,
}
