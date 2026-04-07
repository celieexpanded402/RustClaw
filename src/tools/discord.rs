use std::sync::Arc;

use anyhow::{Context, Result};
use serenity::all::{ChannelType, CreateChannel, EditChannel};
use serenity::http::Http;
use serenity::model::id::{ChannelId, GuildId, UserId};
use tracing::info;

pub async fn create_channel(
    http: &Arc<Http>,
    guild_id: u64,
    name: &str,
    kind: &str,
) -> Result<String> {
    let channel_type = match kind {
        "voice" => ChannelType::Voice,
        "category" => ChannelType::Category,
        _ => ChannelType::Text,
    };

    let guild = GuildId::new(guild_id);
    let builder = CreateChannel::new(name).kind(channel_type);
    let channel = guild
        .create_channel(http.as_ref(), builder)
        .await
        .context("Failed to create channel")?;

    info!(channel_id = %channel.id, %name, %kind, "Created Discord channel");
    Ok(format!(
        "Created {} channel #{} (id: {})",
        kind, channel.name, channel.id
    ))
}

pub async fn delete_channel(http: &Arc<Http>, channel_id: u64) -> Result<String> {
    let channel = ChannelId::new(channel_id);
    channel
        .delete(http.as_ref())
        .await
        .context("Failed to delete channel")?;

    info!(%channel_id, "Deleted Discord channel");
    Ok(format!("Deleted channel {channel_id}"))
}

pub async fn create_role(
    http: &Arc<Http>,
    guild_id: u64,
    name: &str,
    color: u32,
) -> Result<String> {
    let guild = GuildId::new(guild_id);
    let role = guild
        .create_role(http.as_ref(), serenity::builder::EditRole::new().name(name).colour(color))
        .await
        .context("Failed to create role")?;

    info!(role_id = %role.id, %name, "Created Discord role");
    Ok(format!("Created role \"{}\" (id: {})", role.name, role.id))
}

pub async fn set_channel_topic(
    http: &Arc<Http>,
    channel_id: u64,
    topic: &str,
) -> Result<String> {
    let channel = ChannelId::new(channel_id);
    let builder = EditChannel::new().topic(topic);
    channel
        .edit(http.as_ref(), builder)
        .await
        .context("Failed to set channel topic")?;

    info!(%channel_id, "Updated channel topic");
    Ok(format!("Set topic for channel {channel_id}"))
}

pub async fn kick_member(http: &Arc<Http>, guild_id: u64, user_id: u64) -> Result<String> {
    let guild = GuildId::new(guild_id);
    guild
        .kick(http.as_ref(), UserId::new(user_id))
        .await
        .context("Failed to kick member")?;

    info!(%guild_id, %user_id, "Kicked member");
    Ok(format!("Kicked user {user_id} from guild {guild_id}"))
}

pub async fn ban_member(
    http: &Arc<Http>,
    guild_id: u64,
    user_id: u64,
    reason: &str,
) -> Result<String> {
    let guild = GuildId::new(guild_id);
    guild
        .ban_with_reason(http.as_ref(), UserId::new(user_id), 0, reason)
        .await
        .context("Failed to ban member")?;

    info!(%guild_id, %user_id, "Banned member");
    Ok(format!(
        "Banned user {user_id} from guild {guild_id}: {reason}"
    ))
}
