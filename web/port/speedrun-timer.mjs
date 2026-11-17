// Real elapsed time: pausing, deaths, menus and stage transitions never stop it.
export class SpeedrunTimer {
    constructor() { this.reset(); }
    reset(stage = 1) { this.startStage = stage; this.startedAt = null; this.finishedAt = null; }
    start(now) { if (this.startedAt === null) this.startedAt = now; }
    finish(now) { if (this.startedAt !== null && this.finishedAt === null) this.finishedAt = Math.max(this.startedAt, now); }
    elapsed(now) { return this.startedAt === null ? 0 : Math.max(0, (this.finishedAt ?? now) - this.startedAt); }
}
