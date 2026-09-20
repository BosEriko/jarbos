use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::process_monitor::CwdByInstance;

pub const POLL_INTERVAL: Duration = Duration::from_millis(300);

fn classify_event(hook_event_name: &str, notification_type: Option<&str>) -> Option<bool> {
    match hook_event_name {
        "Notification" => match notification_type {
            Some("permission_prompt") => Some(true),
            _ => None,
        },
        "PermissionRequest" => Some(true),
        "PreToolUse" | "UserPromptSubmit" | "Stop" => Some(false),
        _ => None,
    }
}

fn normalize_cwd(cwd: &str) -> &str {
    cwd.trim_end_matches('/')
}

fn find_instance_for_cwd(cwd_by_instance: &HashMap<String, String>, cwd: &str) -> Option<String> {
    let cwd = normalize_cwd(cwd);
    cwd_by_instance
        .iter()
        .find(|(_, instance_cwd)| normalize_cwd(instance_cwd) == cwd)
        .map(|(instance_id, _)| instance_id.clone())
}

fn process_event_file(
    path: &std::path::Path,
    cwd_by_instance: &CwdByInstance,
    on_change: &(dyn Fn(&str, bool) + Send + Sync),
) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };

    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return;
    };
    let _ = fs::remove_file(path);
    let Some(hook_event_name) = payload.get("hook_event_name").and_then(|v| v.as_str()) else {
        return;
    };
    let Some(cwd) = payload.get("cwd").and_then(|v| v.as_str()) else {
        return;
    };
    let notification_type = payload.get("notification_type").and_then(|v| v.as_str());
    let Some(waiting) = classify_event(hook_event_name, notification_type) else {
        return;
    };

    let instance_id = {
        let map = cwd_by_instance.lock().unwrap();
        find_instance_for_cwd(&map, cwd)
    };
    if let Some(instance_id) = instance_id {
        on_change(&instance_id, waiting);
    }
}

pub fn spawn<F>(events_dir: PathBuf, cwd_by_instance: CwdByInstance, on_change: F)
where
    F: Fn(&str, bool) + Send + Sync + 'static,
{
    thread::spawn(move || loop {
        if let Ok(entries) = fs::read_dir(&events_dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect();
            paths.sort();
            for path in paths {
                if path.is_file() {
                    process_event_file(&path, &cwd_by_instance, &on_change);
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn classifies_permission_prompt_notification_as_waiting() {
        assert_eq!(
            classify_event("Notification", Some("permission_prompt")),
            Some(true)
        );
        assert_eq!(classify_event("PermissionRequest", None), Some(true));
    }

    #[test]
    fn ignores_idle_and_other_notification_types() {
        assert_eq!(classify_event("Notification", Some("idle_timeout")), None);
        assert_eq!(classify_event("Notification", None), None);
    }

    #[test]
    fn classifies_resume_events_as_not_waiting() {
        assert_eq!(classify_event("PreToolUse", None), Some(false));
        assert_eq!(classify_event("UserPromptSubmit", None), Some(false));
        assert_eq!(classify_event("Stop", None), Some(false));
    }

    #[test]
    fn ignores_unknown_event_names() {
        assert_eq!(classify_event("SessionStart", None), None);
        assert_eq!(classify_event("PostToolUse", None), None);
    }

    #[test]
    fn matches_cwd_ignoring_trailing_slash() {
        let mut map = HashMap::new();
        map.insert("123".to_string(), "/Users/bos/project".to_string());
        assert_eq!(
            find_instance_for_cwd(&map, "/Users/bos/project/"),
            Some("123".to_string())
        );
        assert_eq!(
            find_instance_for_cwd(&map, "/Users/bos/project"),
            Some("123".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_cwd_matches() {
        let mut map = HashMap::new();
        map.insert("123".to_string(), "/Users/bos/project".to_string());
        assert_eq!(find_instance_for_cwd(&map, "/Users/bos/other"), None);
    }

    #[test]
    fn process_event_file_reports_and_removes_matching_event() {
        let dir =
            std::env::temp_dir().join(format!("jarbos_hook_watch_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("event.json");
        fs::write(
            &file,
            r#"{"hook_event_name":"PermissionRequest","cwd":"/tmp/proj","session_id":"abc"}"#,
        )
        .unwrap();

        let cwd_by_instance: CwdByInstance = Arc::new(Mutex::new(HashMap::from([(
            "42".to_string(),
            "/tmp/proj".to_string(),
        )])));

        let reported: Arc<Mutex<Vec<(String, bool)>>> = Arc::new(Mutex::new(Vec::new()));
        let reported_for_closure = Arc::clone(&reported);
        let on_change = move |instance_id: &str, waiting: bool| {
            reported_for_closure
                .lock()
                .unwrap()
                .push((instance_id.to_string(), waiting));
        };

        process_event_file(&file, &cwd_by_instance, &on_change);

        assert_eq!(*reported.lock().unwrap(), vec![("42".to_string(), true)]);
        assert!(!file.exists(), "event file should be consumed");

        let _ = fs::remove_dir_all(&dir);
    }
}
