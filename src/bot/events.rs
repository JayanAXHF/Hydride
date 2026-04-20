use crate::state::AppState;

pub async fn handle(
    _ctx: &poise::serenity_prelude::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, AppState, anyhow::Error>,
    _data: &AppState,
) -> Result<(), anyhow::Error> {
    if let poise::serenity_prelude::FullEvent::Ready { data_about_bot } = event {
        tracing::info!(user = %data_about_bot.user.tag(), "Discord gateway ready");
    }

    Ok(())
}
