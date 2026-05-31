use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use tokio::time::sleep;
use url::Url;

use crate::config::ResolvedDiscordConfig;
use crate::error::AppError;

const MAX_RETRIES: usize = 3;

#[derive(Debug, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct DiscordRateLimit {
    retry_after: f64,
}

#[derive(Clone)]
pub struct DiscordWebhookClient {
    client: Client,
    config: ResolvedDiscordConfig,
}

impl DiscordWebhookClient {
    pub fn new(config: ResolvedDiscordConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|source| AppError::HttpBuild { source })?;
        Ok(Self { client, config })
    }

    pub async fn sync_messages(&self, chunks: &[String]) -> Result<Vec<String>, AppError> {
        let mut created_ids = Vec::new();
        let mut sent_ids = Vec::new();
        let first_chunk = chunks.first().map(String::as_str).unwrap_or("");

        let root = self
            .edit_or_create_message(&self.config.root_message_id, first_chunk, "root")
            .await?;
        sent_ids.push(root.id.clone());

        let known_overflow = self.config.overflow_message_ids.len();
        for (message_id, chunk) in self
            .config
            .overflow_message_ids
            .iter()
            .zip(chunks.iter().skip(1))
        {
            let message = self
                .edit_or_create_message(message_id, chunk, "overflow")
                .await?;
            sent_ids.push(message.id);
        }

        if chunks.len() > known_overflow + 1 {
            for chunk in chunks.iter().skip(known_overflow + 1) {
                let message = self.create_message(chunk).await?;
                created_ids.push(message.id.clone());
                sent_ids.push(message.id);
            }
        }

        if chunks.len() < known_overflow + 1 {
            for message_id in self
                .config
                .overflow_message_ids
                .iter()
                .skip(chunks.len().saturating_sub(1))
            {
                if let Err(error) = self.edit_message(message_id, "").await {
                    match error {
                        AppError::DiscordApi { status, .. } if status == StatusCode::NOT_FOUND => {
                            tracing::warn!(
                                "Discord overflow message {message_id} no longer exists"
                            );
                        }
                        other => return Err(other),
                    }
                }
            }
        }

        if !created_ids.is_empty() {
            tracing::info!("created overflow message ids: {}", created_ids.join(", "));
        }

        Ok(sent_ids)
    }

    async fn edit_or_create_message(
        &self,
        message_id: &str,
        content: &str,
        kind: &str,
    ) -> Result<DiscordMessage, AppError> {
        match self.edit_message(message_id, content).await {
            Ok(message) => Ok(message),
            Err(AppError::DiscordApi { status, .. }) if status == StatusCode::NOT_FOUND => {
                tracing::warn!(
                    "Discord {kind} message {message_id} was missing; creating a replacement"
                );
                let message = self.create_message(content).await?;
                tracing::info!("created replacement {kind} message id: {}", message.id);
                Ok(message)
            }
            Err(error) => Err(error),
        }
    }

    async fn edit_message(
        &self,
        message_id: &str,
        content: &str,
    ) -> Result<DiscordMessage, AppError> {
        let url = self.message_url(message_id)?;
        self.send_json(Method::PATCH, url, content).await
    }

    async fn create_message(&self, content: &str) -> Result<DiscordMessage, AppError> {
        let url = self.webhook_url(true)?;
        self.send_json(Method::POST, url, content).await
    }

    async fn send_json(
        &self,
        method: Method,
        url: Url,
        content: &str,
    ) -> Result<DiscordMessage, AppError> {
        let payload = serde_json::json!({
            "content": content,
            "allowed_mentions": { "parse": [] }
        });

        for attempt in 0..=MAX_RETRIES {
            let request = self
                .client
                .request(method.clone(), url.clone())
                .json(&payload);
            let response = request
                .send()
                .await
                .map_err(|source| AppError::Http { source })?;
            let status = response.status();
            let body = response
                .text()
                .await
                .map_err(|source| AppError::DiscordResponse { source })?;

            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry = serde_json::from_str::<DiscordRateLimit>(&body)
                    .map_err(|source| AppError::DiscordJson { source })?;
                let delay = Duration::from_secs_f64(retry.retry_after.max(0.0));
                tracing::warn!("Discord rate limited, retrying in {:?}", delay);
                sleep(delay).await;
                if attempt < MAX_RETRIES {
                    continue;
                }
            }

            if !status.is_success() {
                return Err(AppError::DiscordApi { status, body });
            }

            return serde_json::from_str::<DiscordMessage>(&body)
                .map_err(|source| AppError::DiscordJson { source });
        }

        Err(AppError::DiscordApi {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: String::from("rate limit retry budget exhausted"),
        })
    }

    fn message_url(&self, message_id: &str) -> Result<Url, AppError> {
        let mut url = self.webhook_url(false)?;
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| AppError::InvalidWebhook(String::from("invalid webhook path")))?;
            segments.pop_if_empty();
            segments.push("messages").push(message_id);
        }
        Ok(url)
    }

    fn webhook_url(&self, wait: bool) -> Result<Url, AppError> {
        let mut url = self.config.webhook_url.clone();
        url.set_query(None);
        url.set_fragment(None);
        if wait {
            url.query_pairs_mut().append_pair("wait", "true");
        }
        Ok(url)
    }
}
