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
