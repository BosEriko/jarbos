import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { AgentState, AgentStateChangedPayload, InstanceState } from "./agent";

type Listener = (instanceId: string, agentId: string, state: AgentState) => void;

const listeners = new Set<Listener>();

export function onAgentStateChanged(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export async function initAgentStore(): Promise<void> {
  await listen<AgentStateChangedPayload>("agent-state-changed", (event) => {
    const { instanceId, agentId, state } = event.payload;
    for (const listener of listeners) {
      listener(instanceId, agentId, state);
    }
  });
}

export async function fetchCurrentStates(): Promise<Record<string, InstanceState>> {
  return invoke<Record<string, InstanceState>>("get_agent_states");
}
