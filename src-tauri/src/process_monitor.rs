use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use sysinfo::{ProcessesToUpdate, System};

use crate::agent::AgentConfig;

pub const POLL_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Started(String),
    Stopped(String),
}

fn normalize(name: &str) -> String {
    name.strip_suffix(".exe")
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn agent_is_running(agent: &AgentConfig, running_names: &[String]) -> bool {
    agent.process_names.iter().any(|configured| {
        let configured = normalize(configured);
        running_names.contains(&configured)
    })
}

pub fn spawn(agents: Vec<AgentConfig>, tx: Sender<ProcessEvent>) {
    thread::spawn(move || {
        let mut system = System::new();
        let mut alive: HashSet<String> = HashSet::new();

        loop {
            system.refresh_processes(ProcessesToUpdate::All, true);

            let running_names: Vec<String> = system
                .processes()
                .values()
                .map(|process| normalize(&process.name().to_string_lossy()))
                .collect();

            for agent in &agents {
                let is_running = agent_is_running(agent, &running_names);
                let was_alive = alive.contains(&agent.id);

                if is_running && !was_alive {
                    alive.insert(agent.id.clone());
                    if tx.send(ProcessEvent::Started(agent.id.clone())).is_err() {
                        return;
                    }
                } else if !is_running && was_alive {
                    alive.remove(&agent.id);
                    if tx.send(ProcessEvent::Stopped(agent.id.clone())).is_err() {
                        return;
                    }
                }
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
        let running = vec!["claude".to_string(), "zsh".to_string()];
        assert!(agent_is_running(&a, &running));
    }

    #[test]
    fn does_not_match_unrelated_process() {
        let a = agent(&["codex"]);
        let running = vec!["claude".to_string(), "zsh".to_string()];
        assert!(!agent_is_running(&a, &running));
    }

    #[test]
    fn matches_windows_exe_suffix() {
        let a = agent(&["claude"]);
        let running = vec![normalize("claude.exe")];
        assert!(agent_is_running(&a, &running));
    }

    #[test]
    fn matches_case_insensitively() {
        let a = agent(&["Claude"]);
        let running = vec![normalize("claude")];
        assert!(agent_is_running(&a, &running));
    }
}
