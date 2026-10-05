# memmon

A lightweight, terminal-based unified memory (RAM + SSD Swap) visualizer with per-process breakdown for Linux.

## Features
- **Unified Memory Space**: Tracks physical RAM and disk-backed Swap (SSD) as a single pool.
- **Top Consumers Breakdown**: Scans `/proc` to compute private process memory (`RssAnon` + `VmSwap`) and shows the top 10 consumers with script path resolution (`folder/script.py`).
- **Interactive Adaptive TUI**: Automatically adjusts layout, matrix size, and legend when terminal dimensions change.
- **Embedded In-Grid Badges**: Contiguous memory runs display consumer labels and metrics directly inside the colored matrix blocks.
- **Minimal Footprint**: Written in Rust, consumes < 2 MiB of RAM with sub-millisecond `/proc` parsing time.

## Controls
- `q`, `Q`, `Esc`: Exit application
- `Ctrl + C`: Clean shutdown
