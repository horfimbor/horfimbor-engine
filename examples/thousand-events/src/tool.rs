
use eventstore::Client;
use gyg_eventsource::repository::EventRepository;
use gyg_eventsource::state_db_redis::RedisStateDb;
use crate::state::RollState;

pub fn get_event_db() -> Client {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    Client::new(settings).unwrap()
}


pub type RollCache = RedisStateDb<RollState>;

pub fn get_repository() -> EventRepository<RedisStateDb<RollState>, RollState> {
    let redis_client = redis::Client::open("redis://localhost:6379/").unwrap();

    let repo = EventRepository::new(get_event_db(), RollCache::new(redis_client.clone()));
    repo
}
