use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_ENV: &str = "CHAOS_HISTORY";
const HISTORY_FILE: &str = "history.log";

pub(crate) struct HistorySnapshot {
    pub(crate) path: PathBuf,
    pub(crate) lines: Vec<String>,
    pub(crate) total_lines: usize,
}

pub(crate) fn append_action(event: &str, profile: &str, lines: &[String]) -> io::Result<PathBuf> {
    let path = history_path();
    append_entry_to_path(&path, &[("event", event), ("profile", profile)], lines)?;
    Ok(path)
}

pub(crate) fn append_session(
    game_name: &str,
    app_id: &str,
    seconds: u64,
    overdrive: bool,
    source: &str,
) -> io::Result<PathBuf> {
    let path = history_path();
    let seconds = seconds.to_string();
    let overdrive = overdrive.to_string();

    append_entry_to_path(
        &path,
        &[
            ("event", "session"),
            ("game", game_name),
            ("app_id", app_id),
            ("duration_s", &seconds),
            ("overdrive", &overdrive),
            ("source", source),
        ],
        &[],
    )?;
    Ok(path)
}

pub(crate) fn current_path() -> PathBuf {
    history_path()
}

pub(crate) fn read_recent_lines(limit: usize) -> io::Result<HistorySnapshot> {
    read_recent_lines_from_path(&history_path(), limit)
}

fn history_path() -> PathBuf {
    if let Some(path) = std::env::var_os(HISTORY_ENV) {
        return PathBuf::from(path);
    }

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        return exe_dir.join(HISTORY_FILE);
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(HISTORY_FILE)
}

fn read_recent_lines_from_path(path: &Path, limit: usize) -> io::Result<HistorySnapshot> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(HistorySnapshot {
                path: path.to_path_buf(),
                lines: Vec::new(),
                total_lines: 0,
            });
        }
        Err(err) => return Err(err),
    };

    let total_lines = contents.lines().count();
    let skip = total_lines.saturating_sub(limit);
    let lines = contents
        .lines()
        .skip(skip)
        .map(ToString::to_string)
        .collect();

    Ok(HistorySnapshot {
        path: path.to_path_buf(),
        lines,
        total_lines,
    })
}

fn append_entry_to_path(path: &Path, fields: &[(&str, &str)], lines: &[String]) -> io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    write!(file, "==== epoch_s={}", epoch_seconds())?;
    for (key, value) in fields {
        write!(file, " {key}=\"{}\"", escape_field(value))?;
    }
    writeln!(file, " ====")?;

    for line in lines {
        writeln!(file, "{}", one_line(line))?;
    }

    writeln!(file)?;
    Ok(())
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\r', '\n'], " ")
}

fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn append_entry_to_path_should_write_header_fields() {
        let path = unique_history_path();

        append_entry_to_path(
            &path,
            &[("event", "overdrive"), ("profile", "balanced")],
            &["line one".to_string()],
        )
        .expect("history entry should be written");

        let contents = fs::read_to_string(&path).expect("history entry should be readable");

        assert!(contents.contains("event=\"overdrive\" profile=\"balanced\""));

        fs::remove_file(path).expect("history test file should be removed");
    }

    #[test]
    fn append_entry_to_path_should_keep_payload_lines() {
        let path = unique_history_path();

        append_entry_to_path(&path, &[("event", "restore")], &["restore ok".to_string()])
            .expect("history entry should be written");

        let contents = fs::read_to_string(&path).expect("history entry should be readable");

        assert!(contents.contains("restore ok"));

        fs::remove_file(path).expect("history test file should be removed");
    }

    #[test]
    fn escape_field_should_keep_values_single_line() {
        let escaped = escape_field("Neon \"Runner\"\nDeluxe");

        assert_eq!(escaped, "Neon \\\"Runner\\\" Deluxe");
    }

    #[test]
    fn read_recent_lines_from_path_should_return_tail() {
        let path = unique_history_path();
        fs::write(&path, "one\ntwo\nthree\n").expect("history test file should be written");

        let snapshot =
            read_recent_lines_from_path(&path, 2).expect("history tail should be readable");

        assert_eq!(snapshot.lines, vec!["two".to_string(), "three".to_string()]);

        fs::remove_file(path).expect("history test file should be removed");
    }

    #[test]
    fn read_recent_lines_from_path_should_treat_missing_file_as_empty() {
        let path = unique_history_path();

        let snapshot =
            read_recent_lines_from_path(&path, 20).expect("missing history should be readable");

        assert_eq!(snapshot.total_lines, 0);
    }

    fn unique_history_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "chaosgamemode-history-{}-{unique}.log",
            process::id()
        ))
    }
}
