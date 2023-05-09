use gyg_eventsource::model_key::ModelKey;
use thousand_events::state::{RollCommand};
use uuid::Uuid;

use thousand_events::{STREAM_NAME, tool};
use std::time::Instant;

#[tokio::main]
async fn main (){
    let repo = tool::get_repository();

    let start = Instant::now();

    let key = ModelKey::new(STREAM_NAME.to_string(), Uuid::new_v4().to_string());

    for n in 1..100 {
        if n % 3 == 0 {
            repo
                .add_command(&key, RollCommand::UnRoll(6), None)
                .await
                .unwrap();
        }else {
            repo
                .add_command(&key, RollCommand::Roll(10), None)
                .await
                .unwrap();
        }
    }
    repo
        .add_command(&key, RollCommand::End, None)
        .await
        .unwrap();

    let duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);
}
