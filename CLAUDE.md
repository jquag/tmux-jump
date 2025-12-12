# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build           # Debug build
cargo build --release # Release build
cargo run -- <process> <directory>  # Run with arguments
```

## Project Overview

tmux-jump is a Rust CLI utility that finds and switches to a tmux pane based on the running process and directory. It takes two arguments: a process name and a directory path, then locates a tmux pane running that process within that directory (or subdirectory) and switches focus to it.

The tool queries tmux for all panes using `list-panes -a`, matches against the provided criteria, and uses `switch-client` (falling back to `select-pane`) to switch focus.
