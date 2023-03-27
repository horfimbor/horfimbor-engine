use eventstore::{Client as EventClient, Client};
use redis::Commands;
use uuid::Uuid;
use gyg_eventsource::EventSourceError;
use gyg_eventsource::metadata::Metadata;

use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::state::State;
use gyg_eventsource::state_repository::StateRepository;

use crate::redis_state::{PokeCommand, PokeState, RedisStateDb};

mod concurrent;
mod redis_state;
mod simple;

type EasyRedisCache = RedisStateDb<PokeState>;



#[tokio::test]
async fn easy_case() {
    let redis_client = redis::Client::open("redis://localhost:6379/").unwrap();

    let event_store = get_event_db();


    match event_store.create_persistent_subscription("$ce-poke_test", "groupeP", &Default::default()).await {
        Ok(_) => {}
        Err(eventstore::Error::ResourceAlreadyExists) => {}
        Err(_) => { todo!("check error") }
    }


    let repo = StateRepository::new(event_store.clone(), EasyRedisCache::new(redis_client.clone()));

    let key = ModelKey::new("poke_test".to_string(), Uuid::new_v4().to_string());

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &PokeState { nb: 0 });

    let mut connection = redis_client.get_connection().unwrap();
    let data: Option<String> = connection.get(key.format()).unwrap();

    assert_eq!(data, None);

    let added = repo
        .add_command(&key, PokeCommand::Poke(80), None)
        .await
        .unwrap();

    assert_eq!(added, (PokeState { nb: 80 }));

    let mut sub = event_store.subscribe_to_persistent_subscription("$ce-poke_test", "groupeP", &Default::default()).await.unwrap();

    let event = sub.next().await.unwrap();
    dbg!(&event);
    let original_event = event.get_original_event();
    dbg!(&original_event);

    let metadata: Metadata =
        serde_json::from_slice(original_event.custom_metadata.as_ref()).unwrap();

    if metadata.is_event() {
        let event = original_event
            .as_json::<<PokeState as State>::Event>().unwrap();

        dbg!(&event);
    }else{
        let command = original_event
            .as_json::<<PokeState as State>::Command>().unwrap();

        dbg!(&command);
    }


    sub.ack(event).await.unwrap();



    let data: Option<String> = connection.get(key.format()).unwrap();
    assert_eq!(data, Some("Bob".to_string()));
}

fn get_event_db() -> Client {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    EventClient::new(settings).unwrap()
}