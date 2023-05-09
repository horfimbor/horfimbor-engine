// TODO add behind a feature flag
use crate::model_key::ModelKey;
use crate::state_db::{StateDb, StateDbError};
use crate::State;
use redis::{Client, Commands};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RedisStateDb<S> {
    client: Client,
    state: PhantomData<S>,
}

impl<S> RedisStateDb<S> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: PhantomData,
        }
    }
}

impl<S> StateDb<S> for RedisStateDb<S>
where
    S: State,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, StateDbError> {
        let mut connection = self.client.get_connection().unwrap();

        let data: Option<String> = connection.get(key.format()).unwrap();

        Ok(data)
    }

    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), StateDbError> {
        let mut connection = self.client.get_connection().unwrap();

        connection
            .set(key.format(), state)
            .map_err(|err| StateDbError::Internal(err.to_string()))?;

        Ok(())
    }
}
