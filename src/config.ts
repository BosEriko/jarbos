export interface AgentDisplayConfig {
  id: string;
  name: string;
  processNames: string[];
  accentColor: string;
}

export const AGENT_CONFIGS: AgentDisplayConfig[] = [
  {
    id: "claude",
    name: "Claude",
    processNames: ["claude"],
    accentColor: "#d97757",
  },
  {
    id: "codex",
    name: "Codex",
    processNames: ["codex"],
    accentColor: "#10a37f",
  },
];

export const AVATAR_SIZE = 64;
export const BOTTOM_MARGIN = 24;
export const FADE_OUT_MS = 600;

export const DUCK_NAMES: string[] = [
  "Quacker", "Puddles", "Waddles", "Bill", "Feathers", "Splash", "Bubbles", "Nibbles", "Pip", "Biscuit",
  "Noodle", "Waffles", "Pancake", "Peep", "Chirpy", "Dandy", "Marvin", "Ollie", "Wobble", "Ripple",
  "Squirt", "Petal", "Sprout", "Bean", "Pebble", "Muffin", "Cricket", "Sunny", "Breezy", "Misty",
  "Foggy", "Drizzle", "Puddle", "Splashy", "Feathery", "Downy", "Fluff", "Quill", "Webby", "Paddle",
  "Dabble", "Dip", "Skim", "Glide", "Float", "Bob", "Wiggle", "Jiggle", "Twirl", "Spin",
  "Hop", "Skip", "Scoot", "Zippy", "Dash", "Turbo", "Rusty", "Ginger", "Amber", "Honey",
  "Butter", "Toffee", "Caramel", "Maple", "Clover", "Poppy", "Willow", "Hazel", "Olive", "Basil",
  "Sage", "Pepper", "Cinnamon", "Nutmeg", "Clove", "Mochi", "Boba", "Taro", "Sesame", "Peanut",
  "Cashew", "Almond", "Walnut", "Hazelnut", "Chestnut", "Acorn", "Pinecone", "Lily", "Iris", "Fern",
  "Reed", "Rush", "Marsh", "Brook", "Creek", "River", "Delta", "Cove", "Bay", "Lagoon",
];
