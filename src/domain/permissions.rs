use poise::serenity_prelude::{GuildId, Member, PartialGuild, Permissions, RoleId, UserId};

use crate::{
    config::RuntimeGuildSettings,
    error::{AppError, AppResult},
};

pub async fn fetch_guild_and_member(
    ctx: &poise::serenity_prelude::Context,
    guild_id: GuildId,
    user_id: UserId,
) -> AppResult<(PartialGuild, Member)> {
    let guild = guild_id
        .to_partial_guild(&ctx.http)
        .await
        .map_err(|source| AppError::Discord {
            source: Box::new(source),
        })?;
    let member = guild_id
        .member(&ctx.http, user_id)
        .await
        .map_err(|source| AppError::Discord {
            source: Box::new(source),
        })?;
    Ok((guild, member))
}

pub fn ensure_moderator_access(
    guild: &PartialGuild,
    member: &Member,
    settings: &RuntimeGuildSettings,
    required_permission: Permissions,
) -> AppResult<()> {
    if guild.owner_id == member.user.id {
        return Ok(());
    }

    let permissions = guild_permissions(guild, member);
    let has_mod_role = member
        .roles
        .iter()
        .any(|role_id| settings.mod_role_ids.contains(&role_id.get()));

    if permissions.administrator() || permissions.contains(required_permission) || has_mod_role {
        Ok(())
    } else {
        Err(AppError::PermissionDenied {
            message: format!(
                "missing {:?} and no configured moderator role is present",
                required_permission
            ),
        })
    }
}

pub async fn ensure_bot_permissions(
    ctx: &poise::serenity_prelude::Context,
    guild: &PartialGuild,
    required_permission: Permissions,
) -> AppResult<Member> {
    let bot_id = ctx.cache.current_user().id;
    let bot_member =
        guild
            .id
            .member(&ctx.http, bot_id)
            .await
            .map_err(|source| AppError::Discord {
                source: Box::new(source),
            })?;
    let permissions = guild_permissions(guild, &bot_member);

    if permissions.administrator() || permissions.contains(required_permission) {
        Ok(bot_member)
    } else {
        Err(AppError::BotPermissionDenied {
            message: format!("bot is missing {:?}", required_permission),
        })
    }
}

pub fn ensure_targetable(guild: &PartialGuild, actor: &Member, target: &Member) -> AppResult<()> {
    if guild.owner_id == target.user.id {
        return Err(AppError::RoleHierarchy {
            message: "guild owner cannot be moderated".into(),
        });
    }

    if actor.user.id == target.user.id {
        return Err(AppError::InvalidInput {
            message: "you cannot target yourself".into(),
        });
    }

    if target.user.bot {
        return Err(AppError::InvalidInput {
            message: "targeting bots is not supported".into(),
        });
    }

    let actor_position = highest_role_position(guild, actor);
    let target_position = highest_role_position(guild, target);

    if target_position >= actor_position && guild.owner_id != actor.user.id {
        return Err(AppError::RoleHierarchy {
            message: "target has an equal or higher top role than the acting moderator".into(),
        });
    }

    Ok(())
}

pub fn ensure_bot_can_target(
    guild: &PartialGuild,
    bot_member: &Member,
    target: &Member,
) -> AppResult<()> {
    let bot_position = highest_role_position(guild, bot_member);
    let target_position = highest_role_position(guild, target);

    if target_position >= bot_position {
        Err(AppError::RoleHierarchy {
            message: "target has an equal or higher top role than the bot".into(),
        })
    } else {
        Ok(())
    }
}

fn guild_permissions(guild: &PartialGuild, member: &Member) -> Permissions {
    if guild.owner_id == member.user.id {
        return Permissions::all();
    }

    let everyone_role = guild.roles.get(&RoleId::new(guild.id.get()));
    let mut permissions = everyone_role
        .map(|role| role.permissions)
        .unwrap_or_default();

    for role_id in &member.roles {
        if let Some(role) = guild.roles.get(role_id) {
            permissions |= role.permissions;
        }
    }

    if permissions.administrator() {
        Permissions::all()
    } else {
        permissions
    }
}

fn highest_role_position(guild: &PartialGuild, member: &Member) -> i64 {
    member
        .roles
        .iter()
        .filter_map(|role_id| {
            guild
                .roles
                .get(role_id)
                .map(|role| i64::from(role.position))
        })
        .max()
        .unwrap_or(0)
}
