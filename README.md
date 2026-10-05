# memmon

[![Release](https://img.shields.io/github/v/release/lagovi/memmon)](https://github.com/lagovi/memmon/releases/latest)
[![Build Static Binary](https://github.com/lagovi/memmon/actions/workflows/build.yml/badge.svg)](https://github.com/lagovi/memmon/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Binary Size](https://img.shields.io/badge/size-879_KiB-brightgreen.svg)]()

A lightweight, terminal-based **unified memory visualizer** for Linux written in Rust.

Instead of navigating complex process trees, `memmon` models your physical RAM and disk-backed Swap (SSD) as a single proportional matrix. It inspects `/proc` to detail top memory consumers and dynamically embeds live process labels directly into the visual blocks.

---

## Features

- **Unified Memory Space**: Treats physical RAM and disk-backed Swap (SSD) as a single pool.
- **Accurate Process Accounting**: Inspects `/proc/[pid]/status` calculating private anonymous memory (`RssAnon` + `VmSwap`), eliminating shared page double-counting.
- **Smart Script Resolution**: Resolves runtime scripts and working directories via `/proc/[pid]/cmdline` and `/proc/[pid]/cwd`, displaying readable labels (e.g. `python3: torrent-mcp/server.py`).
- **Process Aggregation**: Groups multi-worker processes (e.g. `dockerd (x2)`, `zellij (x4)`) into combined consumer blocks.
- **Embedded In-Grid Badges**: Embeds `[ process · size ]` labels inside the largest contiguous runs in the color matrix.
- **Hamilton Apportionment**: Uses the largest remainder algorithm to ensure grid cells always add up to 100% without rounding drift.
- **Adaptive 2-Column TUI**: Recalculates matrix and legend on terminal resize (`SIGWINCH`), organizing consumers into two balanced columns when width permits.
- **Static Binary**: Standalone executable (~879 KiB, compiled with `musl libc`) with zero shared library dependencies.

---

## Download & Installation

### Option 1: Direct Download (Pre-built Static Binary)

Download the standalone binary from the [latest GitHub Release](https://github.com/lagovi/memmon/releases/latest):

```bash
curl -L -o ~/bin/memmon https://github.com/lagovi/memmon/releases/latest/download/memmon
chmod +x ~/bin/memmon
```

*(Ensure `~/bin` is in your `$PATH`, or move it to `/usr/local/bin/memmon`)*

### Option 2: GitHub CLI

```bash
gh release download v0.1.0 -R lagovi/memmon -p memmon -D ~/bin/
chmod +x ~/bin/memmon
```

### Option 3: Build from Source

```bash
git clone https://github.com/lagovi/memmon.git
cd memmon
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Usage

```bash
memmon
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `q` or `Q` | Exit |
| `Esc` | Exit |
| `Ctrl + C` | Clean terminal teardown and exit |

---

## Architecture

- **`src/mem.rs`**: Parser for `/proc/meminfo` and `/proc/swaps` (filters out zram to accurately track disk swap).
- **`src/process.rs`**: Traverses `/proc/[pid]`, computes anonymous RSS + VmSwap, resolves cwd symlinks, and aggregates instances.
- **`src/layout.rs`**: Geometry engine, Hamilton apportionment, ANSI-aware string width calculation (`visible_width`), and TUI renderer.
- **`src/terminal.rs`**: RAII terminal guard with custom panic hook to guarantee restoration from raw/alternate mode.

---

## License

This project is licensed under the [MIT License](LICENSE).
