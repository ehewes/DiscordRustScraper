use crate::scraper::{convert_jsonl_file_into_json, Scraper};
use crate::utils::message_saver::SaveTarget;
use clap::Parser;
use color_eyre::eyre;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    ConvertToJson(ConvertToJson),
    Scrape(Scrape),
}

#[derive(Parser)]
struct ConvertToJson {
    input_file: PathBuf,
}

#[derive(Parser)]
struct Scrape {
    #[clap(long = "bot_token")]
    bot_token: String,
    #[clap(long = "channel_ids", value_parser, num_args = 1..)]
    channel_ids: Vec<u64>,
    #[clap(long, value_parser)]
    sql: Option<String>, // Database URL for SQL saving
}

pub async fn run() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scrape(arguments) => {
            let scraper = Scraper::new(arguments.bot_token);
            let save_target = if let Some(database_url) = arguments.sql {
                SaveTarget::Sql(database_url)
            } else {
                SaveTarget::Jsonl
            };

            for channel_id in arguments.channel_ids {
                let (output_path, time_it_took_in_secs) = scraper
                    .scrape_channel(channel_id, &save_target)
                    .await?;
                if let Some(path) = output_path {
                    tracing::info!(
                        "Successfully scraped channel `{channel_id}`, took {time_it_took_in_secs}s. Output at `{path:?}`"
                    );
                } else {
                    tracing::info!(
                        "Successfully scraped channel `{channel_id}`, took {time_it_took_in_secs}s. Saved to database"
                    );
                }
            }
        }
        Command::ConvertToJson(arguments) => {
            let json_file_path = convert_jsonl_file_into_json(&arguments.input_file).await?;
            tracing::info!("Converted JSONL to JSON at `{json_file_path:?}`");
        }
    }

    Ok(())
}