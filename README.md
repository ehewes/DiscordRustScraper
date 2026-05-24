# DiscordRustScraper
[![Rust-Scraper-Bannerwide.png](https://i.postimg.cc/CxSB8GDM/Rust-Scraper-Bannerwide.png)](https://postimg.cc/2V9SRB2g)
<p align="center">
	<img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/discord_rust_scraper?label=crates.io%20downloads" />
</p>

---

## Description
DiscordRustScraper is a powerful Discord data scraper built in Rust, designed to extract and format channel data for further analysis. It efficiently pulls message history from specified channels and outputs it in a clean JSON format for easy processing. Optional features include creating backups of messages and storing data in a SQL database for improved performance and organization.

<details>
  <summary>Table of Contents</summary>

- [About](#Description)
- [Commands \& Usage](#commands--usage)
    - [Scrape](#scrape)
    - [Convert-to-json](#convert-to-json)
    - [sql](#sql-optional)
        - [Schema](#schema)
</details>
<br>

**Disclaimer**
<br>
DiscordRustScraper is an open-source tool for ethical use, provided "as is." Users must comply with Discord's terms and laws. Not affiliated with Discord.

---

## Commands & Usage

#### Scrape
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS]``
- Example : ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234``

#### convert-to-json
- Usage: ``cargo run -- convert-to-json <INPUT_FILE>``
- Example: ``cargo run -- convert-to-json on-topic.jsonl``


- `--personal` is now removed due to Discord's Terms of Service. Using user account tokens for automation is against Discord policy and may lead to account bans.
#### sql (optional)
The SQL argument provides an optional feature that enables the use of a SQL database to store messages instead of the default storage method, by passing through a connection string. This is a more efficient way of storing data compared to JSONs.
- Usage : ``cargo run -- scrape --bot_token <BOT_TOKEN> --channel_ids [CHANNEL_IDS] --sql <CONNECTION_STRING>``
- Example : ``cargo run -- scrape --bot_token "your_bot_token" --channel_ids 659069446438125570 806378740917469234 --sql mysql://username:password@127.0.0.1:3306/database``

##### Schema
You'll have to create the database yourself so i've attached the schema below. 
```sql
CREATE TABLE messages (
    channel_id          BIGINT UNSIGNED  NOT NULL,
    channel_name        VARCHAR(100)     NOT NULL DEFAULT '',
    author_id           BIGINT UNSIGNED  NOT NULL,
    author_name         VARCHAR(64)      NOT NULL DEFAULT '',
    message_id          BIGINT UNSIGNED  NOT NULL,
    message             TEXT             NOT NULL,
    has_media           BOOLEAN          NOT NULL,
    timestamp           VARCHAR(64)      NOT NULL DEFAULT '',
    edited_timestamp    VARCHAR(64)      NULL,
    reply_to_message_id BIGINT UNSIGNED  NULL,
    message_type        TINYINT UNSIGNED NOT NULL DEFAULT 0,
    pinned              BOOLEAN          NOT NULL DEFAULT 0,
    attachment_urls     TEXT             NULL,
    embed_count         INT UNSIGNED     NOT NULL DEFAULT 0,
    reactions           TEXT             NULL,
    PRIMARY KEY (message_id)
);
```

To migrate an existing table:
```sql
ALTER TABLE messages
    ADD COLUMN channel_name        VARCHAR(100)     NOT NULL DEFAULT '' AFTER channel_id,
    ADD COLUMN author_name         VARCHAR(64)      NOT NULL DEFAULT '' AFTER author_id,
    ADD COLUMN timestamp           VARCHAR(64)      NOT NULL DEFAULT '' AFTER has_media,
    ADD COLUMN edited_timestamp    VARCHAR(64)      NULL                AFTER timestamp,
    ADD COLUMN reply_to_message_id BIGINT UNSIGNED  NULL                AFTER edited_timestamp,
    ADD COLUMN message_type        TINYINT UNSIGNED NOT NULL DEFAULT 0  AFTER reply_to_message_id,
    ADD COLUMN pinned              BOOLEAN          NOT NULL DEFAULT 0  AFTER message_type,
    ADD COLUMN attachment_urls     TEXT             NULL                AFTER pinned,
    ADD COLUMN embed_count         INT UNSIGNED     NOT NULL DEFAULT 0  AFTER attachment_urls,
    ADD COLUMN reactions           TEXT             NULL                AFTER embed_count;
```
*Inspired by [DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter).*
