# Moderation Bot

A Discord moderation bot written in Rust with `poise`, `serenity`, and `sqlx`.

It supports both prefix commands and slash commands, stores moderation cases in SQLite, and sends audit-style moderation logs to a configured channel.

## Features

- Warn, timeout, kick, ban, and unban members
- Purge recent messages from a channel
- Persist moderation cases in SQLite
- View a single case or recent cases for a user
- Per-guild moderation settings stored in the database
- Configurable moderator roles in addition to Discord permissions
- Automatic database migrations on startup
- Slash command registration either globally or to development guilds only

## Tech Stack

- Rust 2024
- `poise` / `serenity` for Discord bot framework and API access
- `tokio` for async runtime
- `sqlx` with SQLite for persistence
- `tracing` for logging

## Configuration

The bot reads configuration from `config.toml` by default.

You can override the config path with:

```bash
MODBOT_CONFIG=/path/to/config.toml cargo run
```

Use `config.example.toml` as a starting point:

```toml
[discord]
token = "DISCORD_BOT_TOKEN"
prefix = "!"
owner_ids = []
dev_guild_ids = [123456789012345678]
register_globally = false

[database]
url = "sqlite://moderation_bot.db"

[logging]
filter = "info,serenity=warn,sqlx=warn"

[moderation]
default_log_channel_id = 123456789012345678
require_reason = true
ephemeral_slash_responses = true
max_case_results = 10
```

### Config Fields

`[discord]`

- `token`: Bot token
- `prefix`: Prefix for text commands
- `owner_ids`: Optional Discord user IDs treated as bot owners by `poise`
- `dev_guild_ids`: Guild IDs used for slash command registration when `register_globally = false`
- `register_globally`: Register application commands globally instead of per dev guild

`[database]`

- `url`: SQLite connection string

`[logging]`

- `filter`: `tracing_subscriber` env filter string

`[moderation]`

- `default_log_channel_id`: Default moderation log channel used if a guild-specific value is not set
- `require_reason`: Require a reason for moderation commands
- `ephemeral_slash_responses`: Send slash command confirmations ephemerally
- `max_case_results`: Maximum number of cases returned by the `cases` command

## Running

1. Copy `config.example.toml` to `config.toml`.
2. Fill in your Discord bot token and guild/channel IDs.
3. Run the bot:

```bash
cargo run
```

On startup, the bot:

- loads the bootstrap config
- initializes tracing
- connects to SQLite
- runs database migrations from `migrations/`
- registers slash commands
- starts the Discord gateway client

## Required Discord Intents

The bot enables these gateway intents:

- non-privileged intents
- `MESSAGE_CONTENT`
- `GUILD_MEMBERS`
- `GUILD_MESSAGES`
- `GUILD_MODERATION`

Make sure the matching privileged intents are enabled in the Discord developer portal if your bot needs them.

## Commands

All commands are available as both prefix commands and slash commands unless noted otherwise.

### Meta

- `ping`
- `help [command]`

### Moderation

- `warn <user> [reason]`
- `timeout <user> <duration> [reason]`
- `kick <user> [reason]`
- `ban <user> [reason]`
- `unban <user> [reason]`
- `purge [user] <count> [reason]`

Duration format for `timeout` supports `s`, `m`, `h`, and `d`, for example `30m`, `4h`, or `7d`.

### Cases

- `case <case_id>`
- `cases <user> [limit]`

### Configuration

- `config view`
- `config set_log_channel <channel>`
- `config clear_log_channel`
- `config set_require_reason <true|false>`
- `config set_ephemeral <true|false>`
- `config add_mod_role <role>`
- `config remove_mod_role <role>`

## Permissions Model

Moderation commands check both Discord permissions and guild-specific moderator roles.

- Guild owners are always allowed
- Administrators are allowed
- Users with the required Discord permission are allowed
- Users with a configured moderator role are allowed

The bot also checks role hierarchy before acting on a target member.

## Data Model

The SQLite schema includes:

- `guild_settings`: per-guild moderation settings
- `guild_mod_roles`: extra roles allowed to moderate
- `moderation_cases`: stored moderation actions
- `case_notes`: note records attached to cases

Each moderation action creates a case record. If audit log delivery succeeds, the audit message channel and message IDs are stored with the case.

## Audit Logging

Moderation actions are posted as embeds to the configured moderation log channel. Embeds include:

- action type
- moderator
- target
- creation time
- reason
- optional duration / expiry
- optional details

## Current Limitations

- `notes_enabled` and `appeals_enabled` are stored in guild settings but do not have full user-facing flows yet.
- `case_notes` support exists in the database and code, but there is no public command wired in for adding notes.
- `purge` with a specific target user is not complete yet; the filtered fetch helper still ends in `todo!()`.

## Development

Useful commands:

```bash
cargo fmt
cargo check
cargo run
```

For local development, keep `register_globally = false` and set `discord.dev_guild_ids` so slash commands update quickly in a test server.
