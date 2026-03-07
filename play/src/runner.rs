use std::{path::Path, process::Command, sync::mpsc::Sender, thread};

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pass,
    Fail,
    Ignored,
}

pub enum RunnerMsg {
    TestResult { name: String, status: TestStatus },
    Done,
}

/// Spawn a background thread that runs `cargo test -p <pkg>` and sends results.
pub fn spawn(pkg: &str, workspace: &Path, tx: Sender<RunnerMsg>) {
    let pkg       = pkg.to_string();
    let workspace = workspace.to_path_buf();

    thread::spawn(move || {
        let out = Command::new("cargo")
            .args(["test", "-p", &pkg, "--color", "never"])
            .current_dir(&workspace)
            .output();

        if let Ok(output) = out {
            let text = String::from_utf8_lossy(&output.stdout).into_owned()
                + &String::from_utf8_lossy(&output.stderr);
            for line in text.lines() {
                if let Some((name, status)) = parse_line(line) {
                    let _ = tx.send(RunnerMsg::TestResult { name, status });
                }
            }
        }

        let _ = tx.send(RunnerMsg::Done);
    });
}

fn parse_line(line: &str) -> Option<(String, TestStatus)> {
    let line = line.trim();
    if !line.starts_with("test ") { return None; }

    if let Some(rest) = line.strip_suffix(" ... ok") {
        return Some((leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()), TestStatus::Pass));
    }
    if let Some(rest) = line.strip_suffix(" ... FAILED") {
        return Some((leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()), TestStatus::Fail));
    }
    if let Some(rest) = line.strip_suffix(" ... ignored") {
        return Some((leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()), TestStatus::Ignored));
    }
    None
}

fn leaf(full: &str) -> String {
    full.split("::").last().unwrap_or(full).to_string()
}
