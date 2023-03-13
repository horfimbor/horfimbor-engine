use std::fmt::Debug;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub type CommandName = &'static str;
pub type EventName = &'static str;
pub type StateName = &'static str;

pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn command_name(&self) -> CommandName;
}

pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn event_name(&self) -> EventName;

    fn is_state_specific(&self) -> bool {
        true
    }
}

pub trait State: Default + Serialize + DeserializeOwned + Debug + Send + Clone {
    type Event: Event;
    type Command: Command + Sync + Send;

    fn name_prefix() -> StateName;

    fn play_event(&mut self, event: &Self::Event);

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>>;

    fn state_cache_interval() -> Option<u64> {
        None
    }
}
