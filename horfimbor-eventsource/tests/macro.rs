use serde::{Deserialize, Serialize};

use horfimbor_eventsource::{Command, CommandName, Event, EventName, StateName, StateNamed};
use horfimbor_eventsource_derive::{Command, Event, StateNamed};

const MACRO_DEMO_STATE_NAME: StateName = "Something";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, StateNamed)]
#[state(MACRO_DEMO_STATE_NAME)]
pub struct DemoConstState {}

#[derive(Deserialize, Serialize, Clone, Debug, Command)]
#[state(MACRO_DEMO_STATE_NAME)]
pub enum DemoConstCommand {
    FortyTwo,
}

#[derive(Deserialize, Serialize, Clone, Debug, Event)]
#[state(MACRO_DEMO_STATE_NAME)]
pub enum DemoConstEvent {
    FortyTwo,
}

#[test]
fn test_macros() {
    assert_eq!(DemoConstState::state_name(), "Something");
    assert_eq!(
        DemoConstCommand::FortyTwo.command_name(),
        "Something.CMD.FortyTwo"
    );
    assert_eq!(
        DemoConstEvent::FortyTwo.event_name(),
        "Something.evt.forty_two"
    );
}
