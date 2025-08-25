use crate::EventStoreProps;
use reqwasm::http::{Request, Response};
use serde::Serialize;
use std::fmt::Debug;

pub async fn send_command<C: Serialize + Debug, P: EventStoreProps + 'static>(
    cmd: &C,
    props: P,
) -> Result<Response, String> {
    Request::post(&format!(
        "{endpoint}/{path}/{id}",
        endpoint = props.endpoint(),
        path = props.path(),
        id = props.id()
    ))
    .body(serde_json::to_string(&cmd).map_err(|_| format!("cannot serialize cmd {:?}", &cmd))?)
    .header("Content-Type", "application/json")
    .header("Authorization", props.jwt())
    .send()
    .await
    .map_err(|_| "fail to send command".to_string())
}
