# tmux-jump

A CLI utility to find and switch to a tmux pane by foreground process name.

## Usage

```
tmux-jump <process> [-d|--directory <dir>] [-k|--keys <keys>]
```

### Examples

```bash
# Jump to a pane running vim
tmux-jump vim

# Jump to a pane running claude (even if process shows as "node")
tmux-jump claude

# Jump to a pane running nvim in a specific directory
tmux-jump nvim -d ~/projects/myapp

# Jump to a pane and send keys before switching
tmux-jump claude -k "@main.go"
```

## Features

- **Full command line matching**: Matches against the full process command, not just the executable name. This means `tmux-jump claude` works even when the process shows as `node /path/to/claude`.
- **Directory filtering**: Optionally filter panes by working directory (matches subdirectories too).
- **Smart prioritization**: When multiple panes match, prefers panes in your current working directory.
- **Send keys**: Optionally send keys to the pane before switching focus.

## Installation

```bash
cargo install --path .
```

## Requirements

- tmux
- macOS or Linux (uses `ps` for process info)
