use crate::Stream;
use eventstore::Error as EventStoreError;
use eventstore::{
    Client as EventDb, Error, PersistentSubscription, PersistentSubscriptionOptions, RetryOptions,
    StreamPosition, SubscribeToPersistentSubscriptionOptions, SubscribeToStreamOptions,
    Subscription,
};

/// # Errors
///
/// Will return an `Err` if the subscription cannot be created but not when it already exists.
pub async fn create_subscription(
    event_db: &EventDb,
    stream: &Stream,
    group_name: &str,
) -> Result<(), EventStoreError> {
    let opt = PersistentSubscriptionOptions::default().resolve_link_tos(true);

    let created = event_db
        .create_persistent_subscription(stream.to_string(), group_name, &opt)
        .await;

    match created {
        Ok(()) => {}
        Err(e) => match e {
            Error::ResourceAlreadyExists => {}
            _ => return Err(e),
        },
    }

    Ok(())
}

pub async fn get_subscription(
    event_db: &EventDb,
    stream: &Stream,
    position: Option<u64>,
) -> Subscription {
    let mut options = SubscribeToStreamOptions::default().retry_options(RetryOptions::default());

    options = match position {
        None => options.start_from(StreamPosition::Start),
        Some(n) => options.start_from(StreamPosition::Position(n)),
    };

    event_db
        .subscribe_to_stream(stream.to_string(), &options)
        .await
}

/// # Errors
///
/// Will return `Err` if the subscription cannot be created.
pub async fn get_persistent_subscription(
    event_db: &EventDb,
    stream: &Stream,
    group_name: &str,
) -> Result<PersistentSubscription, EventStoreError> {
    create_subscription(event_db, stream, group_name).await?;

    let options = SubscribeToPersistentSubscriptionOptions::default().buffer_size(1);

    event_db
        .subscribe_to_persistent_subscription(stream.to_string(), group_name, &options)
        .await
}
