# DiscordRustScraper
[![Rust-Scraper-Bannerwide.png](https://i.postimg.cc/CxSB8GDM/Rust-Scraper-Bannerwide.png)](https://postimg.cc/2V9SRB2g)
<p align="center">
	<img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/discord_rust_scraper?label=crates.io%20downloads" />
</p>

---

## Description
DiscordRustScraper is a powerful Discord data scraper built in Rust, designed to extract and format channel data for further analysis. It efficiently scrapes message history from specified channels and outputs it in a clean JSON format for easy processing. Optional features include the ability to scrape data from personal accounts, create backups of messages, and store data in a SQL database for improved performance and organization.

<details>
  <summary>Table of Contents</summary>

- [About](#Description)
- [Commands \& Usage](#commands--usage)
    - [Scrape](#scrape)
    - [Convert-to-json](#convert-to-json)
    - [personal](#personal-optional)
        - [backup ](#backup-optionalderived-from-personal)
    - [sql](#sql-optional)
        - [Schema](#schema)
</details>

---

## Commands & Usage

#### Scrape
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS]``
- Example : ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234``

#### convert-to-json
- Usage: ``cargo run -- convert-to-json <INPUT_FILE>``
- Example: ``cargo run -- convert-to-json on-topic.jsonl``

#### personal (optional)
The personal argument allows you to scrape channels from your own account, which is useful for gathering data from private channels or servers where you have access. This feature is particularly beneficial for users who want to analyze their own messages or those in private channels. 
<br>
<br>
For those that do not know how to get your personal discord token, a demonstration video can be found [here](https://www.youtube.com/watch?v=LnBnm_tZlyUn).
- Usage : ``cargo run -- scrape --bot_token <ACCOUNT_TOKEN> --channel_ids [CHANNEL_IDS] --personal``
- Example : ``cargo run -- scrape --bot_token "your_account_token" --channel_ids 659069446438125570 806378740917469234 --personal``

##### backup (optional)(derived from personal)
The backup argument is an optional feature that derives from the personal argument. It allows you to create a backup of your personal discord messages,  this feature is particularly useful for people that want to back up their accounts. This does not require specifying channel ids, as it will scrape all channels that you have access to. Additionally, it will store the ids of all your message sessions.
- Usage : ``cargo run -- scrape --bot_token <ACCOUNT_TOKEN> --personal --backup``
- Example : ``cargo run -- scrape --bot_token "your_account_token" --personal --backup``

#### sql (optional)
The SQL argument provides an optional feature that enables the use of a SQL database to store messages instead of the default storage method, by passing through a connection string. This is a more efficient way of storing data compared to JSONs.
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS] --sql <CONNECTION_STRING>``
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