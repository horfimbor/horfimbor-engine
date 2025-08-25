pub mod input;
pub mod state;

use serde::de::DeserializeOwned;
use yew::prelude::*;

pub trait EventStoreProps: Properties + DeserializeOwned {
    fn endpoint(&self) -> &str;
    fn path(&self) -> &str;
    fn jwt(&self) -> &str;
    fn id(&self) -> &str;
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub endpoint: String,
    pub balise: String,
    pub jwt: String,
    pub id: String,
}

#[function_component]
pub fn LoadExternalComponent(props: &Props) -> Html {
    let key: String = props
        .endpoint
        .clone()
        .chars()
        .filter(|&c| c.is_alphanumeric())
        .collect();

    let script = format!(
        r#"
            import init, {{ run }} from '{endpoint}/client/index.js';

            if (window["{key}"] === undefined){{
                async function main() {{
                    await init();
                    run();
                }}
                main();
            }}
            window["{key}"] = 42;
            "#,
        endpoint = props.endpoint.clone()
    );

    html! {
        <>
            <script type="module">{script}</script>
            <@{props.balise.clone()}
                endpoint={props.endpoint.clone()}
                jwt={props.jwt.clone()}
                id={props.id.clone()}
            />
        </>
    }
}
