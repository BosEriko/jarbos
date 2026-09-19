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

export interface AgentStateChangedPayload {
  id: string;
  state: AgentState;
}
