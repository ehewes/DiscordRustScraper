use super::{DiscordApi, DiscordApiError, FoundableStuff, ParseError};
use reqwest::Method;

impl DiscordApi {
    pub async fn get_last_msg_in_channel(
        &self,
        channel_id: u64,
        wait_for_ratelimit: bool,
    ) -> Result<u64, DiscordApiError> {
        let response = self
            .request_with_relative_url_and_auth_header(
                Method::GET,
                &format!("channels/{channel_id}"),
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

                if let Some(channels) = json_data.as_object() {
                    if let Some(last_message_id) = channels
                        .get("last_message_id")
                        .and_then(|id_value| id_value.as_str())
                        .and_then(|id_string| id_string.parse::<u64>().ok())
                    {
                        return Ok(last_message_id);
                    }

                    return Err(DiscordApiError::NotFound(FoundableStuff::Message(None)));
                };

                Err(DiscordApiError::NotFound(FoundableStuff::Channel(Some(
                    channel_id,
                ))))
            }
            _ => Err(DiscordApiError::UnexpectedResponseStatusCode(
                status,
                Some(response),
            )),
        }
    }
}
