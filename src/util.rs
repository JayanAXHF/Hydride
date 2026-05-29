use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::{AppError, AppResult};

pub fn parse_duration(input: &str) -> AppResult<i64> {
    let trimmed = input.trim();
    if trimmed.len() < 2 {
        return Err(AppError::InvalidInput {
            message: "duration must look like 30m, 4h, or 7d".into(),
        });
    }

    let (amount, unit) = trimmed.split_at(trimmed.len() - 1);
    let value: i64 = amount.parse().map_err(|_| AppError::InvalidInput {
        message: format!("could not parse duration amount from `{trimmed}`"),
    })?;

    let seconds = match unit {
        "s" => value,
        "m" => value * 60,
        "h" => value * 60 * 60,
        "d" => value * 60 * 60 * 24,
        _ => {
            return Err(AppError::InvalidInput {
                message: format!("unsupported duration unit `{unit}`; use s, m, h, or d"),
            });
        }
    };

    if seconds <= 0 {
        return Err(AppError::InvalidInput {
            message: "duration must be greater than zero".into(),
        });
    }

    Ok(seconds)
}

pub fn parse_leave_window(input: &str) -> AppResult<(String, Option<i64>, Option<i64>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput {
            message: "leave window must not be empty".into(),
        });
    }

    if let Some((start, end)) = trimmed.split_once("..") {
        let start = parse_rfc3339_timestamp(start.trim())?;
        let end = parse_rfc3339_timestamp(end.trim())?;

        if end <= start {
            return Err(AppError::InvalidInput {
                message: "leave range end must be after the start".into(),
            });
        }

        return Ok((
            format!("{} .. {}", format_timestamp(start), format_timestamp(end)),
            Some(start),
            Some(end),
        ));
    }

    let _duration_seconds = parse_duration(trimmed)?;
    Ok((trimmed.to_string(), None, None))
}

pub fn format_duration(seconds: i64) -> String {
    if seconds % 86_400 == 0 {
        format!("{}d", seconds / 86_400)
    } else if seconds % 3_600 == 0 {
        format!("{}h", seconds / 3_600)
    } else if seconds % 60 == 0 {
        format!("{}m", seconds / 60)
    } else {
        format!("{}s", seconds)
    }
}

pub fn format_timestamp(timestamp: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .and_then(|ts| ts.format(&Rfc3339).ok())
    {
        Some(value) => value,
        None => timestamp.to_string(),
    }
}

fn parse_rfc3339_timestamp(input: &str) -> AppResult<i64> {
    OffsetDateTime::parse(input, &Rfc3339)
        .map(|timestamp| timestamp.unix_timestamp())
        .map_err(|_| AppError::InvalidInput {
            message: format!(
                "could not parse `{input}` as an RFC3339 timestamp; use a value like 2026-05-29T10:00:00Z"
            ),
        })
}

#[cfg(test)]
mod tests {
    use super::{parse_duration, parse_leave_window};

    #[test]
    fn parses_leave_duration_fallback() {
        let (window, starts_at, ends_at) = parse_leave_window("7d").unwrap();
        assert_eq!(window, "7d");
        assert_eq!(starts_at, None);
        assert_eq!(ends_at, None);
        assert_eq!(parse_duration(&window).unwrap(), 604800);
    }

    #[test]
    fn parses_leave_range() {
        let (window, starts_at, ends_at) =
            parse_leave_window("2026-05-29T10:00:00Z..2026-06-05T18:00:00Z").unwrap();

        assert!(window.contains("2026-05-29T10:00:00Z"));
        assert!(window.contains("2026-06-05T18:00:00Z"));
        assert!(starts_at.is_some());
        assert!(ends_at.is_some());
        assert!(ends_at.unwrap() > starts_at.unwrap());
    }
}
