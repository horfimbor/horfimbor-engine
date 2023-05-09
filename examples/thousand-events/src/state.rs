use rand::Rng;
use serde::{Deserialize, Serialize};

use gyg_eventsource::{Command, Event, EventName, State};
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum RollCommand {
    Roll(u32),
    UnRoll(u32),
    End,
}

impl Command for RollCommand {
    fn command_name(&self) -> &'static str {
        match &self {
            RollCommand::Roll(_) => "Roll",
            RollCommand::UnRoll(_) => { "UnRoll" }
            RollCommand::End => { "End" }
        }
    }
}

#[derive(Error, Debug)]
pub enum RollError {
    #[error("the roll error is `{0}`")]
    Info(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RollEvent {
    Added(u32),
    Removed(u32),
    Ended(u32),
}

impl Event for RollEvent {
    fn event_name(&self) -> &'static str {
        match &self {
            RollEvent::Added(_) => "added",
            RollEvent::Removed(_) => "removed",
            RollEvent::Ended(_) => "ended",
        }
    }

    fn event_list() -> Vec<EventName> {
        vec!["added", "removed", "ended"]
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct RollState {
    pub nb: u32,
}

impl State for RollState {
    type Event = RollEvent;
    type Command = RollCommand;
    type Error = RollError;

    fn name_prefix() -> &'static str {
        "roll"
    }

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            RollEvent::Added(n) => {
                match self.nb.checked_add(*n) {
                    None => {
                        self.nb = u32::MAX
                    }
                    Some(r) => {
                        self.nb = r
                    }
                }
            }
            RollEvent::Removed(n) => {
                match self.nb.checked_sub(*n) {
                    None => {
                        self.nb = 0
                    }
                    Some(r) => {
                        self.nb = r
                    }
                }
            }
            RollEvent::Ended(n) => {
                println!("Ended with : {n}"); // for subscriber output
                self.nb = 0;
            }
        }
    }

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            RollCommand::Roll(n) => {
                if n == 0 {
                    return Err(RollError::Info("cannot roll 0".to_string()));
                }

                let mut rng = rand::thread_rng();
                let k = rng.gen_range(0..n);

                Ok(vec![RollEvent::Added(k + 1)])
            }
            RollCommand::UnRoll(n) => {
                if n == 0 {
                    return Err(RollError::Info("cannot unroll 0".to_string()));
                }

                let mut rng = rand::thread_rng();
                let k = rng.gen_range(0..n);

                Ok(vec![RollEvent::Removed(k + 1)])
            }
            RollCommand::End => {
                Ok(vec![RollEvent::Ended(self.nb)])
            }
        }
    }
}
