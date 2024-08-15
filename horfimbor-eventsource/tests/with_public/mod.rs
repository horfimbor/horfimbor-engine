use crate::with_public::public::{TTTEvents, Victory};
use horfimbor_eventsource::Command;
use horfimbor_eventsource::CommandName;
use horfimbor_eventsource::Event;
use horfimbor_eventsource::EventName;
use horfimbor_eventsource::StateNamed;
use horfimbor_eventsource::{Dto, State, StateName};
use horfimbor_eventsource_derive::{Command, Event, StateNamed};
use public::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub mod public;

// example with Tic Tac Toe with a 2 per 2 grid

pub const TTT_STATE: StateName = "TTT_STATE";

#[derive(Deserialize, Serialize, Debug, Clone, Command)]
#[state(TTT_STATE)]
pub enum TTTCommand {
    Create,
    Cross(usize),
    Circle(usize),
}

#[derive(Deserialize, Serialize, Debug, Clone, Event)]
#[state(TTT_STATE)]
pub enum TTTPlayedPrivate {
    Cross(usize),
    Circle(usize),
}

#[derive(Deserialize, Serialize, Debug, Clone, Event)]
#[composite_state]
#[serde(untagged)]
pub enum TTTPlayed {
    Public(TTTEvents),
    Private(TTTPlayedPrivate),
}

#[derive(Error, Debug)]
pub enum TTTError {
    BadPlayer,
    AlreadyPlayed,
    InvalidPosition,
}

impl Display for TTTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TTTError::BadPlayer => f.write_str("its not your turn"),
            TTTError::AlreadyPlayed => f.write_str("this cell has already being played"),
            TTTError::InvalidPosition => f.write_str("this is not a position on the grid"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, StateNamed)]
#[state(TTT_STATE)]
pub struct TTTState {
    turn: Option<Player>,
    grid: HashMap<usize, Player>,
    winner: Option<Victory>,
}

impl Dto for TTTState {
    type Event = TTTPlayed;
    type Error = TTTError;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            TTTPlayed::Public(p) => match p {
                TTTEvents::Started => {
                    self.turn = Some(Player::Circle);
                }
                TTTEvents::Ended(victory) => {
                    self.turn = None;
                    self.winner = Some(victory.clone());
                }
            },
            TTTPlayed::Private(p) => match p {
                TTTPlayedPrivate::Cross(pos) => {
                    self.grid.insert(*pos, Player::Cross);
                    self.turn = Some(Player::Circle);
                }
                TTTPlayedPrivate::Circle(pos) => {
                    self.grid.insert(*pos, Player::Circle);
                    self.turn = Some(Player::Cross);
                }
            },
        }
    }
}

impl TTTState {
    pub fn get_winner(&self) -> Option<Victory> {
        self.winner.clone()
    }

    fn check_position(&self, pos: usize, p: Player) -> Result<(), TTTError> {
        if self.turn != Some(p) {
            return Err(TTTError::BadPlayer);
        } else if pos >= 4 {
            return Err(TTTError::InvalidPosition);
        } else if self.grid.get(&pos).is_some() {
            return Err(TTTError::AlreadyPlayed);
        };

        Ok(())
    }

    fn check_victory(&self, pos: usize, p: Player) -> Option<Victory> {
        match pos {
            0 => {
                if let Some(previous) = self.grid.get(&1) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
                if let Some(previous) = self.grid.get(&2) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
            }
            1 => {
                if let Some(previous) = self.grid.get(&0) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
                if let Some(previous) = self.grid.get(&3) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
            }
            2 => {
                if let Some(previous) = self.grid.get(&0) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
                if let Some(previous) = self.grid.get(&3) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
            }
            3 => {
                if let Some(previous) = self.grid.get(&1) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
                if let Some(previous) = self.grid.get(&2) {
                    if previous == &p {
                        return Some(Victory::Winner(p));
                    }
                }
            }
            _ => return None,
        }

        if self.grid.len() == 3 {
            // 3 + current pos = 4
            return Some(Victory::Draw);
        }

        None
    }
}

impl State for TTTState {
    type Command = TTTCommand;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            TTTCommand::Create => Ok(vec![TTTPlayed::Public(TTTEvents::Started)]),
            TTTCommand::Cross(pos) => {
                self.check_position(pos, Player::Cross)?;
                let mut res = vec![TTTPlayed::Private(TTTPlayedPrivate::Cross(pos))];
                if let Some(victory) = self.check_victory(pos, Player::Cross) {
                    res.push(TTTPlayed::Public(TTTEvents::Ended(victory)))
                }
                Ok(res)
            }
            TTTCommand::Circle(pos) => {
                self.check_position(pos, Player::Circle)?;
                let mut res = vec![TTTPlayed::Private(TTTPlayedPrivate::Circle(pos))];
                if let Some(victory) = self.check_victory(pos, Player::Circle) {
                    res.push(TTTPlayed::Public(TTTEvents::Ended(victory)))
                }
                Ok(res)
            }
        }
    }
}
