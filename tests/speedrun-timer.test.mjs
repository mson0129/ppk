import assert from 'node:assert/strict';
import test from 'node:test';
import { SpeedrunTimer } from '../web/port/speedrun-timer.mjs';
export const checks = [];
function check(name, fn) { checks.push([name, fn]); test(name, fn); }

check('title waiting stays zero; Space timestamp is the start, not the next game tick', () => {
    const timer = new SpeedrunTimer();
    assert.equal(timer.elapsed(9000), 0);
    timer.start(9012.25);
    assert.equal(timer.elapsed(9060), 47.75);
    timer.start(9100); // repeat keydown cannot restart timing
    assert.equal(timer.elapsed(9212.25), 200);
});
check('real elapsed time includes pause, deaths, continues and inter-stage waits', () => {
    const timer = new SpeedrunTimer(); timer.start(1000);
    // No phase or pause operation can subtract real elapsed time.
    for (const now of [1500, 6000, 31000, 91000]) assert.equal(timer.elapsed(now), now - 1000);
});
check('last candy timestamp freezes the result through bonus screen and stage wrap', () => {
    const timer = new SpeedrunTimer(); timer.start(1000); timer.finish(250123.75);
    assert.equal(timer.elapsed(250200), 249123.75);
    assert.equal(timer.elapsed(999999), 249123.75);
    timer.finish(999999);
    assert.equal(timer.elapsed(1000000), 249123.75);
});
check('stage selection resets the attempt; non-1 starts are distinguishable practice runs', () => {
    const timer = new SpeedrunTimer(); timer.start(1000); timer.finish(2000);
    timer.reset(50); assert.equal(timer.startStage, 50); assert.equal(timer.elapsed(5000), 0);
    timer.start(6000); assert.equal(timer.elapsed(6100), 100);
    timer.reset(); assert.equal(timer.startStage, 1); assert.equal(timer.elapsed(7000), 0);
});
