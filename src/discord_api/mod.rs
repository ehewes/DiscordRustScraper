mod get_channel_messages;
mod get_last_message_id_in_channel;

use reqwest::{header, header::HeaderMap, Method, RequestBuilder, Response};

pub use get_channel_messages::Message;

const DISCORD_API_BASE_URL: &str = "https://discord.com/api/v9";

use std::{
    fmt,
    fmt::{Display, Formatter},
};

pub struct DiscordAuth {
    token: String,
}

impl Display for DiscordAuth {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Bot {}", self.token)
    }
}

pub struct DiscordApi {
    reqwest_client: reqwest::Client,
    auth: DiscordAuth,
}

#[derive(Debug, thiserror::Error)]
pub enum DiscordApiError {
    #[error("Couldn't find: {0:#?}")]
    NotFound(FoundableStuff),

    #[error("Failed to send the request, see {0:#?}")]
    SendingRequest(reqwest::Error),

    //#[error("Ratelimited, {0:#?}")]
    //RateLimited(Option<DiscordApiRatelimitInfo>),

    #[error(transparent)]
    ParseResponse(ParseError),

    #[error("Unexpected status code {0}, see: {1:#?}")]
    UnexpectedResponseStatusCode(u16, Option<reqwest::Response>),

    //#[error("Unexpected error: {0}\n\n{1:#?}")]
    //Unexpected(String, Option<reqwest::Error>),
}

#[derive(Debug)]
pub enum FoundableStuff {
    Channel(Option<u64>),
    Message(Option<u64>),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to deserialize the response body into json, see: {0:#?}")]
    DeserializeBodyIntoJson(reqwest::Error),
}

impl DiscordApi {
    pub fn new<S: ToString>(token: S) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            auth: DiscordAuth {
                token: token.to_string(),
            },
        }
    }

    fn build_request_with_auth_header(&self, method: Method, url: &str) -> RequestBuilder {
        self.reqwest_client
            .request(method, url)
            .header(header::AUTHORIZATION, self.auth.to_string())
    }

    async fn request_with_relative_url_and_auth_header(
        &self,
        method: Method,
        relative_url: &str,
    ) -> Result<Response, DiscordApiError> {
        self.build_request_with_auth_header(
            method,
            &format!("{DISCORD_API_BASE_URL}/{relative_url}"),
        )
        .send()
        .await
        .map_err(DiscordApiError::SendingRequest)
    }

    fn get_ratelimit_info_from_response_header_map(
        header_map: &HeaderMap,
    ) -> Option<DiscordApiRatelimitInfo> {
        let ratelimit_remaining = header_map
            .get("X-RateLimit-Remaining")
            .and_then(|string| string.to_str().ok())
            .and_then(|string| string.parse::<u8>().ok());

        let rate_limit_reset_after = header_map
            .get("X-RateLimit-Reset-After")
            .and_then(|string| string.to_str().ok())
            .and_then(|string| string.parse::<f32>().ok());

        if let (Some(ratelimit_remaining), Some(rate_limit_reset_after)) =
            (ratelimit_remaining, rate_limit_reset_after)
        {
            let milliseconds_until_ratelimit_reset =
                (rate_limit_reset_after * 1000_f32).ceil() as u16;

            return Some(DiscordApiRatelimitInfo {
                remaining_limit: ratelimit_remaining,
                milliseconds_until_ratelimit_reset,
            });
        }

        None
    }

    async fn handle_rate_limit_wait(header_map_with_rate_limit_info: &HeaderMap) {
        if let Some(ratelimit_info) =
            DiscordApi::get_ratelimit_info_from_response_header_map(header_map_with_rate_limit_info)
        {
            if ratelimit_info.remaining_limit == 0 {
                tracing::info!(
                    "Waiting {} milliseconds to respect the rate limit.",
                    ratelimit_info.milliseconds_until_ratelimit_reset
                );

                tokio::time::sleep(tokio::time::Duration::from_millis(
                    ratelimit_info.milliseconds_until_ratelimit_reset.into(),
                ))
                .await;
            }
        }
    }
}

#[derive(Debug)]
struct DiscordApiRatelimitInfo {
    remaining_limit: u8,
    milliseconds_until_ratelimit_reset: u16,
}
