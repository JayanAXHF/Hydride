use std::sync::Arc;

use serenity::{
    all::{Color, CreateEmbed, CreateMessage, EventHandler, Member, UserId},
    async_trait,
};
use tracing::info;

use crate::config::BootstrapConfig;

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
    }
}
