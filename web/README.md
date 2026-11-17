# Raccoon / PPK runtime

This directory contains only the static browser runtime and the optional DOS
reference bundle. Development sources, original EXE/archive, tests and analysis
outputs live in the separate `ppk` project (`~/Documents/Github/ppk`).

- `index.htm`: native WASM player.
- `port/player.js`, `port/speedrun-timer.mjs`, `port/ppk.wasm`: runtime modules.
- `dos.htm`, `ppk.jsdos`: original DOS reference player and its game bundle.

From the website repository root, preview with:

```sh
python3 -m http.server 8080 --bind 127.0.0.1
```

Open http://127.0.0.1:8080/raccoon/index.htm.
Build and local-copy instructions are in the separate development project's README.
