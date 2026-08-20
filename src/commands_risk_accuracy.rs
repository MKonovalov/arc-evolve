//! Prediction-accuracy stats for the `/risk` subsystem — trend detection,
//! aggregate accuracy statistics, and the accuracy report display.
//! Extracted from commands_risk.rs (Day 127) to keep that module focused
//! on scoring and command handling; commands_risk.rs re-exports everything
//! here so call sites are unchanged.

use crate::commands_risk_snapshots::ValidationEvent;
use crate::format::*;

/// The seven risk signal names, index-aligned with the 7-element weight arrays.
pub(crate) const SIGNAL_NAMES: [&str; 7] = [
    "churn",
    "recency",
    "size",
    "complexity",
    "test_density",
    "coupling",
    "revert_history",
];

// ── Risk prediction accuracy tracking ──

/// Trend direction for accuracy over time.
#[derive(Debug, PartialEq)]
pub(crate) enum AccuracyTrend {
    Improving,
    Declining,
    Stable,
    Insufficient, // not enough data points
}

/// Aggregate accuracy statistics computed from validation history.
pub(crate) struct AccuracyStats {
    pub(crate) total_validations: usize,
    pub(crate) total_hits: usize,
    pub(crate) total_changed: usize,
    pub(crate) overall_hit_rate_pct: f64,
    pub(crate) trend: AccuracyTrend,
    pub(crate) best_day: Option<(u32, f64)>, // (day, accuracy_pct)
    pub(crate) worst_day: Option<(u32, f64)>, // (day, accuracy_pct)
    /// Lift factor: how much more often flagged (predicted-high-risk) files
    /// break vs the baseline over all scored files. `None` when no usable
    /// event (carrying `total_scored`/`scored_broke`) exists yet.
    pub(crate) overall_lift: Option<f64>,
    /// Number of validation events that carry the lift fields
    /// (`total_scored`/`scored_broke`/`predicted_count`) and therefore actually
    /// feed the pooled `overall_lift`. This is the "matched prediction-outcome
    /// pair" count the dream tracks: the lift is only a confident measurement
    /// once ≥5 such pairs accumulate (cold-start threshold).
    pub(crate) lift_events_used: usize,
    /// Per-signal hit counts: `per_signal_hits[i]` = how many hit files had
    /// signal `i` elevated. `Some` when the caller supplied signal detail
    /// (from the parsed snapshot/validation cross-reference); `None` when no
    /// signal data is available. Render (and trust) only behind `pairs >= 5`.
    pub(crate) per_signal_hits: Option<[usize; 7]>,
}

/// Compute trend by comparing the average accuracy of the last N events
/// vs the first N events. Uses min(5, len/2) as window size.
fn compute_accuracy_trend(events: &[ValidationEvent]) -> AccuracyTrend {
    if events.len() < 2 {
        return AccuracyTrend::Insufficient;
    }

    let window = std::cmp::min(5, events.len() / 2).max(1);
    let first_avg: f64 =
        events[..window].iter().map(|e| e.accuracy_pct).sum::<f64>() / window as f64;
    let last_avg: f64 = events[events.len() - window..]
        .iter()
        .map(|e| e.accuracy_pct)
        .sum::<f64>()
        / window as f64;

    let diff = last_avg - first_avg;
    if diff > 5.0 {
        AccuracyTrend::Improving
    } else if diff < -5.0 {
        AccuracyTrend::Declining
    } else {
        AccuracyTrend::Stable
    }
}

