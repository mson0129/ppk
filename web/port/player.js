import { SpeedrunTimer } from './speedrun-timer.mjs';
import { GamepadInput } from './gamepad-input.mjs';
const canvas = document.getElementById('game');
const ctx = canvas.getContext('2d', { alpha: false });
const status = document.getElementById('status');
const start = document.getElementById('start');
const overlay = document.getElementById('overlay');
const pauseButton = document.getElementById('pause');
const restartButton = document.getElementById('restart');
const soundButton = document.getElementById('sound');
const stageSelect = document.getElementById('stage-select');
const playTimer = document.getElementById('play-timer');
const fullscreenTimer = document.getElementById('fullscreen-play-timer');
const speedrun = new SpeedrunTimer();
const timerMode = document.getElementById('timer-mode');
let timerUpdatedAt = 0;
const inputHelp = document.getElementById('input-help');
const helpDialog = document.getElementById('controls-help');
const helpTitle = document.getElementById('controls-help-title');
const helpBody = document.getElementById('controls-help-body');
const helpClose = document.getElementById('controls-help-close');
let connectedGamepad = false;
const inputIcons = {
    keyboard: '<rect x="2" y="5" width="20" height="14" rx="2"/><path d="M5 9h1m3 0h1m3 0h1m3 0h1M5 12h1m3 0h1m3 0h1m3 0h1M7 16h10"/>',
    gamepad: '<path d="M7 7h10c3 0 5 10 3 12-1 1-3-1-5-3H9c-2 2-4 4-5 3-2-2 0-12 3-12Z"/><path d="M8 9v6M5 12h6M16 11h.01M19 14h.01"/>'
};
function renderHelp() {
    helpTitle.textContent = connectedGamepad ? '게임패드 조작법' : '키보드 조작법';
    helpBody.innerHTML = connectedGamepad
        ? '<dl><dt>방향키 · 왼쪽 스틱</dt><dd>이동 / 사다리 타기</dd><dt>A · B</dt><dd>시작 / 점프 / 이어하기</dd><dt>Start</dt><dd>일시정지 / 재개</dd></dl><p>타이틀에서 A 또는 B를 누르면 타임어택이 시작됩니다.</p>'
        : '<dl><dt>방향키 ↑ ↓ ← →</dt><dd>이동 / 사다리 타기</dd><dt>Space</dt><dd>시작 / 점프 / 이어하기</dd><dt>P</dt><dd>일시정지 / 재개</dd></dl><p>터치 화면에서는 십자키와 SPACE 버튼으로 조작할 수 있습니다.</p>';
    helpBody.innerHTML += '<p>홈 화면에 추가: iPhone은 Safari 공유 → 홈 화면에 추가, Android는 브라우저 메뉴 → 앱 설치를 선택하세요. 온라인에서 오프라인 저장을 완료한 뒤에는 인터넷 없이 실행할 수 있습니다.</p>';
}
function updateInputHelp(connected) {
    if (connectedGamepad === connected) return;
    connectedGamepad = connected;
    inputHelp.querySelector('svg').innerHTML = inputIcons[connected ? 'gamepad' : 'keyboard'];
    const label = connected ? '게임패드 연결됨 · 게임패드 조작법 보기' : '키보드 조작법 보기';
    inputHelp.setAttribute('aria-label', label); inputHelp.title = label;
    if (helpDialog.open) renderHelp();
}
inputHelp.addEventListener('click', () => { releaseInputs(); renderHelp(); helpDialog.showModal(); if (game) updateSound(); helpClose.focus(); });
helpClose.addEventListener('click', () => helpDialog.close());
helpDialog.addEventListener('close', () => { releaseInputs(); (running && !paused ? canvas : inputHelp).focus({preventScroll:true}); if (game) updateSound(); });

