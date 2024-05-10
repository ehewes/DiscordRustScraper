use crate::discord_api::{DiscordApi, DiscordApiError, Message};
use async_recursion::async_recursion;
use serde_jsonlines::append_json_lines;
use tokio::time;

const SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503: u8 = 20;

pub struct Scraper {
    discord_api_client: DiscordApi,
}

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error(transparent)]
    DiscordApiError(DiscordApiError),
}

impl Scraper {
    pub fn new<S: ToString>(bot_token: S) -> Self {
        Self {
            discord_api_client: DiscordApi::new(bot_token),
        }
    }

    #[async_recursion]
    async fn scrape_msgs_before_msg(
        &self,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Vec<Message>, ScraperError> {
        let possible_messages = self
            .discord_api_client
            .get_channel_msgs_before_msg(channel_id, message_id, true)
            .await;

        match possible_messages {
            Err(error) => match error {
                DiscordApiError::UnexpectedResponseStatusCode(status_code, response) => {
                    if status_code == 503 {
                        tracing::warn!("Received HTTP 503 from the Discord API, waiting {SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503} seconds before retrying. See: {response:#?}");

                        time::sleep(time::Duration::from_secs(
                            SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503 as u64,
                        ))
                        .await;

                        return self.scrape_msgs_before_msg(channel_id, message_id).await;
                    }

                    Err(ScraperError::DiscordApiError(
                        DiscordApiError::UnexpectedResponseStatusCode(status_code, response),
                    ))
                }
                _ => Err(ScraperError::DiscordApiError(error)),
            },
            Ok(messages) => Ok(messages),
        }
    }

    pub async fn scrape_channel(&self, channel_id: u64) -> Result<(), ScraperError> {
        let mut last_message_id = self
            .discord_api_client
            .get_last_msg_in_channel(channel_id, true)
            .await
            .map_err(ScraperError::DiscordApiError)?;

        let start_timestamp = chrono::Local::now().timestamp();

        loop {
            match self
                .scrape_msgs_before_msg(channel_id, last_message_id)
                .await
            {
                Err(error) => {
                    tracing::error!(
                        "Failed to scrape the messages that are before the message ID `{last_message_id}`, retrying. See: {error:#?}"
                    );
                }
                Ok(messages) => {
                    if let Some(last_message) = messages.last() {
                        last_message_id = last_message.message_id;
                    } else {
                        tracing::warn!("No messages were found in the last message batch. Assuming that we scraped every message in the channel.");

                        break;
                    }

                    let output_file_name = format!("{channel_id}.jsonl");
                    let result = append_json_lines(output_file_name, messages);

                    if let Err(error) = result {
                        tracing::error!("Failed to write the message batch into the output file, resuming with the next one. See: {error:#?}");
                    };
                }
            };
        }

        tracing::info!(
            "Scraping is over for the channel {}, it took {} minutes.",
            channel_id,
            (chrono::Local::now().timestamp() - start_timestamp) / 60
        );

        Ok(())
    }
}
