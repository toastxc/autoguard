use dotenv::dotenv;
use futures_util::StreamExt;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::channels::{
        message::{DataBulkDelete, DataMessageSend, Message},
        Channel,
    },
    websocket::{data::WebSocketEvent, error::Error},
};
use std::{env, time::Duration};

const REPLACE: [char; 3] = ['<', '>', '@'];
#[tokio::main]
async fn main() {
    println!("Starting process for AUTOGUARD");
    dotenv().ok();

    let token = env::var("BOT_TOKEN").unwrap();

    let client = {
        println!("Third party instance detected");
        if env::var("OTHER_INSTANCE").unwrap().parse::<bool>().unwrap() {
            let mut client = Client::from_token(&token, true).unwrap();
            client.http.url = env::var("INSTANCE_API_URI").clone().unwrap();
            client.websocket.domain = env::var("INSTANCE_WS_URI").unwrap().clone();
            client
        } else {
            println!("Official instance detected");
            Client::from_token(&token, true).unwrap()
        }
    };
    println!("Created client successfully");

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
                if let Err(error) = message_handle(&client, message).await {
                    println!("{:?}", error)
                }
            });
        }
    }

    // code in container somehow reaches here...
    Ok(())
}

async fn message_handle(client: &Client, message: Message) -> Result<(), DeltaError> {
    let prefix = env::var("COMMAND_PREFIX").unwrap();

    // no current wordlist ban

    // from here on out, this is command handling

    let convec = match message.content_contains(&prefix, " ") {
        None => return Ok(()),
        Some(a) => a,
    };
    // help
    if convec.get(1).cloned() == Some("help".to_string()) {
        client
            .message_send(
                &message.channel,
                &DataMessageSend::new().set_content(&format!(
                    "<prefix> <action> <item>\nprefix is set to {}",
                    prefix
                )),
            )
            .await?;
    }
    // earlyreturn: too short
    if convec.len() < 2 {
        return Ok(());
    };

    // find user
    let (id, reason) = find_id(&message, &convec).await;

    let server_id = match client.channel_fetch(&message.channel).await? {
        Channel::TextChannel { server, .. } => server,
        _ => {
            panic!("not server")
        }
    };

    let user = client.member_fetch(&server_id, message.author).await?;

    if !user.roles.contains(&env::var("ADMIN_ROLE").unwrap()) {
        return Ok(());
    };

    // differ commands
    match (convec.get(1).unwrap().as_str(), id) {
        ("ban", Stuff::One(id)) => {
            client.ban_create(&server_id, id, reason).await?;
        }
        ("kick", Stuff::One(id)) => {
            client.member_kick(&server_id, &id).await?;
        }
        ("unban", Stuff::One(id)) => {
            client.ban_remove(&server_id, &id).await?;
        }
        ("delete", Stuff::One(id)) => {
            client.message_delete(&message.channel, &id).await?;
        }
        ("delete", Stuff::Many(id)) => {
            client
                .message_bulk_delete(message.channel, &DataBulkDelete::new().set_messages(id))
                .await?;
        }
        _ => {}
    }
    Ok(())
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
