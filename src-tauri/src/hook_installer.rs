use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_MARKER: &str = "ph.eriko.jarbos/hook-events";
const INSTALL_SCRIPT: &str = include_str!("../scripts/install-hooks.sh");
const CLAUDE_MARKER_FILE: &str = ".claude_hook_initialized";
const CODEX_MARKER_FILE: &str = ".codex_hook_initialized";

fn contains_marker(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|contents| contents.contains(HOOK_MARKER))
        .unwrap_or(false)
}

fn is_hook_ready(marker_path: &Path, config_path: &Path) -> bool {
    if marker_path.exists() {
        return true;
    }
    if contains_marker(config_path) {
        let _ = std::fs::write(marker_path, "");
        return true;
    }
    false
}

pub fn ensure_installed() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let home = PathBuf::from(home);
    let claude_settings = home.join(".claude/settings.json");
    let codex_hooks = home.join(".codex/hooks.json");

    let claude_ready = is_hook_ready(Path::new(CLAUDE_MARKER_FILE), &claude_settings);
    let codex_ready = is_hook_ready(Path::new(CODEX_MARKER_FILE), &codex_hooks);

    if claude_ready && codex_ready {
        return;
    }

    let script_path = std::env::temp_dir().join("jarbos-install-hooks.sh");
    if std::fs::write(&script_path, INSTALL_SCRIPT).is_err() {
        return;
    }
    let _ = Command::new("sh").arg(&script_path).status();
    let _ = std::fs::remove_file(&script_path);

    let _ = std::fs::write(CLAUDE_MARKER_FILE, "");
    let _ = std::fs::write(CODEX_MARKER_FILE, "");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jarbos_hook_installer_test_{}_{}",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn detects_missing_marker() {
        let dir = temp_path("a");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("settings.json");
        std::fs::write(&file, "{\"hooks\": {}}").unwrap();

        assert!(!contains_marker(&file));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_present_marker() {
        let dir = temp_path("b");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("settings.json");
        std::fs::write(
            &file,
            "{\"hooks\": {\"Notification\": [{\"hooks\": [{\"command\": \"... ph.eriko.jarbos/hook-events ...\"}]}]}}",
        )
        .unwrap();

        assert!(contains_marker(&file));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_has_no_marker() {
        let missing = std::env::temp_dir().join("jarbos_definitely_missing_file.json");
        assert!(!contains_marker(&missing));
    }

    #[test]
    fn is_hook_ready_short_circuits_when_marker_file_exists() {
        let dir = temp_path("c");
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join(".claude_hook_initialized");
        let config = dir.join("settings.json");
        std::fs::write(&marker, "").unwrap();

        assert!(is_hook_ready(&marker, &config));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_hook_ready_backfills_marker_file_from_config() {
        let dir = temp_path("d");
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join(".codex_hook_initialized");
        let config = dir.join("hooks.json");
        std::fs::write(&config, "{\"PreToolUse\": [{\"hooks\": [{\"command\": \"... ph.eriko.jarbos/hook-events ...\"}]}]}").unwrap();

        assert!(!marker.exists());
        assert!(is_hook_ready(&marker, &config));
        assert!(marker.exists(), "marker file should be backfilled");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_hook_ready_false_when_neither_marker_nor_config_present() {
        let dir = temp_path("e");
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join(".claude_hook_initialized");
        let config = dir.join("settings.json");

        assert!(!is_hook_ready(&marker, &config));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
