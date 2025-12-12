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
    let directory = if args.len() >= 3 {
        args[2].clone()
    } else {
        env::current_dir()
            .expect("Failed to get current directory")
            .to_string_lossy()
            .to_string()
    };

    // Normalize directory path
    let directory = match fs::canonicalize(directory) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            eprintln!("Directory not found: {}", directory);
            exit(1);
        }
    };

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
        let normalized_path = fs::canonicalize(path).ok()?.to_string_lossy().to_string();

        if cmd == process_name && normalized_path.starts_with(&directory) {
            Some(id.to_string())
        } else {
            None
        }
    });

    let pane_id = match pane_id {
        Some(id) => id,
        None => {
            eprintln!(
                "No pane found running '{}' in '{}'",
                process_name, directory
            );
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
