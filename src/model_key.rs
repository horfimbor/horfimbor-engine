use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ModelKey {
    stream_name: String,
    stream_id: String,
}

impl ModelKey {
    pub fn new(stream_name: String, stream_id: String) -> Self {
        Self {
            stream_name,
            stream_id,
        }
    }

    pub fn format(&self) -> String {
        format!("{}.{}", self.stream_name.replace('.', "_"), self.stream_id)
    }
}

impl From<String> for ModelKey {
    fn from(value: String) -> Self {
        let mut split = value.split('.');
        let stream_name = split.next().unwrap_or_default();
        let stream_id = split.collect();
        ModelKey {
            stream_name: stream_name.to_string(),
            stream_id,
        }
    }
}
