mod adapter;
mod lang_runner;
mod meta;
mod progress;
mod workspace;

use std::io::{self, BufRead, Write};

use host_protocol::{Envelope, ProtocolError, Request, ResponseEnvelope};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let workspace = adapter::resolve_workspace_root(&current_dir, std::env::args().skip(1))?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let envelope: Envelope<Request> = serde_json::from_str(&line)?;
        let response = match adapter::handle_request(&workspace, envelope.payload) {
            Ok(result) => ResponseEnvelope {
                id: envelope.id,
                ok: true,
                result: Some(result),
                error: None,
            },
            Err(error) => ResponseEnvelope {
                id: envelope.id,
                ok: false,
                result: None,
                error: Some(ProtocolError {
                    code: "request_failed".into(),
                    message: error.to_string(),
                }),
            },
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}
