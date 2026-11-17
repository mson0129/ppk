// W3C Standard Gamepad layout; common unmapped pads also use the first two axes.
export class GamepadInput {
    constructor(deadzone = 0.3) { this.deadzone = deadzone; this.previousKeys = new Set(); this.previousPause = false; }
    update(pads, enabled = true) {
        const keys = new Set(); let pause = false, connected = 0;
        for (const pad of pads || []) {
            if (!pad || pad.connected === false || pad.mapping === 'xr-standard') continue;
            connected++;
            if (!enabled) continue;
            const down = index => !!pad.buttons?.[index]?.pressed || (pad.buttons?.[index]?.value || 0) > 0.5;
            const x = Number(pad.axes?.[0]) || 0, y = Number(pad.axes?.[1]) || 0;
            if (down(14) || x < -this.deadzone) keys.add(75);
            if (down(15) || x > this.deadzone) keys.add(77);
            if (down(12) || y < -this.deadzone) keys.add(72);
            if (down(13) || y > this.deadzone) keys.add(80);
            if (down(0) || down(1)) keys.add(57); // A AND B both map to Space.
            pause ||= down(9);
        }
        const pressed = new Set([...keys].filter(key => !this.previousKeys.has(key)));
        const pausePressed = pause && !this.previousPause;
        this.previousKeys = keys; this.previousPause = pause;
        return { keys, pressed, pausePressed, connected };
    }
}
