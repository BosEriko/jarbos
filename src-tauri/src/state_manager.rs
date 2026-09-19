use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::agent::AgentState;
use crate::process_monitor::ProcessEvent;

pub const SLEEP_DURATION: Duration = Duration::from_secs(6);

type Generations = Arc<Mutex<HashMap<String, u64>>>;
pub type SharedStates = Arc<Mutex<HashMap<String, AgentState>>>;

pub fn run<F>(rx: Receiver<ProcessEvent>, on_change: F) -> SharedStates
where
    F: Fn(&str, AgentState) + Send + Sync + 'static,
{
    run_with_sleep_duration(rx, on_change, SLEEP_DURATION)
}

fn run_with_sleep_duration<F>(
    rx: Receiver<ProcessEvent>,
    on_change: F,
    sleep_duration: Duration,
) -> SharedStates
where
    F: Fn(&str, AgentState) + Send + Sync + 'static,
{
    let on_change = Arc::new(on_change);
    let generations: Generations = Arc::new(Mutex::new(HashMap::new()));
    let states: SharedStates = Arc::new(Mutex::new(HashMap::new()));

    let thread_states = Arc::clone(&states);
    thread::spawn(move || {
        for event in rx {
            match event {
                ProcessEvent::Started(id) => {
                    bump_generation(&generations, &id);
                    record(&thread_states, &on_change, &id, AgentState::Active);
                }
                ProcessEvent::Stopped(id) => {
                    record(&thread_states, &on_change, &id, AgentState::Sleeping);
                    let expected_generation = bump_generation(&generations, &id);

                    let generations = Arc::clone(&generations);
                    let on_change = Arc::clone(&on_change);
                    let states = Arc::clone(&thread_states);
                    thread::spawn(move || {
                        thread::sleep(sleep_duration);
                        let is_still_sleeping = generations
                            .lock()
                            .unwrap()
                            .get(&id)
                            .copied()
                            .map(|current| current == expected_generation)
                            .unwrap_or(false);
                        if is_still_sleeping {
                            record(&states, &on_change, &id, AgentState::Hidden);
                        }
                    });
                }
            }
        }
    });

    states
}

fn record<F>(states: &SharedStates, on_change: &Arc<F>, id: &str, state: AgentState)
where
    F: Fn(&str, AgentState) + Send + Sync + 'static,
{
    states.lock().unwrap().insert(id.to_string(), state);
    on_change(id, state);
}

fn bump_generation(generations: &Generations, id: &str) -> u64 {
    let mut map = generations.lock().unwrap();
    let counter = map.entry(id.to_string()).or_insert(0);
    *counter += 1;
    *counter
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn collect_events(
        rx: Receiver<ProcessEvent>,
        sleep_duration: Duration,
    ) -> mpsc::Receiver<(String, AgentState)> {
        let (out_tx, out_rx) = mpsc::channel::<(String, AgentState)>();
        run_with_sleep_duration(
            rx,
            move |id: &str, state: AgentState| {
                let _ = out_tx.send((id.to_string(), state));
            },
            sleep_duration,
        );
        out_rx
    }

    #[test]
    fn start_transitions_to_active() {
        let (tx, rx) = mpsc::channel();
        let out_rx = collect_events(rx, Duration::from_millis(50));

        tx.send(ProcessEvent::Started("claude".to_string()))
            .unwrap();
        let (id, state) = out_rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(id, "claude");
        assert_eq!(state, AgentState::Active);
    }

    #[test]
    fn stop_then_timeout_transitions_to_hidden() {
        let (tx, rx) = mpsc::channel();
        let out_rx = collect_events(rx, Duration::from_millis(30));

        tx.send(ProcessEvent::Stopped("codex".to_string())).unwrap();

        let (_, sleeping_state) = out_rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(sleeping_state, AgentState::Sleeping);

        let (_, hidden_state) = out_rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(hidden_state, AgentState::Hidden);
    }

    #[test]
    fn restart_during_sleep_cancels_hidden_transition() {
        let (tx, rx) = mpsc::channel();
        let out_rx = collect_events(rx, Duration::from_millis(60));

        tx.send(ProcessEvent::Stopped("claude".to_string()))
            .unwrap();
        let (_, sleeping_state) = out_rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(sleeping_state, AgentState::Sleeping);

        tx.send(ProcessEvent::Started("claude".to_string()))
            .unwrap();
        let (_, active_state) = out_rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(active_state, AgentState::Active);

        let unexpected = out_rx.recv_timeout(Duration::from_millis(150));
        assert!(
            unexpected.is_err(),
            "should not transition to Hidden after restart"
        );
    }

    #[test]
    fn shared_states_reflects_latest_transition() {
        let (tx, rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel::<(String, AgentState)>();
        let states = run_with_sleep_duration(
            rx,
            move |id: &str, state: AgentState| {
                let _ = out_tx.send((id.to_string(), state));
            },
            Duration::from_millis(50),
        );

        tx.send(ProcessEvent::Started("claude".to_string()))
            .unwrap();
        out_rx.recv_timeout(Duration::from_millis(200)).unwrap();

        let snapshot = states.lock().unwrap().clone();
        assert_eq!(snapshot.get("claude"), Some(&AgentState::Active));
    }
}
