use crate::with_public::TTTError;
use horfimbor_eventsource::EventName;
use horfimbor_eventsource::{Dto, Event, StateName};
use horfimbor_eventsource_derive::Event;
use serde::{Deserialize, Serialize};

pub const TTT_STREAM: &'static str = "ttt_stream";
pub const TTT_PUB: StateName = "TTT_PUB";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub enum Player {
    #[default]
    Cross,
    Circle,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Victory {
    Draw,
    Winner(Player),
}

#[derive(Deserialize, Serialize, Debug, Clone, Event, PartialEq)]
#[state(TTT_PUB)]
pub enum TTTEvents {
    Started,
    Ended(Victory),
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct TTTPub {}

impl Dto for TTTPub {
    type Event = TTTEvents;
    type Error = TTTError;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            TTTEvents::Started => {
                println!("STARTED !!");
            }
            TTTEvents::Ended(v) => {
                println!("victory : {v:?}");
            }
        }
    }
}
