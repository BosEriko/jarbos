import { invoke } from "@tauri-apps/api/core";

import type { AgentState } from "./agent";
import type { AgentDisplayConfig } from "./config";
import { AVATAR_SIZE } from "./config";
import { Movement } from "./movement";

const GRAVITY_PX_PER_MS2 = 0.0015;

export interface AvatarRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export class Avatar {
  readonly id: string;
  readonly movement: Movement;

  private readonly root: HTMLDivElement;
  private readonly sprite: HTMLDivElement;
  private readonly glyph: HTMLSpanElement;
  private state: AgentState = "active";

  private liftY = 0;
  private dragging = false;
  private falling = false;
  private fallVelocity = 0;
  private dragPointerId: number | null = null;
  private dragStartClientX = 0;
  private dragStartClientY = 0;
  private dragStartX = 0;
  private dragStartLiftY = 0;

  constructor(config: AgentDisplayConfig, container: HTMLElement, startX: number) {
    this.id = config.id;
    this.movement = new Movement(startX, AVATAR_SIZE);

    this.root = document.createElement("div");
    this.root.className = "avatar";
    this.root.style.setProperty("--accent", config.accentColor);

    const label = document.createElement("div");
    label.className = "avatar-label";
    label.textContent = config.name;

    this.sprite = document.createElement("div");
    this.sprite.className = "avatar-sprite";

    this.glyph = document.createElement("span");
    this.glyph.className = "avatar-glyph";
    this.glyph.textContent = config.glyph;
    this.sprite.appendChild(this.glyph);

    this.root.append(label, this.sprite);
    container.appendChild(this.root);

    this.root.addEventListener("pointerdown", this.handlePointerDown);
    this.root.addEventListener("pointermove", this.handlePointerMove);
    this.root.addEventListener("pointerup", this.handlePointerUp);
    this.root.addEventListener("pointercancel", this.handlePointerUp);

    this.render();
  }

  get isBusy(): boolean {
    return this.dragging || this.falling;
  }

  getRect(): AvatarRect {
    const rect = this.root.getBoundingClientRect();
    return { x: rect.left, y: rect.top, width: rect.width, height: rect.height };
  }

  updatePhysics(dtMs: number): void {
    if (!this.falling) {
      return;
    }
    this.fallVelocity += GRAVITY_PX_PER_MS2 * dtMs;
    this.liftY -= this.fallVelocity * dtMs;
    if (this.liftY <= 0) {
      this.liftY = 0;
      this.falling = false;
    }
  }

  private handlePointerDown = (event: PointerEvent): void => {
    event.preventDefault();
    this.root.setPointerCapture(event.pointerId);
    this.dragging = true;
    this.falling = false;
    this.fallVelocity = 0;
    this.dragPointerId = event.pointerId;
    this.dragStartClientX = event.clientX;
    this.dragStartClientY = event.clientY;
    this.dragStartX = this.movement.x;
    this.dragStartLiftY = this.liftY;
    this.root.classList.add("is-dragging");
    void invoke("begin_drag");
  };

  private handlePointerMove = (event: PointerEvent): void => {
    if (!this.dragging || event.pointerId !== this.dragPointerId) {
      return;
    }
    const dx = event.clientX - this.dragStartClientX;
    const dy = event.clientY - this.dragStartClientY;
    const maxX = Math.max(0, window.innerWidth - AVATAR_SIZE);
    this.movement.x = Math.min(Math.max(this.dragStartX + dx, 0), maxX);
    this.liftY = Math.max(0, this.dragStartLiftY - dy);
    this.render();
  };

  private handlePointerUp = (event: PointerEvent): void => {
    if (!this.dragging || event.pointerId !== this.dragPointerId) {
      return;
    }
    this.dragging = false;
    this.falling = true;
    this.fallVelocity = 0;
    this.dragPointerId = null;
    this.root.classList.remove("is-dragging");
    this.root.releasePointerCapture(event.pointerId);
    void invoke("end_drag");
  };

  applyState(state: AgentState): void {
    this.state = state;
    this.root.dataset.state = state;
    if (state === "hidden") {
      this.root.classList.add("is-hidden");
    } else {
      this.root.classList.remove("is-hidden");
    }
  }

  get currentState(): AgentState {
    return this.state;
  }

  render(): void {
    this.root.style.transform = `translateX(${this.movement.x}px) translateY(${-this.liftY}px)`;
    this.glyph.classList.toggle("facing-left", this.movement.direction === "left");
    this.sprite.classList.toggle("is-walking", this.movement.isWalking && !this.isBusy);
  }

  destroy(): void {
    this.root.remove();
  }
}
