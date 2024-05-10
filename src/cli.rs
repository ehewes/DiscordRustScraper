use crate::scraper::Scraper;
use clap::Parser;
use color_eyre::eyre;

#[derive(Parser)]
struct Cli {
    bot_token: String,
    channel_ids: Vec<u64>,
}

pub async fn run() -> eyre::Result<()> {
    let arguments = Cli::parse();
    let scraper = Scraper::new(arguments.bot_token);

    for channel_id in arguments.channel_ids {
        scraper.scrape_channel(channel_id).await?;
    }

    Ok(())
}
