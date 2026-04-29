# AGENTS.md

## Quick Facts
- Single Rust crate (`Cargo.toml` only; no workspace).
- Main entrypoint is `src/main.rs` -> `src/app.rs` -> `src/bot/framework.rs`.
- Command registration lives in `src/commands/mod.rs`; add new bot commands to `commands::all()` or they will never be exposed.

## Run And Verify
- `cargo check` is the fastest meaningful verification step.
- `cargo test` currently only compiles the crate and runs `0` tests; treat it as a compile/regression check, not behavioral coverage.
- `cargo fmt` is the formatter mentioned by the repo docs; no Clippy, Justfile, Makefile, CI workflow, or pre-commit config is present.

## Config And Runtime
- The bot reads `config.toml` by default; override with `MODBOT_CONFIG=/path/to/config.toml cargo run`.
- `config.toml` and `*.db` files are gitignored local state. Do not commit local bot tokens or SQLite files.
- `src/config/file.rs` enforces: `discord.token` and `discord.prefix` must be non-empty, and you must either set `discord.register_globally = true` or provide at least one `discord.dev_guild_ids` entry.
- For local development, keep `register_globally = false`; slash commands are then registered only into `discord.dev_guild_ids` during startup.

## Database
- SQLite is the only supported database in this repo today (`sqlx` with `sqlite` features only).
- Startup always runs migrations via `sqlx::migrate!("./migrations")`; schema changes belong in `migrations/`, not ad hoc startup code.
- Default database URL is `sqlite://moderation_bot.db`.

## Code Layout
- `src/app.rs`: bootstrap config loading, tracing init, DB connect/migrate, app state construction.
- `src/bot/`: Discord framework setup and event handling.
- `src/commands/`: user-facing command handlers.
- `src/db/`: SQLx pool, records, and repository methods.
- `src/domain/`: moderation actions, permission checks, and audit-log helpers.

## Known Gotcha
- `src/commands/moderation.rs` still contains a `todo!()` in the user-filtered purge fetch path (`get_messages_from_user`), so `purge` with a specific target user is incomplete even though the general purge flow exists.
