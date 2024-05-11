use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct DatastoreValueMeta {
    id: u64,
    change_id: Uuid,
    timestamp: DateTime<Utc>,
}
