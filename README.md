# PPK 50-stage Rust WebAssembly port

The browser game is now built from human-maintained Rust, organized around named
player/enemy/stage state and ordinary functions. It does **not** compile or link
C, execute PPK.EXE, or include DOSBox, x86 instruction decoding, virtual registers,
CPU flags, or a VGA emulator. The WASM module has **zero imports**. JavaScript
continues to supply input, Web Audio, the fixed clock, Canvas and replay storage.

The previous statically recompiled C engine remains in `src/` as an independent
oracle. It is not part of the default build. Both engines are compared with the
original executable offline; that validation machinery never ships to browsers.

- `rust/src/`: readable Rust gameplay, lifecycle, renderer, audio and palette fades.
- `rust/assets/`: original data assets embedded with `include_bytes!`.
- `src/`: frozen C reference implementation and generated C headers.
- `original/`, `analysis/`: supplied DOS distribution, disassembly and verification results.
- `tools/`, `tests/`: reproducible builds and independent differential tests.
- `web/`: canonical browser UI, replay and time-attack source.
- `dist/`: runtime-only static website, including Rust-built `port/ppk.wasm`.

Start reading with [rust/README.md](rust/README.md), then
[rust/src/state.rs](rust/src/state.rs), [rust/src/player.rs](rust/src/player.rs)
and [rust/src/engine.rs](rust/src/engine.rs).

The EXE is the supplied original binary. This project builds the WASM port;
it does not claim to rebuild the original DOS executable from C.

## Build

Install Rust and its WebAssembly standard library (no Cargo dependencies are needed).
The verified compiler is rustc 1.99.1. From the repository root:

```sh
rustup target add wasm32-unknown-unknown
./build.sh
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

The default build uses Cargo with the committed lockfile. No WASI SDK or C compiler
is needed for it. For an optional independent C reference build, use the official
WASI SDK with `WASI_SDK_PATH=/path/to/wasi-sdk sh tools/build-c-reference.sh`.
That writes `analysis/c-reference.wasm`, leaving the Rust web build untouched.

Open `http://127.0.0.1:8081/index.htm`. Static hosting, including GitHub
Pages, needs no extra headers, npm dependencies, CDN, or server-side runtime.
Do not use `file://` for the WebAssembly fetch.

Controls: arrows move/climb; Space jumps; P pauses. The page also provides touch
controls, restart, mute, and fullscreen. On phones the game fills the screen
width, with a left D-pad and right Space button. The top toolbar contains the
status message and icon controls; the original DOS link sits beside the title.
Touch inputs support simultaneous movement/jump, pointer cancellation, and
short taps lasting less than one simulation tick. The original intro is shown
first, with its eye animation and opening melody.
Press Space to enter the selected stage. All 50 original maps are playable.
The stage selector immediately starts a new run at any stage. Clearing a stage
and pressing Space advances to the next; stage 50 loops back to stage 1 as in
the original. Death retries the current stage. The original continue screen
allows resuming that stage with another credit.

The time-attack clock starts on the **Space input event at the title screen**,
not on the browser's Start button or the following simulation tick. It uses a
monotonic real-time timestamp and keeps counting through pause, background-tab
pause, dying/respawn, inter-stage screens and credit continues. It freezes when
stage 50's final candy is collected, before waiting at the clear/bonus screen.
The WASM `run_finished()` signal latches at the candy-update routine and survives
stage wrap; a new run resets it. The frozen clock cannot resume accidentally.

Stage selection now returns to the title so every attempt has the same Space
start boundary. Starting at stage 1 is labeled time attack; selecting stages
2–50 is labeled practice time. Selecting a stage, restarting, or returning to
the title after exhausted credits resets the attempt. The timer also appears
in fullscreen. Its millisecond display refreshes at 10 Hz; the underlying start
and finish timestamps are not rounded to that display interval.

## Update the existing local website

After a successful build and tests, copy only runtime outputs to the website:

```sh
python3 tools/publish.py ../mson0129.github.io/raccoon
```

This changes local files only; it does not commit, push or publish to a host.

## What was recovered

- `PPK.EXE`: LZEXE 0.91 compressed, Borland C++ 16-bit program. The unpacked
  analysis copy is `analysis/PPK-unpacked.EXE`; it is never fetched by the player.
- `STAGE.PPK`: 50 maps, each 11 rows of 28 characters plus two NUL bytes per row
  (330 bytes per stage). Every stage uses the original map reader and constructor.
- `PPK.B16`: four sprite blocks of 48, 49, 31, and 28 128-byte images. Each image
  is 16×16 pixels with four bitplanes. Blocks contain sprites, AND masks,
  terrain/UI tiles, and scenery.
