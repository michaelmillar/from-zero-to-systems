use host_runtime::{AdapterSpec, HostSession};
use std::path::PathBuf;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "inspect" => inspect_command(args.into_iter().skip(1).collect()),
        "web" => web_command(args.into_iter().skip(1).collect()),
        "--help" | "-h" | "" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command: {other}").into()),
    }
}

fn inspect_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let (adapter, adapter_args, adapter_cwd) = parse_inspect_args(args)?;
    let mut session = HostSession::connect_in(&adapter, &adapter_args, adapter_cwd.as_deref())?;
    let overview = session.load_overview()?;

    println!(
        "game: {} ({})",
        overview.handshake.title, overview.handshake.game_id
    );
    println!("challenge: {}", overview.workspace.challenge_id);
    println!(
        "language: {}",
        overview.workspace.language.as_deref().unwrap_or("(none)")
    );
    println!("visible_hints: {}", overview.workspace.hints.len());
    println!("intro: {}", overview.workspace.intro);

    Ok(())
}

fn web_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    host_web::run(parse_web_args(args)?)
}

fn parse_inspect_args(
    args: Vec<String>,
) -> Result<(String, Vec<String>, Option<PathBuf>), Box<dyn std::error::Error>> {
    let mut adapter = None;
    let mut adapter_args = Vec::new();
    let mut adapter_cwd = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--adapter" => {
                adapter = Some(iter.next().ok_or("--adapter requires a program")?);
            }
            "--adapter-cwd" => {
                adapter_cwd = Some(PathBuf::from(
                    iter.next().ok_or("--adapter-cwd requires a path")?,
                ));
            }
            "--adapter-arg" => {
                adapter_args.push(iter.next().ok_or("--adapter-arg requires a value")?);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown inspect argument: {other}").into()),
        }
    }

    let adapter = adapter.ok_or("missing --adapter PROGRAM")?;
    Ok((adapter, adapter_args, adapter_cwd))
}

fn parse_web_args(args: Vec<String>) -> Result<host_web::WebOptions, Box<dyn std::error::Error>> {
    let mut adapter_program = None;
    let mut adapter_args = Vec::new();
    let mut adapter_cwd = None;
    let mut port = host_web::DEFAULT_PORT;
    let mut print_only = false;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--adapter" => {
                adapter_program = Some(iter.next().ok_or("--adapter requires a program")?);
            }
            "--adapter-cwd" => {
                adapter_cwd = Some(PathBuf::from(
                    iter.next().ok_or("--adapter-cwd requires a path")?,
                ));
            }
            "--adapter-arg" => {
                adapter_args.push(iter.next().ok_or("--adapter-arg requires a value")?);
            }
            "--port" => {
                let raw = iter.next().ok_or("--port requires a value")?;
                port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid port: {raw}"))?;
            }
            "--print-only" => {
                print_only = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown web argument: {other}").into()),
        }
    }

    Ok(host_web::WebOptions {
        adapter: AdapterSpec {
            program: adapter_program.ok_or("missing --adapter PROGRAM")?,
            args: adapter_args,
            cwd: adapter_cwd,
        },
        port,
        print_only,
    })
}

fn print_help() {
    eprintln!("challenge-host");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  inspect --adapter PROGRAM [--adapter-cwd PATH] [--adapter-arg ARG]...");
    eprintln!(
        "  web --adapter PROGRAM [--adapter-cwd PATH] [--adapter-arg ARG]... [--port PORT] [--print-only]"
    );
}

#[cfg(test)]
mod tests {
    use super::{parse_inspect_args, parse_web_args};
    use std::path::PathBuf;

    #[test]
    fn parse_inspect_args_accepts_adapter_cwd() {
        let (adapter, adapter_args, adapter_cwd) = parse_inspect_args(vec![
            "--adapter".into(),
            "cargo".into(),
            "--adapter-cwd".into(),
            "/tmp/hazptr".into(),
            "--adapter-arg".into(),
            "run".into(),
        ])
        .unwrap();

        assert_eq!(adapter, "cargo");
        assert_eq!(adapter_args, vec!["run"]);
        assert_eq!(adapter_cwd, Some(PathBuf::from("/tmp/hazptr")));
    }

    #[test]
    fn parse_web_args_accepts_adapter_cwd_port_and_print_only() {
        let parsed = parse_web_args(vec![
            "--adapter".into(),
            "cargo".into(),
            "--adapter-cwd".into(),
            "/tmp/hazptr".into(),
            "--adapter-arg".into(),
            "run".into(),
            "--port".into(),
            "7891".into(),
            "--print-only".into(),
        ])
        .unwrap();

        assert_eq!(parsed.adapter.program, "cargo");
        assert_eq!(parsed.adapter.args, vec!["run"]);
        assert_eq!(parsed.adapter.cwd, Some(PathBuf::from("/tmp/hazptr")));
        assert_eq!(parsed.port, 7891);
        assert!(parsed.print_only);
    }
}
