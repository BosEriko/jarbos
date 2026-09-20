export type AgentState =
  | "hidden"
  | "sleeping"
  | "active"
  | "waiting"
  | "success"
  | "error";

export interface Agent {
  id: string;
  name: string;
  processNames: string[];
  state: AgentState;
}

export interface InstanceState {
  agentId: string;
  state: AgentState;
}

export interface AgentStateChangedPayload {
  instanceId: string;
  agentId: string;
  state: AgentState;
}
