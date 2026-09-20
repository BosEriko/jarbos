import { invoke } from "@tauri-apps/api/core";

import type { AgentState } from "./agent";
import { Avatar } from "./avatar";
import { AGENT_CONFIGS, AVATAR_SIZE, FADE_OUT_MS } from "./config";
import { onAgentStateChanged } from "./agentStore";

const RECT_REPORT_INTERVAL_MS = 100;

export class AvatarManager {
  private readonly avatars = new Map<string, Avatar>();
  private readonly hideTimers = new Map<string, number>();
  private readonly container: HTMLElement;
  private lastFrameTime: number | null = null;
  private rafHandle: number | null = null;
  private rectReportAccumulatorMs = 0;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  start(): void {
    onAgentStateChanged((instanceId, agentId, state) =>
      this.applyState(instanceId, agentId, state),
    );
    this.rafHandle = requestAnimationFrame((t) => this.tick(t));
  }

  applyState(instanceId: string, agentId: string, state: AgentState): void {
    const config = AGENT_CONFIGS.find((c) => c.id === agentId);
    if (!config) {
      return;
    }

    const existingTimer = this.hideTimers.get(instanceId);
    if (existingTimer !== undefined) {
      clearTimeout(existingTimer);
      this.hideTimers.delete(instanceId);
    }

    if (state === "hidden") {
      const avatar = this.avatars.get(instanceId);
      if (!avatar) {
        return;
      }
      avatar.applyState("hidden");
      const timer = window.setTimeout(() => {
        avatar.destroy();
        this.avatars.delete(instanceId);
        this.hideTimers.delete(instanceId);
      }, FADE_OUT_MS);
      this.hideTimers.set(instanceId, timer);
      return;
    }

    let avatar = this.avatars.get(instanceId);
    if (!avatar) {
      const startX = Math.random() * Math.max(0, window.innerWidth - AVATAR_SIZE);
      avatar = new Avatar(config, this.container, startX);
      this.avatars.set(instanceId, avatar);
    }
    avatar.applyState(state);
  }

  private tick(now: number): void {
    const dt = this.lastFrameTime === null ? 0 : now - this.lastFrameTime;
    this.lastFrameTime = now;

    for (const avatar of this.avatars.values()) {
      avatar.updatePhysics(dt);
      if (avatar.currentState === "active" && !avatar.isBusy) {
        avatar.movement.update(dt, window.innerWidth);
      }
      avatar.render();
    }

    this.rectReportAccumulatorMs += dt;
    if (this.rectReportAccumulatorMs >= RECT_REPORT_INTERVAL_MS) {
      this.rectReportAccumulatorMs = 0;
      const rects = Array.from(this.avatars.values()).map((avatar) => avatar.getRect());
      void invoke("update_avatar_rects", { rects });
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
