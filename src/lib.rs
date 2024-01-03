/// re-export import :
pub use chrono_craft_engine_derive;

pub mod cache_db;
pub mod metadata;
pub mod model_key;
pub mod repository;

const COMMAND_PREFIX: &str = "CMD";
const EVENT_PREFIX: &str = "evt";

use crate::cache_db::CacheDbError;
use crate::metadata::MetadataError;
use crate::model_key::ModelKey;
use eventstore::Error as EventStoreError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt::Debug;
use std::str::Utf8Error;
use thiserror::Error;
use uuid::Uuid;

pub type StreamName = &'static str;

pub enum Stream {
    Model(ModelKey),
    Stream(StreamName),
    Event(EventName),
    Correlation(Uuid),
}

impl ToString for Stream {
    fn to_string(&self) -> String {
        match self {
            Stream::Model(m) => m.format(),
            Stream::Stream(stream_name) => {
                let n = stream_name.replace('-', "_");
                format!("$ce-{}", n)
            }
            Stream::Event(e) => {
                format!("$et-{}", e)
            }
            Stream::Correlation(u) => {
                format!("bc-{}", u)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum EventSourceError<S> {
    #[error("Cache error")]
    CacheDbError(CacheDbError),

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

pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn command_name(&self) -> CommandName;
}

pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn event_name(&self) -> EventName;
}

pub trait Dto: Default + Serialize + DeserializeOwned + Debug + Send + Clone + Sync {
    type Event: Event + Sync + Send;
    type Error: Error + Sync + Send;

    fn play_event(&mut self, event: &Self::Event);
}

pub trait State: Dto {
    type Command: Command + Sync + Send;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use gyg_eventsource_derive::Command;
    use gyg_eventsource_derive::Event;
    use serde::Deserialize;

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Command, Event)]
    pub enum ToTest {
        Add(usize),
        Reset,
        SomeOtherVariant { a: String },
    }

    #[test]
    fn it_works() {
        let cmd_add = ToTest::Add(1);
        let cmd_reset = ToTest::Reset;
        let cmd_other = ToTest::SomeOtherVariant {
            a: "ok".to_string(),
        };

        assert_eq!(cmd_add.command_name(), "Add");
        assert_eq!(cmd_reset.command_name(), "Reset");
        assert_eq!(cmd_other.command_name(), "SomeOtherVariant");

        assert_eq!(cmd_add.event_name(), "add");
        assert_eq!(cmd_reset.event_name(), "reset");
        assert_eq!(cmd_other.event_name(), "some_other_variant");
    }
}
