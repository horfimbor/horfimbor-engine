use serde::{Deserialize, Serialize};

use horfimbor_eventsource::{Command, CommandName, Event, EventName, StateName, StateNamed};
use horfimbor_eventsource_derive::{Command, Event, StateNamed};

/// test simple macro

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

/// test macro with public_events attribute.

const STATE_NAME: StateName = "STATE_NAME";
const PUB_STATE_NAME: StateName = "PUB_NAME";

#[derive(Clone, Debug, Default, Serialize, Deserialize, StateNamed)]
#[state(STATE_NAME)]
pub struct TestState {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Command)]
#[state(STATE_NAME)]
pub enum TestCommand {
    Add(usize),
    Restart,
    SomethingElse { a: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Event)]
#[state(PUB_STATE_NAME)]
pub enum PublicTestEvent {
    Added(usize),
    Restarted,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Event)]
#[state(STATE_NAME)]
pub enum PrivateTestEvent {
    OtherStuff { a: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Event)]
#[composite_state]
pub enum TestEvent {
    Public(PublicTestEvent),
    Private(PrivateTestEvent),
}

#[test]
fn it_works() {
    let cmd_add = TestCommand::Add(1);
    let cmd_restart = TestCommand::Restart;
    let cmd_other = TestCommand::SomethingElse {
        a: "ok".to_string(),
    };

    assert_eq!(cmd_add.command_name(), "STATE_NAME.CMD.Add");
    assert_eq!(cmd_restart.command_name(), "STATE_NAME.CMD.Restart");
    assert_eq!(cmd_other.command_name(), "STATE_NAME.CMD.SomethingElse");

    let evt_add = TestEvent::Public(PublicTestEvent::Added(1));
    let evt_restarted = TestEvent::Public(PublicTestEvent::Restarted);
    let evt_other = TestEvent::Private(PrivateTestEvent::OtherStuff {
        a: "ok".to_string(),
    });

    assert_eq!(evt_add.event_name(), "PUB_NAME.evt.added");
    assert_eq!(evt_restarted.event_name(), "PUB_NAME.evt.restarted");
    assert_eq!(evt_other.event_name(), "STATE_NAME.evt.other_stuff");
}
