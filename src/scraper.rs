use crate::discord_api::{DiscordApi, DiscordApiError, Message};
use async_recursion::async_recursion;
use serde_json::Value;
use serde_jsonlines::append_json_lines;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io, io::AsyncBufReadExt, io::AsyncWriteExt, io::BufReader, time};

const SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503: u8 = 20;

pub struct Scraper {
    discord_api_client: DiscordApi,
}

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error(transparent)]
    DiscordApiError(DiscordApiError),
}

#[derive(Debug, thiserror::Error)]
pub enum FileConversionError<'a> {
    #[error("Failed to read the contents of the file located at `{0}`, see: {1:#?}")]
    ReadFileContents(&'a Path, io::Error),

    #[error("Failed to write into the file at `{0}`, see: {1:#?}")]
    WriteIntoFile(PathBuf, io::Error),

    #[error("Provided path doesn't have a file name `{0}`")]
    NoFileName(&'a Path),

    #[error("Failed to create an output file at `{0}`, see: {1:#?}")]
    CreateOutputFile(PathBuf, io::Error),

    #[error("Failed to serialize the items from jsonl into json, see: {0:#?}")]
    SerializeJsonlItems(serde_json::Error),
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

    pub async fn scrape_channel(&self, channel_id: u64) -> Result<(PathBuf, u64), ScraperError> {
        let (channel_last_msg_id, channel_name) = self
            .discord_api_client
            .get_last_msg_in_channel(channel_id, true)
            .await
            .map_err(ScraperError::DiscordApiError)?;

        let mut last_message_id = channel_last_msg_id;
        let start_timestamp = chrono::Local::now().timestamp();

        let output_file_name = format!("{channel_name}.jsonl");

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

                    let result = append_json_lines(&output_file_name, messages);

                    if let Err(error) = result {
                        tracing::error!("Failed to write the message batch into the output file, resuming with the next one. See: {error:#?}");
                    };
                }
            };
        }

        let time_it_took_in_secs =
            ((chrono::Local::now().timestamp() - start_timestamp) / 60) as u64;

        let jsonl_file_path = PathBuf::from(output_file_name);

        let jsonl_file_path_slice = jsonl_file_path.as_path();

        match convert_jsonl_file_into_json(&jsonl_file_path).await {
            Ok(json_file_path) => Ok((json_file_path, time_it_took_in_secs)),
            Err(error) => {
                tracing::error!(
                    "Failed to convert the jsonl file at `{jsonl_file_path_slice:?}` into json. See: {error:#?}`"
                );

                Ok((jsonl_file_path, time_it_took_in_secs))
            }
        }
    }
}

pub async fn convert_jsonl_file_into_json(path: &Path) -> Result<PathBuf, FileConversionError> {
    let jsonl_file = File::open(path)
        .await
        .map_err(|error| FileConversionError::ReadFileContents(path, error))?;

    if path.file_name().is_some() {
        let mut json_file_path = path.to_path_buf();

        json_file_path.set_extension("json");

        let mut jsonl_lines = BufReader::new(jsonl_file).lines();

        return match File::create(&json_file_path).await {
            Err(error) => Err(FileConversionError::CreateOutputFile(json_file_path, error)),
            Ok(mut json_file) => {
                let mut json_value_data: Vec<Value> = Vec::new();

                while let Ok(Some(line)) = jsonl_lines.next_line().await {
                    if let Ok(value) = serde_json::from_str::<Value>(&line) {
                        json_value_data.push(value);
                    }
                }

                let json_string = serde_json::to_string(&json_value_data)
                    .map_err(FileConversionError::SerializeJsonlItems)?;

                if let Err(error) = json_file.write_all(json_string.as_bytes()).await {
                    return Err(FileConversionError::WriteIntoFile(json_file_path, error));
                }

                return Ok(json_file_path);
            }
        };
    }

    Err(FileConversionError::NoFileName(path))
}
