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
    #[clap(long)]
    sql: Option<String>,
    #[clap(long)]
    personal: bool,
    #[clap(long)]
    backup: bool,
}

pub async fn run() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scrape(args) => {
            let scraper = Scraper::new(args.bot_token, args.personal);
            let save_target = if let Some(database_url) = args.sql {
                SaveTarget::Sql(database_url)
            } else {
                SaveTarget::Jsonl
            };

            if args.personal && args.backup {
                scraper.backup_channels(&save_target).await?;
                tracing::info!("Backup users and messages saved using the provided save target");
            } else {
                for channel_id in args.channel_ids {
                    let (output_path, duration) =
                        scraper.scrape_channel(channel_id, &save_target).await?;
                    if let Some(path) = output_path {
                        tracing::info!(
                            "Successfully scraped channel `{}`, took {}s. Output at `{}`",
                            channel_id,
                            duration,
                            path.display()
                        );
                    } else {
                        tracing::info!(
                            "Successfully scraped channel `{}`, took {}s. Saved to database",
                            channel_id,
                            duration
                        );
                    }
                }
            }
        }
        Command::ConvertToJson(args) => {
            let json_file_path = convert_jsonl_file_into_json(&args.input_file).await?;
            tracing::info!("Converted JSONL to JSON at `{}`", json_file_path.display());
        }
    }

    Ok(())
}