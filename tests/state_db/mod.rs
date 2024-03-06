use horfimbor_engine_derive::{Command, Event, StateNamed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use horfimbor_engine::*;

#[derive(Deserialize, Serialize, Clone, Debug, Command)]
#[state(PokeState)]
pub enum PokeCommand {
    Poke(u32),
}

#[derive(Error, Debug)]
pub enum PokeError {
    #[error("the Poke error is `{0}`")]
    Info(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, Event)]
#[state(PokeState)]
pub enum PokeEvent {
    Poked(u32),
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, StateNamed)]
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
