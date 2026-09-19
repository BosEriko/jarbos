import type { AgentState } from "./agent";
import { Avatar } from "./avatar";
import { AGENT_CONFIGS, AVATAR_SIZE, FADE_OUT_MS } from "./config";
import { onAgentStateChanged } from "./agentStore";

export class AvatarManager {
  private readonly avatars = new Map<string, Avatar>();
  private readonly hideTimers = new Map<string, number>();
  private readonly container: HTMLElement;
  private lastFrameTime: number | null = null;
  private rafHandle: number | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  start(): void {
    onAgentStateChanged((id, state) => this.applyState(id, state));
    this.rafHandle = requestAnimationFrame((t) => this.tick(t));
  }

  applyState(id: string, state: AgentState): void {
    const config = AGENT_CONFIGS.find((c) => c.id === id);
    if (!config) {
      return;
    }

    const existingTimer = this.hideTimers.get(id);
    if (existingTimer !== undefined) {
      clearTimeout(existingTimer);
      this.hideTimers.delete(id);
    }

    if (state === "hidden") {
      const avatar = this.avatars.get(id);
      if (!avatar) {
        return;
      }
      avatar.applyState("hidden");
      const timer = window.setTimeout(() => {
        avatar.destroy();
        this.avatars.delete(id);
        this.hideTimers.delete(id);
      }, FADE_OUT_MS);
      this.hideTimers.set(id, timer);
      return;
    }

    let avatar = this.avatars.get(id);
    if (!avatar) {
      const startX = Math.random() * Math.max(0, window.innerWidth - AVATAR_SIZE);
      avatar = new Avatar(config, this.container, startX);
      this.avatars.set(id, avatar);
    }
    avatar.applyState(state);
  }

  private tick(now: number): void {
    const dt = this.lastFrameTime === null ? 0 : now - this.lastFrameTime;
    this.lastFrameTime = now;

    for (const avatar of this.avatars.values()) {
      if (avatar.currentState === "active") {
        avatar.movement.update(dt, window.innerWidth);
      }
      avatar.render();
    }

    this.rafHandle = requestAnimationFrame((t) => this.tick(t));
  }

  stop(): void {
    if (this.rafHandle !== null) {
      cancelAnimationFrame(this.rafHandle);
      this.rafHandle = null;
    }
  }
}
