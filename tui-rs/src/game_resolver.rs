use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::presentmon::FrameMetrics;
use crate::steam::SteamGame;
use crate::system::{ProcessGroup, SystemState};

const REQUIRED_FRAME_SCORE: i32 = 120;
const REQUIRED_DIRECT_SCORE: i32 = 110;
const REQUIRED_FRAME_SAMPLES: usize = 3;

const LAUNCHER_PROCESS_NAMES: &[&str] = &[
    "2klauncher.exe",
    "bethesdanetlauncher.exe",
    "eabackgroundservice.exe",
    "eadesktop.exe",
    "ealauncher.exe",
    "origin.exe",
    "redlauncher.exe",
    "rockstarlauncher.exe",
    "rockstarservice.exe",
    "socialclubhelper.exe",
    "ubisoftconnect.exe",
    "upc.exe",
];

const NOISE_PROCESS_NAMES: &[&str] = &[
    "chaosgamemode.exe",
    "chrome.exe",
    "cmd.exe",
    "codex.exe",
    "crashhandler.exe",
    "crashpad_handler.exe",
    "discord.exe",
    "dropbox.exe",
    "explorer.exe",
    "gameoverlayui.exe",
    "msedge.exe",
    "msedgewebview2.exe",
    "obs64.exe",
    "onedrive.exe",
    "opencode.exe",
    "overwolf.exe",
    "powershell.exe",
    "presentmon.exe",
    "pwsh.exe",
    "amdrssrv.exe",
    "radeonsoftware.exe",
    "nvcontainer.exe",
    "steam.exe",
    "steamerrorreporter.exe",
    "steamservice.exe",
    "steamwebhelper.exe",
    "windowsterminal.exe",
];

const GENERIC_HELPER_TERMS: &[&str] = &[
    "bootstrap",
    "config",
    "crash",
    "helper",
    "installer",
    "launcher",
    "patcher",
    "setup",
    "unins",
    "updater",
];

#[derive(Default)]
pub(crate) struct GameProcessResolver {
    active_app_id: Option<String>,
    baseline_processes: HashSet<String>,
    blocked_processes: HashSet<String>,
    frame_counts: HashMap<String, usize>,
    best: Option<ResolvedCandidate>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedCandidate {
    pub(crate) process_name: String,
    pub(crate) score: i32,
}

impl GameProcessResolver {
    pub(crate) fn reset(&mut self) {
        self.active_app_id = None;
        self.baseline_processes.clear();
        self.blocked_processes.clear();
        self.frame_counts.clear();
        self.best = None;
    }

    pub(crate) fn start(&mut self, game: &SteamGame, state: &SystemState) {
        if self.active_app_id.as_deref() == Some(game.app_id.as_str()) {
            return;
        }

        self.active_app_id = Some(game.app_id.clone());
        self.baseline_processes = all_processes(state)
            .map(|(name, _)| normalize_process_name(name))
            .collect();
        self.blocked_processes.clear();
        self.frame_counts.clear();
        self.best = None;
    }

    pub(crate) fn is_active_for(&self, app_id: &str) -> bool {
        self.active_app_id.as_deref() == Some(app_id)
    }

    pub(crate) fn best(&self) -> Option<&ResolvedCandidate> {
        self.best.as_ref()
    }

    pub(crate) fn block_process(&mut self, process_name: &str) {
        self.blocked_processes
            .insert(normalize_process_name(process_name));
        if self
            .best
            .as_ref()
            .is_some_and(|best| best.process_name.eq_ignore_ascii_case(process_name))
        {
            self.best = None;
        }
    }

    pub(crate) fn direct_target(
        &mut self,
        game: &SteamGame,
        state: &SystemState,
    ) -> Option<String> {
        self.update_best_from_state(game, state);
        self.best
            .as_ref()
            .filter(|candidate| candidate.score >= REQUIRED_DIRECT_SCORE)
            .map(|candidate| candidate.process_name.clone())
    }

