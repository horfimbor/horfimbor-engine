use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::state_db::{StateDb, StateDbError};
use gyg_eventsource::{Command, Event, State};
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

impl State for PokeState {
    type Event = PokeEvent;
    type Command = PokeCommand;
    type Error = PokeError;

    fn name_prefix() -> &'static str {
        "test-Poke"
    }
    fn play_event(&mut self, event: &Self::Event) {
        match event {
            PokeEvent::Poked(n) => self.nb += n,
        }
    }

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

impl<S> StateDb<S> for NoCache<S>
where
    S: State,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<Option<String>, StateDbError> {
        Ok(None)
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), StateDbError> {
        Ok(())
    }
}
