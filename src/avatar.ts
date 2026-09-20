import { invoke } from "@tauri-apps/api/core";

import type { AgentState } from "./agent";
import type { AgentDisplayConfig } from "./config";
import { AVATAR_SIZE, DUCK_NAMES } from "./config";
import { Movement } from "./movement";

const GRAVITY_PX_PER_MS2 = 0.0015;

const DUCK_SVG = `<svg viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg" class="avatar-duck">
  <ellipse class="duck-shadow" cx="30" cy="61" rx="20" ry="2.5" fill="rgba(0,0,0,0.22)" />
  <path class="duck-foot duck-foot-back" d="M19 53 H22 V57 L26 57 Q28 57 27 58.5 Q31 58 29 60 Q31 62 27 62 H19 Q16 62 17 60 L19 58 Z" fill="#e58b1b" />
  <path class="duck-foot duck-foot-front" d="M33 53 H36 V57 L40 57 Q42 57 41 58.5 Q45 58 43 60 Q45 62 41 62 H33 Q30 62 31 60 L33 58 Z" fill="#f5a623" />
  <g class="duck-body">
  <ellipse cx="30" cy="42" rx="22" ry="15" fill="currentColor" />
  <circle cx="44" cy="24" r="13" fill="currentColor" />
  <path d="M55 21 Q66 23 57 28 Q52 26 55 21 Z" fill="#f5a623" />
  <circle cx="47" cy="20" r="2.2" fill="#222" />
  <ellipse class="duck-wing" cx="24" cy="44" rx="9" ry="6" fill="rgba(0,0,0,0.16)" />
  </g>
</svg>`;

function randomDuckName(): string {
  return DUCK_NAMES[Math.floor(Math.random() * DUCK_NAMES.length)];
}

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
  private readonly duck: SVGSVGElement;
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
    this.root.dataset.state = this.state;
    this.root.style.setProperty("--accent", config.accentColor);

    const label = document.createElement("div");
    label.className = "avatar-label";
    label.textContent = randomDuckName();

    this.sprite = document.createElement("div");
    this.sprite.className = "avatar-sprite";
    this.sprite.innerHTML = DUCK_SVG;
    this.duck = this.sprite.querySelector("svg") as SVGSVGElement;

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
    this.duck.classList.toggle("facing-left", this.movement.direction === "left");
    this.sprite.classList.toggle("is-walking", this.movement.isWalking && !this.isBusy);
  }

  destroy(): void {
    this.root.remove();
  }
}
