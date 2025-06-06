use super::{DiscordApi, DiscordApiError, ParseError};
use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub author_id: u64,
    pub message_id: u64,
    pub message: String,
    pub has_media: bool,
}

impl DiscordApi {
    async fn process_messages(
        response: Response,
        channel_id: u64,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        if wait_for_ratelimit {
            DiscordApi::handle_rate_limit_wait(response.headers()).await;
        }
        let json_data = response.json::<serde_json::Value>().await.map_err(|error| {
            DiscordApiError::ParseResponse(ParseError::DeserializeBodyIntoJson(error))
        })?;
        let mut messages_vec: Vec<Message> = Vec::new();
        if let Some(message_array) = json_data.as_array() {
            for message_object in message_array {
                let message_id = message_object
                    .get("id")
                    .and_then(|id| id.as_str())
                    .and_then(|id| id.parse::<u64>().ok());
                let author_id = message_object
                    .get("author")
                    .and_then(|author| author.as_object())
                    .and_then(|author| author.get("id"))
                    .and_then(|id| id.as_str())
                    .and_then(|id| id.parse::<u64>().ok());
                let content = message_object
                    .get("content")
                    .and_then(|content| content.as_str())
                    .map(|s| s.to_string());
                let has_media = !message_object
                    .get("attachments")
                    .and_then(|att| att.as_array())
                    .map(|arr| arr.is_empty())
                    .unwrap_or(true);
                if let (Some(mid), Some(aid), Some(text)) = (message_id, author_id, content) {
                    messages_vec.push(Message {
                        channel_id,
                        author_id: aid,
                        message_id: mid,
                        message: text,
                        has_media,
                    });
                }
            }
        }
        Ok(messages_vec)
    }

    pub async fn get_channel_msgs_before_msg(
        &self,
        channel_id: u64,
        message_id: u64,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        let url = format!("channels/{}/messages?before={}&limit=100", channel_id, message_id);
        let response = self.request_with_relative_url_and_auth_header(Method::GET, &url).await?;
        let status = response.status().as_u16();
        match status {
            200 => Self::process_messages(response, channel_id, wait_for_ratelimit).await,
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(status, Some(response))),
        }
    }

    pub async fn get_channel_msgs(
        &self,
        channel_id: u64,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        let url = format!("channels/{}/messages?limit=100", channel_id);
        let response = self.request_with_relative_url_and_auth_header(Method::GET, &url).await?;
        let status = response.status().as_u16();
        match status {
            200 => Self::process_messages(response, channel_id, wait_for_ratelimit).await,
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(status, Some(response))),
        }
    }
}