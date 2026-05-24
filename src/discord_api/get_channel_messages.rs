use super::{DiscordApi, DiscordApiError, ParseError};
use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub channel_name: String,
    pub author_id: u64,
    pub author_name: String,
    pub message_id: u64,
    pub message: String,
    pub has_media: bool,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
    pub reply_to_message_id: Option<u64>,
    pub message_type: u8,
    pub pinned: bool,
    pub attachment_urls: Vec<String>,
    pub embed_count: u32,
    pub reactions: Vec<Reaction>,
}

impl DiscordApi {
    async fn process_messages(
        response: Response,
        channel_id: u64,
        channel_name: String,
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
                let author_name = message_object
                    .get("author")
                    .and_then(|a| a.as_object())
                    .and_then(|a| {
                        a.get("global_name")
                            .and_then(|n| n.as_str())
                            .filter(|s| !s.is_empty())
                            .or_else(|| a.get("username").and_then(|n| n.as_str()))
                    })
                    .unwrap_or("")
                    .to_string();
                let content = message_object
                    .get("content")
                    .and_then(|content| content.as_str())
                    .map(|s| s.to_string());
                let attachment_urls: Vec<String> = message_object
                    .get("attachments")
                    .and_then(|att| att.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|a| a.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let has_media = !attachment_urls.is_empty();
                let timestamp = message_object
                    .get("timestamp")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let edited_timestamp = message_object
                    .get("edited_timestamp")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());
                let reply_to_message_id = message_object
                    .get("referenced_message")
                    .and_then(|rm| rm.as_object())
                    .and_then(|rm| rm.get("id"))
                    .and_then(|id| id.as_str())
                    .and_then(|id| id.parse::<u64>().ok());
                let message_type = message_object
                    .get("type")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as u8;
                let pinned = message_object
                    .get("pinned")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);
                let embed_count = message_object
                    .get("embeds")
                    .and_then(|e| e.as_array())
                    .map(|arr| arr.len() as u32)
                    .unwrap_or(0);
                let reactions: Vec<Reaction> = message_object
                    .get("reactions")
                    .and_then(|r| r.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|r| {
                                let count = r.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                                let emoji = r
                                    .get("emoji")
                                    .and_then(|e| e.as_object())
                                    .and_then(|e| e.get("name").and_then(|n| n.as_str()))
                                    .unwrap_or("")
                                    .to_string();
                                Reaction { emoji, count }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if let (Some(mid), Some(aid), Some(text)) = (message_id, author_id, content) {
                    messages_vec.push(Message {
                        channel_id,
                        channel_name: channel_name.clone(),
                        author_id: aid,
                        author_name,
                        message_id: mid,
                        message: text,
                        has_media,
                        timestamp,
                        edited_timestamp,
                        reply_to_message_id,
                        message_type,
                        pinned,
                        attachment_urls,
                        embed_count,
                        reactions,
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
        channel_name: String,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        let url = format!("channels/{}/messages?before={}&limit=100", channel_id, message_id);
        let response = self.request_with_relative_url_and_auth_header(Method::GET, &url).await?;
        let status = response.status().as_u16();
        match status {
            200 => Self::process_messages(response, channel_id, channel_name, wait_for_ratelimit).await,
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(status, Some(response))),
        }
    }

    pub async fn get_channel_msgs(
        &self,
        channel_id: u64,
        channel_name: String,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        let url = format!("channels/{}/messages?limit=100", channel_id);
        let response = self.request_with_relative_url_and_auth_header(Method::GET, &url).await?;
        let status = response.status().as_u16();
        match status {
            200 => Self::process_messages(response, channel_id, channel_name, wait_for_ratelimit).await,
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(status, Some(response))),
        }
    }
}