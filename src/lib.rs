/// re-export import :
pub use serde;
pub use gyg_eventsource_derive;

pub mod metadata;
pub mod model_key;
pub mod repository;
pub mod state_db;

const COMMAND_PREFIX: &str = "cmd";
const EVENT_PREFIX: &str = "evt";

use crate::metadata::MetadataError;
use crate::state_db::StateDbError;
use eventstore::Error as EventStoreError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt::Debug;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventSourceError<S> {
    #[error("Cache error")]
    StateDbError(StateDbError),

    #[error("Event store error")]
    EventStore(EventStoreError),

    #[error("Event store postion error")]
    Position(String),

    #[error("Utf8 error")]
    Utf8(Utf8Error),

    #[error("Metadata error")]
    Metadata(MetadataError),

    #[error("Serde error")]
    Serde(SerdeError),

    #[error("State error")]
    State(S),

    #[error("unknown cache db error")]
    Unknown,
}

pub type CommandName = &'static str;
pub type EventName = &'static str;
pub type StateName = &'static str;

pub enum EventType {
    State,
    Event,
}

pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn command_name(&self) -> CommandName;
}

pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn event_name(&self) -> EventName;

    fn get_type(&self) -> EventType {
        EventType::Event
    }
}

pub trait State: Default + Serialize + DeserializeOwned + Debug + Send + Clone {
    type Event: Event;
    type Command: Command + Sync + Send;
    type Error: Error;

    fn name_prefix() -> StateName;

    fn play_event(&mut self, event: &Self::Event);

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}


#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use gyg_eventsource_derive::Command;
    use gyg_eventsource_derive::Event;
    use super::*;

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Command, Event)]
    pub enum ToTest {
        Add(usize),
        Reset,
        SomeOtherVariant{a: String}
    }

    #[test]
    fn it_works() {

        let cmd_add = ToTest::Add(1);
        let cmd_reset = ToTest::Reset;
        let cmd_other = ToTest::SomeOtherVariant{a:"ok".to_string()};

        assert_eq!(cmd_add.command_name(), "Add");
        assert_eq!(cmd_reset.command_name(), "Reset");
        assert_eq!(cmd_other.command_name(), "SomeOtherVariant");

        assert_eq!(cmd_add.event_name(), "add");
        assert_eq!(cmd_reset.event_name(), "reset");
        assert_eq!(cmd_other.event_name(), "some_other_variant");

    }
}