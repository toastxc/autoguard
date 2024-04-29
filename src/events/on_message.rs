use crate::db::{Db, Warning};
// use crate::text::{help, links};
use crate::REPLACE;
use rand::random;
use reywen::structures::server::Role;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::channels::message::{DataMessageSend, Message},
};
use std::env;

use crate::doc::{help, links};
use std::sync::Arc;

pub async fn message_handle(
    client: Arc<Client>,
    message: Message,
    db: Db,
) -> Result<(), DeltaError> {
    let bot_id = env::var("BOT_ID").unwrap();
    let prefix = env::var("COMMAND_PREFIX").unwrap();
    if message.content == Some(format!("<@{bot_id}>")) {
        client
            .message_send(
                &message.channel,
                &DataMessageSend::from_content(format!(
                    "Hi! I'm Autoguard. my prefix is `{prefix}`. \nRun `{prefix} help` for help!"
                )),
            )
            .await?;
        return Ok(());
    }

    let Some(convec) = message.content_contains(&prefix, " ") else {
        return Ok(());
    };

    // quick help
    let quick = match convec.get(1).map(|a| a.as_str()) {
        Some("help") => Some(help(prefix)),
        Some("links") => Some(links()),
        _ => None,
    };

    if let Some(quick) = quick {
        return client
            .message_send(&message.channel, &DataMessageSend::from_content(quick))
            .await
            .map(|_| {});
    };

    if convec.len() <= 2 {
        return Ok(());
    };

    let Some(server_id) = db.c_alias_poll(&message.channel, &client).await.unwrap() else {
        return client
            .message_send(
                &message.channel,
                &DataMessageSend::from_embed_text("Error! operation is only possible for a server"),
            )
            .await
            .map(|_| {});
    };

    // apply for convec
    // ?? ban user reason
    // unwraps SHOULD be infallible
    let (operation, user, reason) = (
        convec.get(1).unwrap(),
        convec.get(2).unwrap().replace(REPLACE, ""),
        convec.get(3).cloned(),
    );

    let auth = auth(&client, &message, &server_id).await?;
    // if command is privileged and user is unauthorised
    if !auth && PRIV.contains(&operation.as_str()) {
        return client
            .message_send(
                &message.channel,
                &DataMessageSend::from_embed_text(
                    "Unauthorised! (you require a role named \"ADMIN\") ",
                ),
            )
            .await
            .map(|_| {});
    };

    if auth && user == bot_id {
        return client
            .message_send(&message.channel, &DataMessageSend::from_content("No?"))
            .await
            .map(|_| {});
    }

    match operation.as_str() {
        "ban" => {
            client.ban_create(&server_id, &user, reason).await?;
        }
        "unban" | "!ban" => {
            client.ban_remove(server_id, &user).await?;
        }
        "kick" => {
            client.member_kick(&server_id, &user).await?;
        }
        "delete" | "del" => {
            client.message_delete(&message.channel, &user).await?;
        }
        "warn" => {
            let reason_str = reason.clone().unwrap_or("no reason provided".to_string());
            let msg = DataMessageSend::from_embed_text(format!(
                "Infraction added for: {}. {}",
                user, reason_str
            ));
            client.message_send(&message.channel, &msg).await?;
            db.warn_user(Warning {
                _id: random::<u32>().to_string(),
                server_id,
                user_id: user,
                message: reason,
            })
            .await
            .unwrap();
        }
        "warns" => {
            let warns = db.warnings(user, server_id).await.unwrap();

            client
                .message_send(
                    message.channel,
                    &DataMessageSend::from_embed_text(&format!(
                        "{} warning(s) for user",
                        warns.len()
                    )),
                )
                .await?;
        }

        _ => {}
    }

    Ok(())
}

const PRIV: [&str; 5] = ["ban", "unban", "kick", "delete", "warn"];

async fn auth(
    client: &Arc<Client>,
    message: &Message,
    server_id: &String,
) -> Result<bool, DeltaError> {
    let target = env::var("ADMIN_ROLE_NAME").unwrap();
    for (_, Role { name, .. }) in client
        .member_fetch_roles(server_id, &message.author)
        .await?
        .roles
    {
        if name == target {
            return Ok(true);
        }
    }
    Ok(false)
}
