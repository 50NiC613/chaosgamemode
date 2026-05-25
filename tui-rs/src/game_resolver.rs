use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::frames::FrameMetrics;
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
    "redprelauncher.exe",
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
    "crashreporter.exe",
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
    "pwsh.exe",
    "amdrssrv.exe",
    "radeonsoftware.exe",
    "nvcontainer.exe",
    "redengineerrorreporter.exe",
    "steam.exe",
    "steamerrorreporter.exe",
    "steamservice.exe",
    "steamwebhelper.exe",
    "windowsterminal.exe",
];

const GENERIC_HELPER_TERMS: &[&str] = &[
    "bootstrap",
    "config",
    "crashhandler",
    "crashreporter",
    "errorreporter",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameCandidateDecision {
    Accepted,
    Probing,
    Watching,
    Rejected,
}

#[derive(Clone, Debug)]
pub(crate) struct FrameCandidateDiagnostic {
    pub(crate) process_name: String,
    pub(crate) decision: FrameCandidateDecision,
    pub(crate) score: i32,
    pub(crate) frame_samples: usize,
    pub(crate) memory_mb: Option<f64>,
    pub(crate) reason: String,
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
        let normalized = normalize_process_name(process_name);
        let frame_count = {
            let count = self.frame_counts.entry(normalized.clone()).or_default();
            *count += 1;
            *count
        };
        if is_rejected_process(process_name) || self.is_blocked(process_name) {
            return None;
        }

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

    pub(crate) fn diagnostics(
        &self,
        game: &SteamGame,
        state: &SystemState,
    ) -> Vec<FrameCandidateDiagnostic> {
        let mut names = HashSet::new();

        for (name, group) in all_processes(state) {
            let frame_count = self.frame_count(name);
            if frame_count > 0 || process_is_relevant_to_game(name, Some(group), game) {
                names.insert(name.clone());
            }
        }

        names.extend(self.frame_counts.keys().cloned());

        let mut diagnostics = names
            .into_iter()
            .map(|name| {
                let frame_count = self.frame_count(&name);
                let group = find_process_group_case_insensitive(state, &name);
                let score =
                    score_process(&name, group, game, &self.baseline_processes, frame_count);
                let (decision, reason) = self.candidate_decision(&name, score, frame_count);

                FrameCandidateDiagnostic {
                    process_name: name,
                    decision,
                    score,
                    frame_samples: frame_count,
                    memory_mb: group.map(|group| group.memory_mb),
                    reason,
                }
            })
            .collect::<Vec<_>>();

        diagnostics.sort_by(|a, b| {
            decision_rank(a.decision)
                .cmp(&decision_rank(b.decision))
                .then_with(|| b.score.cmp(&a.score))
                .then_with(|| b.frame_samples.cmp(&a.frame_samples))
                .then_with(|| a.process_name.cmp(&b.process_name))
        });
        diagnostics
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

    fn frame_count(&self, process_name: &str) -> usize {
        self.frame_counts
            .get(&normalize_process_name(process_name))
            .copied()
            .unwrap_or_default()
    }

    fn candidate_decision(
        &self,
        process_name: &str,
        score: i32,
        frame_count: usize,
    ) -> (FrameCandidateDecision, String) {
        if self.is_blocked(process_name) {
            return (
                FrameCandidateDecision::Rejected,
                "stale target blocked".to_string(),
            );
        }
        if let Some(reason) = rejection_reason(process_name) {
            return (FrameCandidateDecision::Rejected, reason.to_string());
        }
        if frame_count >= REQUIRED_FRAME_SAMPLES && score >= REQUIRED_FRAME_SCORE {
            return (
                FrameCandidateDecision::Accepted,
                "RTSS frames + Steam path".to_string(),
            );
        }
        if self
            .best
            .as_ref()
            .is_some_and(|best| best.process_name.eq_ignore_ascii_case(process_name))
        {
            return (
                FrameCandidateDecision::Probing,
                "best match so far".to_string(),
            );
        }
        if score >= REQUIRED_DIRECT_SCORE {
            return (
                FrameCandidateDecision::Probing,
                "strong Steam path match".to_string(),
            );
        }
        if frame_count > 0 {
            return (
                FrameCandidateDecision::Probing,
                "RTSS frames below threshold".to_string(),
            );
        }

        (
            FrameCandidateDecision::Watching,
            "Steam install match".to_string(),
        )
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
    rejection_reason(process_name).is_some()
}

fn is_unknown_process_name(process_name: &str) -> bool {
    matches!(
        process_name.trim_matches(|ch| ch == '<' || ch == '>' || ch == '"' || ch == '\''),
        "" | "unknown" | "unk" | "n/a" | "na"
    )
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
    if matches_helper_term(&normalized) {
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

fn find_process_group_case_insensitive<'a>(
    state: &'a SystemState,
    process_name: &str,
) -> Option<&'a ProcessGroup> {
    find_process_group(state, process_name).or_else(|| {
        all_processes(state)
            .find(|(name, _)| name.eq_ignore_ascii_case(process_name))
            .map(|(_, group)| group)
    })
}

fn is_exact_match(process_name: &str, matches: &[&str]) -> bool {
    matches.contains(&process_name)
}

fn rejection_reason(process_name: &str) -> Option<&'static str> {
    let normalized = normalize_process_name(process_name);
    if is_unknown_process_name(&normalized) {
        return Some("unknown RTSS entry");
    }
    if is_exact_match(&normalized, LAUNCHER_PROCESS_NAMES) {
        return Some("launcher/helper");
    }
    if is_exact_match(&normalized, NOISE_PROCESS_NAMES) {
        return Some(
            if normalized.contains("reporter") || normalized.contains("crash") {
                "reporter/crash helper"
            } else {
                "background/system noise"
            },
        );
    }
    if matches_helper_term(&normalized) {
        return Some("helper process");
    }
    None
}

fn normalize_process_name(process_name: &str) -> String {
    process_name.trim().to_ascii_lowercase()
}

fn matches_helper_term(process_name: &str) -> bool {
    let stem = process_name.strip_suffix(".exe").unwrap_or(process_name);
    GENERIC_HELPER_TERMS.iter().any(|term| {
        stem == *term || stem.ends_with(term) || (*term == "unins" && stem.starts_with("unins"))
    })
}

fn process_is_relevant_to_game(
    process_name: &str,
    group: Option<&ProcessGroup>,
    game: &SteamGame,
) -> bool {
    process_name_matches_game(process_name, &game.name)
        || group
            .and_then(|group| group.exe_path.as_deref())
            .is_some_and(|path| is_inside_install_dir(path, &game.install_dir))
}

fn decision_rank(decision: FrameCandidateDecision) -> u8 {
    match decision {
        FrameCandidateDecision::Accepted => 0,
        FrameCandidateDecision::Probing => 1,
        FrameCandidateDecision::Watching => 2,
        FrameCandidateDecision::Rejected => 3,
    }
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
    fn resolver_should_ignore_redengine_error_reporter() {
        let game = game();
        let state = state_with(
            "REDengineErrorReporter.exe",
            r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077\bin\x64\REDengineErrorReporter.exe",
            300.0,
        );
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        let metrics = FrameMetrics {
            process_name: Some("REDengineErrorReporter.exe".to_string()),
            ..FrameMetrics::idle()
        };

        assert!(resolver.observe_frame(&metrics, &game, &state).is_none());
        assert!(is_rejected_process("REDengineErrorReporter.exe"));
    }

    #[test]
    fn diagnostics_should_explain_rejected_and_accepted_candidates() {
        let game = game();
        let mut state = state_with(
            "Cyberpunk2077.exe",
            r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe",
            4_000.0,
        );
        state.observed_processes.insert(
            "REDengineErrorReporter.exe".to_string(),
            ProcessGroup {
                count: 1,
                memory_mb: 300.0,
                exe_path: Some(
                    r"D:\SteamLibrary\steamapps\common\Cyberpunk 2077\bin\x64\REDengineErrorReporter.exe"
                        .to_string(),
                ),
            },
        );
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        for process_name in [
            "Cyberpunk2077.exe",
            "Cyberpunk2077.exe",
            "Cyberpunk2077.exe",
            "REDengineErrorReporter.exe",
        ] {
            let metrics = FrameMetrics {
                process_name: Some(process_name.to_string()),
                ..FrameMetrics::idle()
            };
            let _ = resolver.observe_frame(&metrics, &game, &state);
        }

        let diagnostics = resolver.diagnostics(&game, &state);
        let game_candidate = diagnostics
            .iter()
            .find(|candidate| candidate.process_name == "Cyberpunk2077.exe")
            .expect("game candidate should be present");
        let reporter_candidate = diagnostics
            .iter()
            .find(|candidate| candidate.process_name == "REDengineErrorReporter.exe")
            .expect("reporter candidate should be present");

        assert_eq!(game_candidate.decision, FrameCandidateDecision::Accepted);
        assert_eq!(
            reporter_candidate.decision,
            FrameCandidateDecision::Rejected
        );
        assert!(reporter_candidate.reason.contains("reporter"));
    }

    #[test]
    fn resolver_should_not_reject_game_names_that_contain_helper_words() {
        assert!(!is_rejected_process("CrashBandicoot4.exe"));
        assert!(!is_rejected_process("ConfigurableAdventure.exe"));
    }

    #[test]
    fn resolver_should_ignore_unknown_frame_source() {
        let game = game();
        let state = state_with("Unknown", "", 0.0);
        let mut resolver = GameProcessResolver::default();
        resolver.start(&game, &SystemState::empty_for_test());

        let metrics = FrameMetrics {
            process_name: Some("Unknown".to_string()),
            ..FrameMetrics::idle()
        };

        assert!(resolver.observe_frame(&metrics, &game, &state).is_none());
        assert!(is_rejected_process("<unknown>"));
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
