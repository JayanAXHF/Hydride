use std::io::{self, Write};

use owo_colors::{OwoColorize, Stream::Stdout};

use crate::{
    bar,
    stats::{self, ChannelSnapshot},
};

/// Fixed label column width (characters), matching hyperfine's alignment style.
const W: usize = 22;

// ── Public API ────────────────────────────────────────────────────────────────

/// Prints a channel statistics dashboard to `writer`, styled after hyperfine.
///
/// Respects `NO_COLOR` / `FORCE_COLOR` and tty detection via `owo-colors`'
/// `supports-colors` feature. Add to `Cargo.toml`:
///
/// ```toml
/// owo-colors = { version = "4", features = ["supports-colors"] }
/// ```
pub fn print_channel_stats(out: &mut impl Write, snap: &ChannelSnapshot) -> io::Result<()> {
    let stats = stats::compute(snap);

    let mut sorted: Vec<f64> = snap.daily_counts.iter().map(|&x| x as f64).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p25 = stats::percentile(&sorted, 0.25);
    let p75 = stats::percentile(&sorted, 0.75);

    // ── Header ────────────────────────────────────────────────────────────────
    writeln!(out)?;
    writeln!(
        out,
        "  {}  {}",
        format!("#{}", snap.channel_name)
            .bold()
            .if_supports_color(Stdout, |t| t.bold()),
        format!(
            "·  {}d window  ·  {} messages",
            snap.window_days,
            fmt_n(snap.total_messages)
        )
        .if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(out)?;

    // ── Volume ────────────────────────────────────────────────────────────────
    //   Volume (mean ± σ):      42.3 msg/day  ±   8.1    [23 unique authors]
    writeln!(
        out,
        "{}{}  ±  {}    {}",
        lbl("Volume (mean ± σ)"),
        fmt_val(format!("{:>7.1} msg/day", stats.mean_per_day))
            .green()
            .bold(),
        fmt_val(format!("{:>5.1}", stats.stddev_per_day)).if_supports_color(Stdout, |t| t.cyan()),
        format!("[{} unique authors]", snap.unique_authors)
            .if_supports_color(Stdout, |t| t.dimmed()),
    )?;

    //   Range (min … max):          12  …    87  msg/day
    writeln!(
        out,
        "{}{}  …  {}  msg/day",
        lbl("Range (min … max)"),
        format!("{:>7}", stats.min_per_day),
        format!("{:>4}", stats.max_per_day).if_supports_color(Stdout, |t| t.bold()),
    )?;

    //   Trend:               ▲  +1.40 msg/day²
    let (arrow, slope) = trend_strs(stats.trend_slope);
    writeln!(out, "{}{}  {}", lbl("Trend"), arrow, slope)?;

    //   CV (σ/mean):         0.19  ·  low variability
    writeln!(
        out,
        "{}{}",
        lbl("CV (σ/mean)"),
        format!("{:.2}  ·  {}", stats.cv, cv_label(stats.cv))
            .if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(out)?;

    // ── Spark bars ────────────────────────────────────────────────────────────
    writeln!(
        out,
        "  {}  {}",
        "Hourly activity".if_supports_color(Stdout, |t| t.bold()),
        "(UTC 00 → 23)".if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(out, "  {}", bar::hourly_spark(&snap.hourly_buckets))?;
    writeln!(
        out,
        "  {}  {}",
        "Peak:".if_supports_color(Stdout, |t| t.dimmed()),
        format!("{:02}:00 UTC", stats.peak_hour_utc).cyan().bold()
    )?;
    writeln!(out)?;

    writeln!(
        out,
        "  {}  {}{}{}",
        "Daily trend".if_supports_color(Stdout, |t| t.bold()),
        format!("(last {} days  ·  ", snap.window_days).if_supports_color(Stdout, |t| t.dimmed()),
        "◆".if_supports_color(Stdout, |t| t.yellow()),
        " = outlier)".if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(
        out,
        "  {}",
        bar::daily_spark(&snap.daily_counts, &stats.outlier_days)
    )?;
    writeln!(out)?;

    // ── Secondary metrics ─────────────────────────────────────────────────────
    writeln!(
        out,
        "{}avg {}  {}",
        lbl("Message length"),
        format!(
            "{:.0} ± {:.0} chars",
            stats.mean_length, stats.stddev_length
        )
        .if_supports_color(Stdout, |t| t.bold()),
        format!("·  med {:.0}", stats.median_length).if_supports_color(Stdout, |t| t.dimmed()),
    )?;

    writeln!(out, "{}{}", lbl("Authors"), author_line(snap))?;

    if let Some(ref rs) = snap.role_share {
        writeln!(
            out,
            "{}{}  {}",
            lbl("Role share"),
            format!("{:.1}%", rs.percentage).if_supports_color(Stdout, |t| t.cyan()),
            format!(
                "(<@&{}>  ·  {}/{} msgs)",
                rs.role_id, rs.role_messages, rs.total_messages
            )
            .if_supports_color(Stdout, |t| t.dimmed()),
        )?;
    }

    writeln!(
        out,
        "{}Gini {}  {}",
        lbl("Concentration"),
        format!("{:.2}", stats.gini).if_supports_color(Stdout, |t| t.bold()),
        format!("·  {}", gini_label(stats.gini)).if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(out)?;

    // ── Percentile table ──────────────────────────────────────────────────────
    writeln!(
        out,
        "  {}  {}",
        "Percentiles".if_supports_color(Stdout, |t| t.bold()),
        "(msg/day)".if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(
        out,
        "  {}",
        "    p5     p25    p50    p75    p95".if_supports_color(Stdout, |t| t.dimmed()),
    )?;
    writeln!(
        out,
        "  {}  {}  {}  {}  {}",
        format!("{:>5.0}", stats.p5_per_day).if_supports_color(Stdout, |t| t.bold()),
        format!("{:>6.0}", p25),
        format!("{:>6.0}", stats.median_per_day).if_supports_color(Stdout, |t| t.bold()),
        format!("{:>6.0}", p75),
        format!("{:>6.0}", stats.p95_per_day),
    )?;
    writeln!(out)?;

    // ── Outlier warning (mirrors hyperfine's "Warning:" footer) ──────────────
    if !stats.outlier_days.is_empty() {
        let n = stats.outlier_days.len();
        writeln!(
            out,
            "  {}  {} outlier {} detected (>mean + 2σ). Consider investigating spikes \
             in the daily trend above.",
            "Warning:".if_supports_color(Stdout, |t| t.yellow()),
            n,
            if n == 1 { "day" } else { "days" },
        )?;
        writeln!(out)?;
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Left-pads a label to `W` chars, matching hyperfine's two-space indent + colon style.
fn lbl(name: &str) -> String {
    format!("  {:<W$}", format!("{name}:"))
}

/// Wraps a pre-formatted numeric string so `.if_supports_color` can be chained.
fn fmt_val(s: String) -> String {
    s
}

/// Returns colored (arrow, slope) strings for the trend row.
fn trend_strs(slope: f64) -> (String, String) {
    if slope > 0.05 {
        (
            "▲".if_supports_color(Stdout, |t| t.green()).to_string(),
            format!("{:>+6.2} msg/day²", slope)
                .if_supports_color(Stdout, |t| t.green())
                .to_string(),
        )
    } else if slope < -0.05 {
        (
            "▼".if_supports_color(Stdout, |t| t.red()).to_string(),
            format!("{:>+6.2} msg/day²", slope)
                .if_supports_color(Stdout, |t| t.red())
                .to_string(),
        )
    } else {
        (
            "─".if_supports_color(Stdout, |t| t.dimmed()).to_string(),
            format!("{:>+6.2} msg/day²", slope)
                .if_supports_color(Stdout, |t| t.dimmed())
                .to_string(),
        )
    }
}

fn author_line(snap: &ChannelSnapshot) -> String {
    match snap.top_posters.first() {
        Some((id, count)) => format!(
            "{}  {}",
            snap.unique_authors
                .to_string()
                .if_supports_color(Stdout, |t| t.bold()),
            format!("unique  ·  top: @{id} ({count} msgs)")
                .if_supports_color(Stdout, |t| t.dimmed()),
        ),
        None => {
            snap.unique_authors
                .to_string()
                .if_supports_color(Stdout, |t| t.bold())
                .to_string()
                + " unique"
        }
    }
}

fn cv_label(cv: f64) -> &'static str {
    if cv < 0.10 {
        "very stable"
    } else if cv < 0.30 {
        "low variability"
    } else if cv < 0.60 {
        "moderate variability"
    } else {
        "high variability"
    }
}

fn gini_label(g: f64) -> &'static str {
    if g < 0.20 {
        "very uniform"
    } else if g < 0.40 {
        "mostly uniform"
    } else if g < 0.60 {
        "moderate"
    } else if g < 0.80 {
        "concentrated"
    } else {
        "highly concentrated"
    }
}

/// Insert thousands separators: 1234567 → "1,234,567".
fn fmt_n(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_snap() -> ChannelSnapshot {
        ChannelSnapshot {
            channel_id: 123456789,
            channel_name: "general".into(),
            window_days: 30,
            daily_counts: vec![10, 20, 15, 30, 25, 50, 80, 60, 40, 20],
            message_lengths: vec![40, 55, 30, 200, 80, 10, 120, 60],
            hourly_buckets: {
                let mut b = [0u32; 24];
                b[14] = 200;
                b[15] = 180;
                b[13] = 150;
                b[10] = 80;
                b
            },
            top_posters: vec![(111111111, 87), (222222222, 43)],
            unique_authors: 23,
            total_messages: 1247,
            role_share: None,
        }
    }

    #[test]
    fn smoke_no_panic() {
        let mut out = Vec::new();
        print_channel_stats(&mut out, &stub_snap()).unwrap();
        // Strip ANSI codes before asserting on content
        let raw = String::from_utf8(out).unwrap();
        let plain = strip_ansi(&raw);
        assert!(plain.contains("general"));
        assert!(plain.contains("msg/day"));
        assert!(plain.contains("Percentiles"));
    }

    #[test]
    fn fmt_n_inserts_commas() {
        assert_eq!(fmt_n(1_234_567), "1,234,567");
        assert_eq!(fmt_n(999), "999");
        assert_eq!(fmt_n(1_000), "1,000");
    }

    #[test]
    fn label_width_is_consistent() {
        // Every label is exactly 2 (indent) + W chars before the value.
        let rendered = lbl("Volume (mean ± σ)");
        assert_eq!(rendered.chars().count(), 2 + W);
    }

    #[test]
    fn no_warning_when_no_outliers() {
        let mut snap = stub_snap();
        snap.daily_counts = vec![20; 30]; // flat — no outliers
        let mut out = Vec::new();
        print_channel_stats(&mut out, &snap).unwrap();
        assert!(!String::from_utf8(out).unwrap().contains("Warning:"));
    }

    /// Naive ANSI stripping for test assertions.
    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape && c == 'm' {
                in_escape = false;
            } else if !in_escape {
                out.push(c);
            }
        }
        out
    }
}
