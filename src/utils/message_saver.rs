use crate::discord_api::Message;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use tokio::io::{AsyncWriteExt, BufWriter}; // Use Tokio's BufWriter and AsyncWriteExt
use tokio::fs::{File, OpenOptions};

pub enum SaveTarget {
    Jsonl,
    Sql(String),
}

#[async_trait]
pub trait MessageSaver {
    async fn save_messages(&mut self, messages: &[Message]) -> Result<()>;
}

pub struct JsonlSaver {
    writer: BufWriter<File>,
}

impl JsonlSaver {
    pub async fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        let writer = BufWriter::new(file);
        Ok(Self { writer })
    }
}

#[async_trait]
impl MessageSaver for JsonlSaver {
    async fn save_messages(&mut self, messages: &[Message]) -> Result<()> {
        for message in messages {
            let json_line = serde_json::to_string(message)? + "\n";
            self.writer.write_all(json_line.as_bytes()).await?;
        }
        self.writer.flush().await?;
        Ok(())
    }
}

pub struct SqlSaver {
    pool: sqlx::MySqlPool,
}

impl SqlSaver {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::MySqlPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl MessageSaver for SqlSaver {
    async fn save_messages(&mut self, messages: &[Message]) -> Result<()> {
        for message in messages {
            sqlx::query(
                "INSERT INTO messages (channel_id, author_id, message_id, message, has_media) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(message.channel_id)
            .bind(message.author_id)
            .bind(message.message_id)
            .bind(&message.message)
            .bind(message.has_media)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}