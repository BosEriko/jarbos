use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Hidden,
    Sleeping,
    Active,
    Waiting,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub process_names: Vec<String>,
}

pub fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            process_names: vec!["claude".to_string()],
        },
        AgentConfig {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            process_names: vec!["codex".to_string()],
        },
    ]
}
