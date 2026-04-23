# horfimbor-eventsource-derive

[![Crates.io](https://img.shields.io/crates/v/horfimbor-eventsource-derive.svg)](https://crates.io/crates/horfimbor-eventsource-derive)

Procedural derive macros for [`horfimbor-eventsource`](https://crates.io/crates/horfimbor-eventsource).

You do not need to add this crate directly — it is re-exported from `horfimbor-eventsource`.

## Macros

### `#[derive(Command)]`

Implements `Command` for an enum. Requires the `#[state(CONST)]` attribute pointing to a `&'static str` constant.

```rust
const PLAYER: &str = "player";

#[derive(Debug, Clone, Serialize, Deserialize, Command)]
#[state(PLAYER)]
pub enum PlayerCommand {
    Join { name: String },
    Leave,
    SendMessage(String),
}
```

Generated `command_name()` values:

| Variant | `command_name()` |
|---|---|
| `Join { .. }` | `"player.CMD.Join"` |
| `Leave` | `"player.CMD.Leave"` |
| `SendMessage(..)` | `"player.CMD.SendMessage"` |

The variant name is kept in PascalCase exactly as written.

### `#[derive(Event)]`

Implements `Event` for an enum. The variant name is converted to `snake_case`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[state(PLAYER)]
pub enum PlayerEvent {
    Joined { name: String },
    Left,
    MessageSent(String),
}
```

Generated `event_name()` values:

| Variant | `event_name()` |
|---|---|
| `Joined { .. }` | `"player.evt.joined"` |
| `Left` | `"player.evt.left"` |
| `MessageSent(..)` | `"player.evt.message_sent"` |

#### Composite events

When a single event enum wraps multiple sub-event enums (e.g. to unify public and private events), use `#[composite_state]` instead of `#[state(...)]`. The `event_name()` call is delegated to the inner wrapped event.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[composite_state]
pub enum AllPlayerEvents {
    Player(PlayerEvent),
    Chat(ChatEvent),
}
```

Each inner variant must be a single-field tuple variant wrapping a type that already implements `Event`.

### `#[derive(StateNamed)]`

Implements `StateNamed` for a struct, returning the constant referenced by `#[state(CONST)]`.

```rust
#[derive(Debug, Default, Serialize, Deserialize, StateNamed)]
#[state(PLAYER)]
pub struct PlayerState {
    pub name: String,
    pub online: bool,
}
```

This is required for `State` and `Dto` implementations to work with the repository.

## Naming Convention

All names are built at **compile time** as `&'static str` values:

```
Commands : "<STATE_CONST>.CMD.<VariantName>"   (PascalCase preserved)
Events   : "<STATE_CONST>.evt.<variant_name>"  (converted to snake_case)
```

These strings are stored in KurrentDB. Renaming a variant or changing the `STATE_CONST` is a **breaking change** — existing events in the database will no longer be recognized.
