use gyg_eventsource::cache_db::{CacheDb, CacheDbError};
use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::{Command, Dto, Event, State};
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

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

impl<S> CacheDb<S> for RedisStateDb<S>
where
    S: State,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, CacheDbError> {
        let mut connection = self.client.get_connection().unwrap();

        let data: Option<String> = connection.get(key.format()).unwrap();

        Ok(data)
    }

    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), CacheDbError> {
        let mut connection = self.client.get_connection().unwrap();

        connection
            .set(key.format(), state)
            .map_err(|err| CacheDbError::Internal(err.to_string()))?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum PokeCommand {
    Poke(u32),
}

impl Command for PokeCommand {
    fn command_name(&self) -> &'static str {
        match &self {
            PokeCommand::Poke(_) => "Poke",
        }
    }
}

#[derive(Error, Debug)]
pub enum PokeError {
    #[error("the Poke error is `{0}`")]
    Info(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PokeEvent {
    Poked(u32),
}

impl Event for PokeEvent {
    fn event_name(&self) -> &'static str {
        match &self {
            PokeEvent::Poked(_) => "poked",
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct PokeState {
    pub nb: u32,
}

impl Dto for PokeState {
    type Event = PokeEvent;
    type Error = PokeError;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            PokeEvent::Poked(n) => self.nb += n,
        }
    }
}

impl State for PokeState {
    type Command = PokeCommand;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            PokeCommand::Poke(n) => {
                if self.nb.checked_add(n).is_none() {
                    Err(PokeError::Info(format!(
                        "{} cannot be added to {}",
                        n, self.nb
                    )))
                } else {
                    Ok(vec![PokeEvent::Poked(n)])
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct NoCache<S> {
    state: PhantomData<S>,
}

impl<S> NoCache<S> {
    pub fn new() -> Self {
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
    S: State,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<Option<String>, CacheDbError> {
        Ok(None)
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), CacheDbError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct DtoNoCache<S> {
    state: PhantomData<S>,
}

impl<S> DtoNoCache<S> {
    pub fn new() -> Self {
        Self { state: PhantomData }
    }
}

impl<S> Default for DtoNoCache<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> CacheDb<S> for DtoNoCache<S>
where
    S: Dto,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<Option<String>, CacheDbError> {
        Ok(None)
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), CacheDbError> {
        Err(CacheDbError::Internal("Not allowed for dto".to_string()))
    }
}
