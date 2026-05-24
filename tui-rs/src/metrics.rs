use std::time::Duration;

use std::collections::HashMap;

use crate::system::{ProcessGroup, SystemState};

pub(crate) const HISTORY_CAPACITY: usize = 72;

pub(crate) struct ScaledHistory {
    pub(crate) values: Vec<u64>,
    pub(crate) min: u64,
    pub(crate) max: u64,
}

pub(crate) fn push_history(history: &mut Vec<u64>, value: u64) {
    history.push(value);
    if history.len() > HISTORY_CAPACITY {
        history.remove(0);
    }
}

pub(crate) fn percent(value: f64, total: f64) -> u16 {
    if total <= 0.0 {
        return 0;
    }
    ((value / total) * 100.0).clamp(0.0, 100.0).round() as u16
}

pub(crate) fn percent_from_f32(value: f32) -> u16 {
    value.clamp(0.0, 100.0).round() as u16
}

pub(crate) fn sorted_processes(state: &SystemState) -> Vec<(&String, &ProcessGroup)> {
    sorted_process_groups(&state.processes)
}

pub(crate) fn sorted_filtered_observed_processes<'a>(
    state: &'a SystemState,
    filter: &str,
) -> Vec<(&'a String, &'a ProcessGroup)> {
    sorted_filtered_process_groups(&state.observed_processes, filter)
}

pub(crate) fn sorted_filtered_hidden_processes<'a>(
    state: &'a SystemState,
    filter: &str,
) -> Vec<(&'a String, &'a ProcessGroup)> {
    sorted_filtered_process_groups(&state.hidden_processes, filter)
}

fn sorted_process_groups(groups: &HashMap<String, ProcessGroup>) -> Vec<(&String, &ProcessGroup)> {
    let mut sorted: Vec<_> = groups.iter().collect();
    sorted.sort_by(|a, b| b.1.memory_mb.total_cmp(&a.1.memory_mb));
    sorted
}

fn sorted_filtered_process_groups<'a>(
    groups: &'a HashMap<String, ProcessGroup>,
    filter: &str,
) -> Vec<(&'a String, &'a ProcessGroup)> {
    let filter = filter.trim().to_ascii_lowercase();
    if filter.is_empty() {
        return sorted_process_groups(groups);
    }

    let mut sorted: Vec<_> = groups
        .iter()
        .filter(|(name, group)| process_matches_filter(name, group, &filter))
        .collect();
    sorted.sort_by(|a, b| b.1.memory_mb.total_cmp(&a.1.memory_mb));
    sorted
}

fn process_matches_filter(name: &str, group: &ProcessGroup, filter: &str) -> bool {
    name.to_ascii_lowercase().contains(filter)
        || group
            .exe_path
            .as_deref()
            .is_some_and(|path| path.to_ascii_lowercase().contains(filter))
}

pub(crate) fn readiness_score(state: &SystemState) -> u16 {
    let waste_penalty = (state.total_waste_mb / 140.0).min(40.0);
    let ram_penalty = (100.0 - f64::from(state.ram_free_pct())) / 4.0;
    let cpu_penalty = f64::from(state.cpu_usage) / 8.0;
    let desktop_penalty = if state.explorer_on { 8.0 } else { 0.0 };
    let plan_bonus = if state.power_plan == "Alto Rendimiento" {
        7.0
    } else {
        0.0
    };

    (100.0 - waste_penalty - ram_penalty - cpu_penalty - desktop_penalty + plan_bonus)
        .clamp(0.0, 100.0)
        .round() as u16
}

pub(crate) fn truncate(value: &str, max_chars: usize) -> String {
    let mut result = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
}

pub(crate) fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3_600;
    let minutes = (total_secs % 3_600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub(crate) fn scaled_history(history: &[u64]) -> ScaledHistory {
    let Some(min) = history.iter().min().copied() else {
        return ScaledHistory {
            values: Vec::new(),
            min: 0,
            max: 0,
        };
    };
    let max = history.iter().max().copied().unwrap_or(min);

    if min == max {
        return ScaledHistory {
            values: vec![50; history.len()],
            min,
            max,
        };
    }

    let span = max - min;
    let values = history
        .iter()
        .map(|value| (((value - min) * 92) / span) + 8)
        .collect();

    ScaledHistory { values, min, max }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_history_should_show_flat_values_at_mid_height() {
        let scaled = scaled_history(&[83, 83, 83]);

        assert_eq!(scaled.values, vec![50, 50, 50]);
    }

    #[test]
    fn scaled_history_should_expand_small_deltas() {
        let scaled = scaled_history(&[80, 81, 82]);

        assert_eq!(scaled.values, vec![8, 54, 100]);
    }

    #[test]
    fn sorted_filtered_process_groups_should_match_exe_path() {
        let mut groups = std::collections::HashMap::new();
        groups.insert(
            "helper.exe".to_string(),
            ProcessGroup {
                count: 1,
                memory_mb: 42.0,
                exe_path: Some("C:\\Program Files\\Dropbox\\helper.exe".to_string()),
            },
        );

        let sorted = sorted_filtered_process_groups(&groups, "dropbox");

        assert_eq!(sorted.len(), 1);
    }
}
