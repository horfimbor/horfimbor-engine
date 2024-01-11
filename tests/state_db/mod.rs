use chrono_craft_engine::{Command, Dto, Event, State};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
