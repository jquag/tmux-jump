// src/main.rs
use std::env;
use std::fs;
use std::process::{Command, exit};

struct Args {
    process_name: String,
    directory: Option<String>,
    keys: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();
    let mut process_name: Option<String> = None;
    let mut directory: Option<String> = None;
    let mut keys: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-k" | "--keys" => {
                if i + 1 < args.len() {
                    keys = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --keys requires a value");
                    exit(1);
                }
            }
            "-d" | "--directory" => {
                if i + 1 < args.len() {
                    directory = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --directory requires a value");
                    exit(1);
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                exit(1);
            }
            _ => {
                if process_name.is_none() {
                    process_name = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    let process_name = match process_name {
        Some(p) => p,
        None => {
            eprintln!("Usage: tmux-jump <process> [-d|--directory <dir>] [-k|--keys <keys>]");
            exit(1);
        }
    };

    let directory = directory.map(|d| match fs::canonicalize(&d) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            eprintln!("Directory not found: {}", d);
            exit(1);
        }
    });

    Args { process_name, directory, keys }
}

fn main() {
    let args = parse_args();
    let process_name = &args.process_name;
    let directory = args.directory;

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

    // Get current working directory for prioritization
    let cwd = env::current_dir()
        .ok()
        .and_then(|p| fs::canonicalize(p).ok())
        .map(|p| p.to_string_lossy().to_string());

    // Collect all matching panes with their paths
    let matching_panes: Vec<(String, String)> = panes
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() != 3 {
                return None;
            }

            let (id, cmd, path) = (parts[0], parts[1], parts[2]);

            if cmd != process_name {
                return None;
            }

            let normalized_path = fs::canonicalize(path).ok()?.to_string_lossy().to_string();

            if let Some(ref dir) = directory {
                if !normalized_path.starts_with(dir) {
                    return None;
                }
            }

            Some((id.to_string(), normalized_path))
        })
        .collect();

    // Prioritize panes: exact CWD match > subdirectory of CWD > others
    let pane_id = if let Some(ref cwd) = cwd {
        matching_panes
            .iter()
            .find(|(_, path)| path == cwd)
            .or_else(|| matching_panes.iter().find(|(_, path)| path.starts_with(cwd)))
            .or_else(|| matching_panes.first())
            .map(|(id, _)| id.clone())
    } else {
        matching_panes.first().map(|(id, _)| id.clone())
    };

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

    if let Some(keys) = args.keys {
        send_keys(&pane_id, &keys);
    }

    switch_to_pane(&pane_id);
}

fn send_keys(pane_id: &str, keys: &str) {
    Command::new("tmux")
        .args(["send-keys", "-t", pane_id, keys])
        .status()
        .expect("Failed to send keys");
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
