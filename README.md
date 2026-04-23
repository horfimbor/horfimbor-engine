[![Rust](https://github.com/horfimbor/horfimbor-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/horfimbor/horfimbor-engine/actions/workflows/rust.yml)

# Horfimbor Engine

A Rust workspace implementing event sourcing on top of [KurrentDB](https://www.kurrent.io/) (formerly EventStoreDB),
with WASM browser client support, JWT authentication, and game-time utilities.

More complete examples are available in [poc-monorepo](https://github.com/horfimbor/poc-monorepo/), [horfimbor-template](https://github.com/horfimbor/horfimbor-template) and [hfb-auth](https://github.com/horfimbor/hfb-auth/).

## Workspace Crates

| Crate                                                            | Description                              |
|------------------------------------------------------------------|------------------------------------------|
| [`horfimbor-callback-recall`](./horfimbor-eventsource)           | Persistent scheduled callback library    |
| [`horfimbor-eventsource`](./horfimbor-eventsource)               | Core event-sourcing engine (server-side) |
| [`horfimbor-eventsource-derive`](./horfimbor-eventsource-derive) | Derive macros for eventsource traits     |
| [`horfimbor-jwt`](./horfimbor-jwt)                               | Shared JWT authentication                |
| [`horfimbor-client`](./horfimbor-client)                         | WASM/Yew browser client via SSE          |
| [`horfimbor-client-derive`](./horfimbor-client-derive)           | Derive macro for Web Components          |
| [`horfimbor-time`](./horfimbor-time)                             | Game-time / real-time converter          |

## Architecture

```
┌─────────────────────────────────────────────┐
│                  Browser                    │
│  horfimbor-client + horfimbor-client-derive │
│  (Yew WASM components, SSE, HTTP POST)      │
└────────────────────┬────────────────────────┘
                     │ SSE / HTTP
┌────────────────────▼────────────────────────┐
│               Your Service                  │
│  horfimbor-eventsource + derive             │
│  horfimbor-jwt [server feature]             │
│  horfimbor-time (optional)                  │
|  horfimbor-callback-recall (optional)       |
└──────────┬──────────────────────────────────┘
           │
    ┌──────▼──────┐    ┌──────────┐
    │  KurrentDB  │    │  Redis   │
    │  (events)   │    │  (cache) │
    └─────────────┘    └──────────┘
```

### Write Path

1. A command arrives at your service.
2. `StateRepository::add_command` loads the current state (from Redis cache + KurrentDB replay).
3. `State::try_command` validates the command and returns a list of events.
4. Events are appended to KurrentDB using **optimistic concurrency** (automatic retry on version conflict).

### Read Path

1. `DtoRepository::get_model` checks Redis for a cached `(position, model)` pair.
2. Only events newer than the cached position are replayed from KurrentDB.
3. `Repository::cache_dto` runs a background loop using a **persistent subscription** to keep Redis up to date.

### Real-Time Push

`helper::get_subscription` opens a volatile subscription to a KurrentDB stream, suitable for feeding a WebSocket or SSE
endpoint to browser clients.

## Infrastructure

A `docker-compose.yaml` is provided with KurrentDB 25.1 (port 2113) and Redis 7.2 (port 6379).

```sh
# Start infrastructure
just dc-up

# Run all checks (test + clippy + fmt)
just precommit

# Generate docs for a crate
just doc horfimbor-eventsource
```

## Development

Please follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Workspace-level lints enforce:

- `unsafe_code = "forbid"`
- `clippy::unwrap_used = "warn"`
