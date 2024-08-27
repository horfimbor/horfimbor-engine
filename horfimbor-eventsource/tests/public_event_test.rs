use crate::with_public::public::Player::Circle;
use crate::with_public::public::{TTTEvents, Victory, TTT_PUB, TTT_STREAM};
use crate::with_public::{TTTCommand, TTTState};
use eventstore::Client as EventClient;
use horfimbor_eventsource::cache_db::NoCache;
use horfimbor_eventsource::helper::get_persistent_subscription;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::{Repository, StateRepository};
use horfimbor_eventsource::Stream;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

mod with_public;

type TicTacToeState = NoCache<TTTState>;

#[tokio::test]
async fn test_with_public_event() {
    let repo_state = StateRepository::new(get_event_db(), TicTacToeState::new());

    tokio::spawn(async move {
        // this would typically be on another microservice

        let mut nb = 0;

        let stream = Stream::Stream(TTT_STREAM);

        let event_db = get_event_db();

        let mut sub = get_persistent_subscription(&event_db, &stream, "beta")
            .await
            .expect("cannot create subscribe");

        loop {
            nb += 1;
            let rcv_event = sub.next().await.expect("cannot resolve event");
            let event = match rcv_event.event.as_ref() {
                None => {
                    continue;
                }
                Some(event) => event,
            };

            if event.event_type.starts_with(TTT_PUB) {
                let pub_event = event
                    .as_json::<TTTEvents>()
                    .expect("cannot deserialize public event");

                if nb == 2 {
                    assert_eq!(pub_event, TTTEvents::Started);
                }
                if nb == 8 {
                    assert_eq!(pub_event, TTTEvents::Ended(Victory::Winner(Circle)));
                }
            }

            sub.ack(rcv_event).await.expect("cannot ack the event");

            dbg!(nb);
        }
    });

    let key = ModelKey::new(TTT_STREAM, Uuid::new_v4());

    repo_state
        .add_command(&key, TTTCommand::Create, None)
        .await
        .expect("cannot play command create");

    repo_state
        .add_command(&key, TTTCommand::Circle(1), None)
        .await
        .expect("circle cannot play");

    let result = repo_state
        .add_command(&key, TTTCommand::Circle(1), None)
        .await;

    assert!(result.is_err());

    repo_state
        .add_command(&key, TTTCommand::Cross(0), None)
        .await
        .expect("cross cannot play");

    repo_state
        .add_command(&key, TTTCommand::Circle(3), None)
        .await
        .expect("circle cannot play twice");

    let model = repo_state.get_model(&key).await.expect("cannot load TTT");

    assert_eq!(model.state().get_winner(), Some(Victory::Winner(Circle)));

    sleep(Duration::from_secs(3)).await;
}

fn get_event_db() -> EventClient {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    EventClient::new(settings).unwrap()
}
