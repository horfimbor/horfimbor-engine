use crate::database::{CallBackRow, Pool};
use crate::{HandlerFn, Inner};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

pub async fn run<P: Pool>(inner: Inner<P>, duration: Duration) {
    let mut interval = tokio::time::interval(duration);
    loop {
        interval.tick().await;

        let due_before = Utc::now() + duration;

        match inner.pool.fetch_due_soon(due_before).await {
            Ok(rows) => {
                for row in rows {
                    tokio::spawn(run_one(inner.pool.clone(), inner.handlers.clone(), row));
                }
            }
            Err(e) => {
                eprintln!("callback-recall: failed to fetch due callbacks: {e}");
            }
        }
    }
}

async fn run_one<P: Pool>(pool: P, handlers: HashMap<String, HandlerFn>, row: CallBackRow) {
    let now = Utc::now();

    if row.due_date > now
        && let Ok(delta) = (row.due_date - now).to_std()
    {
        tokio::time::sleep(delta).await;
    }

    let Some(handler) = handlers.get(&row.identifier) else {
        if let Err(e) = pool.mark_failed(row.id, "no handler registered").await {
            eprintln!(
                "callback-recall: failed to mark callback {} as failed: {e}",
                row.id
            );
        }
        return;
    };

    let res = tokio::spawn(handler(row.payload)).await;

    match res {
        Ok(Ok(())) => {
            if let Err(e) = pool.mark_fired(row.id).await {
                eprintln!(
                    "callback-recall: failed to mark callback {} as fired: {e}",
                    row.id
                );
            }
        }
        Ok(Err(err)) => {
            let err = format!("event failed : {err}");
            if let Err(e) = pool.mark_failed(row.id, &err).await {
                eprintln!(
                    "callback-recall: failed to mark callback {} as failed: {e}",
                    row.id
                );
            }
        }
        Err(err) => {
            let err = format!("fire failed : {err}");
            if let Err(e) = pool.mark_failed(row.id, &err).await {
                eprintln!(
                    "callback-recall: failed to mark callback {} as failed: {e}",
                    row.id
                );
            }
        }
    }
}
