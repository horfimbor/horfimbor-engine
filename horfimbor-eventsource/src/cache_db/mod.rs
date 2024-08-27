//! handle the cache

use std::marker::PhantomData;

use thiserror::Error;

use crate::model_key::ModelKey;
use crate::repository::ModelWithPosition;
use crate::Dto;

#[cfg(feature = "cache-redis")]
pub mod redis;

/// `CacheDb` has only one purpose, reading and writing state somewhere
pub trait CacheDb<S>: Clone + Send + Sync
where
    S: Dto,
{
    /// internal function to read from the db
    ///
    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, DbError>;

    /// internal function to write in the db
    ///
    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), DbError>;

    /// public function to read the db
    ///
    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn get(&self, key: &ModelKey) -> Result<ModelWithPosition<S>, DbError> {
        let data = self.get_from_db(key);

        match data {
            Ok(None) => Ok(ModelWithPosition::default()),
            Ok(Some(value)) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
            Err(err) => Err(err),
        }
    }

    /// public function to write in the db
    ///
    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn set(&self, key: &ModelKey, data: ModelWithPosition<S>) -> Result<(), DbError> {
        let s = serde_json::to_string(&data)
            .map_err(|_err| todo!("error in StateDb.set is not handled yet"))?;
        self.set_in_db(key, s)
    }
}

/// cache db can fail in multiple ways.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum DbError {
    /// data store disconnected
    #[error("data store disconnected `{0}`")]
    Disconnect(String),

    /// internal error can be anything depending on the `cache_db`
    #[error("internal `{0}`")]
    Internal(String),
}

/// `NoCache` is a placeholder allowing quick development,
/// not recommended for production usage
#[derive(Clone)]
pub struct NoCache<S> {
    state: PhantomData<S>,
}

impl<S> NoCache<S> {
    /// simple constructor
    #[must_use]
    pub const fn new() -> Self {
        Self { state: PhantomData }
    }
}

impl<S> Default for NoCache<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> CacheDb<S> for NoCache<S>
where
    S: Dto,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<Option<String>, DbError> {
        Ok(None)
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), DbError> {
        Ok(())
    }
}
