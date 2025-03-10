# DiscordRustScraper
[![Rust-Scraper-Bannerwide.png](https://i.postimg.cc/CxSB8GDM/Rust-Scraper-Bannerwide.png)](https://postimg.cc/2V9SRB2g)

## Description
DiscordRustScraper is a powerful Discord data scraper built in Rust, designed to extract and format channel data for further analysis. It efficiently scrapes message history from specified channels and outputs it in a clean JSON format for easy processing.

### Commands & Usage

#### scrape

- Usage: ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS]``
- Example: ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234``

#### convert-to-json

- Usage: oretary-rust-scraper convert-to-json <INPUT_FILE>
- Example: oretary-rust-scraper convert-to-json 999.jsonl


