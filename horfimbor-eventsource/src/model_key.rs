//! `ModelKey` is the entity unique id

use crate::StreamName;
use serde::{Deserialize, Serialize};
use uuid::{Error as UuidError, Uuid};

/// container for the entity key
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ModelKey {
    stream_name: String,
    stream_id: Uuid,
}

/// the model key allow to create in a safe way the identifier of the entity
impl ModelKey {
    /// the model key is created with a stream name representing the domain
    /// and an uuid, the id of the entity
    #[must_use]
    pub fn new(stream_name: StreamName, stream_id: Uuid) -> Self {
        // maybe replace with an error ?
        let name = stream_name.replace('-', "_");
        Self {
            stream_name: name,
            stream_id,
        }
    }

    /// the main purpose of the `ModelKey` is to provide this string.
    #[must_use]
    pub fn format(&self) -> String {
        format!("{}-{}", self.stream_name.replace('.', "_"), self.stream_id)
    }
}

impl TryFrom<&str> for ModelKey {
    type Error = UuidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split = value.split('-');
        let stream_name = split.next().unwrap_or_default();
        let to_check_uuid = split.collect::<Vec<&str>>().join("-");
        let stream_id = Uuid::parse_str(&to_check_uuid)?;
        Ok(Self {
            stream_name: stream_name.to_string(),
            stream_id,
        })
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
            Uuid::parse_str("01797a2e-19de-467c-bda2-eddc2a2cbf8c").unwrap(),
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
            Uuid::parse_str("01797a2e-19de-467c-bda2-eddc2a2cbf8c").unwrap(),
        );

        let f: ModelKey = "mpzNpYJ-01797a2e-19de-467c-bda2-eddc2a2cbf8c"
            .try_into()
            .unwrap();

        assert_eq!(f, m);
    }
}
