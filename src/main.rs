use dotenv::dotenv;
use futures_util::StreamExt;
use reywen::client::Client;
use reywen::structures::channels::message::{DataBulkDelete, DataMessageSend, Message};
use reywen::structures::channels::Channel;
use reywen::websocket::data::WebSocketEvent;
use reywen::websocket::error::Error;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("BOT_TOKEN").unwrap();

    let client = {
        if env::var("OTHER_INSTANCE").unwrap().parse::<bool>().unwrap() {
            let mut client = Client::from_token(&token, true).unwrap();
            client.http.url = env::var("INSTANCE_API_URI").clone().unwrap();
            client.websocket.domain = env::var("INSTANCE_WS_URI").unwrap().clone();
            client
        } else {
            Client::from_token(&token, true).unwrap()
        }
    };

    let client = Arc::new(client);

    loop {
        let client = Arc::clone(&client);
        ws_2(client).await.ok();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn ws_2(client: Arc<Client>) -> Result<(), Error> {
    let (read, _) = client.websocket.dual_async().await.unwrap();

    let mut read = read;

    while let Some(item) = read.next().await {
        if let WebSocketEvent::Message { message } = item {
            let client = Arc::clone(&client);
            tokio::spawn(async move { message_handle(client, message).await });
        }
    }
    Ok(())
}
async fn message_handle(client: Arc<Client>, message: Message) {
    println!("1");
    let prefix = env::var("COMMAND_PREFIX").unwrap();
    println!("1");

    // no current wordlist ban

    // from here on out, this is command handling

    let convec = match message.content_contains(&prefix, " ") {
        None => return,
        Some(a) => a,
    };
    println!("1");
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
            .await
            .unwrap();
    }
    // earlyreturn: too short
    if convec.len() < 2 {
        return;
    };

    // find user
    let (id, reason) = find_id(&message, &convec).await;

    let server_id = match client.channel_fetch(&message.channel).await.unwrap() {
        Channel::TextChannel { server, .. } => server,
        _ => {
            panic!("not server")
        }
    };

    let user = client
        .member_fetch(&server_id, message.author)
        .await
        .unwrap();

    if !user.roles.contains(&env::var("ADMIN_ROLE").unwrap()) {
        return;
    };

    // differ commands
    match (convec.get(1).unwrap().as_str(), id) {
        ("ban", Stuff::One(id)) => {
            client.ban_create(&server_id, id, reason).await;
        }
        ("kick", Stuff::One(id)) => {
            client.member_kick(&server_id, &id).await;
        }
        ("unban", Stuff::One(id)) => {
            client.ban_remove(&server_id, &id).await;
        }
        ("delete", Stuff::One(id)) => {
            client.message_delete(&message.channel, &id).await;
        }
        ("delete", Stuff::Many(id)) => {
            client
                .message_bulk_delete(message.channel, &DataBulkDelete::new().set_messages(id))
                .await;
        }
        _ => {}
    }
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
                Stuff::One(convec.get(3).unwrap().replace(['<', '>', '@'], "")),
                None,
            )
        }

        (None, 4) => (
            // username / id | with reason
            Stuff::One(convec.get(3).unwrap().replace(['<', '>', '@'], "")),
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
