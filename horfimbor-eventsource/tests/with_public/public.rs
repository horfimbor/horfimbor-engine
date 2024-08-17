use horfimbor_eventsource::EventName;
use horfimbor_eventsource::{Event, StateName};
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
