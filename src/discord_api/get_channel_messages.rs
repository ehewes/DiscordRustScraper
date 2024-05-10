use super::{DiscordApi, DiscordApiError, ParseError};
use reqwest::Method;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
    pub author_id: u64,
    pub message_id: u64,
    pub message: String,
}

impl DiscordApi {
    pub async fn get_channel_msgs_before_msg(
        &self,
        channel_id: u64,
        message_id: u64,
        wait_for_ratelimit: bool,
    ) -> Result<Vec<Message>, DiscordApiError> {
        let response = self
            .request_with_relative_url_and_auth_header(
                Method::GET,
                &format!("channels/{channel_id}/messages?before={message_id}&limit=100"),
            )
            .await?;

        let status = response.status().as_u16();

        match status {
            200 => {
                if wait_for_ratelimit {
                    DiscordApi::handle_rate_limit_wait(response.headers()).await;
                }

                let json_data = response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|error| {
                        DiscordApiError::ParseResponse(ParseError::DeserializeBodyIntoJson(error))
                    })?;

                let mut messages_vec: Vec<Message> = vec![];

                if let Some(message_array) = json_data.as_array() {
                    message_array.iter().for_each(|message_object| {
                        let message_id = message_object
                            .get("id")
                            .and_then(|message_id| message_id.as_str())
                            .and_then(|message_id| message_id.parse::<u64>().ok());

                        let author_id = message_object
                            .get("author")
                            .and_then(|author| author.as_object())
                            .and_then(|author| author.get("id"))
                            .and_then(|author_id| author_id.as_str())
                            .and_then(|author_id| author_id.parse::<u64>().ok());

                        let string_content = message_object
                            .get("content")
                            .and_then(|content| content.as_str())
                            .map(|content| content.to_string());

                        if let (Some(message_id), Some(author_id), Some(string_content)) =
                            (message_id, author_id, string_content)
                        {
                            let message_struct = Message {
                                author_id,
                                message_id,
                                message: string_content,
                            };

                            messages_vec.push(message_struct);
                        }
                    });
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
