export interface AgentDisplayConfig {
  id: string;
  name: string;
  processNames: string[];
  glyph: string;
  accentColor: string;
}

export const AGENT_CONFIGS: AgentDisplayConfig[] = [
  {
    id: "claude",
    name: "Claude",
    processNames: ["claude"],
    glyph: "\u{1F7E0}",
    accentColor: "#d97757",
  },
  {
    id: "codex",
    name: "Codex",
    processNames: ["codex"],
    glyph: "\u{2B1B}",
    accentColor: "#10a37f",
  },
];

export const AVATAR_SIZE = 64;
export const BOTTOM_MARGIN = 24;
export const FADE_OUT_MS = 600;
