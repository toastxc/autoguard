mod mention;
mod on_message;
mod ticket;

use dotenv::dotenv;
use futures_util::StreamExt;

use crate::on_message::embed_error;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::{channels::message::Message, server::Role},
    websocket::{data::WebSocketEvent, error::Error},
};
use std::collections::HashMap;
use std::{env, time::Duration};

const REPLACE: [char; 3] = ['<', '>', '@'];
#[tokio::main]
async fn main() {
    println!("Starting process for AUTOGUARD");
    dotenv().ok();

    let token = env::var("BOT_TOKEN").unwrap();
    let id = env::var("BOT_ID").unwrap();

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

    while let Some(item) = read.next().await {
        let client = client.clone();
        if let WebSocketEvent::Message { message } = item {
            tokio::spawn(async move {
                if let Err(error) = on_message::message_handle(&client, message.clone()).await {
                    println!("{:?}", error);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    client
                        .message_send(
                            &message.channel,
                            &embed_error(
                                format!("{:?}", error),
                                Some("An error has occured, the bot could not finish its tasks"),
                            ),
                        )
                        .await
                        .unwrap();
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

async fn find_id(message: &Message, convec: &[String]) -> (Stuff<String>, Option<String>) {
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
                Stuff::One(convec.get(3).unwrap().replace(REPLACE, "")),
                None,
            )
        }

        (None, 4) => (
            // username / id | with reason
            Stuff::One(convec.get(3).unwrap().replace(REPLACE, "")),
            Some(convec.get(3).unwrap().clone()),
        ),
        (None, _) => (Stuff::None, None),
    }
}

pub enum Stuff<T> {
    Many(Vec<T>),
    One(T),
    None,
}
