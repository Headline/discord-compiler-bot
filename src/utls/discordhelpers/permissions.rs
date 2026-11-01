use serenity::http::{Http, HttpError};
use serenity::model::channel::Channel;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::Permissions;

use crate::framework::CommandError;
use crate::utls::constants::{MISSING_PERMISSIONS_CODE, REQUIRED_PERMISSIONS};

/// True if the error is Discord rejecting a request for missing permissions
pub fn is_missing_permissions(err: &CommandError) -> bool {
    matches!(
        err.downcast_ref::<serenity::Error>(),
        Some(serenity::Error::Http(HttpError::UnsuccessfulRequest(res)))
            if res.error.code == MISSING_PERMISSIONS_CODE
    )
}

/// Builds the error text shown to users. Discord's "Missing Permissions"
/// response doesn't say which permission was missing, so in that case we
/// look up the bot's permissions in the channel and list the ones it lacks.
pub async fn describe_error(
    http: &Http,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
    err: &CommandError,
) -> String {
    if !is_missing_permissions(err) {
        return err.to_string();
    }

    let Some(guild_id) = guild_id else {
        return err.to_string();
    };

    let required = REQUIRED_PERMISSIONS.get_permission_names().join(", ");
    match missing_permissions(http, guild_id, channel_id).await {
        Some(missing) if !missing.is_empty() => format!(
            "Missing Permissions: I'm missing **{}** in this channel.",
            missing.get_permission_names().join(", ")
        ),
        _ => format!(
            "Missing Permissions: please make sure I have these permissions in this channel: {}",
            required
        ),
    }
}

/// Returns the required permissions the bot doesn't have in the channel,
/// or None if they couldn't be looked up.
async fn missing_permissions(
    http: &Http,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<Permissions> {
    let Ok(Channel::Guild(mut channel)) = channel_id.to_channel(http).await else {
        return None;
    };

    let mut required = REQUIRED_PERMISSIONS;
    // threads inherit permissions from their parent channel
    if channel.thread_metadata.is_some() {
        required |= Permissions::SEND_MESSAGES_IN_THREADS;
        required.remove(Permissions::SEND_MESSAGES);
        let parent = channel.parent_id?;
        let Ok(Channel::Guild(parent)) = parent.to_channel(http).await else {
            return None;
        };
        channel = parent;
    }

    let guild = guild_id.to_partial_guild(http).await.ok()?;
    let member = guild_id.current_user_member(http).await.ok()?;
    let have = guild.user_permissions_in(&channel, &member);

    Some(required - have)
}
