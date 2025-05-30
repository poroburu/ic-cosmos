use candid::CandidType;
use serde::{Deserialize, Serialize};

use super::cosmos_common::NodeInfo;

/// Represents the net info response from a Cosmos node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NetInfoResponse {
    /// The JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: i32,
    /// The net info result
    pub result: NetInfo,
}

/// Represents network information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct NetInfo {
    /// Whether the node is listening
    pub listening: bool,
    /// The list of listeners
    pub listeners: Vec<String>,
    /// The number of peers
    pub n_peers: String,
    /// The list of peers
    pub peers: Vec<Peer>,
}

/// Represents a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "snake_case")]
pub struct Peer {
    /// The node information
    pub node_info: NodeInfo,
    /// Whether the peer is outbound
    pub is_outbound: bool,
    /// The connection status
    pub connection_status: ConnectionStatus,
    /// The remote IP address
    pub remote_ip: String,
}

/// Represents connection status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct ConnectionStatus {
    /// The duration of the connection
    #[serde(rename = "Duration")]
    pub duration: String,
    /// The send monitor
    #[serde(rename = "SendMonitor")]
    pub send_monitor: Monitor,
    /// The receive monitor
    #[serde(rename = "RecvMonitor")]
    pub recv_monitor: Monitor,
    /// The channels
    #[serde(rename = "Channels")]
    pub channels: Vec<Channel>,
}

/// Represents a monitor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct Monitor {
    /// Whether the monitor is active
    #[serde(rename = "Active")]
    pub active: bool,
    /// The start time
    #[serde(rename = "Start")]
    pub start: String,
    /// The duration
    #[serde(rename = "Duration")]
    pub duration: String,
    /// The idle time
    #[serde(rename = "Idle")]
    pub idle: String,
    /// The number of bytes
    #[serde(rename = "Bytes")]
    pub bytes: String,
    /// The number of samples
    #[serde(rename = "Samples")]
    pub samples: String,
    /// The instantaneous rate
    #[serde(rename = "InstRate")]
    pub inst_rate: String,
    /// The current rate
    #[serde(rename = "CurRate")]
    pub cur_rate: String,
    /// The average rate
    #[serde(rename = "AvgRate")]
    pub avg_rate: String,
    /// The peak rate
    #[serde(rename = "PeakRate")]
    pub peak_rate: String,
    /// The remaining bytes
    #[serde(rename = "BytesRem")]
    pub bytes_rem: String,
    /// The remaining time
    #[serde(rename = "TimeRem")]
    pub time_rem: String,
    /// The progress
    #[serde(rename = "Progress")]
    pub progress: i32,
}

/// Represents a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct Channel {
    /// The channel ID
    #[serde(rename = "ID")]
    pub id: i32,
    /// The send queue capacity
    #[serde(rename = "SendQueueCapacity")]
    pub send_queue_capacity: String,
    /// The send queue size
    #[serde(rename = "SendQueueSize")]
    pub send_queue_size: String,
    /// The priority
    #[serde(rename = "Priority")]
    pub priority: String,
    /// The recently sent count
    #[serde(rename = "RecentlySent")]
    pub recently_sent: String,
}
