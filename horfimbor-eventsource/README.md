# horfimbor-eventsource

[![Crates.io](https://img.shields.io/crates/v/horfimbor-eventsource.svg)](https://crates.io/crates/horfimbor-eventsource)

Core event-sourcing library for the Horfimbor engine, built on top of [`KurrentDB`](https://www.kurrent.io/) with optional Redis caching.

## Features

- `cache-redis` *(default)* — Redis-backed state cache via `StateDb<S>`

## Quick Start

```toml
[dependencies]
horfimbor-eventsource = "0.4"
```

Infrastructure required: `KurrentDB` and (optionally) Redis. See the workspace `docker-compose.yaml`.

## Core Concepts

### Traits

Define your domain model by implementing four traits:

```rust
use horfimbor_eventsource::{Command, CommandName, Event, EventName, State, StateNamed, StateName, Dto};
use horfimbor_eventsource_derive::{Command, Event, StateNamed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const COUNTER: &str = "counter";

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[state(COUNTER)]
pub enum CounterEvent {
    Incremented,
    Decremented,
    Set { value: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Command)]
#[state(COUNTER)]
pub enum CounterCommand {
    Increment,
    Decrement,
    Set(i64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, StateNamed)]
#[state(COUNTER)]
pub struct Counter {
    pub value: i64,
}

#[derive(Debug, Error)]
pub enum CounterError {
    #[error("value cannot be negative")]
    NegativeValue,
}

impl Dto for Counter {
    type Event = CounterEvent;
    fn play_event(&mut self, event: &CounterEvent) {
        match event {
            CounterEvent::Incremented   => self.value += 1,
            CounterEvent::Decremented   => self.value -= 1,
            CounterEvent::Set { value } => self.value = *value,
        }
    }
}

impl State for Counter {
    type Command = CounterCommand;
    type Error   = CounterError;

    fn try_command(&self, cmd: CounterCommand) -> Result<Vec<CounterEvent>, CounterError> {
        match cmd {
            CounterCommand::Increment => Ok(vec![CounterEvent::Incremented]),
            CounterCommand::Decrement => {
                if self.value == 0 { return Err(CounterError::NegativeValue); }
                Ok(vec![CounterEvent::Decremented])
            }
            CounterCommand::Set(v) => Ok(vec![CounterEvent::Set { value: v }]),
        }
    }
}
```

### `ModelKey`

Every entity is identified by a `ModelKey` combining a stream name and a UUID:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use horfimbor_eventsource::model_key::ModelKey;

// New entity with a random time-ordered UUID
let key = ModelKey::new_uuid_v7("counter");

// Deterministic UUID from content (useful for singleton entities)
let key = ModelKey::new_uuid_v8("counter", "kind", "some-unique-data");

// Parse from string format "stream_name-uuid"
let key: ModelKey = "counter-01956b3a-0000-7000-0000-000000000000".try_into()?;
# Ok(())
# }
```

### Repository

Use `StateRepository` (read + write) or `DtoRepository` (read-only) to interact with `KurrentDB`:

```rust,no_run
# use horfimbor_eventsource::{Command, CommandName, Event, EventName, State, StateNamed, StateName, Dto};
# use horfimbor_eventsource_derive::{Command, Event, StateNamed};
# use serde::{Deserialize, Serialize};
# use thiserror::Error;
# const COUNTER: &str = "counter";
# #[derive(Debug, Clone, Serialize, Deserialize, Event)]
# #[state(COUNTER)]
# pub enum CounterEvent { Incremented }
# #[derive(Debug, Clone, Serialize, Deserialize, Command)]
# #[state(COUNTER)]
# pub enum CounterCommand { Increment }
# #[derive(Debug, Clone, Default, Serialize, Deserialize, StateNamed)]
# #[state(COUNTER)]
# pub struct Counter { pub value: i64 }
# #[derive(Debug, Error)]
# pub enum CounterError { #[error("e")] E }
# impl Dto for Counter { type Event = CounterEvent; fn play_event(&mut self, _: &CounterEvent) {} }
# impl State for Counter { type Command = CounterCommand; type Error = CounterError; fn try_command(&self, _: CounterCommand) -> Result<Vec<CounterEvent>, CounterError> { Ok(vec![]) } }
use horfimbor_eventsource::repository::{StateRepository, Repository};
use horfimbor_eventsource::cache_db::NoCache;
use horfimbor_eventsource::model_key::ModelKey;
use kurrentdb::Client;

async fn example(db: Client) -> Result<(), Box<dyn std::error::Error>> {
    let key = ModelKey::new_uuid_v7("counter");
    let repo: StateRepository<Counter, _> = StateRepository::new(db, NoCache::default());

    // Send a command — handles optimistic concurrency automatically
    let updated_state: Counter = repo.add_command(&key, CounterCommand::Increment, None).await?;
    println!("Counter is now: {}", updated_state.value);

    // Read current state
    let model = repo.get_model(&key).await?;
    println!("Counter: {}", model.state().value);
    Ok(())
}
```

With Redis caching:

```rust,no_run
# use horfimbor_eventsource::{Command, CommandName, Event, EventName, State, StateNamed, StateName, Dto};
# use horfimbor_eventsource_derive::{Command, Event, StateNamed};
# use serde::{Deserialize, Serialize};
# use thiserror::Error;
# const COUNTER: &str = "counter";
# #[derive(Debug, Clone, Serialize, Deserialize, Event)]
# #[state(COUNTER)]
# pub enum CounterEvent { Incremented }
# #[derive(Debug, Clone, Serialize, Deserialize, Command)]
# #[state(COUNTER)]
# pub enum CounterCommand { Increment }
# #[derive(Debug, Clone, Default, Serialize, Deserialize, StateNamed)]
# #[state(COUNTER)]
# pub struct Counter { pub value: i64 }
# #[derive(Debug, Error)]
# pub enum CounterError { #[error("e")] E }
# impl Dto for Counter { type Event = CounterEvent; fn play_event(&mut self, _: &CounterEvent) {} }
# impl State for Counter { type Command = CounterCommand; type Error = CounterError; fn try_command(&self, _: CounterCommand) -> Result<Vec<CounterEvent>, CounterError> { Ok(vec![]) } }
use horfimbor_eventsource::cache_db::redis::StateDb;
use horfimbor_eventsource::repository::{StateRepository, Repository};
use kurrentdb::Client;

async fn example(db: Client) -> Result<(), Box<dyn std::error::Error>> {
    let redis_client = redis::Client::open("redis://localhost:6379")?;
    let cache = StateDb::<Counter>::new(redis_client);
    let _repo: StateRepository<Counter, _> = StateRepository::new(db, cache);
    Ok(())
}
```

### Streams and Subscriptions

Events are stored in per-entity streams and projected by `KurrentDB` into category / event-type streams:

```rust,no_run
use horfimbor_eventsource::{Stream, helper};
use horfimbor_eventsource::model_key::ModelKey;
use kurrentdb::Client;
use uuid::Uuid;

async fn example(db: Client) -> Result<(), Box<dyn std::error::Error>> {
    let key = ModelKey::new_uuid_v7("counter");
    let correlation_uuid = Uuid::now_v7();

    // Subscribe to all events for a specific entity
    let _sub = helper::get_subscription(&db, &Stream::Model(key), None).await;

    // Subscribe to all events across an entire category (e.g. all counters)
    let _sub = helper::get_subscription(&db, &Stream::Stream("counter"), None).await;

    // Subscribe to a specific event type across all streams
    let _sub = helper::get_subscription(&db, &Stream::Event("counter.evt.incremented"), None).await;

    // Subscribe by correlation ID (trace all events from one originating command)
    let _sub = helper::get_subscription(&db, &Stream::Correlation(correlation_uuid), None).await;

    // Durable persistent subscription (survives restarts, supports competing consumers)
    let _sub = helper::get_persistent_subscription(&db, &Stream::Stream("counter"), "my-group").await?;
    Ok(())
}
```

### Metadata and Event Correlation

Every event written by this library carries `Metadata` that enables `KurrentDB`'s built-in correlation projections:

- `$correlationId` — UUID of the originating command; shared across all events in a causal chain.
- `$causationId` — UUID of the direct parent event or command.

### Cache Warming

Run a background task to keep Redis in sync with `KurrentDB` via a persistent subscription:

```rust,no_run
# use horfimbor_eventsource::{Command, CommandName, Event, EventName, State, StateNamed, StateName, Dto};
# use horfimbor_eventsource_derive::{Command, Event, StateNamed};
# use serde::{Deserialize, Serialize};
# use thiserror::Error;
# const COUNTER: &str = "counter";
# #[derive(Debug, Clone, Serialize, Deserialize, Event)]
# #[state(COUNTER)]
# pub enum CounterEvent { Incremented }
# #[derive(Debug, Clone, Serialize, Deserialize, Command)]
# #[state(COUNTER)]
# pub enum CounterCommand { Increment }
# #[derive(Debug, Clone, Default, Serialize, Deserialize, StateNamed)]
# #[state(COUNTER)]
# pub struct Counter { pub value: i64 }
# #[derive(Debug, Error)]
# pub enum CounterError { #[error("e")] E }
# impl Dto for Counter { type Event = CounterEvent; fn play_event(&mut self, _: &CounterEvent) {} }
# impl State for Counter { type Command = CounterCommand; type Error = CounterError; fn try_command(&self, _: CounterCommand) -> Result<Vec<CounterEvent>, CounterError> { Ok(vec![]) } }
use horfimbor_eventsource::repository::{StateRepository, Repository};
use horfimbor_eventsource::cache_db::NoCache;
use horfimbor_eventsource::Stream;
use kurrentdb::Client;

async fn example(db: Client) {
    let repo = StateRepository::new(db, NoCache::<Counter>::default());
    // cache_dto loops forever — run it in a spawned task
    tokio::spawn(async move {
        let _ = repo.cache_dto(&Stream::Stream("counter"), "cache-warmer-group").await;
    });
}
```

## Event and Command Naming

The derive macros generate stable, namespaced string identifiers:

| Type | Format | Example |
|---|---|---|
| Command | `"<STATE>.CMD.<VariantName>"` | `"counter.CMD.Increment"` |
| Event | `"<STATE>.evt.<variant_snake_case>"` | `"counter.evt.incremented"` |

These names are stored in `KurrentDB` and must remain stable. Renaming variants is a breaking change.

## Error Handling

- `EventSourceError` — database, serialization, and position errors.
- `EventSourceStateError` — wraps `EventSourceError` plus your `State::Error`.

## Integration Tests

The `tests/` directory contains complete examples:

| Test | Demonstrates |
|---|---|
| `state_only_test.rs` | Basic CRUD and concurrent command retry |
| `state_with_cache_test.rs` | Redis cache integration |
| `public_event_test.rs` | Tic-Tac-Toe with public/private event split and persistent subscriptions |

Run them with `KurrentDB` and Redis running:

```sh
just dc-up
cargo test
```
