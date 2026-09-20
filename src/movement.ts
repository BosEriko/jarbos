export type Direction = "left" | "right";

export interface MovementConfig {
  minSpeed: number;
  maxSpeed: number;
  minPauseMs: number;
  maxPauseMs: number;
  minWalkMs: number;
  maxWalkMs: number;
  turnChance: number;
}

const DEFAULT_CONFIG: MovementConfig = {
  minSpeed: 30,
  maxSpeed: 70,
  minPauseMs: 1000,
  maxPauseMs: 4000,
  minWalkMs: 1500,
  maxWalkMs: 5000,
  turnChance: 0.3,
};

type Phase = "walking" | "paused";

function randomBetween(min: number, max: number): number {
  return min + Math.random() * (max - min);
}

export class Movement {
  x: number;
  direction: Direction;

  private speed: number;
  private phase: Phase;
  private phaseRemainingMs: number;
  private avatarWidth: number;
  private readonly config: MovementConfig;

  constructor(
    startX: number,
    avatarWidth: number,
    config: Partial<MovementConfig> = {},
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.x = startX;
    this.avatarWidth = avatarWidth;
    this.direction = Math.random() < 0.5 ? "left" : "right";
    this.speed = randomBetween(this.config.minSpeed, this.config.maxSpeed);
    this.phase = "paused";
    this.phaseRemainingMs = randomBetween(
      this.config.minPauseMs,
      this.config.maxPauseMs,
    );
  }

  get isWalking(): boolean {
    return this.phase === "walking";
  }

  setWidth(width: number): void {
    this.avatarWidth = width;
  }

  update(dtMs: number, viewportWidth: number): void {
    const maxX = Math.max(0, viewportWidth - this.avatarWidth);
    this.x = Math.min(this.x, maxX);

    this.phaseRemainingMs -= dtMs;
    if (this.phaseRemainingMs <= 0) {
      this.beginNextPhase();
    }

    if (this.phase !== "walking") {
      return;
    }

    const deltaX = (this.speed * dtMs) / 1000;
    const nextX = this.direction === "right" ? this.x + deltaX : this.x - deltaX;

    if (nextX <= 0) {
      this.x = 0;
      this.direction = "right";
      this.beginWalking();
    } else if (nextX >= maxX) {
      this.x = maxX;
      this.direction = "left";
      this.beginWalking();
    } else {
      this.x = nextX;
    }
  }

  private beginNextPhase(): void {
    if (this.phase === "paused") {
      this.beginWalking();
    } else {
      this.phase = "paused";
      this.phaseRemainingMs = randomBetween(
        this.config.minPauseMs,
        this.config.maxPauseMs,
      );
    }
  }

  private beginWalking(): void {
    this.phase = "walking";
    this.speed = randomBetween(this.config.minSpeed, this.config.maxSpeed);
    if (Math.random() < this.config.turnChance) {
      this.direction = this.direction === "left" ? "right" : "left";
    }
    this.phaseRemainingMs = randomBetween(
      this.config.minWalkMs,
      this.config.maxWalkMs,
    );
  }
}
