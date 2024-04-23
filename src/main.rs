mod db;
mod events;
mod text;

use crate::db::Db;
use dotenv::dotenv;
use futures_util::StreamExt;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::server::Role,
    websocket::{data::WebSocketEvent, error::Error},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, time::Duration};

const REPLACE: [char; 5] = ['<', '>', '@', ' ', '\n'];
#[tokio::main]
async fn main() {
    println!("Starting process for AUTOGUARD");
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

    let client = {
        if env::var("OTHER_INSTANCE")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or_default()
        {
            println!("Third party instance detected");
            let mut client = Client::from_token(&token, id, true).unwrap();
            client.http.url = env::var("INSTANCE_API_URI").clone().unwrap();
            client.websocket.domain = env::var("INSTANCE_WS_URI").unwrap().clone();
            client
        } else {
            println!("Official instance detected");
            Client::from_token(&token, id, true).unwrap()
        }
    };

    println!("Created client successfully");

    println!("INIT: MONGO");
    let db = Db::init().await;

    loop {
        println!("Websocket process started");

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
                    events::on_message::message_handle(client, message.clone(), db).await
                {
                    println!("{:?}", error);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        }
    }

    Ok(())
}

pub async fn roles_get(
    member_roles: &[String],
    client: &Client,
    server_id: &String,
) -> Result<HashMap<String, Role>, DeltaError> {
    Ok(client
        .server_fetch(server_id)
        .await?
        .roles
        .into_iter()
        .filter(|a| member_roles.contains(&a.0))
        .collect())
}
