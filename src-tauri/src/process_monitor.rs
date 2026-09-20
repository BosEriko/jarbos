use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::agent::AgentConfig;

pub type CwdByInstance = Arc<Mutex<HashMap<String, String>>>;

pub const POLL_INTERVAL: Duration = Duration::from_millis(1000);

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing().with_cwd(UpdateKind::Always)
}

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Started {
        agent_id: String,
        instance_id: String,
    },
    Stopped {
        agent_id: String,
        instance_id: String,
    },
}

fn normalize(name: &str) -> String {
    name.strip_suffix(".exe")
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn matching_pids(agent: &AgentConfig, processes: &[(u32, String)]) -> HashSet<u32> {
    let configured: Vec<String> = agent.process_names.iter().map(|n| normalize(n)).collect();
    processes
        .iter()
        .filter(|(_, name)| configured.contains(name))
        .map(|(pid, _)| *pid)
        .collect()
}

pub fn spawn(agents: Vec<AgentConfig>, tx: Sender<ProcessEvent>, cwd_by_instance: CwdByInstance) {
    thread::spawn(move || {
        let mut system = System::new();
        let mut alive: HashMap<String, HashSet<u32>> = HashMap::new();

        loop {
            system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                process_refresh_kind(),
            );

            let processes: Vec<(u32, String)> = system
                .processes()
                .values()
                .map(|process| {
                    (
                        process.pid().as_u32(),
                        normalize(&process.name().to_string_lossy()),
                    )
                })
                .collect();

            let cwd_by_pid: HashMap<u32, String> = system
                .processes()
                .values()
                .filter_map(|process| {
                    process
                        .cwd()
                        .map(|cwd| (process.pid().as_u32(), cwd.to_string_lossy().to_string()))
                })
                .collect();

            for agent in &agents {
                let current = matching_pids(agent, &processes);
                let previous = alive.entry(agent.id.clone()).or_default();

                for &pid in current.difference(previous) {
                    let event = ProcessEvent::Started {
                        agent_id: agent.id.clone(),
                        instance_id: pid.to_string(),
                    };
                    if let Some(cwd) = cwd_by_pid.get(&pid) {
                        cwd_by_instance
                            .lock()
                            .unwrap()
                            .insert(pid.to_string(), cwd.clone());
                    }
                    if tx.send(event).is_err() {
                        return;
                    }
                }

                for &pid in previous.difference(&current) {
                    let event = ProcessEvent::Stopped {
                        agent_id: agent.id.clone(),
                        instance_id: pid.to_string(),
                    };
                    cwd_by_instance.lock().unwrap().remove(&pid.to_string());
                    if tx.send(event).is_err() {
                        return;
                    }
                }

                *previous = current;
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(process_names: &[&str]) -> AgentConfig {
        AgentConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            process_names: process_names.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn matches_exact_process_name() {
        let a = agent(&["claude"]);
        let processes = vec![(1, "claude".to_string()), (2, "zsh".to_string())];
        assert_eq!(matching_pids(&a, &processes), HashSet::from([1]));
    }

    #[test]
    fn does_not_match_unrelated_process() {
        let a = agent(&["codex"]);
        let processes = vec![(1, "claude".to_string()), (2, "zsh".to_string())];
        assert!(matching_pids(&a, &processes).is_empty());
    }

    #[test]
    fn matches_windows_exe_suffix() {
        let a = agent(&["claude"]);
        let processes = vec![(1, normalize("claude.exe"))];
        assert_eq!(matching_pids(&a, &processes), HashSet::from([1]));
    }

    #[test]
    fn matches_case_insensitively() {
        let a = agent(&["Claude"]);
        let processes = vec![(1, normalize("claude"))];
        assert_eq!(matching_pids(&a, &processes), HashSet::from([1]));
    }

    #[test]
    fn matches_multiple_instances_of_same_agent() {
        let a = agent(&["claude"]);
        let processes = vec![
            (1, "claude".to_string()),
            (2, "claude".to_string()),
            (3, "zsh".to_string()),
        ];
        assert_eq!(matching_pids(&a, &processes), HashSet::from([1, 2]));
    }

    #[test]
    fn sysinfo_reports_cwd_for_a_running_process() {
        use std::process::{Command, Stdio};

        let tmp_dir = std::env::temp_dir().join(format!("jarbos_cwd_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let mut child = Command::new("sleep")
            .arg("2")
            .current_dir(&tmp_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn test process");

        thread::sleep(Duration::from_millis(300));

        let mut system = System::new();
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh_kind());
        let cwd = system
            .process(sysinfo::Pid::from_u32(child.id()))
            .and_then(|p| p.cwd())
            .map(|c| c.to_path_buf());

        let expected = tmp_dir.canonicalize().unwrap_or_else(|_| tmp_dir.clone());

        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&tmp_dir);

        assert_eq!(cwd, Some(expected));
    }
}
