# DiscordRustScraper
[![Rust-Scraper-Bannerwide.png](https://i.postimg.cc/CxSB8GDM/Rust-Scraper-Bannerwide.png)](https://postimg.cc/2V9SRB2g)
<p align="center">
	<img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/discord_rust_scraper?label=crates.io%20downloads" />
</p>

---

## Description
DiscordRustScraper is a powerful Discord data scraper built in Rust, designed to extract and format channel data for further analysis. It efficiently scrapes message history from specified channels and outputs it in a clean JSON format for easy processing.

## Commands & Usage

#### Scrape
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS]``
- Example : ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234``

#### convert-to-json
- Usage: ``cargo run -- convert-to-json <INPUT_FILE>``
- Example: ``cargo run -- convert-to-json on-topic.jsonl``

#### sql (optional)
The SQL argument provides an optional feature that enables the use of a SQL database to store messages instead of the default storage method, by passing through a connection string. This is a more efficient way of storing data compared to JSONs.
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS] --sql mysql://username:password@127.0.0.1:3306/database``
- Example : ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234 --sql mysql://username:password@127.0.0.1:3306/database``

##### Schema
You'll have to create the database yourself so i've attached the schema below. 
```sql
CREATE TABLE messages (
    channel_id BIGINT UNSIGNED NOT NULL,
    author_id BIGINT UNSIGNED NOT NULL,
    message_id BIGINT UNSIGNED NOT NULL,
    message TEXT NOT NULL,
    has_media BOOLEAN NOT NULL,
    PRIMARY KEY (message_id)
);
```