function acceptStart(scan) {
    if (scan === 57 && running && !paused && game.get_phase() === 3) {
        speedrun.start(performance.now());
        updateTimer(performance.now(), true);
    }
}
const keyMap = { ArrowLeft: 75, ArrowRight: 77, ArrowUp: 72, ArrowDown: 80, Space: 57, Enter: 28 };
const touchButtons = [...document.querySelectorAll('[data-key]')];
const pointerKeys = new Map();
const keyboardKeys = new Set();
const pendingKeys = new Set();
const gamepadInput = new GamepadInput();
let gamepadKeys = new Set();
let gamepadNeedsNeutral = false;
function syncKeys() {
    if (!game) return;
    for (const scan of new Set(Object.values(keyMap))) {
        const held = running && !paused && (pendingKeys.has(scan) || gamepadKeys.has(scan) || keyboardKeys.has(scan) || [...pointerKeys.values()].includes(scan));
        game.key(scan, held ? 1 : 0);
        for (const button of touchButtons) if (Number(button.dataset.key) === scan) {
            button.classList.toggle('is-held', held);
            if (button.getAttribute('aria-pressed') !== String(held)) button.setAttribute('aria-pressed', String(held));
        }
    }
}
function releaseInputs() {
    keyboardKeys.clear(); pointerKeys.clear(); pendingKeys.clear(); gamepadKeys.clear(); gamepadNeedsNeutral = true;
    if (game) game.release_keys();
    syncKeys();
}
let pixels;
let game, running = false, paused = false, sound = true, last = 0, accumulator = 0;
let audio, oscillator, gain, deathBuffer, deathSource;
let lastAudioEvent = 0, lastHz = -1;
function setStatus(text) { if (status.textContent !== text) status.textContent = text; }
let metricStart = performance.now(), renderCount = 0, tickCount = 0, maxTickMs = 0, maxFrameMs = 0;
// Original game loop waits for the BIOS 18.2065 Hz clock (PIT / 65536).
const stepMs = 1000 * 65536 / 1193182;
function enableAudio() {
    if (!audio) {
        audio = new AudioContext();
        oscillator = audio.createOscillator();
        oscillator.type = 'square';
        gain = audio.createGain(); gain.gain.value = 0;
        oscillator.connect(gain).connect(audio.destination); oscillator.start();
    }
    audio.resume().catch(() => {});
}
function updateSound() {
    const event = game.audio_event();
    if (event !== lastAudioEvent) {
        lastAudioEvent = event;
        if (audio && sound) {
            if (!deathBuffer) {
                const packed = new Uint8Array(game.memory.buffer, game.data_pointer() + 0xf0a0 + 0x15f0, 160);
                deathBuffer = audio.createBuffer(1, packed.length * 8, 6000);
                const data = deathBuffer.getChannelData(0);
                for (let i = 0; i < data.length; i++) data[i] = ((packed[i >> 3] >> (7 - (i & 7))) & 1) ? 0.1 : -0.1;
            }
            deathSource?.stop(); deathSource = audio.createBufferSource();
            deathSource.buffer = deathBuffer; deathSource.connect(audio.destination); deathSource.start();
        }
    }
    if (!audio) return;
    const hz = running && !paused && !helpDialog.open && sound ? game.audio_frequency() : 0;
    if (hz === lastHz) return;
    lastHz = hz;
    oscillator.frequency.setValueAtTime(Math.max(1, Math.min(hz, 18000)), audio.currentTime);
    gain.gain.setTargetAtTime(hz ? 0.035 : 0, audio.currentTime, 0.004);
}
function draw(fraction = 1000) {
    game.render_interpolated(fraction);
    ctx.putImageData(pixels, 0, 0);
}
const icons = {
    pause: '<path d="M8 5v14M16 5v14"/>',
    play: '<path d="m8 4 12 8-12 8V4Z"/>',
    sound: '<path d="M11 5 6 9H3v6h3l5 4V5ZM15 8a6 6 0 0 1 0 8M18 5a10 10 0 0 1 0 14"/>',
    mute: '<path d="M11 5 6 9H3v6h3l5 4V5ZM16 9l6 6M22 9l-6 6"/>'
};
function setIcon(button, name, label) {
    button.querySelector('svg').innerHTML = icons[name];
    button.setAttribute('aria-label', label); button.title = label;
}
function updateTimer(now = performance.now(), force = false) {
    if (!force && now - timerUpdatedAt < 100) return;
    const ms = Math.floor(speedrun.elapsed(now));
    const text = `${String(Math.floor(ms / 60000)).padStart(2, '0')}:${String(Math.floor(ms / 1000) % 60).padStart(2, '0')}.${String(ms % 1000).padStart(3, '0')}`;
    if (playTimer.textContent !== text) playTimer.textContent = text;
    if (fullscreenTimer.textContent !== text) fullscreenTimer.textContent = text;
    timerUpdatedAt = now;
}
function setPaused(value) {
    paused = value; releaseInputs(); accumulator = 0; last = performance.now();
    setIcon(pauseButton, paused ? 'play' : 'pause', paused ? '계속하기' : '일시정지');
    setStatus(paused ? '일시정지됨' : '방향키로 이동하고 Space로 점프하세요.');
    updateSound(); updateTimer(performance.now(), true); if (!paused) canvas.focus({preventScroll:true});
}
function begin() {
    releaseInputs(); gamepadNeedsNeutral = true;
    game.start_stage(Number(stageSelect.value) || 1);
    game.show_intro();
    speedrun.reset(Number(stageSelect.value) || 1); timerMode.textContent = speedrun.startStage === 1 ? '타임어택' : '연습 시간'; updateTimer(performance.now(), true); lastAudioEvent = 0; deathSource?.stop(); deathSource = null; running = true; paused = false; accumulator = 0; last = performance.now();
    touchButtons.forEach(button => { button.disabled = false; });
    overlay.hidden = true; pauseButton.disabled = restartButton.disabled = false;
    setIcon(pauseButton, 'pause', '일시정지');
    setStatus('인트로 화면입니다. Space를 눌러 선택한 스테이지를 시작하세요.');
    enableAudio(); draw(); canvas.focus({preventScroll:true});
}
function pollGamepads() {
    let pads = [];
    try { pads = navigator.getGamepads?.() || []; } catch {}
    const enabled = !document.hidden && document.hasFocus();
    const input = gamepadInput.update(pads, enabled);
    updateInputHelp(input.connected > 0);
    if (helpDialog.open) { releaseInputs(); return; }
    if (!enabled) { gamepadNeedsNeutral = true; gamepadKeys.clear(); syncKeys(); return; }
    if (gamepadNeedsNeutral) {
        gamepadKeys.clear();
        if (!input.keys.size && !gamepadInput.previousPause) gamepadNeedsNeutral = false;
        syncKeys(); return;
    }
    if (!game || !game.memory) return;
    if (!running) {
        if (input.pressed.has(57) && !start.disabled) begin();
        return;
    }
    if (input.pausePressed) { setPaused(!paused); gamepadNeedsNeutral = true; return; }
    gamepadKeys = paused ? new Set() : input.keys;
    if (!paused) for (const scan of input.pressed) { acceptStart(scan); pendingKeys.add(scan); }
    syncKeys();
}
window.addEventListener('gamepaddisconnected', () => { gamepadKeys.clear(); syncKeys(); });
function frame(now) {
    pollGamepads();
    if (running && !paused && !helpDialog.open) {
        const beforePhase = game.get_phase();
        accumulator += Math.min(now - last, 1000);
        while (accumulator >= stepMs) {
            const tickStart = performance.now(), tickPhase = game.get_phase();
            syncKeys(); game.tick();
            if (game.run_finished()) speedrun.finish(performance.now());
            if (tickPhase === 2 && game.get_phase() === 3) {
                speedrun.reset(1); timerMode.textContent = '타임어택';
                releaseInputs(); // A held continue key must not skip the new title/start boundary.
            }
            pendingKeys.clear(); syncKeys();
            maxTickMs = Math.max(maxTickMs, performance.now() - tickStart);
            tickCount++; accumulator -= stepMs;
        }
        if (game.get_error()) {
            running = false; setStatus(`실행 오류: ${game.get_error().toString(16)}`);
        } else if (game.get_phase() === 3) {
            setStatus('인트로 화면입니다. Space를 눌러 선택한 스테이지를 시작하세요.');
        } else if (game.get_phase() === 2) {
            setStatus(game.debug_state(0x159a) > 0 ? `이어하기 ${game.debug_state(0x159a)}회 · Space로 현재 스테이지 계속` : '게임 오버 · Space로 인트로 돌아가기');
        } else if (game.get_phase() === 4) {
            const stage = game.get_stage(), next = stage % game.stage_count() + 1;
            setStatus(game.run_finished() ? `50 스테이지 클리어! ${speedrun.startStage === 1 ? '타임어택' : '연습'} 기록 종료 · Space로 1 스테이지` : `스테이지 ${stage} 클리어! Space로 ${next} 스테이지 진행`);
        } else {
            setStatus(`스테이지 ${game.get_stage()} · 방향키 이동 / Space 점프`);
        }
        if (stageSelect.value !== String(game.get_stage())) stageSelect.value = String(game.get_stage());
        if (beforePhase !== game.get_phase()) updateTimer(performance.now(), true);
        draw(Math.round(accumulator / stepMs * 1000)); updateSound();
    }
    updateTimer(performance.now());
    maxFrameMs = Math.max(maxFrameMs, now - last); renderCount++;
    if (now - metricStart >= 1000) { canvas.dataset.performance = JSON.stringify({fps: +(renderCount * 1000 / (now - metricStart)).toFixed(1), ticks: tickCount, maxTickMs, maxFrameMs}); metricStart = now; renderCount = tickCount = maxTickMs = maxFrameMs = 0; }
    last = now; requestAnimationFrame(frame);
}
start.addEventListener('click', begin);
restartButton.addEventListener('click', begin);
stageSelect.addEventListener('change', begin);
pauseButton.addEventListener('click', () => setPaused(!paused));
soundButton.addEventListener('click', () => {
    sound = !sound; if (!sound) { deathSource?.stop(); deathSource = null; } setIcon(soundButton, sound ? 'sound' : 'mute', sound ? '소리 끄기' : '소리 켜기');
    soundButton.setAttribute('aria-pressed', String(sound)); if (sound) enableAudio(); updateSound();
});
document.getElementById('fullscreen').addEventListener('click', async () => {
    try { if (document.fullscreenElement) await document.exitFullscreen(); else await document.getElementById('screen').requestFullscreen(); }
    catch { setStatus('이 브라우저에서는 전체 화면을 열 수 없습니다.'); }
});
window.addEventListener('keydown', event => {
    if (helpDialog.open || !running || event.target.closest?.('button,select,input,a')) return;
    if (event.code === 'KeyP' && !event.repeat) { event.preventDefault(); setPaused(!paused); return; }
    if (keyMap[event.code] !== undefined) { event.preventDefault(); if (!paused) { acceptStart(keyMap[event.code]); keyboardKeys.add(keyMap[event.code]); if (!event.repeat) pendingKeys.add(keyMap[event.code]); syncKeys(); } }
});
window.addEventListener('keyup', event => {
    if (game && keyMap[event.code] !== undefined) { event.preventDefault(); keyboardKeys.delete(keyMap[event.code]); syncKeys(); }
});
// Prevent native selection/callout from stealing a long-held touch gesture.
const appRoot = document.querySelector('main');
document.addEventListener('selectstart', event => {
    if (appRoot.contains(event.target)) event.preventDefault();
});
document.addEventListener('selectionchange', () => {
    const selection = window.getSelection();
    if (selection?.rangeCount && appRoot.contains(selection.anchorNode)) selection.removeAllRanges();
});
appRoot.addEventListener('contextmenu', event => {
    if (!event.target.closest('select,option')) event.preventDefault();
});
appRoot.addEventListener('dragstart', event => event.preventDefault());
window.addEventListener('blur', () => { releaseInputs(); gamepadNeedsNeutral = true; });
document.addEventListener('visibilitychange', () => { if (running && document.hidden && !paused) setPaused(true); });
const controller = document.querySelector('.touch');
const controllerHandle = document.querySelector('.controller-handle');
let controllerDrag = null;
let controllerPosition = 1;
try { const saved = localStorage.getItem('ppk-controller-position'); if (saved !== null && Number.isFinite(Number(saved))) controllerPosition = Math.max(0, Math.min(1, Number(saved))); } catch {}
function controllerBounds() {
    const viewport = window.visualViewport;
    const min = (viewport?.offsetTop || 0) + 8;
    const max = Math.max(min, (viewport?.offsetTop || 0) + (viewport?.height || innerHeight) - controller.getBoundingClientRect().height - 16);
    return {min, max};
}
function moveController(top, save = false) {
    if (!controller.getBoundingClientRect().height) return;
    const {min, max} = controllerBounds();
    top = Math.max(min, Math.min(max, top));
    controllerPosition = max > min ? (top - min) / (max - min) : 1;
    controller.style.top = `${top}px`; controller.style.bottom = 'auto';
    controllerHandle.setAttribute('aria-valuenow', String(Math.round(controllerPosition * 100)));
    controllerHandle.setAttribute('aria-valuetext', `화면 위에서 ${Math.round(controllerPosition * 100)}%`);
    if (save) { try { localStorage.setItem('ppk-controller-position', String(controllerPosition)); } catch {} }
}
function fitController() {
    const {min, max} = controllerBounds();
    moveController(min + controllerPosition * (max - min));
}
controllerHandle.addEventListener('pointerdown', event => {
    if (event.pointerType === 'mouse' && event.button !== 0) return;
    event.preventDefault(); releaseInputs();
    controllerDrag = {id: event.pointerId, y: event.clientY, top: controller.getBoundingClientRect().top};
    controllerHandle.setPointerCapture(event.pointerId); controller.classList.add('is-dragging');
});
controllerHandle.addEventListener('pointermove', event => {
    if (controllerDrag?.id !== event.pointerId) return;
    event.preventDefault(); moveController(controllerDrag.top + event.clientY - controllerDrag.y);
});
function endControllerDrag(event) {
    if (controllerDrag?.id !== event.pointerId) return;
    controllerDrag = null; controller.classList.remove('is-dragging');
    moveController(controller.getBoundingClientRect().top, true);
}
for (const event of ['pointerup', 'pointercancel', 'lostpointercapture']) controllerHandle.addEventListener(event, endControllerDrag);
controllerHandle.addEventListener('keydown', event => {
    if (!['ArrowUp','ArrowDown','Home','End'].includes(event.code)) return;
    event.preventDefault(); event.stopPropagation();
    const {min,max} = controllerBounds();
    const current = controller.getBoundingClientRect().top;
    moveController(event.code === 'Home' ? min : event.code === 'End' ? max : current + (event.code === 'ArrowUp' ? -16 : 16), true);
});
window.addEventListener('resize', fitController);
window.visualViewport?.addEventListener('resize', fitController);
document.addEventListener('fullscreenchange', fitController);
requestAnimationFrame(fitController);

