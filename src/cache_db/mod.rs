#[cfg(feature = "cache-redis")]
pub mod redis;

use crate::model_key::ModelKey;
use crate::repository::ModelWithPosition;
use crate::Dto;
use thiserror::Error;

pub trait CacheDb<S>: Clone + Send + Sync
where
    S: Dto,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, CacheDbError>;
    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), CacheDbError>;

    fn get(&self, key: &ModelKey) -> Result<ModelWithPosition<S>, CacheDbError> {
        let data = self.get_from_db(key);

        match data {
            Ok(None) => Ok(ModelWithPosition::default()),
            Ok(Some(value)) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
            Err(err) => Err(err),
        }
    }

    fn set(&self, key: &ModelKey, data: ModelWithPosition<S>) -> Result<(), CacheDbError> {
        let s = serde_json::to_string(&data)
            .map_err(|_err| todo!("error in StateDb.set is not handled yet"))?;
        self.set_in_db(key, s)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CacheDbError {
    #[error("Not found")]
    NotFound,

    #[error("data store disconnected `{0}`")]
    Disconnect(String),

    #[error("unknown cache db error")]
    Unknown,

    #[error("internal `{0}`")]
    Internal(String),
}
