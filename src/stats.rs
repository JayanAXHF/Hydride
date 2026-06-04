use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct ChannelSnapshot {
    pub channel_id: u64,
    pub channel_name: String,
    pub window_days: u32,
    /// One entry per calendar day in the window, ordered oldest->newest.
    pub daily_counts: Vec<u32>,
    /// Raw message lengths (Unicode char count) for sampled messages.
    pub message_lengths: Vec<u32>,
    /// Indexed [0..24] - UTC hour -> message count.
    pub hourly_buckets: [u32; 24],
    /// (user_id, message_count) sorted descending.
    pub top_posters: Vec<(u64, u32)>,
    pub unique_authors: u32,
    pub total_messages: u64,
    pub role_share: Option<RoleMessageShare>,
}

#[derive(Debug, Clone)]
pub struct RoleMessageShare {
    pub role_id: u64,
    pub role_messages: u64,
    pub total_messages: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    // Volume
    pub mean_per_day: f64,
    pub stddev_per_day: f64,
    pub median_per_day: f64,
    pub p5_per_day: f64,
    pub p95_per_day: f64,
    pub min_per_day: u32,
    pub max_per_day: u32,
    pub cv: f64,

    // Message length
    pub mean_length: f64,
    pub stddev_length: f64,
    pub median_length: f64,

    // Activity shape
    pub peak_hour_utc: u8,
    pub peak_day_count: u32,
    pub gini: f64,
    pub trend_slope: f64,
    pub outlier_days: Vec<usize>,
}

pub fn compute(snap: &ChannelSnapshot) -> ChannelStats {
    let daily = to_f64(&snap.daily_counts);
    let lengths = to_f64(&snap.message_lengths);
    let mut sorted_daily = daily.clone();
    sorted_daily.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mut sorted_lengths = lengths.clone();
    sorted_lengths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let mean_per_day = mean(&daily);
    let stddev_per_day = stddev(&daily);
    let median_per_day = percentile(&sorted_daily, 0.5);
    let p5_per_day = percentile(&sorted_daily, 0.05);
    let p95_per_day = percentile(&sorted_daily, 0.95);
    let min_per_day = snap.daily_counts.iter().copied().min().unwrap_or(0);
    let max_per_day = snap.daily_counts.iter().copied().max().unwrap_or(0);

    let mean_length = mean(&lengths);
    let stddev_length = stddev(&lengths);
    let median_length = percentile(&sorted_lengths, 0.5);

    let peak_hour_utc = snap
        .hourly_buckets
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(hour, _)| hour as u8)
        .unwrap_or(0);

    let peak_day_count = max_per_day;
    let gini = gini_coefficient(&daily);
    let trend_slope = linear_slope(&daily);
    let outlier_threshold = mean_per_day + (2.0 * stddev_per_day);
    let outlier_days = snap
        .daily_counts
        .iter()
        .enumerate()
        .filter_map(|(index, &count)| (count as f64 > outlier_threshold).then_some(index))
        .collect();

    let cv = if mean_per_day.abs() < f64::EPSILON {
        0.0
    } else {
        stddev_per_day / mean_per_day
    };

    ChannelStats {
        mean_per_day,
        stddev_per_day,
        median_per_day,
        p5_per_day,
        p95_per_day,
        min_per_day,
        max_per_day,
        cv,
        mean_length,
        stddev_length,
        median_length,
        peak_hour_utc,
        peak_day_count,
        gini,
        trend_slope,
        outlier_days,
    }
}

pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }

    xs.iter().sum::<f64>() / xs.len() as f64
}

pub fn variance(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }

    let mean = mean(xs);
    xs.iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f64>()
        / xs.len() as f64
}

pub fn stddev(xs: &[f64]) -> f64 {
    variance(xs).sqrt()
}

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    let p = p.clamp(0.0, 1.0);
    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = p * (sorted.len() as f64 - 1.0);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        return sorted[lower];
    }

    let fraction = rank - lower as f64;
    sorted[lower] + (sorted[upper] - sorted[lower]) * fraction
}

pub fn gini_coefficient(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }

    let mut sorted = xs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let sum: f64 = sorted.iter().sum();
    if sum.abs() < f64::EPSILON {
        return 0.0;
    }

    let n = sorted.len() as f64;
    let mut weighted_sum = 0.0;
    for (index, value) in sorted.iter().enumerate() {
        weighted_sum += (index as f64 + 1.0) * *value;
    }

    let gini = (2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n;
    gini.clamp(0.0, 1.0)
}

pub fn linear_slope(ys: &[f64]) -> f64 {
    if ys.len() < 2 {
        return 0.0;
    }

    let n = ys.len() as f64;
    let mean_x = (n - 1.0) / 2.0;
    let mean_y = mean(ys);

    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (index, y) in ys.iter().enumerate() {
        let x = index as f64;
        let x_delta = x - mean_x;
        numerator += x_delta * (*y - mean_y);
        denominator += x_delta * x_delta;
    }

    if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

fn to_f64(xs: &[u32]) -> Vec<f64> {
    xs.iter().map(|&value| value as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_empty_is_zero() {
        assert_eq!(percentile(&[], 0.5), 0.0);
    }

    #[test]
    fn gini_uniform_is_zero() {
        let value = gini_coefficient(&[1.0, 1.0, 1.0, 1.0]);
        assert!(value.abs() < 1e-12);
    }

    #[test]
    fn linear_slope_single_item_is_zero() {
        assert_eq!(linear_slope(&[4.0]), 0.0);
    }
}
