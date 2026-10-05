# memmon

[![Build Static Binary](https://github.com/lagovi/memmon/actions/workflows/build.yml/badge.svg)](https://github.com/lagovi/memmon/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Binary Size](https://img.shields.io/badge/size-879_KiB-brightgreen.svg)]()
[![Target](https://img.shields.io/badge/target-x86__64--unknown--linux--musl-blue.svg)]()

A lightweight, terminal-based **unified memory visualizer** for Linux written in Rust.

Instead of looking at raw numbers in `free -m` or navigating complex process trees in `htop`, `memmon` models your physical RAM and SSD swap as a single proportional matrix. It inspects `/proc` in sub-millisecond time, details the top memory consumers, and embeds live process labels directly into the visual blocks.

---

## Key Features

- **Unified Memory Space**: Treats physical RAM and disk-backed Swap (SSD) as a single pool, showing true total memory headroom.
- **Accurate Process Accounting**: Inspects `/proc/[pid]/status` calculating private anonymous memory (`RssAnon` + `VmSwap`), eliminating shared library double-counting.
- **Smart Script Resolution**: Resolves runtime scripts and working directories via `/proc/[pid]/cmdline` and `/proc/[pid]/cwd`, displaying readable labels like `python3: torrent-mcp/server.py` instead of generic interpreter names.
- **Process Aggregation**: Groups multi-worker processes (e.g. `dockerd (x2)`, `zellij (x4)`) into combined consumer blocks.
- **Embedded In-Grid Badges**: Dynamically embeds `[ process · size ]` badges inside the largest contiguous runs in the color matrix.
- **Hamilton Apportionment**: Uses the largest remainder algorithm to ensure visual grid cells and percentages always add up to 100% without rounding drift.
- **Adaptive 2-Column TUI**: Automatically recalculates matrix rows/columns on terminal resize (`SIGWINCH`), organizing the legend into two balanced columns when window width allows.
- **Zero-Dependency Static Binary**: Compiled with `musl libc`, producing a completely standalone executable (~879 KiB) that runs on any Linux distribution without external dependencies.

---

## Real-World Performance

Measured on Ubuntu 24.04 LTS (x86_64):

| Metric | Measurement |
| :--- | :--- |
| **Binary Size** | **879 KiB** (fully static, zero shared libraries) |
| **Peak Memory (RSS)** | **~11.2 MiB** (vs ~40+ MiB for Python runtime) |
| **`/proc` Scan Latency** | **< 1 ms** per refresh cycle |
| **CPU Utilization** | **< 0.1%** |

---

## Installation & Download

### Option 1: Download Pre-built Static Binary (GitHub CLI)

If you have `gh` installed:

```bash
mkdir -p ~/bin
gh run download -R lagovi/memmon -n memmon-linux-x86_64 -D ~/bin/
chmod +x ~/bin/memmon
```

### Option 2: Build from Source

Ensure you have Rust and the `musl` target installed:

```bash
git clone https://github.com/lagovi/memmon.git
cd memmon

# Build fully static binary
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Resulting binary will be located at:
# target/x86_64-unknown-linux-musl/release/memmon
```

---

## Usage

Simply run:

```bash
memmon
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `q` or `Q` | Exit `memmon` |
| `Esc` | Exit `memmon` |
| `Ctrl + C` | Clean terminal teardown and exit |

---

## Architecture

- **`src/mem.rs`**: High-speed parser for `/proc/meminfo` and `/proc/swaps` (filters out zram to accurately track disk swap).
- **`src/process.rs`**: Traverses `/proc/[pid]`, computes uncompressed anonymous RSS + VmSwap, resolves cwd symlinks, and aggregates instances.
- **`src/layout.rs`**: Geometry engine, Hamilton apportionment, ANSI-aware string width calculation (`visible_width`), and TUI renderer.
- **`src/terminal.rs`**: RAII terminal guard implementing `Drop` and custom panic hooks to guarantee restoration from raw/alternate mode under any exit condition.

---

## License

This project is licensed under the [MIT License](LICENSE).
