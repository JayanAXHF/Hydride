use std::sync::Arc;

use serenity::{
    all::{
        ChannelId, Color, CreateEmbed, CreateMessage, EventHandler, Member, Mentionable, UserId,
    },
    async_trait,
};
use tracing::{error, info};

use crate::config::{BootstrapConfig, WelcomeMessageConfig};

pub struct Handler {
    cfg: Arc<BootstrapConfig>,
}

impl Handler {
    pub fn new(cfg: Arc<BootstrapConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(
        &self,
        ctx: poise::serenity_prelude::Context,
        new_member: Member,
    ) {
        let new_member_uuid: u64 = new_member.user.id.into();
        if self.cfg.banlist.ids.contains(&new_member_uuid) {
            info!(new_member_uuid, dname = ?new_member.display_name(), "Member on banlist joined");
            let embed = CreateEmbed::new()
                .color(Color::RED)
                .field("Joined UUID", new_member_uuid.to_string(), true)
                .field("Joined Username", new_member.display_name(), true)
                .title("User on Banlist Joined")
                .description("The following user is in the banlist and has joined the server. This might be a ban orpunishment evader. Proceed with caution");
            let message = CreateMessage::new().embed(embed);
            for id in self
                .cfg
                .join_pinglist
                .members
                .iter()
                .map(|id| UserId::from(*id))
            {
                let _ = id.direct_message(&ctx, message.clone()).await;
            }
        }
        let embed = build_welcome_embed(new_member, &self.cfg.welcome_msg);
        let msg = CreateMessage::new().embed(embed);
        let channel = ChannelId::new(self.cfg.welcome_msg.welcome_channel_id);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            error!(?e, "Error occured:");
        }
    }
}

macro_rules! define_channel {
    ($($name: ident => $id: expr);*) => {
            $(
                const $name: serenity::all::ChannelId = serenity::all::ChannelId::new($id);
            )*
    };
}

fn build_welcome_embed(member: Member, welcome_msg_config: &WelcomeMessageConfig) -> CreateEmbed {
    define_channel! {
        RULES_CHANNEL => 1487171920410443836;
        FAQ_CHANNEL => 1494822429711663297;
        REACTION_ROLES_CHANNEL =>1488934150689001723;
        PPG_CHAT_CHANNEL => 1488959010332872815;
        GENERAL_CHANNEL => 1482030003993706653
    };
    let lines = [
        format!(
            "Welcome {} to the discord Plushy's Playground Investigation! <:download_15:1482229485947457680>\n",
            member.mention()
        ),
        "We're a server dedicated to solving the mystery behind the Plushys Playground ARG, and we need your help!\n".to_string(),
        "Make sure to:".to_string(),
        format!("- Follow {}", RULES_CHANNEL.mention()),
        format!("- Read the {} to gain access to {}", FAQ_CHANNEL.mention(), PPG_CHAT_CHANNEL.mention()),
        format!("Customize yourself in {}", REACTION_ROLES_CHANNEL.mention()),
        format!("And when you're done, come join us either in {} or {}! <:Tickles:1482232113578115143>", GENERAL_CHANNEL.mention(), PPG_CHAT_CHANNEL.mention())
    ];

    CreateEmbed::new()
        .title("Welcome to the Server!")
        .description(lines.join("\n"))
        .thumbnail(&welcome_msg_config.main_image_url)
        .image(&welcome_msg_config.footer_image_url)
}
