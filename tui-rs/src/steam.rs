use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::metrics::format_duration;
use crate::system::SystemState;

#[derive(Clone)]
pub(crate) struct SteamGame {
    pub(crate) app_id: String,
    pub(crate) name: String,
    pub(crate) install_dir: PathBuf,
    pub(crate) library_dir: PathBuf,
}

pub(crate) struct SteamLibrary {
    pub(crate) games: Vec<SteamGame>,
    pub(crate) selected: usize,
    pub(crate) status: String,
    pub(crate) scanning: bool,
}

pub(crate) struct SteamScanResult {
    games: Vec<SteamGame>,
    status: String,
}

#[derive(Clone)]
pub(crate) struct RunningSteamGame {
    pub(crate) app_id: String,
    pub(crate) process_name: String,
    pub(crate) exe_path: String,
}

pub(crate) struct ActiveSession {
    pub(crate) app_id: String,
    pub(crate) name: String,
    pub(crate) started_at: Instant,
    pub(crate) overdrive: bool,
    pub(crate) source: SessionSource,
}

pub(crate) struct CompletedSession {
    pub(crate) app_id: String,
    pub(crate) name: String,
    pub(crate) seconds: u64,
    pub(crate) overdrive: bool,
    pub(crate) source: SessionSource,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionSource {
    Manual,
    AutoDetected,
}

impl SessionSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::AutoDetected => "auto-detected",
        }
    }
}

#[derive(Default)]
pub(crate) struct SessionState {
    pub(crate) active: Option<ActiveSession>,
    pub(crate) last_completed: Option<String>,
}

impl SteamLibrary {
    pub(crate) fn loading() -> Self {
        Self {
            games: Vec::new(),
            selected: 0,
            status: "scanning Steam libraries...".to_string(),
            scanning: true,
        }
    }

    pub(crate) fn selected_game(&self) -> Option<&SteamGame> {
        self.games.get(self.selected)
    }

    pub(crate) fn select_next(&mut self) {
        if !self.games.is_empty() {
            self.selected = (self.selected + 1) % self.games.len();
        }
    }

    pub(crate) fn select_previous(&mut self) {
        if !self.games.is_empty() {
            self.selected = (self.selected + self.games.len() - 1) % self.games.len();
        }
    }

    pub(crate) fn apply_scan(&mut self, result: SteamScanResult) {
        self.games = result.games;
        self.status = result.status;
        self.scanning = false;
        if self.selected >= self.games.len() {
            self.selected = self.games.len().saturating_sub(1);
        }
    }

    pub(crate) fn detect_running_game(&self, state: &SystemState) -> Option<&SteamGame> {
        self.detect_running_game_process(state)
            .and_then(|running| self.games.iter().find(|game| game.app_id == running.app_id))
    }

    pub(crate) fn detect_running_game_process(
        &self,
        state: &SystemState,
    ) -> Option<RunningSteamGame> {
        self.running_processes(state)
            .max_by_key(|running| running.exe_path.len())
    }

    pub(crate) fn running_process_for_app(
        &self,
        app_id: &str,
        state: &SystemState,
    ) -> Option<RunningSteamGame> {
        self.running_processes(state)
            .filter(|running| running.app_id == app_id)
            .max_by_key(|running| running.exe_path.len())
    }

    fn running_processes<'a>(
        &'a self,
        state: &'a SystemState,
    ) -> impl Iterator<Item = RunningSteamGame> + 'a {
        state
            .observed_processes
            .iter()
            .chain(state.hidden_processes.iter())
            .filter_map(|(process_name, group)| {
                let exe_path = group.exe_path.as_deref()?;
                let (game, _) = match_game_by_exe_path(&self.games, exe_path)?;
                Some(RunningSteamGame {
                    app_id: game.app_id.clone(),
                    process_name: process_name.clone(),
                    exe_path: exe_path.to_string(),
                })
            })
    }
}

impl SessionState {
    pub(crate) fn start(&mut self, game: &SteamGame, overdrive: bool) {
        self.start_with_source(game, overdrive, SessionSource::Manual);
    }

    pub(crate) fn start_detected(&mut self, game: &SteamGame) {
        self.start_with_source(game, false, SessionSource::AutoDetected);
    }

    fn start_with_source(&mut self, game: &SteamGame, overdrive: bool, source: SessionSource) {
        self.active = Some(ActiveSession {
            app_id: game.app_id.clone(),
            name: game.name.clone(),
            started_at: Instant::now(),
            overdrive,
            source,
        });
    }

    pub(crate) fn active_app_id(&self) -> Option<&str> {
        self.active.as_ref().map(|session| session.app_id.as_str())
    }

