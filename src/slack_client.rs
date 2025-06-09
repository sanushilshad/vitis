use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::configuration::SlackChannel;
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SlackBlockType {
    Header,
    Section,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SlackTextType {
    PlainText,
    Mrkdwn,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackText {
    #[serde(rename = "type")]
    text_type: SlackTextType,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackBlock {
    #[serde(rename = "type")]
    block_type: SlackBlockType,
    text: SlackText,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackNotificationPayload {
    text: String,
    blocks: Vec<SlackBlock>,
}

pub struct SlackNotification {
    payload: SlackNotificationPayload,
}

#[derive(Debug)]
pub struct SlackClient {
    http_client: Client,
    base_url: String,
    channel: SlackChannel,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
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

    #[tracing::instrument(skip(self, blocks))]
    pub async fn send_notification(
        &self,
        title: &str,
        blocks: Vec<SlackBlock>,
        channel: SecretString,
    ) -> Result<(), anyhow::Error> {
        let payload = SlackNotificationPayload {
            text: title.to_string(),
            blocks,
        };
        let final_url = format!("{}/{}", &self.base_url, channel.expose_secret());
        let _response = self
            .http_client
            .post(final_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
