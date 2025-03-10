use crate::scraper::{convert_jsonl_file_into_json, Scraper};
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
}

pub async fn run() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scrape(arguments) => {
            let scraper = Scraper::new(arguments.bot_token);

            for channel_id in arguments.channel_ids {
                let (output_file_path, time_it_took_in_secs) =
                    scraper.scrape_channel(channel_id).await?;

                tracing::info!("Successfully scraped the channel `{channel_id}`, it took {time_it_took_in_secs} seconds. Output file is located at `{output_file_path:?}`");
            }
        }
        Command::ConvertToJson(arguments) => {
            let json_file_path = convert_jsonl_file_into_json(&arguments.input_file).await?;

            tracing::info!("Successfully converted the jsonl file into json, it's located at `{json_file_path:?}`");
        }
    };

    Ok(())
}
