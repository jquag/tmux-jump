// src/main.rs
use std::env;
use std::fs;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: tmux-jump <process> [directory]");
        exit(1);
    }

    let process_name = &args[1];
    let directory = args.get(2).map(|d| match fs::canonicalize(d) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            eprintln!("Directory not found: {}", d);
            exit(1);
        }
    });

    // Get tmux panes
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_id}|#{pane_current_command}|#{pane_current_path}",
        ])
        .output()
        .expect("Failed to run tmux");

    let panes = String::from_utf8_lossy(&output.stdout);

    // Find matching pane
    let pane_id = panes.lines().find_map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            return None;
        }

        let (id, cmd, path) = (parts[0], parts[1], parts[2]);

        if cmd != process_name {
            return None;
        }

        if let Some(ref dir) = directory {
            let normalized_path = fs::canonicalize(path).ok()?.to_string_lossy().to_string();
            if !normalized_path.starts_with(dir) {
                return None;
            }
        }

        Some(id.to_string())
    });

    let pane_id = match pane_id {
        Some(id) => id,
        None => {
            match directory {
                Some(dir) => eprintln!("No pane found running '{}' in '{}'", process_name, dir),
                None => eprintln!("No pane found running '{}'", process_name),
            }
            exit(1);
        }
    };

    switch_to_pane(&pane_id);
}

fn switch_to_pane(pane_id: &str) {
    // Try switch-client first (for detached sessions), fall back to select-pane
    let result = Command::new("tmux")
        .args(["switch-client", "-t", pane_id])
        .status();

    if result.is_err() || !result.unwrap().success() {
        Command::new("tmux")
            .args(["select-pane", "-t", pane_id])
            .status()
            .expect("Failed to select pane");
    }
}