/// Compute aggregate accuracy statistics from validation events.
pub(crate) fn compute_accuracy_stats(events: &[ValidationEvent]) -> AccuracyStats {
    if events.is_empty() {
        return AccuracyStats {
            total_validations: 0,
            total_hits: 0,
            total_changed: 0,
            overall_hit_rate_pct: 0.0,
            trend: AccuracyTrend::Insufficient,
            best_day: None,
            worst_day: None,
            overall_lift: None,
            lift_events_used: 0,
            per_signal_hits: None,
        };
    }

    let total_validations = events.len();
    let total_hits: usize = events.iter().map(|e| e.hit_count).sum();
    let total_changed: usize = events.iter().map(|e| e.total_changed).sum();
    let overall_hit_rate_pct = if total_changed > 0 {
        (total_hits as f64 / total_changed as f64) * 100.0
    } else {
        0.0
    };

    // Group by day — average accuracy per day for best/worst
    let mut day_accuracies: std::collections::BTreeMap<u32, Vec<f64>> =
        std::collections::BTreeMap::new();
    for e in events {
        day_accuracies
            .entry(e.day)
            .or_default()
            .push(e.accuracy_pct);
    }

    let mut best_day: Option<(u32, f64)> = None;
    let mut worst_day: Option<(u32, f64)> = None;
    for (&day, accs) in &day_accuracies {
        let avg = accs.iter().sum::<f64>() / accs.len() as f64;
        let avg_rounded = (avg * 10.0).round() / 10.0;
        match best_day {
            None => best_day = Some((day, avg_rounded)),
            Some((_, best_acc)) if avg_rounded > best_acc => best_day = Some((day, avg_rounded)),
            _ => {}
        }
        match worst_day {
            None => worst_day = Some((day, avg_rounded)),
            Some((_, worst_acc)) if avg_rounded < worst_acc => worst_day = Some((day, avg_rounded)),
            _ => {}
        }
    }

    let trend = compute_accuracy_trend(events);

    // Discriminative breakage-rate lift: do flagged (high-risk) files break at
    // a higher rate than the baseline over ALL scored files? Aggregate across
    // events that carry `total_scored`/`scored_broke` (the lift feature);
    // events without them (older format) are skipped without panic.
    //   flagged_rate = sum(hit_count) / sum(predicted_count)
    //   baseline_rate = sum(scored_broke) / sum(total_scored)
    //   lift = flagged_rate / baseline_rate  (guard division by zero → None)
    let mut lift_hit_sum: usize = 0;
    let mut lift_predicted_sum: usize = 0;
    let mut lift_total_scored_sum: usize = 0;
    let mut lift_scored_broke_sum: usize = 0;
    let mut lift_events_used = 0usize;
    for e in events {
        // A usable event needs the flagged population (predicted_count), the
        // whole scored population (total_scored), and the scored-broke count.
        if let (Some(pred), Some(total_scored), Some(scored_broke)) =
            (e.predicted_count, e.total_scored, e.scored_broke)
        {
            lift_hit_sum += e.hit_count;
            lift_predicted_sum += pred;
            lift_total_scored_sum += total_scored;
            lift_scored_broke_sum += scored_broke;
            lift_events_used += 1;
        }
        // else: older event without the lift fields — skip
    }
    let overall_lift = if lift_events_used > 0
        && lift_predicted_sum > 0
        && lift_total_scored_sum > 0
        && lift_scored_broke_sum > 0
    {
        let flagged_rate = lift_hit_sum as f64 / lift_predicted_sum as f64;
        let baseline_rate = lift_scored_broke_sum as f64 / lift_total_scored_sum as f64;
        if baseline_rate > 0.0 {
            Some(flagged_rate / baseline_rate)
        } else {
            None
        }
    } else {
        None
    };

    AccuracyStats {
        total_validations,
        total_hits,
        total_changed,
        overall_hit_rate_pct,
        trend,
        best_day,
        worst_day,
        overall_lift,
        lift_events_used,
        per_signal_hits: None,
    }
}

/// Compute per-signal hit counts from per-hit signal indices.
///
/// `hit_signals` is one entry per hit file, each a list of signal indices that
/// were elevated for that file (the same `DetailedValidationEvent.hit_signals`
/// shape the weight-learning path already parses in `commands_risk.rs`).
/// Returns a 7-element array where `[i]` = number of hit files that had signal
/// `i` elevated.
pub(crate) fn compute_per_signal_hits(hit_signals: &[Vec<usize>]) -> [usize; 7] {
    let mut counts = [0usize; 7];
    for signals in hit_signals {
        for &idx in signals {
            if idx < 7 {
                counts[idx] += 1;
            }
        }
    }
    counts
}

