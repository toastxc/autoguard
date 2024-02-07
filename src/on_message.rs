use crate::{find_id, mention, roles_get, ticket, Stuff};
use reywen::structures::channels::message::SendableEmbed;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::channels::message::{DataBulkDelete, DataMessageSend, Message},
};
use std::env;
use std::fmt::Write;
use std::sync::Arc;

pub enum EmbedColour {
    Error,
    Warning,
    Success,
}
impl EmbedColour {
    pub fn display(&self) -> String {
        format!(
            "var(--{})",
            match self {
                EmbedColour::Error => "error",
                EmbedColour::Warning => "warning",
                EmbedColour::Success => "success",
            }
        )
    }
}

pub fn embed_error(text: impl Into<String>, description: Option<&str>) -> DataMessageSend {
    let mut embed = SendableEmbed::default()
        .set_title(text)
        .set_colour(EmbedColour::Error.display());
    embed.description = description.map(|a| a.to_string());
    DataMessageSend::from_embed(embed)
}

pub async fn message_handle(client: Arc<Client>, message: Message) -> Result<(), DeltaError> {

    // this system will ban mass spam accounts
    let spam_protect = true;
    // make sure to change the keyword
    if message.content_contains("KEYWORD", " ").is_some() && spam_protect {
        client
            .ban_create(
                "SERVER_ID",
                message.author,
                Some(String::from("raid")),
            )
            .await?;
        println!("banned");
        return Ok(());
    }




    let prefix = env::var("COMMAND_PREFIX").unwrap();

    // no current wordlist ban

    // from here on out, this is command handling

    let convec = match message.content_contains(&prefix, " ") {
        None => return Ok(()),
        Some(a) => a,
    };
    // help
    if convec.get(1) == Some(&"help".to_string()) {
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

    // early return: too short
    if convec.len() < 2 {
        return Ok(());
    };

    if convec.get(1) == Some(&"dbg".to_string()) {
        let content = match convec.get(2).map(|a| a.as_str()) {
            Some("categories") => {
                println!("fetching channel");
                let server_id = &client
                    .channel_fetch(&message.channel)
                    .await?
                    .server_id()
                    .unwrap();

                println!("fetchin server");
                let cat = client.server_fetch(server_id).await?.categories;

                if let Some(categories) = cat {
                    categories
                        .iter()
                        // .map(|a| format!("```text\n{}\n```\n", a.title.clone()))
                        .fold(String::new(), |mut output, b| {
                            let _ = write!(output, "```text\n{}\n```\n", b.title);
                            output
                        })
                } else {
                    String::from("none")
                }
            }
            _ => "notvalid".to_string(),
        };
        println!("sending message");
        let data = DataMessageSend::from_content(content);
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        client.message_send(&message.channel, &data).await?;

        println!("done");
        return Ok(());
    }

    let channel = client.channel_fetch(&message.channel).await?;

    let server_id = &channel.server_id().unwrap();

    println!("{:?}", convec);

    // ?/ ticket
    if convec.get(1) == Some(&"ticket".to_string()) {
        return ticket::process(&client, &message, &convec, &channel).await;
    }

    let member_roles = client
        .member_fetch(server_id.clone(), &message.author)
        .await?
        .roles;

    if roles_get(&member_roles, &client, server_id)
        .await?
        .get(&env::var("ADMIN_ROLE_NAME").unwrap())
        .is_none()
    {
        return Ok(());
    };

    // mention everyone
    // each ping is 30 chars

    if let Some("mention") = convec.get(1).map(|a| a.as_str()) {
        return mention::process(&client, server_id, &message, &convec).await;
    }

    // differ commands
    let (id, reason) = find_id(&message, &convec).await;
    match (convec.get(1).unwrap().as_str(), id) {
        ("ban", Stuff::One(id)) => {
            client.ban_create(server_id, id, reason).await?;
        }
        ("kick", Stuff::One(id)) => {
            client.member_kick(server_id, &id).await?;
        }
        ("unban", Stuff::One(id)) => {
            client.ban_remove(server_id, &id).await?;
        }
        ("delete", Stuff::One(id)) => {
            client.message_delete(&message.channel, &id).await?;
        }
        ("delete", Stuff::Many(id)) => {
            client
                .message_bulk_delete(&message.channel, &DataBulkDelete::new().set_messages(id))
                .await?;
        }
        _ => {}
    }
    Ok(())
}