- `STAGE.PP3`: 16 RGB triplets, 6 bits/channel. Original attribute-table mapping
  `[0,2,4,6,8,10,12,14,1,3,5,7,9,11,13,15]` is applied before RGB conversion.
- The font comes from initialized EXE data at `DS:-110 + character*8`.
- `STAGE.PP4`: the original packed 1-bit death sound, now played with Web Audio.
  Its 6 kHz playback rate is an approximation of the DOS software delay loop.
- `STAGE.PP1`: original intro screen, with a 128-byte header, 16-color palette,
  and PCX-style RLE. Scanline planes are ordered 3,0,1,2. The native decoder
  exactly matches the original `0327:2477` loader.
- `STAGE.PP2`: original continue screen, now decoded for same-stage credit continues.

For the retained C reference, `tools/translate.py` traverses the control-flow graph of reachable game functions
and emits direct C calls, branches, arithmetic, and memory accesses. Original
16-bit values and arithmetic flags are preserved to retain edge behavior.
There is no runtime opcode dispatch. `src/game.generated.c` holds 16 routines
covering 2,964 original instructions. Original binary offsets remain beside the
translated statements for review. `src/port.c` replaces the DOS library, graphics,
keyboard, speaker, and main-loop integration.

Useful routine addresses in segment `0327`:

| Offset | Purpose |
| --- | --- |
| `04db` | Construct stage collision and actor data |
| `0d98` | Player movement, climbing, jump tables, dying animation |
| `11c1`, `12f5`, `1429` | Enemy behavior and collisions |
| `15a3` | Terrain, fruit, bonus, hazard interactions |
| `1cca` | Floating score sprites |
| `1db1` | Score display |
| `1e09` | Stage timer |
| `1f34` | Stage background and HUD |
| `260d` | Reset player and transient state |
| `09e9` | Composite actors and present |

The original BIOS clock (1193182 / 65536 Hz) controls simulation. Presentation
interpolates actor positions independently between ticks. This cannot change
collision/game state; interpolation at 100% reproduces the original composited
frame. Long browser frames retain their elapsed time instead of dropping it.
ImageData and the palette lookup table are reused.

## Verification

Install optional analysis/test packages in a virtual environment:

```sh
python3 -m venv /tmp/ppk-tools
/tmp/ppk-tools/bin/pip install -r requirements-analysis.txt
/tmp/ppk-tools/bin/python -m unittest discover -s tests -v
```

Eighteen WASM tests cover all 50 map loads, fruit counts, scenery selection,
all sequential clear transitions (including 50 → 1), selected-stage intro and
respawn, same-stage credit continue, invalid selections, the standalone module,
stage-50 final-candy completion latch, initial map, movement/wall,
original jump curve, climbing, fruit/score, death/respawn, timeout, clear/restart,
intro-to-stage transition,
and render interpolation (some cases share a test). They execute `ppk.wasm`.

Offline differential testing uses Unicorn **only as a test oracle**, never in
the player. On macOS:

```sh
clang -O1 -Wno-parentheses-equality -DTEST_ORACLE -dynamiclib \
  src/port.c -o /tmp/ppk-port.dylib
/tmp/ppk-tools/bin/python tools/compare_original.py /tmp/ppk-port.dylib
```

The test executes original x86 gameplay routines and compares every byte of game
data and all 32,000 background-plane bytes against the C port. It covers 50
stages × 80 seeded input frames × 8 routines = **32,000 comparisons**.
The native intro decoder also matches all 32,000 bytes produced by the
original EXE screen loader. A separate WASM test checks that render interpolation never mutates game data
and its endpoint matches the original compositor output.

These tests establish exact equality for the exercised gameplay paths, not a
proof for every possible input or original hardware's analog sound/timing.
Death-sound timing and general menu transitions are not cycle-exact. Palette fades now reproduce the original DAC steps and retrace counts. Stage-clear
transitions are tested with completion fixtures; full human playthroughs of
every stage have not been recorded.

Four JavaScript timer tests additionally verify title-input start timing,
uninterrupted elapsed time, frozen final-candy results and practice-run reset:

```sh
node --test tests/speedrun-timer.test.mjs
```

## Re-run static translation

Normal builds use the checked-in Rust source and need no emulator/Capstone.
Only to regenerate the historical C reference after analysis:

```sh
/tmp/ppk-tools/bin/python tools/translate.py
python3 tools/assets.py
```

