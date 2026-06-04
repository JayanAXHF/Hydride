# Channel Statistics Dashboard Spec

## Purpose

Add a per-channel message statistics dashboard to the moderation bot. The output should feel like a compact benchmark report: text-first, monospace-friendly, dense with numbers, and visually structured without becoming noisy.

## Hyperfine Style Reference

I checked `hyperfine` directly. The style cues worth carrying over are:

- A clear top line that identifies the subject of the report.
- A small amount of incidental warning text when needed, never as the main focus.
- Primary metrics presented as short numeric rows.
- A secondary range row using a dedicated separator.
- Tight monospace spacing and minimal ornamentation.

Translate that into the dashboard as:

- one summary title,
- one primary volume row,
- one range row,
- compact spark bars,
- then secondary distribution stats.

## Repository Constraints

This repo is not the same as the original plan assumed.

- The bot is SQLite-only today.
- The logging module lives at `src/domain/logging.rs`, not `src/logging.rs`.
- Command registration lives in `src/commands/mod.rs`.
- Module declarations live in `src/main.rs`.
- The current schema does not store raw channel messages, so a DB-backed dashboard needs either:
  - a new message archive table plus ingestion, or
  - a live Discord history scan fallback.

This spec assumes the long-term solution is a lightweight message archive in SQLite, because the dashboard needs repeatable analytics and should not depend on an ad hoc live scan every time a user runs `/stats`.

## Scope

The feature adds:

- pure statistical computation,
- compact ASCII/Unicode spark bars,
- an embed renderer,
- a slash/prefix command,
- a persisted message archive to support analytics.

## Data Model

Add a new table for message analytics. The bot does not need to store full message content for this feature.

Suggested columns:

- `message_id INTEGER PRIMARY KEY`
- `guild_id INTEGER NOT NULL`
- `channel_id INTEGER NOT NULL`
- `author_id INTEGER NOT NULL`
- `created_at INTEGER NOT NULL` as Unix seconds, matching the rest of the repo
- `content_len INTEGER NOT NULL`

Suggested indexes:

- `(channel_id, created_at DESC)`
- `(channel_id, author_id, created_at DESC)`
- `(guild_id, channel_id, created_at DESC)` if guild-scoped lookups are needed

The ingestion path should be driven from gateway events already enabled by the bot:

- `GUILD_MESSAGES`
- `MESSAGE_CONTENT`

Only store metadata required for analytics. If privacy or retention policy matters later, this table can be trimmed further without changing the dashboard math.

## Statistics Module

Create a pure Rust stats module with no new dependencies.

### Inputs

Define a snapshot struct that holds the values needed to compute the dashboard:

- `channel_id`
- `channel_name`
- `window_days`
- `daily_counts`
- `message_lengths`
- `hourly_buckets`
- `top_posters`
- `unique_authors`
- `total_messages`

### Output

Define a computed stats struct containing:

- volume metrics:
  - mean per day
  - standard deviation per day
  - median per day
  - p5 and p95
  - min and max
  - coefficient of variation
- message length metrics:
  - mean
  - standard deviation
  - median
- activity shape:
  - peak UTC hour
  - peak day count
  - gini coefficient
  - trend slope
  - outlier day indices

### Required Behavior

- `percentile` on an empty slice must return `0.0`.
- `gini_coefficient` on a uniform distribution must evaluate to `0.0` within floating-point tolerance.
- `linear_slope` on a single-element slice must return `0.0`.
- `variance` must be population variance.
- All helpers operate on `&[f64]`; callers convert integer values before computation.

## Spark Bars

Create a small rendering module for compact one-line charts.

### Hourly Spark

- Accept 24 hourly buckets.
- Normalize to the range `0..=7`.
- Use the Unicode block scale `▁▂▃▄▅▆▇█`.
- Zero still renders as `▁`, never as empty space.
- Keep the output short enough for a Discord embed field.

### Daily Spark

- Accept up to 30 daily counts.
- Normalize to the same block scale.
- Mark outlier days with `◆`.
- The spark should remain readable at a glance, even when the distribution is flat.

## Embed Layout

The embed should mirror hyperfine’s information hierarchy: summary first, detail second, visual aids third.

### Title

- `📊 #channel-name · 30d window`

If the channel name is long, prefer truncation over wrapping.

### Field Order

Use Discord embed fields in this order:

1. `Volume (mean ± σ)` inline
2. `Range` inline
3. `Trend` inline
4. `Hourly activity` non-inline
5. `Daily trend (last N days)` non-inline
6. `Message length` inline
7. `Authors` inline
8. `Concentration (Gini)` inline
9. `Percentiles` non-inline
10. `Outlier days` non-inline, only when outliers exist

### Field Formatting

Every numeric value shown inside a field body should be wrapped in a code span so Discord renders it in monospace.

Use plain backticks, for example:

- `` `42.3` ``
- `` `min 12` ``
- `` `p95 79` ``

Avoid exotic punctuation or markdown that would fight the compact benchmark-style layout.

### Color

The embed color should encode activity level based on mean messages per day:

- under `10` msg/day: grey `0x95a5a6`
- `10` to `50` msg/day: blue `0x3498db`
- `50` to `200` msg/day: green `0x2ecc71`
- above `200` msg/day: gold `0xf1c40f`

## Command

Add `/stats channel [channel] [days]`.

Behavior:

1. Resolve the target channel, defaulting to the invocation channel.
2. Clamp the lookback window to the supported range, defaulting to 30 days.
3. Load the analytics snapshot from SQLite.
4. Compute the derived stats in Rust.
5. Send the result as an embed.
6. Defer ephemerally while the snapshot is being built if the command path needs extra time.

The command should be registered in `src/commands/mod.rs` so it is actually exposed by the bot.

## Database Query Shape

Because this repo uses SQLite, the query layer should use SQLite idioms, not PostgreSQL idioms.

Important adjustments from the original plan:

- no `PgPool`
- no `generate_series`
- no `NOW() - $2::interval`

Instead:

- use `SqlitePool`
- use recursive CTEs to generate the day spine for zero-filled daily counts
- use `datetime(..., 'unixepoch')` / `strftime(...)` for time bucketing
- keep statistical math out of SQL

The query layer should return:

- zero-filled daily counts for the full window,
- 24 UTC hourly buckets,
- the top 5 authors by message count,
- up to 500 recent message lengths,
- distinct author count,
- total message count.

## Acceptance Criteria

The feature is done when:

- `/stats` appears in the bot command list,
- the embed layout is compact and benchmark-like,
- the output uses the spark bars and range-style rows described above,
- the stats helpers are unit-testable in isolation,
- the SQLite data path is explicit and does not assume PostgreSQL,
- the repo still passes `cargo check`.

## Notes

The core aesthetic goal is density without clutter.

Hyperfine does this by:

- making the subject obvious,
- keeping the main metrics in one visual block,
- separating range and summary metrics,
- avoiding decorative layout noise.

The dashboard should follow that same pattern.
