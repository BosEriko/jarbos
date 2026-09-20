use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_MARKER: &str = "ph.eriko.jarbos/hook-events";
const INSTALL_SCRIPT: &str = include_str!("../scripts/install-hooks.sh");

fn contains_marker(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|contents| contents.contains(HOOK_MARKER))
        .unwrap_or(false)
}

pub fn ensure_installed() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let home = PathBuf::from(home);
    let claude_settings = home.join(".claude/settings.json");
    let codex_hooks = home.join(".codex/hooks.json");

    if contains_marker(&claude_settings) && contains_marker(&codex_hooks) {
        return;
    }

    let script_path = std::env::temp_dir().join("jarbos-install-hooks.sh");
    if std::fs::write(&script_path, INSTALL_SCRIPT).is_err() {
        return;
    }
    let _ = Command::new("sh").arg(&script_path).status();
    let _ = std::fs::remove_file(&script_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_marker() {
        let dir = std::env::temp_dir().join(format!(
            "jarbos_hook_installer_test_a_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("settings.json");
        std::fs::write(&file, "{\"hooks\": {}}").unwrap();

        assert!(!contains_marker(&file));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_present_marker() {
        let dir = std::env::temp_dir().join(format!(
            "jarbos_hook_installer_test_b_{}",
            std::process::id()
        ));
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
}
