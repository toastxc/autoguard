use reywen::client::Client;
use reywen::reywen_http::results::DeltaError;
use reywen::structures::channels::message::{
    DataBulkDelete, DataEditMessage, DataMessageSend, Message,
};

pub async fn process(
    client: &Client,
    server_id: &String,
    message: &Message,
    convec: &[String],
) -> Result<(), DeltaError> {
    if convec.get(2).map(|a| a.as_str()) != Some("everyone") {
        return Ok(());
    };
    // buffers - max 2000 char
    let mut buffer = Vec::new();

    let users = client.member_fetch_all(server_id).await?;

    for user in users.users {
        buffer.push(format!("\n<@{}>", user.id));
    }

    if buffer.len() < 66 {
        let content: String = buffer.into_iter().collect();
        let id = client
            .message_send(
                &message.channel,
                &DataMessageSend::new().set_content(content),
            )
            .await?;
        client
            .message_edit(
                &message.channel,
                id.id,
                &DataEditMessage::new().set_content("@everyone"),
            )
            .await?;
    } else {
        let mut buffer2: Vec<String> = Vec::new();
        buffer2.push(String::new());
        let mut iterator = 0;
        let mut current_count = 0;

        for item in buffer.clone() {
            if current_count == 66 {
                buffer2.push(String::new());
                iterator += 1;
                current_count = 0;
            };

            current_count += 1;
            buffer2[iterator] += &item
        }

        let mut id = Vec::new();
        for item in &buffer2 {
            client
                .message_send(&message.channel, &DataMessageSend::new().set_content(item))
                .await
                .map(|a| id.push(a.id))
                .ok();
        }

        client
            .message_bulk_delete(&message.channel, &DataBulkDelete { ids: id })
            .await
            .ok();

        client
            .message_send(
                &message.channel,
                &DataMessageSend::new().set_content("@everyone"),
            )
            .await
            .ok();

        println!(
            "number of messages: {}\ntotal chars: {}",
            buffer2.len(),
            buffer.into_iter().collect::<String>().len()
        );
    }
    Ok(())
}
