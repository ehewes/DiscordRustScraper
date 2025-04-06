use super::{DiscordApi, DiscordApiError, ParseError};
use reqwest::{Method};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub author_id: u64,
    pub message_id: u64,
    pub message: String,
    pub has_media: bool,
}

impl DiscordApi {
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
            200 => {
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
                        let string_content = message_object
                            .get("content")
                            .and_then(|content| content.as_str())
                            .map(|content| content.to_string());
                        let has_media = !message_object
                            .get("attachments")
                            .and_then(|attach| attach.as_array())
                            .map(|a| a.is_empty())
                            .unwrap_or(true);

                        if let (Some(message_id), Some(author_id), Some(string_content)) =
                            (message_id, author_id, string_content)
                        {
                            let message_struct = Message {
                                channel_id,
                                author_id,
                                message_id,
                                message: string_content,
                                has_media,
                            };
                            messages_vec.push(message_struct);
                        }
                    }
                }
                Ok(messages_vec)
            }
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(
                status,
                Some(response),
            )),
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
            200 => {
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
                        let string_content = message_object
                            .get("content")
                            .and_then(|content| content.as_str())
                            .map(|content| content.to_string());
                        let has_media = !message_object
                            .get("attachments")
                            .and_then(|attach| attach.as_array())
                            .map(|a| a.is_empty())
                            .unwrap_or(true);

                        if let (Some(message_id), Some(author_id), Some(string_content)) =
                            (message_id, author_id, string_content)
                        {
                            let message_struct = Message {
                                channel_id,
                                author_id,
                                message_id,
                                message: string_content,
                                has_media,
                            };
                            messages_vec.push(message_struct);
                        }
                    }
                }
                Ok(messages_vec)
            }
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(
                status,
                Some(response),
            )),
        }
    }
}