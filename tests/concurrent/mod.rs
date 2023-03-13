use std::thread;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use gyg_eventsource::state::{Command, Event, State};

use crate::concurrent::ConcurrentEvent::TimeTaken;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum ConcurrentCommand {
    TakeTime(u8, String),
}

impl Command for ConcurrentCommand {
    fn command_name(&self) -> &'static str {
        match &self {
            ConcurrentCommand::TakeTime(_, _) => "take_time",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConcurrentEvent {
    TimeTaken(String),
}

impl Event for ConcurrentEvent {
    fn event_name(&self) -> &'static str {
        match &self {
            TimeTaken(_) => "time_taken",
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConcurrentState {
    pub names: Vec<String>,
}

impl State for ConcurrentState {
    type Event = ConcurrentEvent;
    type Command = ConcurrentCommand;

    fn name_prefix() -> &'static str {
        "concurrent"
    }

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            TimeTaken(name) => {
                self.names.push(name.clone());
            }
        }
    }

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>> {
        match command {
            ConcurrentCommand::TakeTime(time, name) => {
                let wait = Duration::from_millis((100 * time) as u64);

                thread::sleep(wait);

                Ok(vec![TimeTaken(name.clone())])
            }
        }
    }
}
