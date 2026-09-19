import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { AgentState, AgentStateChangedPayload } from "./agent";

type Listener = (id: string, state: AgentState) => void;

const listeners = new Set<Listener>();

export function onAgentStateChanged(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export async function initAgentStore(): Promise<void> {
  await listen<AgentStateChangedPayload>("agent-state-changed", (event) => {
    const { id, state } = event.payload;
    for (const listener of listeners) {
      listener(id, state);
    }
  });
}

export async function fetchCurrentStates(): Promise<Record<string, AgentState>> {
  return invoke<Record<string, AgentState>>("get_agent_states");
}
