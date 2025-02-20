//! Redis implementation of the `CacheDb`

use std::marker::PhantomData;

use redis::{Client, Commands};

use crate::Dto;
use crate::cache_db::{CacheDb, DbError};
use crate::model_key::ModelKey;

/// The `StateDb` is a container for the Type system and a db connection
#[derive(Clone)]
pub struct StateDb<S> {
    client: Client,
    state: PhantomData<S>,
}

impl<S> StateDb<S> {
    /// simple constructor
    #[must_use]
    pub const fn new(client: Client) -> Self {
        Self {
            client,
            state: PhantomData,
        }
    }
}

impl<S> CacheDb<S> for StateDb<S>
where
    S: Dto,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, DbError> {
        let mut connection = self
            .client
            .get_connection()
            .map_err(|e| DbError::Disconnect(e.to_string()))?;

        let data: Option<String> = connection
            .get(key.format())
            .map_err(|e| DbError::Internal(e.to_string()))?;

        Ok(data)
    }

    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), DbError> {
        let mut connection = self
            .client
            .get_connection()
            .map_err(|e| DbError::Disconnect(e.to_string()))?;

        connection
            .set::<_, _, ()>(key.format(), state)
            .map_err(|err| DbError::Internal(err.to_string()))?;

        Ok(())
    }
}