    pub(crate) fn active_is_auto_detected(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|session| session.source == SessionSource::AutoDetected)
    }

    pub(crate) fn stop(&mut self) -> Option<CompletedSession> {
        let session = self.active.take()?;
        let ActiveSession {
            app_id,
            name,
            started_at,
            overdrive,
            source,
        } = session;
        let seconds = started_at.elapsed().as_secs();
        let label = format!(
            "{name} ended after {} ({})",
            format_duration(Duration::from_secs(seconds)),
            source.as_str()
        );
        self.last_completed = Some(label);
        Some(CompletedSession {
            app_id,
            name,
            seconds,
            overdrive,
            source,
        })
    }
}

fn match_game_by_exe_path<'a>(
    games: &'a [SteamGame],
    exe_path: &str,
) -> Option<(&'a SteamGame, usize)> {
    let exe_path = normalize_path_string(exe_path);
    games
        .iter()
        .filter_map(|game| {
            let install_dir = normalize_path(&game.install_dir);
            path_is_inside_dir(&exe_path, &install_dir).then_some((game, install_dir.len()))
        })
        .max_by_key(|(_, match_len)| *match_len)
}

fn normalize_path(path: &Path) -> String {
    normalize_path_string(&path.display().to_string())
}

fn normalize_path_string(path: &str) -> String {
    path.trim_matches('"')
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn path_is_inside_dir(path: &str, dir: &str) -> bool {
    let dir = dir.trim_end_matches('/');
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub(crate) fn spawn_steam_scan() -> Receiver<SteamScanResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(scan_steam_library());
    });
    rx
}

fn scan_steam_library() -> SteamScanResult {
    let roots = discover_steam_roots();
    if roots.is_empty() {
        return SteamScanResult {
            games: Vec::new(),
            status: "Steam no encontrado en rutas conocidas".to_string(),
        };
    }

    let mut libraries = roots.clone();
    for root in &roots {
        let libraryfolders = root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(contents) = fs::read_to_string(libraryfolders) {
            libraries.extend(
                extract_vdf_values(&contents, "path")
                    .into_iter()
                    .map(PathBuf::from),
            );
        }
    }

    libraries.sort();
    libraries.dedup();

    let mut games = Vec::new();
    for library in &libraries {
        let steamapps = library.join("steamapps");
        let Ok(entries) = fs::read_dir(&steamapps) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"));
            if !is_manifest {
                continue;
            }

            if let Some(game) = parse_app_manifest(&path, library) {
                games.push(game);
            }
        }
    }

    games.sort_by_key(|game| game.name.to_lowercase());
    let status = format!("{} Steam games detected", games.len());
    SteamScanResult { games, status }
}

fn discover_steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(steam_dir) = std::env::var("STEAM_DIR") {
        roots.push(PathBuf::from(steam_dir));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        roots.push(PathBuf::from(program_files_x86).join("Steam"));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        roots.push(PathBuf::from(program_files).join("Steam"));
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    roots.push(PathBuf::from(r"C:\Program Files\Steam"));

    roots.sort();
    roots.dedup();
    roots
        .into_iter()
        .filter(|path| path.join("steamapps").is_dir())
        .collect()
}

fn parse_app_manifest(path: &Path, library: &Path) -> Option<SteamGame> {
    let contents = fs::read_to_string(path).ok()?;
    let app_id = extract_vdf_value(&contents, "appid")?;
    let name = extract_vdf_value(&contents, "name")?;
    let install_dir = extract_vdf_value(&contents, "installdir").unwrap_or_else(|| name.clone());

    Some(SteamGame {
        app_id,
        name,
        install_dir: library.join("steamapps").join("common").join(install_dir),
        library_dir: library.to_path_buf(),
    })
}

fn extract_vdf_value(contents: &str, key: &str) -> Option<String> {
    extract_vdf_values(contents, key).into_iter().next()
}

fn extract_vdf_values(contents: &str, key: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let tokens = quoted_tokens(line);
            if tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case(key) {
                Some(tokens[1].clone())
            } else {
                None
            }
        })
        .collect()
}