    pub(crate) fn observe_frame(
        &mut self,
        metrics: &FrameMetrics,
        game: &SteamGame,
        state: &SystemState,
    ) -> Option<String> {
        let process_name = metrics.process_name.as_deref()?;
        if is_rejected_process(process_name) || self.is_blocked(process_name) {
            return None;
        }

        let normalized = normalize_process_name(process_name);
        let frame_count = {
            let count = self.frame_counts.entry(normalized.clone()).or_default();
            *count += 1;
            *count
        };

        let group = find_process_group(state, process_name);
        let score = score_process(
            process_name,
            group,
            game,
            &self.baseline_processes,
            frame_count,
        );
        self.remember_best(process_name, score);

        (frame_count >= REQUIRED_FRAME_SAMPLES && score >= REQUIRED_FRAME_SCORE)
            .then(|| process_name.to_string())
    }

    fn update_best_from_state(&mut self, game: &SteamGame, state: &SystemState) {
        let best = all_processes(state)
            .filter(|(name, _)| !is_rejected_process(name) && !self.is_blocked(name))
            .map(|(name, group)| {
                let frame_count = self
                    .frame_counts
                    .get(&normalize_process_name(name))
                    .copied()
                    .unwrap_or_default();
                let score = score_process(
                    name,
                    Some(group),
                    game,
                    &self.baseline_processes,
                    frame_count,
                );
                ResolvedCandidate {
                    process_name: name.clone(),
                    score,
                }
            })
            .max_by_key(|candidate| candidate.score);

        if let Some(candidate) = best {
            self.remember_best(&candidate.process_name, candidate.score);
        }
    }

    fn remember_best(&mut self, process_name: &str, score: i32) {
        if self.best.as_ref().is_none_or(|best| score > best.score) {
            self.best = Some(ResolvedCandidate {
                process_name: process_name.to_string(),
                score,
            });
        }
    }

    fn is_blocked(&self, process_name: &str) -> bool {
        self.blocked_processes
            .contains(&normalize_process_name(process_name))
    }
}

pub(crate) fn discovery_exclusions() -> Vec<String> {
    LAUNCHER_PROCESS_NAMES
        .iter()
        .chain(NOISE_PROCESS_NAMES.iter())
        .map(|name| (*name).to_string())
        .collect()
}

pub(crate) fn is_rejected_process(process_name: &str) -> bool {
    let normalized = normalize_process_name(process_name);
    is_exact_match(&normalized, LAUNCHER_PROCESS_NAMES)
        || is_exact_match(&normalized, NOISE_PROCESS_NAMES)
        || GENERIC_HELPER_TERMS
            .iter()
            .any(|term| normalized.contains(term))
}

fn score_process(
    process_name: &str,
    group: Option<&ProcessGroup>,
    game: &SteamGame,
    baseline_processes: &HashSet<String>,
    frame_count: usize,
) -> i32 {
    let normalized = normalize_process_name(process_name);
    let mut score = 0;

    if frame_count > 0 {
        score += 90;
        score += frame_count.min(10) as i32;
    }

    if !baseline_processes.contains(&normalized) {
        score += 45;
    }

    if process_name_matches_game(process_name, &game.name) {
        score += 30;
    }

    if let Some(group) = group {
        if group.memory_mb >= 300.0 {
            score += 15;
        }
        if group
            .exe_path
            .as_deref()
            .is_some_and(|path| is_inside_install_dir(path, &game.install_dir))
        {
            score += 70;
        }
    }

    if is_exact_match(&normalized, LAUNCHER_PROCESS_NAMES) {
        score -= 200;
    }
    if is_exact_match(&normalized, NOISE_PROCESS_NAMES) {
        score -= 120;
    }
    if GENERIC_HELPER_TERMS
        .iter()
        .any(|term| normalized.contains(term))
    {
        score -= 70;
    }

    score
}

fn all_processes(state: &SystemState) -> impl Iterator<Item = (&String, &ProcessGroup)> {
    state
        .observed_processes
        .iter()
        .chain(state.hidden_processes.iter())
}

fn find_process_group<'a>(state: &'a SystemState, process_name: &str) -> Option<&'a ProcessGroup> {
    state
        .observed_processes
        .get(process_name)
        .or_else(|| state.hidden_processes.get(process_name))
}

