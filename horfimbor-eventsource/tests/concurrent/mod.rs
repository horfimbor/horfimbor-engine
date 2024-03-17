use std::thread;

use horfimbor_eventsource_derive::{Command, Event, StateNamed};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::Duration;

use horfimbor_eventsource::{
    Command, CommandName, Dto, Event, EventName, State, StateName, StateNamed,
};

use crate::concurrent::ConcurrentEvent::TimeTaken;

const CONCURRENT_STATE_NAME: StateName = "CONCURRENT_STATE_NAME";

#[derive(Deserialize, Serialize, Clone, Debug, Command)]
#[state(CONCURRENT_STATE_NAME)]
pub enum ConcurrentCommand {
    TakeTime(u8, String),
}

#[derive(Deserialize, Serialize, Debug, Clone, Event)]
#[state(CONCURRENT_STATE_NAME)]
pub enum ConcurrentEvent {
    TimeTaken(String),
}

#[derive(Error, Debug)]
pub enum ConcurrentError {}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, StateNamed)]
#[state(CONCURRENT_STATE_NAME)]
pub struct ConcurrentState {
    pub names: Vec<String>,
}

impl Dto for ConcurrentState {
    type Event = ConcurrentEvent;
    type Error = ConcurrentError;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            TimeTaken(name) => {
                self.names.push(name.clone());
            }
        }
    }
}

impl State for ConcurrentState {
    type Command = ConcurrentCommand;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            ConcurrentCommand::TakeTime(time, name) => {
                let wait = Duration::from_millis((100 * time) as u64);

                println!("wait : {:?}", wait);

                thread::sleep(wait);

                Ok(vec![TimeTaken(name.clone())])
            }
        }
    }
}
