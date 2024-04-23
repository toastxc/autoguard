use crate::db::{Db, Warning};
use crate::text::{help, links};
use crate::REPLACE;
use rand::random;
use reywen::structures::server::Role;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::channels::message::{DataMessageSend, Message},
};
use std::env;

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

    let Some(server_id) = db.c_poll(&message.channel, &client).await.unwrap() else {
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
    // unwraps SHOULD be infaliable
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
            db.warning_add(Warning {
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

// pub async fn message_handle(
//     client: Arc<Client>,
//     message: Message,
//     vdb: Vdb,
// ) -> Result<(), DeltaError> {
//     let prefix = env::var("COMMAND_PREFIX").unwrap();
//
//
//
//     let convec = match message.content_contains(&prefix, " ") {
//         None => return Ok(()),
//         Some(a) => a,
//     };
//     // help
//     // if convec.get(1) == Some(&"help".to_string()) {
//     //     client
//     //         .message_send(
//     //             &message.channel,
//     //             &DataMessageSend::new().set_content(&format!(
//     //
//     //             )),
//     //         )
//     //         .await?;
//     // }
//
//     // early return: too short
//     if convec.len() <= 2 {
//         return Ok(());
//     };
//
//     let channel = client.channel_fetch(&message.channel).await?;
//
//     let server_id = &channel.server_id().unwrap();
//
//     println!("{:?}", convec.get(1));
//     // differ commands
//     let (id, reason) = find_id(&message, &convec).await;
//     println!("id: {:?}", id);
//     match (convec.get(1).unwrap().as_str(), id) {
//         ("ban", Stuff::One(id)) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 client.ban_create(server_id, id, reason).await?;
//             }
//         }
//
//         ("kick", Stuff::One(id)) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 client.member_kick(server_id, &id).await?;
//             }
//         }
//
//         ("unban", Stuff::One(id)) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 client.ban_remove(server_id, &id).await?;
//             }
//         }
//         ("delete", Stuff::One(id)) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 client.message_delete(&message.channel, &id).await?;
//             }
//         }
//
//         ("delete", Stuff::Many(id)) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 client
//                     .message_bulk_delete(&message.channel, &DataBulkDelete::new().set_messages(id))
//                     .await?;
//             }
//         }
//         ("warn", _) => {
//             if auth_check(&client, server_id, &message.author, &message.channel).await? {
//                 if let Some(id) = convec.get(2) {
//                     let id = id.replace(REPLACE, "");
//                     let reason = reason.unwrap_or("no reason provided".to_string());
//                     let msg = DataMessageSend::from_embed_text(format!(
//                         "Infraction added for: {}. {}",
//                         id, reason
//                     ));
//                     client.message_send(&message.channel, &msg).await?;
//                 }
//             }
//         }
//         _ => return Ok(()),
//     }
//
//     Ok(())
// }
//
// // used before execution of a privileged command
// // returns true for root, false for not
// async fn auth_check(
//     client: &Client,
//     server_id: impl Into<String>,
//     message_author: impl Into<String>,
//     channel_id: impl Into<String>,
// ) -> Result<bool, DeltaError> {
//     let server_id = server_id.into();
//     let message_author = message_author.into();
//
//     let root = roles_get(
//         &client
//             .member_fetch(&server_id, &message_author)
//             .await?
//             .roles,
//         &client,
//         &server_id,
//     )
//     .await?;
//     println!("{:?}", root);
//     let target = env::var("ADMIN_ROLE_NAME").unwrap();
//     // let root =  root.get(&env::var("ADMIN_ROLE_NAME").unwrap())
//     //     .is_some();
//
//     let mut rootis = false;
//     for x in root.values() {
//         if &x.name == &target {
//             rootis = true;
//         }
//     }
//
//     if !rootis {
//         client
//             .message_send(
//                 channel_id.into(),
//                 &DataMessageSend::from_embed_text("You do not have permission for these commands"),
//             )
//             .await?;
//     };
//     Ok(rootis)
// }
