import type { AgentState } from "./agent";
import type { AgentDisplayConfig } from "./config";
import { AVATAR_SIZE } from "./config";
import { Movement } from "./movement";

export class Avatar {
  readonly id: string;
  readonly movement: Movement;

  private readonly root: HTMLDivElement;
  private readonly sprite: HTMLDivElement;
  private readonly glyph: HTMLSpanElement;
  private state: AgentState = "active";

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

    this.render();
  }

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
    this.root.style.transform = `translateX(${this.movement.x}px)`;
    this.glyph.classList.toggle("facing-left", this.movement.direction === "left");
    this.sprite.classList.toggle("is-walking", this.movement.isWalking);
  }

  destroy(): void {
    this.root.remove();
  }
}
