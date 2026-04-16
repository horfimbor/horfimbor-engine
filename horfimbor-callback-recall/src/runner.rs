use crate::database::{CallBackRow, Pool};
use crate::{HandlerFn, Inner};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

pub(crate) async fn run<P: Pool>(inner: Inner<P>, duration: Duration) {
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
                println!("{e:?}");
            }
        }

        println!("foobar !");
    }
}

async fn run_one<P: Pool>(pool: P, handlers: HashMap<String, HandlerFn>, row: CallBackRow) {
    let now = Utc::now();

    if row.due_date > now {
        if let Ok(delta) = (row.due_date - now).to_std() {
            tokio::time::sleep(delta).await;
        }
    }

    let Some(handler) = handlers.get(&row.identifier) else {
        let res = pool.mark_failed(row.id, "no handler registered").await;
        if res.is_err() {
            dbg!(&res);
            todo!("handle error")
        }
        return;
    };

    let res = tokio::spawn(handler(row.payload)).await;

    match res {
        Ok(_) => {
            let res = pool.mark_fired(row.id).await;
            if res.is_err() {
                dbg!(&res);
                todo!("handle error")
            }
        }
        Err(err) => {
            let err = format!("fire failed : {err}");
            let res = pool.mark_failed(row.id, &err).await;
            if res.is_err() {
                dbg!(&res);
                todo!("handle error")
            }
        }
    }
}
