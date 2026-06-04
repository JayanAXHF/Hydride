use std::collections::HashSet;

const GLYPHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn hourly_spark(buckets: &[u32; 24]) -> String {
    render_spark(buckets)
}

pub fn daily_spark(counts: &[u32], outlier_indices: &[usize]) -> String {
    let outliers: HashSet<usize> = outlier_indices.iter().copied().collect();
    if counts.is_empty() {
        return String::new();
    }

    let max = counts.iter().copied().max().unwrap_or(0);
    counts
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            if outliers.contains(&index) {
                '◆'
            } else {
                scale(value, max)
            }
        })
        .collect()
}

fn render_spark(counts: &[u32]) -> String {
    let max = counts.iter().copied().max().unwrap_or(0);
    counts
        .iter()
        .copied()
        .map(|value| scale(value, max))
        .collect()
}

fn scale(value: u32, max: u32) -> char {
    if max == 0 {
        return GLYPHS[0];
    }

    let scaled = ((value as f64 / max as f64) * 7.0).round().clamp(0.0, 7.0) as usize;
    GLYPHS[scaled]
}
