# horfimbor-client-derive

[![Crates.io](https://img.shields.io/crates/v/horfimbor-client-derive.svg)](https://crates.io/crates/horfimbor-client-derive)

Procedural macro for [`horfimbor-client`](https://crates.io/crates/horfimbor-client).

Generates a [Web Component](https://developer.mozilla.org/en-US/docs/Web/Web_Components) wrapper around a Yew component, so it can be used as a standard HTML custom element (`<my-widget />`). The wrapper handles attribute observation, prop propagation, and lifecycle management.

More complete examples are available in [poc-monorepo](https://github.com/horfimbor/poc-monorepo/) and [horfimbor-template](https://github.com/horfimbor/horfimbor-template).

## Usage

```toml
[dependencies]
horfimbor-client-derive = "0.1"
```

## `#[derive(WebComponent)]`

Apply to a struct that lists the HTML attributes your custom element will expose. Each field maps to an HTML attribute (underscores are converted to hyphens).

```rust
use horfimbor_client_derive::WebComponent;

#[derive(WebComponent)]
#[component(CounterComponent)]  // the Yew component to wrap
pub struct CounterWidget {
    pub endpoint: String,
    pub id: String,
    pub jwt: String,
}
```

This generates a `pub mod optional` module containing everything needed to register the custom element.

### Registering the custom element

```rust
use custom_elements::define;
use counter_widget::optional::ComponentWrapper;

fn main() {
    define::<ComponentWrapper>("counter-widget");
    // <counter-widget endpoint="..." id="..." jwt="..." /> can now be used in HTML
}
```

### Optional attributes

Fields annotated with `#[optionnal]` (note: two `n`s — matches the crate spelling) become `Option<T>` in the generated props. The component renders normally even when optional attributes are absent or empty.

Fields **without** `#[optionnal]` are mandatory. The component renders empty HTML until all mandatory attributes are provided.

```rust
#[derive(WebComponent)]
#[component(PlayerCard)]
pub struct PlayerWidget {
    pub endpoint: String,   // mandatory — component hidden until set
    pub id: String,         // mandatory

    #[optionnal]
    pub theme: String,      // optional — rendered as Option<String>
}
```

## How it works

The macro generates a `pub mod optional` containing:

- A `#[derive(Properties)] struct <StructName>` with all fields as `Option<T>`.
- A `WrappedComponent` Yew component that gates rendering behind all mandatory fields being non-empty.
- A `ComponentWrapper` implementing `custom_elements::CustomElement`:
  - `inject_children` — mounts the Yew app into the shadow root.
  - `observed_attributes` — returns all field names as hyphenated HTML attribute names.
  - `attribute_changed_callback` — propagates HTML attribute changes to Yew props via `AppHandle::update`.
  - `disconnected_callback` — destroys the Yew app when the element is removed from the DOM.

## Attribute name mapping

Struct field names use underscores; HTML attribute names use hyphens. The mapping is automatic:

| Struct field | HTML attribute |
|---|---|
| `endpoint` | `endpoint` |
| `account_name` | `account-name` |
| `refresh_interval` | `refresh-interval` |

## Full example

```rust
use horfimbor_client_derive::WebComponent;
use yew::prelude::*;

// Your Yew component
#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: Option<String>,
    pub label: Option<String>,
}

#[function_component]
pub fn DisplayComponent(props: &Props) -> Html {
    html! {
        <div>
            <span>{ props.label.as_deref().unwrap_or("") }</span>
            <strong>{ props.value.as_deref().unwrap_or("—") }</strong>
        </div>
    }
}

// Generate the Web Component wrapper
#[derive(WebComponent)]
#[component(DisplayComponent)]
pub struct DisplayWidget {
    pub value: String,

    #[optionnal]
    pub label: String,
}

// Register and use in HTML:
// <display-widget value="42" label="Score:" />
```
