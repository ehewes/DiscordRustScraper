use crate::discord_api::{DiscordApi, DiscordApiError};
use crate::utils::message_saver::{JsonlSaver, MessageSaver, SaveTarget, SqlSaver};
use async_recursion::async_recursion;
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    time,
};

const SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503: u8 = 20;

pub struct Scraper {
    discord_api_client: DiscordApi,
}

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error(transparent)]
    DiscordApiError(DiscordApiError),
    #[error("Failed to save messages: {0}")]
    SaveError(#[from] color_eyre::eyre::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum FileConversionError {
    #[error("Failed to read the contents of the file located at `{0}`, see: {1:#?}")]
    ReadFileContents(PathBuf, io::Error),
    #[error("Failed to write into the file at `{0}`, see: {1:#?}")]
    WriteIntoFile(PathBuf, io::Error),
    #[error(transparent)]
    InvalidPath(InvalidPathError),
    #[error("Failed to create an output file at `{0}`, see: {1:#?}")]
    CreateOutputFile(PathBuf, io::Error),
    #[error("Failed to serialize the items from jsonl into json, see: {0:#?}")]
    SerializeJsonlItems(serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidPathError {
    #[error("Provided path doesn\\'t have a file stem `{0}`")]
    NoFileStem(PathBuf),
    #[error("Provided path doesn\\'t have a parent directory `{0}`")]
    NoParentDir(PathBuf),
}

impl Scraper {
    pub fn new<S: ToString>(bot_token: S, personal: bool) -> Self {
        Self {
            discord_api_client: DiscordApi::new(bot_token, personal),
        }
    }
    #[async_recursion]
    async fn scrape_msgs_before_msg(
        &self,
        channel_id: u64,
        message_id: u64,
        use_personal: bool,
    ) -> Result<Vec<crate::discord_api::Message>, ScraperError> {
        let possible_messages = if message_id == 0 {
            self.discord_api_client.get_channel_msgs(channel_id, use_personal).await
        } else {
            self.discord_api_client
                .get_channel_msgs_before_msg(channel_id, message_id, use_personal)
                .await
        };
        match possible_messages {
            Err(error) => match error {
                DiscordApiError::UnexpectedResponseStatusCode(status_code, response) => {
                    if status_code == 503 {
                        tracing::warn!(
                            "Received HTTP 503 from the Discord API, waiting {} seconds before retrying. See: {:#?}",
                            SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503,
                            response
                        );
                        time::sleep(time::Duration::from_secs(SECONDS_TO_WAIT_IN_CASE_OF_HTTP_503 as u64))
                            .await;
                        return self.scrape_msgs_before_msg(channel_id, message_id, use_personal).await;
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

    pub async fn scrape_channel(
        &self,
        channel_id: u64,
        save_target: &SaveTarget,
    ) -> Result<(Option<PathBuf>, u64), ScraperError> {
        let (channel_last_msg_id, channel_name) =
            match self.discord_api_client.get_last_msg_in_channel(channel_id, true).await {
                Ok(data) => data,
                Err(e) => {
                    if e.to_string().contains("ChannelName") {
                        tracing::warn!(
                            "Channel name not found for channel {}: falling back to DM mode.",
                            channel_id
                        );
                        let channel_last_msg_id = match self.discord_api_client.get_last_msg_in_channel(channel_id, false).await {
                            Ok((id, _)) => id,
                            Err(_) => 0,
                        };
                        (channel_last_msg_id, format!("dm_{}", channel_id))
                    } else {
                        return Err(ScraperError::DiscordApiError(e));
                    }
                }
            };

        let start_timestamp = chrono::Local::now().timestamp();

        let mut saver: Box<dyn MessageSaver + Send + Sync> = match save_target {
            SaveTarget::Jsonl => {
                std::fs::create_dir_all("storage").unwrap();
                let output_file_name = format!("storage/{}.jsonl", channel_name);
                Box::new(JsonlSaver::new(&output_file_name).await?)
            }
            SaveTarget::Sql(database_url) => Box::new(SqlSaver::new(database_url).await?),
        };

        let mut last_message_id = channel_last_msg_id;
        let format_request = !channel_name.starts_with("dm_");

        loop {
            let messages = self
                .scrape_msgs_before_msg(channel_id, last_message_id, format_request)
                .await?;
            if let Some(last_message) = messages.last() {
                last_message_id = last_message.message_id;
            } else {
                tracing::info!("No more messages to scrape.");
                break;
            }
            if let Err(error) = saver.save_messages(&messages).await {
                tracing::error!("Failed to save message batch: {:#?}", error);
            }
        }

        let time_it_took_in_secs =
            ((chrono::Local::now().timestamp() - start_timestamp) / 60) as u64;
        let output_path = if let SaveTarget::Jsonl = save_target {
            Some(PathBuf::from(format!("{}.jsonl", channel_name)))
        } else {
            None
        };

        Ok((output_path, time_it_took_in_secs))
    }
}

pub async fn convert_jsonl_file_into_json(path: &Path) -> Result<PathBuf, FileConversionError> {
    let jsonl_file_path_buf = path.to_path_buf();
    let jsonl_file = File::open(path).await.map_err(|error| {
        FileConversionError::ReadFileContents(jsonl_file_path_buf.clone(), error)
    })?;

    if let Some(jsonl_file_stem) = path.file_stem() {
        if let Some(dir_path) = jsonl_file_path_buf.parent() {
            let jsonl_file_stem_string = jsonl_file_stem.to_string_lossy();
            let json_file_name = format!("{}.json", jsonl_file_stem_string);
            let mut json_file_path = dir_path.to_path_buf();

            json_file_path.push(json_file_name);

            let mut jsonl_lines = BufReader::new(jsonl_file).lines();
            let mut json_file = File::create(json_file_path.clone())
                .await
                .map_err(|error| {
                    FileConversionError::CreateOutputFile(json_file_path.clone(), error)
                })?;

            let mut json_value_data: Vec<Value> = Vec::new();

            while let Ok(Some(line)) = jsonl_lines.next_line().await {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    json_value_data.push(value);
                }
            }

            let json_string = serde_json::to_string_pretty(&json_value_data)
                .map_err(FileConversionError::SerializeJsonlItems)?;

            json_file
                .write_all(json_string.as_bytes())
                .await
                .map_err(|error| {
                    FileConversionError::WriteIntoFile(json_file_path.clone(), error)
                })?;

            Ok(json_file_path)
        } else {
            Err(FileConversionError::InvalidPath(
                InvalidPathError::NoParentDir(jsonl_file_path_buf),
            ))
        }
    } else {
        Err(FileConversionError::InvalidPath(
            InvalidPathError::NoFileStem(jsonl_file_path_buf),
        ))
    }
}