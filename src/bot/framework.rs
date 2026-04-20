use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents, GuildId, UserId};

use crate::{commands, state::AppState};

pub async fn run(state: AppState) -> anyhow::Result<()> {
    let token = state.config().discord.token.clone();
    let prefix = state.config().discord.prefix.clone();
    let owners: HashSet<UserId> = state
        .config()
        .discord
        .owner_ids
        .iter()
        .copied()
        .map(UserId::new)
        .collect();

    let setup_state = state.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(crate::bot::events::handle(ctx, event, framework, data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix),
                mention_as_prefix: true,
                ..Default::default()
            },
            owners,
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let state = setup_state.clone();
            Box::pin(async move {
                tracing::info!(bot = %ready.user.tag(), "registering commands");
                register_application_commands(ctx, framework, state.config()).await?;
                Ok(state)
            })
        })
        .build();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MODERATION;

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("failed to create Discord client")?;

    client.start().await.context("Discord client exited")
}

async fn register_application_commands(
    ctx: &poise::serenity_prelude::Context,
    framework: &poise::Framework<AppState, anyhow::Error>,
    config: &Arc<crate::config::BootstrapConfig>,
) -> anyhow::Result<()> {
    if config.discord.register_globally {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    } else {
        for guild_id in &config.discord.dev_guild_ids {
            poise::builtins::register_in_guild(
                ctx,
                &framework.options().commands,
                GuildId::new(*guild_id),
            )
            .await?;
        }
    }

    Ok(())
}
