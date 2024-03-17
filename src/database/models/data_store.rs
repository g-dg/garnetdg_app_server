use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct DataStoreTreeNode {
    id: u64,
    parent_id: Option<u64>,
    key: String,
}

pub struct DataStoreValue {
    id: u64,
    tree_node_id: Option<i64>,
    change_id: Uuid,
    timestamp: DateTime<Utc>,
    value_serialized: String,
}
