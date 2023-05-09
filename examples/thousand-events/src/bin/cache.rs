use thousand_events::{STREAM_NAME, tool};


#[tokio::main]
async fn main (){

    let repo = tool::get_repository();

    match repo
        .create_subscription(STREAM_NAME, STREAM_NAME)
        .await {
        Ok(_) => {}
        Err(e) => {
            dbg!(e);
        }
    }

    repo
        .listen(STREAM_NAME, STREAM_NAME)
        .await
        .unwrap();
}