/// Format the accuracy report as a compact box display, followed by a
/// per-signal breakdown once enough matched pairs have accumulated (≥5).
pub(crate) fn format_accuracy_report(stats: &AccuracyStats) -> String {
    if stats.total_validations == 0 {
        return format!(
            "\n{BOLD}{CYAN}  No prediction accuracy data yet.{RESET}\n\n\
             {DIM}  Accuracy tracking starts automatically when watch commands\n\
             {DIM}  detect failures and validate them against risk predictions.\n\n\
             {DIM}  Run {RESET}/risk snapshot{DIM} first, then trigger a watch failure{RESET}\n\
             {DIM}  to begin collecting data.{RESET}\n"
        );
    }

    let hit_rate_rounded = (stats.overall_hit_rate_pct * 10.0).round() / 10.0;
    let trend_str = match stats.trend {
        AccuracyTrend::Improving => format!("{GREEN}↑ Improving{RESET}"),
        AccuracyTrend::Declining => format!("{RED}↓ Declining{RESET}"),
        AccuracyTrend::Stable => format!("{YELLOW}→ Stable{RESET}"),
        AccuracyTrend::Insufficient => format!("{DIM}? Too few data points{RESET}"),
    };

    let best_str = match stats.best_day {
        Some((day, pct)) => format!("Day {day} ({pct:.0}%)"),
        None => "—".to_string(),
    };
    let worst_str = match stats.worst_day {
        Some((day, pct)) => format!("Day {day} ({pct:.0}%)"),
        None => "—".to_string(),
    };

    // Cold-start / milestone legibility for the lift: the dream's active
    // milestone is to accumulate ≥5 matched prediction-outcome pairs (events
    // that actually feed the lift), then judge whether the lift is real. Below
    // that threshold the lift is noise — a single event presented as a firm
    // "2.5×" is exactly the cold-start problem the dream names. `lift_events_used`
    // IS the matched-pair count: each lift-carrying validation event is one
    // matched prediction-outcome pair feeding the pooled lift.
    let pairs = stats.lift_events_used;
    let pairs_str = format!("Pairs: {pairs}/5");

    // Labels are byte-safe (no slicing); width handled by `{:<13}` padding.
    let lift_str = match stats.overall_lift {
        Some(lift) if lift.is_finite() && pairs >= 5 => format!("{lift:.1}×"),
        Some(lift) if lift.is_finite() => {
            // Sub-threshold: label the measurement as provisional, not firm.
            format!("~{lift:.1}× (provisional)")
        }
        _ if pairs > 0 => {
            // Pairs exist but below both lift-availability and threshold.
            format!("— (cold start: {pairs}/5)")
        }
        _ => "—".to_string(),
    };

    let box_str = format!(
        "\n{BOLD}  ╭─ Risk Prediction Accuracy ─╮{RESET}\n\
         {BOLD}  │{RESET} Validations:  {:<13}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} Hit rate:     {:<13}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} Trend:        {:<16}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} Best day:     {:<13}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} Worst day:    {:<13}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} {:<27}{BOLD}│{RESET}\n\
         {BOLD}  │{RESET} Lift:         {:<13}{BOLD}│{RESET}\n\
         {BOLD}  ╰───────────────────────────╯{RESET}\n",
        stats.total_validations,
        format!(
            "{hit_rate_rounded:.0}% ({}/{})",
            stats.total_hits, stats.total_changed
        ),
        trend_str,
        best_str,
        worst_str,
        pairs_str,
        lift_str,
    );

    // Per-signal breakdown (the dream's next measurement step): behind the same
    // `pairs >= 5` cold-start gate as the lift, show which risk signals actually
    // co-occurred with the hits. Only rendered when the caller supplied the
    // signal detail (from the parsed snapshot/validation cross-reference).
    let mut out = box_str;
    if let Some(per_signal_hits) = &stats.per_signal_hits {
        let signal_block = fmt_per_signal_block(per_signal_hits, stats.total_hits, pairs);
        out.push_str(&signal_block);
    }
    out
}