fn is_exact_match(process_name: &str, matches: &[&str]) -> bool {
    matches.contains(&process_name)
}

fn normalize_process_name(process_name: &str) -> String {
    process_name.trim().to_ascii_lowercase()
}

fn is_inside_install_dir(exe_path: &str, install_dir: &Path) -> bool {
    let exe_path = normalize_path(exe_path);
    let install_dir = normalize_path(&install_dir.display().to_string());
    !install_dir.is_empty() && exe_path.starts_with(&install_dir)
}

fn normalize_path(path: &str) -> String {
    path.replace('/', "\\").to_ascii_lowercase()
}

fn process_name_matches_game(process_name: &str, game_name: &str) -> bool {
    let process_name = process_name
        .trim_end_matches(".exe")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    game_name_tokens(game_name).any(|token| process_name.contains(&token))
}

fn game_name_tokens(game_name: &str) -> impl Iterator<Item = String> + '_ {
    game_name
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| token.len() >= 4)
        .filter(|token| {
            !matches!(
                token.as_str(),
                "definitive" | "director" | "edition" | "game" | "remastered" | "steam"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> SteamGame {
        SteamGame {
            app_id: "1091500".to_string(),
            name: "Cyberpunk 2077".to_string(),
            install_dir: Path::new(r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077")
                .to_path_buf(),
            library_dir: Path::new(r"D:\SteamLibrary").to_path_buf(),
        }
    }

    fn state_with(process_name: &str, exe_path: &str, memory_mb: f64) -> SystemState {
        let mut state = SystemState::empty_for_test();
        state.observed_processes.insert(
            process_name.to_string(),
            ProcessGroup {
                count: 1,
                memory_mb,
                exe_path: Some(exe_path.to_string()),
            },
        );
        state
    }

    #[test]
    fn resolver_should_pick_frame_process_inside_game_install_dir() {
        let game = game();
        let state = state_with(
            "Cyberpunk2077.exe",
            r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe",
            4_000.0,
        );
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        let metrics = FrameMetrics {
            process_name: Some("Cyberpunk2077.exe".to_string()),
            ..FrameMetrics::idle()
        };

        assert!(resolver.observe_frame(&metrics, &game, &state).is_none());
        assert!(resolver.observe_frame(&metrics, &game, &state).is_none());
        assert_eq!(
            resolver.observe_frame(&metrics, &game, &state).as_deref(),
            Some("Cyberpunk2077.exe")
        );
    }

    #[test]
    fn resolver_should_ignore_known_launcher_frames() {
        let game = game();
        let state = state_with(
            "REDlauncher.exe",
            r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077\REDlauncher.exe",
            500.0,
        );
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        let metrics = FrameMetrics {
            process_name: Some("REDlauncher.exe".to_string()),
            ..FrameMetrics::idle()
        };

        assert!(resolver.observe_frame(&metrics, &game, &state).is_none());
    }

    #[test]
    fn resolver_should_pick_new_external_frame_process() {
        let game = game();
        let state = state_with(
            "Cyberpunk2077.exe",
            r"C:\Program Files\External Launcher\Cyberpunk2077.exe",
            4_000.0,
        );
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        let metrics = FrameMetrics {
            process_name: Some("Cyberpunk2077.exe".to_string()),
            ..FrameMetrics::idle()
        };

        resolver.observe_frame(&metrics, &game, &state);
        resolver.observe_frame(&metrics, &game, &state);
        assert_eq!(
            resolver.observe_frame(&metrics, &game, &state).as_deref(),
            Some("Cyberpunk2077.exe")
        );
    }
}
