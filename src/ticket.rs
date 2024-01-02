use crate::roles_get;
use reywen::{
    client::Client,
    reywen_http::results::DeltaError,
    structures::{
        channels::{
            message::{DataMessageSend, Message},
            Channel, DataEditChannel,
        },
        permissions::{calculator::Permissions, definitions::Permission, DataRoleCreate},
        server::{member::DataMemberEdit, Category, DataChannelCreate, DataEditServer, Server},
    },
};
use std::env;
use std::time::Duration;

// currently non functional and slow :/
pub async fn process(
    client: &Client,
    message: &Message,
    convec: &[String],
    channel: &Channel,
) -> Result<(), DeltaError> {
    let start_channel = env::var("TICKET_START_CHANNEL").unwrap();
    let ticket_category = env::var("TICKET_CATEGORY").unwrap();

    // ?/ ticket lodge
    match convec.get(2).unwrap().as_str() {
        "lodge" => {
            println!("starting lodge");
            match channel {
                Channel::TextChannel { name, .. } => {
                    if name != &start_channel {
                        return Ok(());
                    };
                }
                _ => return Ok(()),
            };

            let server_id = &channel.server_id().unwrap();

            let user = client.user_fetch(&message.author).await?;

            println!("creating channel");
            let ticket_channel = client
                .channel_create(
                    server_id,
                    &DataChannelCreate::new(format!(
                        "Issue-{}-{}",
                        user.username, user.discriminator
                    )),
                )
                .await?;

            tokio::time::sleep(Duration::from_secs(5)).await;

            client
                .message_send(
                    ticket_channel.id(),
                    &DataMessageSend::from_embed_text("This will take a while..."),
                )
                .await?;

            println!("fetching categories");
            // new cato
            let server = client.server_fetch(server_id).await?;
            let mut category = String::new();
            let mut new_cat = Vec::new();

            if let Server {
                categories: Some(cat),
                ..
            } = server
            {
                // for every category, find the category with the right name and update its channels
                cat.iter().enumerate().for_each(|(iter, data)| {
                    if data.title.to_ascii_lowercase() == ticket_category.to_ascii_lowercase() {
                        category = data.clone().id;
                        let mut new_data = data.clone();
                        new_cat = cat.clone();
                        new_cat.remove(iter);
                        new_data.channels.push(ticket_channel.id());
                        new_cat.push(new_data);
                    }
                });
            };

            if category.is_empty() | new_cat.is_empty() {
                println!(
                    "No category found! required {} for ticketer",
                    ticket_category
                );
                return Ok(());
            };

            println!("server edit");
            client
                .server_edit(server_id, &DataEditServer::new().set_categories(new_cat))
                .await?;

            let mut role_id = String::new();

            println!("create user role");
            let mut member_roles = client
                .member_fetch(server_id.clone(), &message.author)
                .await?
                .roles;

            tokio::time::sleep(Duration::from_secs(4)).await;

            println!("getting roles");
            let roles = roles_get(&member_roles, client, server_id).await?;

            for (a, b) in roles {
                if b.name == message.author {
                    role_id = a;
                }
            }

            if role_id.is_empty() {
                println!("role not present, creating...");

                tokio::time::sleep(Duration::from_secs(10)).await;
                let new_role = client
                    .roles_create(
                        server_id,
                        &DataRoleCreate::new(&message.author).set_rank(100),
                    )
                    .await?;
                role_id = new_role.id;

                member_roles.push(role_id.clone());

                println!("editing user");
                client
                    .member_edit(
                        server_id,
                        &message.author,
                        &DataMemberEdit::default().set_roles(member_roles.clone()),
                    )
                    .await?;
                println!("editing self");
                client
                    .member_edit(
                        server_id,
                        client.self_id.clone().unwrap(),
                        &DataMemberEdit::default().add_role(&role_id),
                    )
                    .await?;
            };

            tokio::time::sleep(Duration::from_secs(10)).await;
            println!("allow user & bot");
            client
                .channel_permissions_set(
                    &ticket_channel.id(),
                    &role_id,
                    &Permissions::new()
                        .add_allow(Permission::ViewChannel)
                        .add_allow(Permission::SendMessage)
                        .add_allow(Permission::ReadMessageHistory)
                        .add_allow(Permission::ManageMessages),
                )
                .await?;

            println!("allow admin");

            let role_name = env::var("ADMIN_ROLE_NAME").unwrap();

            let mut admin_role_id = String::new();
            client.server_fetch(server_id).await?.roles.iter().for_each(|(id, name)| if name.name == role_name {


                admin_role_id = id.clone();
            });


            if !admin_role_id.is_empty() {
                client
                    .channel_permissions_set(
                        &ticket_channel.id(),
                        &admin_role_id,
                        &Permissions::new()
                            .add_allow(Permission::ViewChannel)
                            .add_allow(Permission::SendMessage)
                            .add_allow(Permission::ReadMessageHistory)
                            .add_allow(Permission::ManageMessages),
                    )
                    .await?;
            }


            println!("edit message");
            client
                .message_send(
                    &ticket_channel.id(),
                    &DataMessageSend::from_embed_text(
                        "Ticket Successfully created\nPlease wait for a moderator",
                    ),
                )
                .await?;

            println!("set default perm");
            client
                .channel_permissions_set_default(
                    ticket_channel.id(),
                    &Permissions::new()
                        .add_deny(Permission::SendMessage)
                        .add_deny(Permission::ViewChannel),
                )
                .await?;

            client
                .message_send(
                    &message.channel,
                    &DataMessageSend::from_content(format!(
                        "<#{}>\n[](<@{}>)",
                        ticket_channel.id(),
                        &message.author
                    ))
                    .add_embed_text("The ticket is ready"),
                )
                .await?;
        }
        "complete" => {
            println!("starting complete");

            if let Some(cat) = client
                .server_fetch(channel.server_id().unwrap())
                .await?
                .categories
            {
                for item in &cat {
                    if item.channels.contains(&message.channel)
                        && client.channel_fetch(&message.channel).await?.name()
                            != Some(start_channel.clone())
                    {
                        ticket_complete(
                            client,
                            channel,
                            message,
                            cat.clone(),
                            ticket_category.clone(),
                        )
                        .await?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

async fn ticket_complete(
    client: &Client,
    channel: &Channel,
    message: &Message,
    cat: Vec<Category>,
    ticket_category: String,
) -> Result<(), DeltaError> {
    // conformation message
    client
        .message_send(
            &message.channel,
            &DataMessageSend::from_embed_text("Removing..."),
        )
        .await?;

    // archiving message
    client
        .channel_edit(&message.channel, &DataEditChannel::new().set_archived(true))
        .await?;

    let member_roles = client
        .member_fetch(channel.server_id().unwrap(), &message.author)
        .await?
        .roles;

    let mut role_id = String::new();
    let roles = roles_get(&member_roles, client, &channel.server_id().unwrap()).await?;

    for (a, b) in roles {
        if b.name == message.author {
            role_id = a;
        }
    }

    client
        .channel_permissions_set(
            &message.channel,
            &role_id,
            &Permissions::new()
                .add_deny(Permission::SendMessage)
                .add_deny(Permission::ReadMessageHistory)
                .add_deny(Permission::ManageMessages),
        )
        .await?;

    // new categories

    let archive_category = env::var("ARCHIVE_CATEGORY").unwrap();

    let mut new_cat = cat.clone();

    for category in new_cat.clone().iter_mut() {
        for channel_id in &category.channels {
            // if channel ID match
            if channel_id == &message.channel {
                // remove old channel

                new_cat = new_cat
                    .iter_mut()
                    .map(|a| {
                        if a.title == archive_category {
                            a.channels.push(channel_id.clone());
                        } else if a.title == ticket_category {
                            for (c_iter, s_data) in a.channels.clone().into_iter().enumerate() {
                                if s_data == message.channel {
                                    a.channels.remove(c_iter);
                                }
                            }
                        }
                        a.clone()
                    })
                    .collect::<Vec<Category>>();
            }
        }
    }

    client
        .server_edit(
            channel.server_id().unwrap(),
            &DataEditServer::new().set_categories(new_cat.clone()),
        )
        .await?;
    Ok(())
}
//
// pub fn cat_display(input: Vec<Category>) -> Vec<Category> {
//     input
//         .into_iter()
//         .filter(|a| a.title == "✦  Do you need help? ✦" || a.title == "Archive")
//         .collect()
// }