fn quoted_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;

    for ch in line.chars() {
        if !in_quote {
            if ch == '"' {
                in_quote = true;
                current.clear();
            }
            continue;
        }

        if escaped {
            current.push(match ch {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            tokens.push(current.clone());
            current.clear();
            in_quote = false;
        } else {
            current.push(ch);
        }
    }

    tokens
}

pub(crate) fn launch_steam_game(game: &SteamGame) -> bool {
    open_steam_uri(&steam_app_uri("run", &game.app_id))
}

pub(crate) fn install_steam_game(game: &SteamGame) -> bool {
    open_steam_uri(&steam_app_uri("install", &game.app_id))
}

pub(crate) fn uninstall_steam_game(game: &SteamGame) -> bool {
    open_steam_uri(&steam_app_uri("uninstall", &game.app_id))
}

pub(crate) fn validate_steam_game(game: &SteamGame) -> bool {
    open_steam_uri(&steam_app_uri("validate", &game.app_id))
}

pub(crate) fn open_steam_game_properties(game: &SteamGame) -> bool {
    open_steam_uri(&steam_app_uri("gameproperties", &game.app_id))
}

pub(crate) fn open_steam_downloads() -> bool {
    open_steam_uri("steam://open/downloads")
}

fn open_steam_uri(uri: &str) -> bool {
    Command::new("cmd")
        .args(["/C", "start", "", uri])
        .spawn()
        .is_ok()
}

fn steam_app_uri(command: &str, app_id: &str) -> String {
    format!("steam://{command}/{app_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn quoted_tokens_should_parse_escaped_vdf_values() {
        let tokens = quoted_tokens(r#""name" "Neon \"Runner\" \\ Deluxe""#);

        assert_eq!(tokens, vec!["name", r#"Neon "Runner" \ Deluxe"#]);
    }

    #[test]
    fn parse_app_manifest_should_build_common_install_path() {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "chaosgamemode-test-{}-{unique}",
            std::process::id()
        ));
        let steamapps = root.join("steamapps");
        fs::create_dir_all(&steamapps).expect("test steamapps directory should be created");
        let manifest = steamapps.join("appmanifest_123.acf");
        fs::write(
            &manifest,
            r#"
"AppState"
{
    "appid" "123"
    "name" "Neon Runner"
    "installdir" "NeonRunner"
}
"#,
        )
        .expect("test manifest should be written");

        let game = parse_app_manifest(&manifest, &root).expect("manifest should parse");

        assert_eq!(game.app_id, "123");
        assert_eq!(game.name, "Neon Runner");
        assert_eq!(
            game.install_dir,
            root.join("steamapps").join("common").join("NeonRunner")
        );

        fs::remove_dir_all(root).expect("test directory should be removed");
    }

    #[test]
    fn match_game_by_exe_path_should_match_exe_inside_install_dir() {
        let games = vec![SteamGame {
            app_id: "123".to_string(),
            name: "Neon Runner".to_string(),
            install_dir: PathBuf::from(r"D:\SteamLibrary\steamapps\common\NeonRunner"),
            library_dir: PathBuf::from(r"D:\SteamLibrary"),
        }];

        let matched = match_game_by_exe_path(
            &games,
            r"D:\SteamLibrary\steamapps\common\NeonRunner\bin\game.exe",
        );

        assert_eq!(matched.map(|(game, _)| game.app_id.as_str()), Some("123"));
    }

    #[test]
    fn match_game_by_exe_path_should_reject_prefix_sibling_dirs() {
        let games = vec![SteamGame {
            app_id: "123".to_string(),
            name: "Game".to_string(),
            install_dir: PathBuf::from(r"D:\SteamLibrary\steamapps\common\Game"),
            library_dir: PathBuf::from(r"D:\SteamLibrary"),
        }];

        let matched = match_game_by_exe_path(
            &games,
            r"D:\SteamLibrary\steamapps\common\Game Plus\game.exe",
        );

        assert!(matched.is_none());
    }

    #[test]
    fn match_game_by_exe_path_should_prefer_longest_install_dir() {
        let games = vec![
            SteamGame {
                app_id: "base".to_string(),
                name: "Game".to_string(),
                install_dir: PathBuf::from(r"D:\SteamLibrary\steamapps\common\Game"),
                library_dir: PathBuf::from(r"D:\SteamLibrary"),
            },
            SteamGame {
                app_id: "deluxe".to_string(),
                name: "Game Deluxe".to_string(),
                install_dir: PathBuf::from(r"D:\SteamLibrary\steamapps\common\Game\Deluxe"),
                library_dir: PathBuf::from(r"D:\SteamLibrary"),
            },
        ];

        let matched = match_game_by_exe_path(
            &games,
            r"D:\SteamLibrary\steamapps\common\Game\Deluxe\run.exe",
        );

        assert_eq!(
            matched.map(|(game, _)| game.app_id.as_str()),
            Some("deluxe")
        );
    }

    #[test]
    fn running_process_for_app_should_include_process_name() {
        let mut state = crate::system::SystemState::empty_for_test();
        state.observed_processes.insert(
            "NeonRunner.exe".to_string(),
            crate::system::ProcessGroup {
                count: 1,
                memory_mb: 512.0,
                exe_path: Some(
                    r"D:\SteamLibrary\steamapps\common\NeonRunner\NeonRunner.exe".to_string(),
                ),
            },
        );
        let library = SteamLibrary {
            games: vec![SteamGame {
                app_id: "123".to_string(),
                name: "Neon Runner".to_string(),
                install_dir: PathBuf::from(r"D:\SteamLibrary\steamapps\common\NeonRunner"),
                library_dir: PathBuf::from(r"D:\SteamLibrary"),
            }],
            selected: 0,
            status: String::new(),
            scanning: false,
        };

        let running = library
            .running_process_for_app("123", &state)
            .expect("running process should be detected");

        assert_eq!(running.process_name, "NeonRunner.exe");
    }

    #[test]
    fn steam_app_uri_should_build_client_protocol_links() {
        assert_eq!(
            steam_app_uri("validate", "1091500"),
            "steam://validate/1091500"
        );
    }
}
