use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

// Hard limits to prevent memory spikes and runaway processes.
const MAX_OUTPUT_LINES: usize = 5_000;
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pass,
    Fail,
    Ignored,
}

pub enum RunnerMsg {
    TestResult { name: String, status: TestStatus },
    /// Cargo exited with non-zero status before any test results were emitted.
    /// Almost always means a compile error.
    BuildFailed,
    Done,
}

/// Returned to the caller so it can abort an in-flight run by setting true.
pub type CancelToken = Arc<AtomicBool>;

/// Spawn a background thread that streams `cargo test -p <pkg>` output
/// line by line instead of buffering it all into memory first.
///
/// Returns a `CancelToken`; set it to `true` to kill the child process.
pub fn spawn(pkg: &str, workspace: &Path, tx: Sender<RunnerMsg>) -> CancelToken {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = Arc::clone(&cancel);

    let pkg = pkg.to_string();
    let workspace = workspace.to_path_buf();

    thread::spawn(move || {
        let mut child = match Command::new("cargo")
            .args(["test", "-p", &pkg, "--color", "never"])
            .current_dir(&workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                let _ = tx.send(RunnerMsg::Done);
                return;
            }
        };

        // Drain stderr in a separate thread so the child never blocks on a
        // full pipe buffer, which would deadlock the whole process.
        let stderr = child.stderr.take().expect("stderr piped");
        thread::spawn(move || {
            BufReader::new(stderr)
                .lines()
                .take(MAX_OUTPUT_LINES)
                .for_each(drop);
        });

        let stdout = child.stdout.take().expect("stdout piped");
        let deadline = Instant::now() + TIMEOUT;
        let mut lines_read = 0usize;
        let mut results_sent = 0usize;

        'read: for line in BufReader::new(stdout).lines() {
            // Check abort conditions before processing each line.
            if cancel_clone.load(Ordering::Relaxed)
                || Instant::now() > deadline
                || lines_read >= MAX_OUTPUT_LINES
            {
                let _ = child.kill();
                break 'read;
            }
            lines_read += 1;

            if let Ok(line) = line {
                if let Some((name, status)) = parse_line(&line) {
                    // If the receiver was dropped (app quit), kill and stop.
                    if tx.send(RunnerMsg::TestResult { name, status }).is_err() {
                        let _ = child.kill();
                        break 'read;
                    }
                    results_sent += 1;
                }
            }
        }

        // Non-zero exit with no test output means compile error, not test failure.
        let exit_ok = child.wait().map(|s| s.success()).unwrap_or(false);
        if !exit_ok && results_sent == 0 {
            let _ = tx.send(RunnerMsg::BuildFailed);
        }
        let _ = tx.send(RunnerMsg::Done);
    });

    cancel
}

fn parse_line(line: &str) -> Option<(String, TestStatus)> {
    let line = line.trim();
    if !line.starts_with("test ") {
        return None;
    }
    if let Some(rest) = line.strip_suffix(" ... ok") {
        return Some((leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()), TestStatus::Pass));
    }
    if let Some(rest) = line.strip_suffix(" ... FAILED") {
        return Some((leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()), TestStatus::Fail));
    }
    if let Some(rest) = line.strip_suffix(" ... ignored") {
        return Some((
            leaf(rest.strip_prefix("test ").unwrap_or(rest).trim()),
            TestStatus::Ignored,
        ));
    }
    None
}

fn leaf(full: &str) -> String {
    full.split("::").last().unwrap_or(full).to_string()
}
