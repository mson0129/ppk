# PPK 50-stage WebAssembly source port

This is an ahead-of-time static recompilation of the original game's routines,
with native browser platform replacements. It does **not** execute PPK.EXE,
DOSBox, EmulatorJS, js-dos, an x86 instruction decoder, or a VGA device at runtime.
The shipped `ppk.wasm` has **zero imports**. JavaScript only supplies input,
Web Audio playback, a fixed clock, and Canvas presentation.

This repository contains the original files, C source, web source, tests and
analysis outputs. The website checkout contains only the deployable runtime.

- `original/`: original `PPK.EXE`, game/map files and `ppk.zip` archive.
- `src/`, `assets/`: WASM C sources, generated headers and initialized data.
- `tools/`, `tests/`, `analysis/`: build/analysis tools, tests and recovered data.
- `web/`: canonical browser HTML, JavaScript and time-attack timer source.
- `dos/`: DOSBox configuration for the optional original DOS reference.
- `dist/`: complete static website build, including `port/ppk.wasm`.

The EXE is the supplied original binary. This project builds the WASM port;
it does not claim to rebuild the original DOS executable from C.

## Build

Install the official [WASI SDK](https://github.com/WebAssembly/wasi-sdk/releases).
The WASM output was built with wasi-sdk 30.0 (macOS arm64).
From the repository root:

```sh
WASI_SDK_PATH=/path/to/wasi-sdk ./build.sh
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

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

`tools/translate.py` traverses the control-flow graph of reachable game functions
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

Sixteen WASM tests cover all 50 map loads, fruit counts, scenery selection,
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
Death-sound timing and fade/menu transitions are not cycle-exact. Stage-clear
transitions are tested with completion fixtures; full human playthroughs of
every stage have not been recorded.

Four JavaScript timer tests additionally verify title-input start timing,
uninterrupted elapsed time, frozen final-candy results and practice-run reset:

```sh
node --test tests/speedrun-timer.test.mjs
```

## Re-run static translation

Normal builds use the checked-in generated C and need no emulator/Capstone.
To regenerate after analysis:

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

Open the installed app online once and wait for **오프라인 준비됨**. Its 11
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

Validation: 3 Python PWA metadata/hash tests and 7 service-worker tests cover
offline HTML/WASM reads, failed installs, integrity mismatches, scope-limited
cleanup, explicit activation and evicted-file repair. A real browser tab was
reloaded with network access disabled and successfully ran the WASM game.
Home-screen installation on physical iOS/Android hardware was not tested.

```sh
node --test tests/service-worker.test.mjs
```

Platform documentation: https://web.dev/learn/pwa/installation/ and
https://webkit.org/blog/13878/
