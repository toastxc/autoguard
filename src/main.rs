mod mention;
mod on_message;
mod ticket;

use dotenv::dotenv;
use futures_util::StreamExt;

use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::{channels::message::Message, server::Role},
    websocket::{data::WebSocketEvent, error::Error},
};
use std::collections::HashMap;

use std::sync::Arc;
use std::{env, time::Duration};

const REPLACE: [char; 3] = ['<', '>', '@'];
#[tokio::main]
async fn main() {
    println!("Starting process for AUTOGUARD");
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

    println!("{:?}", client);

    loop {
        println!("Websocket process started");

        let client = client.clone();
        if let Err(error) = ws_2(client).await {
            println!("{:?}", error);
        };
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Restarting Websocket...");
    }
}

async fn ws_2(client: Client) -> Result<(), Error> {
    let (mut read, _) = client.websocket.dual_async().await?;

    let client = Arc::new(client);
    while let Some(item) = read.next().await {
        let client = Arc::clone(&client);
        if let WebSocketEvent::Message { message } = item {
            tokio::spawn(async move {
                if let Err(error) = on_message::message_handle(client, message.clone()).await {
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

async fn find_id(message: &Message, con_vec: &[String]) -> (Stuff<String>, Option<String>) {
    match (
        message.clone().replies,
        message.content.clone().unwrap().len(),
    ) {
        // reply
        (Some(replies), _) => {
            match replies.as_slice() {
                [] => {
                    unreachable!()
                }
                // reply to single message
                [item] => (Stuff::One(item.clone()), None),
                // reply to many message - delete message only
                items => (Stuff::Many(items.to_vec()), None),
            }
        }

        (None, 3) => {
            // username / id

            (
                Stuff::One(con_vec.get(3).unwrap().replace(REPLACE, "")),
                None,
            )
        }

        (None, 4) => (
            // username / id | with reason
            Stuff::One(con_vec.get(3).unwrap().replace(REPLACE, "")),
            Some(con_vec.get(3).unwrap().clone()),
        ),
        (None, _) => (Stuff::None, None),
    }
}

pub enum Stuff<T> {
    Many(Vec<T>),
    One(T),
    None,
}
