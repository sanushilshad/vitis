use crate::configuration::SlackChannel;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlackBlockType {
    Header,
    Section,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlackTextType {
    PlainText,
    Mrkdwn,
}

#[derive(Debug, Serialize)]
pub struct SlackText {
    #[serde(rename = "type")]
    pub r#type: SlackTextType,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct SlackBlock {
    #[serde(rename = "type")]
    pub r#type: SlackBlockType,
    pub text: SlackText,
}

#[derive(Debug, Serialize)]
pub struct SlackNotificationPayload {
    text: String,
    blocks: Vec<SlackBlock>,
}

impl SlackNotificationPayload {
    pub fn new(text: String) -> Self {
        Self {
            text,
            blocks: Vec::new(),
        }
    }

    pub fn add_section(
        mut self,
        text: String,
        block_type: SlackBlockType,
        text_type: SlackTextType,
    ) -> Self {
        let block = SlackBlock {
            r#type: block_type,
            text: SlackText {
                r#type: text_type,
                text,
            },
        };
        self.blocks.push(block);
        self
    }

    pub fn build(self) -> SlackNotificationPayload {
        SlackNotificationPayload {
            text: self.text,
            blocks: self.blocks,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SlackClient {
    http_client: Client,
    base_url: String,
    pub channel: SlackChannel,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SlackChannelType {
    Leave,
}

impl SlackClient {
    #[tracing::instrument]
    pub fn new(base_url: String, timeout: std::time::Duration, channel: SlackChannel) -> Self {
        tracing::info!("Establishing connection to the slack server.");
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            channel,
        }
    }

    #[tracing::instrument(skip(self, payload, channel))]
    pub async fn send_notification(
        &self,
        payload: SlackNotificationPayload,
        channel: &SecretString,
    ) -> Result<(), anyhow::Error> {
        let final_url = format!("{}/{}", &self.base_url, channel.expose_secret());
        let _ = self
            .http_client
            .post(final_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {
                tracing::error!("HTTP request failed: {}", e);
                e
            })?;
        Ok(())
    }
}
