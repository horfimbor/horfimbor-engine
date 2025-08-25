use crate::EventStoreProps;
use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use rand::Rng;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Error;
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;
use weblog::{console_info, console_warn};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;
use yew::{Component, Context, Html};

pub struct EventStoreState<DTO, EVENT, PROP>
where
    DTO: Default + DeserializeOwned,
    EVENT: DeserializeOwned,
    PROP: EventStoreProps + DeserializeOwned,
{
    es: Option<EventSource>,
    dto: Result<DTO, String>,
    message: PhantomData<EventStoreMessage<DTO, EVENT>>,
    props: PhantomData<PROP>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(bound = "DTO:  DeserializeOwned, EVENT:  DeserializeOwned")]
pub enum EventStoreMessage<DTO, EVENT>
where
    DTO: DeserializeOwned,
    EVENT: DeserializeOwned,
{
    Dto(DTO),
    Event(EVENT),
    Error(String),
    Reconnect,
}

pub trait AddEvent<EVENT, PROP>
where
    PROP: 'static,
{
    fn play_event(&mut self, event: &EVENT);
    fn get_view(&self, props: PROP) -> Html;
}

impl<DTO, EVENT, PROP> EventStoreState<DTO, EVENT, PROP>
where
    DTO: Default + DeserializeOwned + 'static + AddEvent<EVENT, PROP>,
    EVENT: DeserializeOwned + 'static,
    PROP: EventStoreProps + Clone + 'static,
{
    fn connect(&mut self, ctx: &Context<Self>) {
        if self.es.is_some() {
            return;
        }

        self.dto = Ok(DTO::default());

        let jwt = ctx.props().jwt();
        let id = ctx.props().id();
        let path = ctx.props().path();

        let mut rng = rand::rng();
        let Ok(mut url) = Url::parse(ctx.props().endpoint()) else {
            console_warn!(ctx.props().endpoint());
            console_warn!("cannot parse endpoint");
            return;
        };

        let Some(host) = url.host() else {
            console_warn!("no host");
            return;
        };

        let new_host = format!("sse{:06}.{host}", rng.random_range(1..1_000_000));
        match url.set_host(Some(&new_host)) {
            Ok(_) => {}
            Err(e) => {
                console_warn!("cannot set host : ");
                console_warn!(new_host);
                console_warn!(e.to_string());
                return;
            }
        }
        let endpoint = url.to_string();
        console_info!(endpoint.clone());

        let mut es = match EventSource::new(format!("{endpoint}/{path}/{id}/{jwt}").as_str()) {
            Ok(es) => es,
            Err(_) => {
                self.dto = Err(format!(
                    "cannot open eventsource to {endpoint}/{path}/{id}/<jwt>"
                ));
                return;
            }
        };

        let mut stream = match es.subscribe("message") {
            Ok(stream) => stream,
            Err(_) => {
                self.dto = Err("cannot subscribe to all messages".to_string());
                return;
            }
        };

        let link = ctx.link().clone();
        spawn_local(async move {
            while let Some(Ok((_, msg))) = stream.next().await {
                if let Some(json) = msg.data().as_string() {
                    let message: Result<EventStoreMessage<DTO, EVENT>, Error> =
                        serde_json::from_str(json.as_str());

                    let link = link.clone();
                    match message {
                        Ok(m) => {
                            link.send_message(m);
                        }
                        Err(_) => {
                            link.send_message(EventStoreMessage::<DTO, EVENT>::Error(
                                "stream closed".to_string(),
                            ));
                        }
                    }
                }
            }
            link.send_message(EventStoreMessage::<DTO, EVENT>::Error(
                "EventSource closed".to_string(),
            ));
        });

        self.es = Some(es);
    }
}

impl<DTO, EVENT, PROP> Component for EventStoreState<DTO, EVENT, PROP>
where
    DTO: Default + DeserializeOwned + 'static + AddEvent<EVENT, PROP>,
    EVENT: DeserializeOwned + 'static,
    PROP: EventStoreProps + Clone + 'static,
{
    type Message = EventStoreMessage<DTO, EVENT>;
    type Properties = PROP;

    fn create(ctx: &Context<Self>) -> Self {
        let mut state = Self {
            es: None,
            dto: Ok(DTO::default()),
            props: PhantomData,
            message: PhantomData,
        };

        state.connect(ctx);

        state
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventStoreMessage::<DTO, EVENT>::Dto(d) => {
                self.dto = Ok(d);
                true
            }
            EventStoreMessage::<DTO, EVENT>::Event(e) => match &mut self.dto {
                Ok(dto) => {
                    dto.play_event(&e);
                    true
                }
                Err(_) => false,
            },
            EventStoreMessage::<DTO, EVENT>::Error(e) => {
                self.dto = Err(e);
                self.es = None;

                let link = ctx.link().clone();

                spawn_local(async move {
                    sleep(Duration::from_secs(5)).await;
                    link.send_message(EventStoreMessage::<DTO, EVENT>::Reconnect);
                });
                true
            }
            EventStoreMessage::<DTO, EVENT>::Reconnect => {
                self.connect(ctx);
                if self.dto.is_err() {
                    let link = ctx.link().clone();
                    spawn_local(async move {
                        sleep(Duration::from_secs(5)).await;
                        link.send_message(EventStoreMessage::<DTO, EVENT>::Reconnect);
                    });
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = move || -> Html {
            match &self.dto {
                Ok(dto) => dto.get_view(ctx.props().clone()),
                Err(e) => {
                    html! {
                        <div class="event_source_error">
                            {e}
                        </div>
                    }
                }
            }
        };

        html! {
            {state()}
        }
    }
}