/// Render the compact per-signal accuracy breakdown.
///
/// One line per signal: name and hits-with-signal/total-hits. Gated on
/// `pairs >= 5` so we never over-claim at cold-start N (aligns with the lift's
/// cold-start discipline). `total_hits` is the denominator; a single hit file
/// can have multiple signals elevated, so per-signal counts are not exclusive.
fn fmt_per_signal_block(per_signal_hits: &[usize; 7], total_hits: usize, pairs: usize) -> String {
    if pairs < 5 {
        return String::new();
    }
    let mut out = format!("\n{BOLD}  Per-Signal Accuracy{RESET}\n");
    out.push_str(&format!(
        "  {:<16}{:<12}{}\n",
        "Signal", "In hits", "Share of hits"
    ));
    for i in 0..7 {
        let count = per_signal_hits[i];
        let share = if total_hits > 0 {
            count as f64 / total_hits as f64 * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "  {:<16}{:<12}{share:.0}%\n",
            SIGNAL_NAMES[i],
            format!("{count}/{total_hits}"),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test category 7: Accuracy tracking ──

    #[test]
    fn test_compute_accuracy_stats_empty() {
        let stats = compute_accuracy_stats(&[]);
        assert_eq!(stats.total_validations, 0);
        assert_eq!(stats.trend, AccuracyTrend::Insufficient);
        assert!(stats.best_day.is_none());
        assert!(stats.worst_day.is_none());
    }

    #[test]
    fn test_compute_accuracy_stats_single_entry() {
        let events = vec![ValidationEvent {
            day: 110,
            hit_count: 3,
            total_changed: 5,
            accuracy_pct: 60.0,
            ..Default::default()
        }];
        let stats = compute_accuracy_stats(&events);
        assert_eq!(stats.total_validations, 1);
        assert_eq!(stats.total_hits, 3);
        assert_eq!(stats.total_changed, 5);
        assert!((stats.overall_hit_rate_pct - 60.0).abs() < 0.1);
        assert_eq!(stats.trend, AccuracyTrend::Insufficient);
        assert_eq!(stats.best_day, Some((110, 60.0)));
        assert_eq!(stats.worst_day, Some((110, 60.0)));
    }

    #[test]
    fn test_compute_accuracy_trend_improving() {
        let events = vec![
            ValidationEvent {
                day: 100,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 20.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 101,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 25.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 102,
                hit_count: 2,
                total_changed: 5,
                accuracy_pct: 40.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 103,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 60.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 104,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 80.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 105,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 80.0,
                ..Default::default()
            },
        ];
        let trend = compute_accuracy_trend(&events);
        assert_eq!(trend, AccuracyTrend::Improving);
    }

    #[test]
    fn test_compute_accuracy_trend_declining() {
        let events = vec![
            ValidationEvent {
                day: 100,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 80.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 101,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 75.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 102,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 60.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 103,
                hit_count: 2,
                total_changed: 5,
                accuracy_pct: 40.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 104,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 20.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 105,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 15.0,
                ..Default::default()
            },
        ];
        let trend = compute_accuracy_trend(&events);
        assert_eq!(trend, AccuracyTrend::Declining);
    }

    #[test]
    fn test_compute_accuracy_trend_stable() {
        let events = vec![
            ValidationEvent {
                day: 100,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 60.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 101,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 58.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 102,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 62.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 103,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 59.0,
                ..Default::default()
            },
        ];
        let trend = compute_accuracy_trend(&events);
        assert_eq!(trend, AccuracyTrend::Stable);
    }

    #[test]
    fn test_compute_accuracy_trend_insufficient() {
        let events = vec![ValidationEvent {
            day: 100,
            hit_count: 3,
            total_changed: 5,
            accuracy_pct: 60.0,
            ..Default::default()
        }];
        let trend = compute_accuracy_trend(&events);
        assert_eq!(trend, AccuracyTrend::Insufficient);
    }

    #[test]
    fn test_compute_accuracy_trend_two_events_boundary() {
        // The smallest series that can yield a direction: 2 events. The window
        // must shrink to 1 so the first/last averaging windows are each a
        // single event and must NOT overlap (a >5% swing is a valid signal).
        let events = vec![
            ValidationEvent {
                day: 100,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 20.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 101,
                hit_count: 3,
                total_changed: 5,
                accuracy_pct: 70.0,
                ..Default::default()
            },
        ];
        assert_eq!(compute_accuracy_trend(&events), AccuracyTrend::Improving);
    }

    #[test]
    fn test_compute_accuracy_trend_odd_length_no_overlap() {
        // 5 events -> window = min(5, 2) = 2, first/last windows are disjoint
        // slices ([0..2] and [3..5]). A monotonic rise must read as Improving,
        // which fails if windows overlapped and the "last avg" diluted the rise
        // with early high values.
        let events: Vec<ValidationEvent> = [20.0, 40.0, 60.0, 80.0, 95.0]
            .iter()
            .enumerate()
            .map(|(i, &pct)| ValidationEvent {
                day: 100 + i as u32,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: pct,
                ..Default::default()
            })
            .collect();
        assert_eq!(compute_accuracy_trend(&events), AccuracyTrend::Improving);
    }

    #[test]
    fn test_compute_accuracy_stats_best_worst_day() {
        let events = vec![
            ValidationEvent {
                day: 108,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 20.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 110,
                hit_count: 2,
                total_changed: 5,
                accuracy_pct: 40.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 115,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 80.0,
                ..Default::default()
            },
        ];
        let stats = compute_accuracy_stats(&events);
        assert_eq!(stats.best_day, Some((115, 80.0)));
        assert_eq!(stats.worst_day, Some((108, 20.0)));
    }

    #[test]
    fn test_compute_accuracy_stats_multiple_events_same_day() {
        let events = vec![
            ValidationEvent {
                day: 110,
                hit_count: 1,
                total_changed: 5,
                accuracy_pct: 20.0,
                ..Default::default()
            },
            ValidationEvent {
                day: 110,
                hit_count: 4,
                total_changed: 5,
                accuracy_pct: 80.0,
                ..Default::default()
            },
        ];
        let stats = compute_accuracy_stats(&events);
        // Average for day 110 = (20 + 80) / 2 = 50
        assert_eq!(stats.best_day, Some((110, 50.0)));
        assert_eq!(stats.worst_day, Some((110, 50.0)));
    }

    #[test]
    fn test_compute_accuracy_stats_lift_aggregation_mixed_events() {
        // Mix of events: some carry the lift fields (total_scored/scored_broke/
        // predicted_count), some are old-format events WITHOUT them. The absent-
        // field events must be skipped without panic, and the aggregated lift
        // must be computed only from the fields present.
        let events = vec![
            // Old-format event — no lift fields → must be skipped, no panic.
            ValidationEvent {
                day: 110,
                hit_count: 1,
                total_changed: 3,
                accuracy_pct: 33.3,
                ..Default::default()
            },
            // Lift event 1: 2 hits out of 4 predicted; 3 scored-broke of 7 scored.
            ValidationEvent {
                day: 111,
                hit_count: 2,
                total_changed: 3,
                accuracy_pct: 66.7,
                predicted_count: Some(4),
                total_scored: Some(7),
                scored_broke: Some(3),
            },
            // Lift event 2: 1 hit out of 2 predicted; 2 scored-broke of 5 scored.
            ValidationEvent {
                day: 112,
                hit_count: 1,
                total_changed: 2,
                accuracy_pct: 50.0,
                predicted_count: Some(2),
                total_scored: Some(5),
                scored_broke: Some(2),
            },
            // Old-format event — again no lift fields, must be skipped.
            ValidationEvent {
                day: 113,
                hit_count: 0,
                total_changed: 1,
                accuracy_pct: 0.0,
                ..Default::default()
            },
        ];

        let stats = compute_accuracy_stats(&events);
        // The two old-format events were handled without panic, so they still
        // contribute to the headline aggregates.
        assert_eq!(stats.total_validations, 4);
        assert_eq!(stats.total_hits, 4); // 1 + 2 + 1 + 0
        assert_eq!(stats.total_changed, 9); // 3 + 3 + 2 + 1

        // Pooled lift over lift-carrying events only:
        //   flagged / predicted = (2 + 1) / (4 + 2) = 3/6 = 0.5
        //   baseline = (3 + 2) / (7 + 5) = 5/12 ≈ 0.4167
        //   lift = 0.5 / (5/12) = 1.2
        let expected_lift = (3.0_f64 / 6.0) / (5.0_f64 / 12.0);
        let lift = stats.overall_lift.expect("mixed events produce a lift");
        assert!(
            (lift - expected_lift).abs() < 1e-9,
            "pooled lift = {lift}, expected {expected_lift}"
        );
    }

    #[test]
    fn test_compute_accuracy_stats_lift_no_fields_is_none() {
        // No event carries the lift fields → overall_lift must be None (and the
        // report renders "—"), not an incorrect number.
        let events = vec![
            ValidationEvent {
                day: 110,
                hit_count: 1,
                total_changed: 3,
                accuracy_pct: 33.3,
                ..Default::default()
            },
            ValidationEvent {
                day: 111,
                hit_count: 2,
                total_changed: 2,
                accuracy_pct: 100.0,
                ..Default::default()
            },
        ];
        let stats = compute_accuracy_stats(&events);
        assert!(
            stats.overall_lift.is_none(),
            "no lift fields → unknown lift"
        );

        let report = format_accuracy_report(&stats);
        assert!(
            report.contains("Lift:") && report.contains("—"),
            "report shows the n/a dash when no lift data exists"
        );
    }

    #[test]
    fn test_format_accuracy_report_empty() {
        let stats = compute_accuracy_stats(&[]);
        let report = format_accuracy_report(&stats);
        assert!(report.contains("No prediction accuracy data yet"));
        assert!(report.contains("/risk snapshot"));
    }

    #[test]
    fn test_format_accuracy_report_with_data() {
        let stats = AccuracyStats {
            total_validations: 12,
            total_hits: 7,
            total_changed: 12,
            overall_hit_rate_pct: 58.333,
            overall_lift: Some(2.5),
            lift_events_used: 7,
            trend: AccuracyTrend::Improving,
            best_day: Some((115, 80.0)),
            worst_day: Some((108, 20.0)),
            per_signal_hits: None,
        };
        let report = format_accuracy_report(&stats);
        assert!(report.contains("Risk Prediction Accuracy"));
        assert!(report.contains("12"));
        assert!(report.contains("58%"));
        assert!(report.contains("7/12"));
        assert!(report.contains("Improving"));
        assert!(report.contains("Day 115"));
        assert!(report.contains("Day 108"));
        assert!(report.contains("Pairs: 7/5"));
        assert!(report.contains("2.5×"));
        assert!(!report.contains("provisional"));
    }

    #[test]
    fn test_lift_single_pair_is_provisional() {
        // A single lift-carrying event computes an overall_lift, but the report
        // must mark it provisional (below the dream's ≥5-pair threshold) rather
        // than presenting a 1-event lift as a confident measurement.
        let events = vec![ValidationEvent {
            day: 172,
            hit_count: 2,
            total_changed: 3,
            accuracy_pct: 66.7,
            predicted_count: Some(4),
            total_scored: Some(7),
            scored_broke: Some(3),
        }];
        let stats = compute_accuracy_stats(&events);
        assert!(
            stats.overall_lift.is_some(),
            "single pair still computes lift"
        );
        assert_eq!(stats.lift_events_used, 1);

        let report = format_accuracy_report(&stats);
        assert!(report.contains("Pairs: 1/5"));
        assert!(
            report.contains("provisional"),
            "sub-threshold lift is labeled provisional"
        );
        assert!(
            report.contains("~"),
            "provisional lift is shown as approximate"
        );
    }

    #[test]
    fn test_lift_ge5_pairs_is_confident() {
        // Five or more lift-carrying events produce a confident lift label (no
        // "provisional" qualifier) per the dream's cold-start threshold.
        let events: Vec<ValidationEvent> = (0..5)
            .map(|i| ValidationEvent {
                day: 170 + i as u32,
                hit_count: 2,
                total_changed: 3,
                accuracy_pct: 66.7,
                predicted_count: Some(4),
                total_scored: Some(7),
                scored_broke: Some(3),
            })
            .collect();
        let stats = compute_accuracy_stats(&events);
        assert_eq!(stats.lift_events_used, 5);
        assert!(stats.overall_lift.is_some());

        let report = format_accuracy_report(&stats);
        assert!(report.contains("Pairs: 5/5"));
        assert!(report.contains("×"), "confident lift renders the number");
        assert!(!report.contains("provisional"));
    }

    #[test]
    fn test_lift_events_used_counts_only_lift_carrying_events() {
        // `lift_events_used` counts only events that carry the lift fields;
        // old-format events without them contribute to total_validations but
        // NOT to the matched-pair count feeding the lift.
        let events = vec![
            // Old-format — no lift fields.
            ValidationEvent {
                day: 110,
                hit_count: 1,
                total_changed: 3,
                accuracy_pct: 33.3,
                ..Default::default()
            },
            // Lift-carrying event 1.
            ValidationEvent {
                day: 111,
                hit_count: 2,
                total_changed: 3,
                accuracy_pct: 66.7,
                predicted_count: Some(4),
                total_scored: Some(7),
                scored_broke: Some(3),
            },
            // Lift-carrying event 2.
            ValidationEvent {
                day: 112,
                hit_count: 1,
                total_changed: 3,
                accuracy_pct: 33.3,
                predicted_count: Some(2),
                total_scored: Some(5),
                scored_broke: Some(2),
            },
        ];
        let stats = compute_accuracy_stats(&events);
        assert_eq!(stats.total_validations, 3);
        assert_eq!(
            stats.lift_events_used, 2,
            "only the two lift-carrying events feed the lift"
        );

        let report = format_accuracy_report(&stats);
        assert!(
            report.contains("Pairs: 2/5") && report.contains("provisional"),
            "below threshold → provisional label"
        );
    }

    #[test]
    fn test_lift_none_but_pairs_present_shows_cold_start() {
        // When overall_lift is None (no usable lift) but lift-carrying pairs
        // exist, the report shows a cold-start progress line rather than a bare
        // "—" for the lift.
        let events = vec![
            // Lift-carrying event with scored_broke = 0 → lift resolves to None.
            ValidationEvent {
                day: 172,
                hit_count: 0,
                total_changed: 3,
                accuracy_pct: 0.0,
                predicted_count: Some(4),
                total_scored: Some(7),
                scored_broke: Some(0),
            },
        ];
        let stats = compute_accuracy_stats(&events);
        assert!(stats.overall_lift.is_none());
        assert_eq!(stats.lift_events_used, 1);

        let report = format_accuracy_report(&stats);
        assert!(report.contains("Pairs: 1/5"));
        assert!(
            report.contains("cold start") && report.contains("1/5"),
            "None lift with pairs shows the cold-start progress"
        );
    }

    #[test]
    fn test_compute_per_signal_hits_counts_elevated_signals() {
        // One hit file per Vec<usize>; a file can have multiple elevated signals.
        // Here 5 hit files, four carrying real signal detail:
        //   file A → signals [0 (churn), 3 (complexity)]
        //   file B → signals [0 (churn)]
        //   file C → signals [1 (recency)]
        //   file D → signals [4 (test_density)]
        //   file E → signals [99]  (out-of-range index must be ignored, no panic)
        let hit_signals: Vec<Vec<usize>> = vec![
            vec![0, 3],
            vec![0],
            vec![1],
            vec![4],
            vec![99], // out of range → must be ignored (no panic)
        ];
        let counts = compute_per_signal_hits(&hit_signals);
        assert_eq!(counts[0], 2, "churn appears in A and B");
        assert_eq!(counts[1], 1, "recency appears in C");
        assert_eq!(counts[2], 0, "size not elevated in any hit");
        assert_eq!(counts[3], 1, "complexity appears in A");
        assert_eq!(counts[4], 1, "test_density appears in D");
        assert_eq!(counts[5], 0, "coupling not elevated");
        assert_eq!(counts[6], 0, "revert_history not elevated");
    }
}