function releasePointer(event) {
    if (pointerKeys.delete(event.pointerId)) syncKeys();
}
for (const button of touchButtons) {
    button.addEventListener('contextmenu', event => event.preventDefault());
    button.addEventListener('pointerdown', event => {
        event.preventDefault();
        if (!running || paused || (event.pointerType === 'mouse' && event.button !== 0)) return;
        button.setPointerCapture(event.pointerId);
        pointerKeys.set(event.pointerId, Number(button.dataset.key));
        acceptStart(Number(button.dataset.key));
        pendingKeys.add(Number(button.dataset.key));
        syncKeys(); enableAudio();
    });
    button.addEventListener('pointermove', event => {
        if (!pointerKeys.has(event.pointerId) || !button.closest('.dpad')) return;
        const target = document.elementFromPoint(event.clientX, event.clientY)?.closest('.dpad [data-key]');
        pointerKeys.set(event.pointerId, target ? Number(target.dataset.key) : 0);
        syncKeys();
    });
    for (const event of ['pointerup', 'pointercancel', 'lostpointercapture']) button.addEventListener(event, releasePointer);
}
window.addEventListener('pointerup', releasePointer);
window.addEventListener('pointercancel', releasePointer);

try {
    const response = await fetch(new URL('./ppk.wasm', import.meta.url));
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const bytes = await response.arrayBuffer();
    const result = await WebAssembly.instantiate(bytes, {});
    game = result.instance.exports;
    for (let stage = 1; stage <= game.stage_count(); stage++) {
        const option = document.createElement('option'); option.value = String(stage); option.textContent = String(stage).padStart(2, '0'); stageSelect.append(option);
    }
    stageSelect.disabled = false;
    pixels = new ImageData(new Uint8ClampedArray(game.memory.buffer, game.framebuffer(), 320 * 200 * 4), 320, 200);
    game.init(); game.show_intro(); draw(); start.disabled = false; start.textContent = '게임 시작';
    setStatus('게임 시작을 누르세요.'); requestAnimationFrame(frame);
} catch (error) {
    setStatus(`게임을 불러오지 못했습니다: ${error.message}`);
    start.textContent = '불러오기 실패'; console.error(error);
}
