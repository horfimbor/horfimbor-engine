# horfimbor-client

[![Crates.io](https://img.shields.io/crates/v/horfimbor-client.svg)](https://crates.io/crates/horfimbor-client)

WASM/Yew browser client library for [horfimbor-engine](https://github.com/horfimbor/horfimbor-engine) backends.

Provides:
- Live state components driven by **Server-Sent Events (SSE)** with automatic reconnection.
- A typed `send_command` helper for posting commands over HTTP.
- A `LoadExternalComponent` component for dynamically importing a remote WASM module.

More complete examples are available in [poc-monorepo](https://github.com/horfimbor/poc-monorepo/).

## Setup

```toml
[dependencies]
horfimbor-client = "0.1"
yew = { version = "0.23", features = ["csr"] }
```

This crate targets WASM. Build with `wasm-pack` or `trunk`.

## Live State Component

`EventStoreState<DTO, EVENT, PROP>` is a Yew component that maintains a live, up-to-date copy of a server-side entity by connecting to an SSE endpoint.

### 1. Define your DTO and Event types

These should match what your backend produces. The `Event` type is the incremental update, and `DTO` is the full snapshot.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Counter { pub value: i64 }

#[derive(Debug, Clone, Deserialize)]
pub enum CounterEvent {
    Incremented,
    Decremented,
    Set { value: i64 },
}
```

### 2. Define your Props

Props must implement `horfimbor_client::EventStoreProps` (and Yew's `Properties`):

```rust
use horfimbor_client::EventStoreProps;
use yew::Properties;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CounterProps {
    pub endpoint: AttrValue,  // base URL, e.g. "https://api.example.com"
    pub id: AttrValue,        // entity UUID
    pub jwt: AttrValue,       // JWT token
}

impl EventStoreProps for CounterProps {
    fn endpoint(&self) -> &str { &self.endpoint }
    fn path(&self)     -> &str { "counter" }      // API route segment
    fn jwt(&self)      -> &str { &self.jwt }
    fn id(&self)       -> &str { &self.id }
}
```

### 3. Implement `AddEvent` and render

```rust
use horfimbor_client::state::AddEvent;
use yew::Html;
use yew::html;

impl AddEvent<CounterEvent, CounterProps> for Counter {
    fn play_event(&mut self, event: &CounterEvent) {
        match event {
            CounterEvent::Incremented   => self.value += 1,
            CounterEvent::Decremented   => self.value -= 1,
            CounterEvent::Set { value } => self.value = *value,
        }
    }

    fn get_view(&self, _props: CounterProps) -> Html {
        html! { <p>{ format!("Counter: {}", self.value) }</p> }
    }
}
```

### 4. Use the component in Yew

```rust
use horfimbor_client::state::EventStoreState;

html! {
    <EventStoreState<Counter, CounterEvent, CounterProps>
        endpoint="https://api.example.com"
        id="01956b3a-..."
        jwt={jwt_token}
    />
}
```

The component connects to `{endpoint}/{path}/{id}/{jwt}` over SSE. The server can send:

- A full `DTO` snapshot — replaces the entire local state.
- An `EVENT` — applied incrementally via `play_event`.
- An error string — clears state and triggers a reconnect after 5 seconds.

A random subdomain prefix (`sse<N>.`) is prepended to the endpoint for load-balancing across multiple SSE connections.

## Sending Commands

```rust
use horfimbor_client::input::send_command;

// Inside an async callback or effect:
send_command(&CounterCommand::Increment, props.clone()).await?;
```

This POSTs the command as JSON to `{endpoint}/{path}/{id}` with an `Authorization: <jwt>` header.

## Loading a Remote WASM Component

`LoadExternalComponent` dynamically imports a compiled WASM component hosted on a remote server. Useful for micro-frontend architectures where each service ships its own UI.

```rust
use horfimbor_client::{LoadExternalComponent, Props};

html! {
    <LoadExternalComponent
        endpoint="https://counter-service.example.com"
        balise="counter-widget"
        jwt={jwt_token}
        id="01956b3a-..."
    />
}
```

This loads `{endpoint}/client/index.js` once per endpoint and renders `<counter-widget endpoint jwt id />` inside a custom element. The initialization is deduplicated — loading the same endpoint multiple times only calls the module once.

## `EventStoreProps` Trait

If the built-in `Props` struct does not fit your needs, implement `EventStoreProps` on any Yew `Properties` struct:

```rust
pub trait EventStoreProps: Properties {
    fn endpoint(&self) -> &str;
    fn path(&self) -> &str;
    fn jwt(&self) -> &str;
    fn id(&self) -> &str;
}
```
