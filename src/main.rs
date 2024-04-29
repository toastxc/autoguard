mod db;
mod doc;
mod events;

use crate::db::Db;
use dotenv::dotenv;
use futures_util::StreamExt;
use reywen::structures::channels::message::DataMessageSend;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    websocket::{data::WebSocketEvent, error::Error},
};
use std::sync::Arc;
use std::{env, time::Duration};

const REPLACE: [char; 5] = ['<', '>', '@', ' ', '\n'];
#[tokio::main]
async fn main() {
    println!("Starting process for AUTOGUARD...");
    println!("INIT: ENV");
    dotenv().ok();

    let token = match (env::var("BOT_TOKEN"), env::var("SELF_TOKEN")) {
        (Ok(_), Ok(_)) => None,
        (Ok(token), _) | (_, Ok(token)) => Some(token),
        _ => None,
    };
    let Some(token) = token else {
        panic!("invalid token fields in .env");
    };

    let id = env::var("BOT_ID").expect("system requires self ID");

    println!("TOKEN: {}", token);

    let client = {
        if env::var("OTHER_INSTANCE")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or_default()
        {
            println!("INFO: Third party instance detected");
            let mut client = Client::from_token(&token, id, true).unwrap();
            client.http.url = env::var("INSTANCE_API_URI").clone().unwrap();
            client.websocket.domain = env::var("INSTANCE_WS_URI").unwrap().clone();
            client
        } else {
            println!("INFO: Official instance detected");
            Client::from_token(&token, id, true).unwrap()
        }
    };
    println!("INIT: Reywen");

    let db = Db::init().await;
    println!("INIT: MONGO");
    loop {
        println!("INIT: Websocket");

        let client = client.clone();
        if let Err(error) = ws_2(client, db.clone()).await {
            println!("{:?}", error);
        };
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Restarting Websocket...");
    }
}

async fn ws_2(client: Client, db: Db) -> Result<(), Error> {
    let (mut read, _) = client.websocket.dual_async().await?;

    let client = Arc::new(client);
    while let Some(item) = read.next().await {
        let db = db.clone();
        let client = Arc::clone(&client);
        if let WebSocketEvent::Message { message } = item {
            tokio::spawn(async move {
                if let Err(error) =
                    events::on_message::message_handle(client.clone(), message.clone(), db).await
                {
                    println!("{:?}", error);
                    if let DeltaError::StatusCode(a) = error {
                        _ = client
                            .message_send(
                                &message.channel,
                                &DataMessageSend::from_embed_text(format!("Error {}!", a.as_u16()))
                                    .add_reply_str(&message.id),
                            )
                            .await;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        }
    }

    Ok(())
}
