use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Default, Hash)]
pub struct ModelKey {
    stream_name: String,
    stream_id: Uuid,
}
