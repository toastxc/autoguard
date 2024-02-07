use reywen::structures::channels::message::{DataMessageSend, SendableEmbed};

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
