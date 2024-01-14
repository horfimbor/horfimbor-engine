use serde::{Deserialize, Serialize};

use crate::StreamName;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ModelKey {
    stream_name: String,
    stream_id: String,
}

impl ModelKey {
    pub fn new(stream_name: StreamName, stream_id: String) -> Self {
        // maybe replace with an error ?
        let name = stream_name.replace('-', "_");
        Self {
            stream_name: name,
            stream_id,
        }
    }

    pub fn format(&self) -> String {
        format!("{}-{}", self.stream_name.replace('.', "_"), self.stream_id)
    }
}

impl From<&str> for ModelKey {
    fn from(value: &str) -> Self {
        let mut split = value.split('-');
        let stream_name = split.next().unwrap_or_default();
        let stream_id = split.collect::<Vec<&str>>().join("-");
        ModelKey {
            stream_name: stream_name.to_string(),
            stream_id,
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_format() {
        let m = ModelKey::new(
            "mpzNpYJ",
            "01797a2e-19de-467c-bda2-eddc2a2cbf8c".to_string(),
        );

        assert_eq!(
            m.format(),
            "mpzNpYJ-01797a2e-19de-467c-bda2-eddc2a2cbf8c".to_string()
        );
    }

    #[test]
    fn test_from() {
        let m = ModelKey::new(
            "mpzNpYJ",
            "01797a2e-19de-467c-bda2-eddc2a2cbf8c".to_string(),
        );

        let f: ModelKey = "mpzNpYJ-01797a2e-19de-467c-bda2-eddc2a2cbf8c".into();

        assert_eq!(f, m);
    }
}