The unpacked analysis image was produced using
[UNLZEXE](https://github.com/mywave82/unlzexe) from the supplied original EXE.
`analysis/asset-hashes.json` records the recovered asset hashes.

## Physical gamepad support

Bluetooth/USB controllers exposed by the browser Gamepad API are polled every
frame. D-pad and left stick move/climb; both A (standard button 0) and B
(button 1) map to Space (title start, jump and continue); Start (button 9)
toggles pause. The stick has a 0.3 deadzone. Touch and keyboard controls can be
used together with a gamepad. Disconnect/blur releases keys, and returning
to the page requires neutral controls before accepting a held controller.

On iOS/Android pair the controller in the OS settings, open the HTTPS game page,
and press a controller button to expose it to the browser. Tap the page's Start
button once if the mobile browser requires a touch gesture for audio. Browser
and controller compatibility varies; physical iOS/Android hardware was not
available for end-to-end device testing.

Specification: https://www.w3.org/TR/gamepad/

Gamepad mapping tests: `node --test tests/gamepad-input.test.mjs` (7 cases).

The top-right input icon shows a gamepad when a controller is detected, or a
keyboard otherwise. Clicking it opens the matching control guide in a modal
with a centered Close button. The modal suspends game inputs and simulation,
while the real-time speedrun clock keeps counting.

## Installable / offline web app

The static build now includes `app.webmanifest`, Apple touch icons and
`sw.js`. Serve it over HTTPS (GitHub Pages works); localhost is suitable for
development. On iPhone/iPad use Safari's Share → Add to Home Screen; on Android
use the browser's Install app / Add to Home Screen menu. Chromium's native
install prompt is also exposed through an install button when available.

Open the installed app online once and wait for **오프라인 준비됨**. Its 13
pre-cached files include the HTML, manifest, all JS modules, icons and the
compiled WASM (all 50 stages are embedded). It can then relaunch and play
without a network connection. The separate DOS reference remains online-only
because it loads js-dos from an external CDN. Clearing browser/app storage
requires preparing the offline cache online again.

`tools/build-service-worker.py` versions every cached file by its actual SHA-256
and also includes the worker template in the version. Installation verifies
every response before activating a complete package. Incomplete/mismatched
updates are discarded, preserving the previous working cache. Updates wait
while the game is open; **새 버전 적용** explicitly reloads into the new version.
Old cache cleanup is confined to this app's service-worker scope.

Both `tools/package-web.py` (when a compiled WASM exists) and `tools/publish.py`
refresh the generated worker. `build.sh` refreshes it again after compiling
WASM, so code/asset edits cannot silently retain an old cache version.
To recreate the checked-in sprite-derived icons, install Pillow and run
`python tools/build-icons.py`. Icon generation is not required for normal builds.

Validation: 4 Python PWA metadata/hash tests and 7 service-worker tests cover
offline HTML/WASM reads, failed installs, integrity mismatches, scope-limited
cleanup, explicit activation and evicted-file repair. A real browser tab was
reloaded with network access disabled and successfully ran the WASM game.
Home-screen installation on physical iOS/Android hardware was not tested.

```sh
node --test tests/service-worker.test.mjs
```

Platform documentation: https://web.dev/learn/pwa/installation/ and
https://webkit.org/blog/13878/

## Mobile touch and directory entry

Native text selection, image dragging and long-press callouts are disabled for
the game app, including labels below the floating controls. Native stage-picker
interaction remains enabled. A selection that iOS creates anyway is cleared.

The build generates `dist/index.html` from `web/index.htm`; there is only one
editable entry source. `/raccoon/` and `/raccoon/index.htm` carry identical
manifest/name/icon metadata, and directory navigation canonicalizes to
`index.htm` before installation. The manifest app ID remains unchanged. The
Apple touch icon and favicon use a cache-refresh query parameter. Existing
incorrectly created iOS shortcuts should be removed and added again after
updating to this release; a page cannot rewrite an already-created OS shortcut.

## Replays and title start input

The toolbar's replay icon opens the most recent attempt. Recording begins with
the intro and becomes saveable on the title Space input. One attempt is saved
locally in IndexedDB every five seconds, on pause, on restart, on page hide and
at the stage-50 finish. The recording is an input movie, not video. File export
and import are local browser operations; no replay is uploaded to a server.

Playback offers pause, 1x/2x/4x speed and exit to a new title screen. Physical
gameplay input is disabled and floating touch controls are hidden during playback. Viewing a replay ends the current
attempt after archiving it. Start stages other than 1 remain practice runs.
Playback displays recorded RTA, including original pauses and between-stage
waits. Pausing the viewer stops its timeline; it does not alter the stored RTA.

Format v1 records the WASM SHA-256, start stage, key-mask changes, tick wall
timestamps, timer start/finish and state digests at 180-tick intervals plus the
last tick. Bit 6 preserves a Space release/re-press occurring between fixed
ticks. This matters because the title-start release latch responds to key-up.
Playback rejects another engine version and stops if a checkpoint differs.
The digest excludes presentation buffers; rendering/audio reads cannot alter
logical movie state. Imported JSON is validated as data, with bounded file
size, frame count, timestamps, masks, checkpoints and per-render catch-up work.

Limits: the latest attempt only, 200,000 simulation ticks (about three hours),
12 MB per imported file and at most 24 hours of recorded wall time including
pauses. Storage failures leave an in-memory movie available for file export.
Browser eviction/private browsing can remove local replays; export a file to
retain an important run. A five-second save interval can lose the last few
seconds on abrupt browser termination.

Title Space still starts RTA at the actual input event. WASM now consumes this
Space until release, so holding keyboard Space, touch SPACE or gamepad A/B
across the title boundary does not jump on stage entry. Movement is available
immediately and a new Space press jumps normally.

Validation: two new WASM regression tests check held-title Space and movement;
ten replay tests reproduce tick states, pauses, stage-27 practice, 2x/4x
speed, exact final millisecond time, sub-tick Space edges, invalid imports,
state mismatches and recording limits. Browser QA verifies file export/import,
IndexedDB restoration, 4x completion, frozen viewer time on pause and mobile
offline playback using the verified service-worker package.

```sh
node --test tests/*.test.mjs
```

## Autosave timing regression

`requestAnimationFrame` supplies the frame-start timestamp. The live autosave
previously reused it after WASM ticking, hashing and rendering, even though the
latest recorded tick already had a later `performance.now()` timestamp. Replay
validation could then reject the inconsistent duration/RTA pair. The uncaught
exception prevented scheduling the next animation frame and froze gameplay.

Periodic autosave now samples a fresh clock value after the tick and uses it
for both duration and RTA. Snapshot/storage exceptions are isolated from the
animation loop; recording is disabled for that attempt, the previous replay
is retained, and gameplay continues. A replay-dialog note reports this case.
Tick recording is also guarded. The WASM binary and replay format are unchanged.
Regression tests reproduce a 5000ms RAF timestamp followed by a 5004ms tick,
verify a consistent fresh snapshot, and inject snapshot/storage failures.

## Palette fade timing

`0327:02e6` writes 64 decremented RGB DAC palettes, waiting for one retrace
(`02fe:000b`, VGA port 03DA bit 3) each time. `0327:01d1` writes initial black
at one retrace, then 64 incremented palettes. Every component is clamped to
0..63; a CSS opacity fade would produce different intermediate colors.
The native transition uses the mode-0D VGA clock 25.175 MHz / (800×449),
about 70.0863 retraces/s, independently of the 18.2065 Hz gameplay clock.
Fade-out is 64 retraces (913.16 ms), fade-in 65 (927.43 ms). Render interpolation
samples this rational clock without advancing gameplay or changing replay state.
At a clear, the last gameplay image fades out before the bonus screen; the next
stage fades in. Last-candy RTA completion is latched before either effect.

The supplied executable's title exit calls the immediate-black palette writer
`0327:019b` before the stage fade-in; it does not call the gradual fade-out there.
The port follows that title sequence: Space immediately switches to black,
then the stage uses the original 65-retrace fade-in. There is no outgoing
title fade. The 64-retrace outgoing fade remains on stage clears.

`tests/test_fade_original.py` executes the two original routines in Unicorn and
compares every pixel across title fade-in and stage-clear fade-out retrace positions against WASM, as well as confirming
64/65 original retrace waits and frozen actor positions. The module still ships
without any CPU emulator. A browser running below 70 Hz may skip visual samples;
the elapsed time and selected palette step do not slow down with display FPS.

## Rust rewrite validation

`tests/compare_c_rust.py` compares all 0x1784 bytes of the game-data image, all
32,000 planar background bytes, all 64,000 RGBA pixels, phase/stage, completion,
audio and frame counter on 24,500 ticks across 50 stages. It includes randomized
input, title/respawn/continue transitions, forced clears, next stages and wrapping.
Results and exact module hashes are saved in `analysis/rust-verification.json`.

```sh
WASI_SDK_PATH=/path/to/wasi-sdk sh tools/build-c-reference.sh
python tests/compare_c_rust.py
```

The original executable oracle also supports Rust directly. Build the isolated
test adapter (excluded from production), then run the same 32,000-routine suite:

```sh
(cd rust && CARGO_TARGET_DIR=target/oracle cargo build --locked --release \
  --target wasm32-unknown-unknown --features oracle)
python tools/compare_original.py --wasm \
  rust/target/oracle/wasm32-unknown-unknown/release/raccoon_engine.wasm
```

The existing Python WASM/palette/PWA and JavaScript replay/controller/timer/cache
tests run against the production Rust module: 25 Python and 28 JavaScript tests.
The Rust rewrite does not claim proof for every possible input; these tests use
exact equality for the exercised paths, including the documented original quirks.

Browser ABI names and controls remain compatible. Replay files identify their
engine by the full WASM SHA-256, so recordings made with the old C module remain
version-incompatible with this Rust module; new recordings replay deterministically.
