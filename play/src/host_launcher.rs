use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type DynError = Box<dyn std::error::Error>;

const DEFAULT_PORT: u16 = 7878;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WebLaunchArgs {
    pub(crate) port: u16,
    pub(crate) print_only: bool,
    pub(crate) host_path: Option<PathBuf>,
    pub(crate) adapter_args: Vec<String>,
}

pub(crate) fn run_host_web(
    repo: &Path,
    local: bool,
    passthrough: &[String],
) -> Result<(), DynError> {
    let invocation_dir = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    let repo = workspace_root(&invocation_dir);
    let options = parse_web_args(&invocation_dir, local, passthrough)?;
    let adapter_bin = ensure_adapter_built(&repo)?;
    let host_root = options
        .host_path
        .clone()
        .map(|path| path.canonicalize().unwrap_or(path))
        .unwrap_or_else(|| default_host_path(&repo));
    let manifest = host_root.join("Cargo.toml");
    if !manifest.exists() {
        return Err(format!(
            "challenge-host manifest not found at {}",
            manifest.display()
        )
        .into());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("-p")
        .arg("host-cli")
        .arg("--")
        .arg("web")
        .arg("--adapter")
        .arg(adapter_bin)
        .arg("--adapter-cwd")
        .arg(&repo);

    for arg in &options.adapter_args {
        cmd.arg("--adapter-arg").arg(arg);
    }

    cmd.arg("--port").arg(options.port.to_string());
    if options.print_only {
        cmd.arg("--print-only");
    }

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .current_dir(host_root);

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("challenge-host web command exited with status {status}").into());
    }

    Ok(())
}

pub(crate) fn parse_web_args(
    current_dir: &Path,
    local: bool,
    args: &[String],
) -> Result<WebLaunchArgs, DynError> {
    let mut parsed = WebLaunchArgs {
        port: DEFAULT_PORT,
        print_only: false,
        host_path: None,
        adapter_args: Vec::new(),
    };
    let mut adapter_local = local;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--port" => {
                let value = iter.next().ok_or("--port requires a value")?;
                parsed.port = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid port: {value}"))?;
            }
            "--print-only" => parsed.print_only = true,
            "--host-path" => {
                let value = iter.next().ok_or("--host-path requires a value")?;
                let path = PathBuf::from(value);
                parsed.host_path = Some(if path.is_absolute() {
                    path
                } else {
                    current_dir.join(path)
                });
            }
            "--workspace" => {
                let value = iter.next().ok_or("--workspace requires a value")?;
                let path = PathBuf::from(value);
                let resolved = if path.is_absolute() {
                    path
                } else {
                    current_dir.join(path)
                };
                parsed.adapter_args.push(arg.clone());
                parsed
                    .adapter_args
                    .push(resolved.to_string_lossy().to_string());
            }
            "--local" => adapter_local = true,
            "--help" | "-h" => {
                println!(
                    "Usage: play web [--port PORT] [--workspace PATH] [--local] [--print-only] [--host-path PATH]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown web option: {other}").into()),
        }
    }

    if adapter_local {
        parsed.adapter_args.insert(0, "--local".to_string());
    }

    Ok(parsed)
}

fn workspace_root(start: &Path) -> PathBuf {
    for ancestor in start.ancestors() {
        if ancestor.join("Cargo.toml").exists()
            && ancestor.join("play").join("Cargo.toml").exists()
            && ancestor.join("crates").is_dir()
        {
            return ancestor.to_path_buf();
        }
    }

    start.to_path_buf()
}

pub(crate) fn default_host_path(start: &Path) -> PathBuf {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join("challenge-host");
        if candidate.exists() {
            return candidate;
        }
    }

    start.join("challenge-host")
}

fn ensure_adapter_built(repo: &Path) -> Result<PathBuf, DynError> {
    let bin = repo.join("target").join("debug").join("fzts-adapter");
    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("play")
        .arg("--bin")
        .arg("fzts-adapter")
        .current_dir(repo)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(format!("cargo build for fzts-adapter exited with {status}").into());
    }
    let bin = bin.canonicalize().unwrap_or(bin);
    if !bin.exists() {
        return Err("fzts-adapter binary missing after build".into());
    }
    Ok(bin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "play-host-launcher-{prefix}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn parse_web_args_splits_host_flags_from_adapter_flags() {
        let repo = temp_dir("parse");
        let parsed = parse_web_args(
            &repo,
            true,
            &[
                "--port".into(),
                "9000".into(),
                "--print-only".into(),
                "--host-path".into(),
                "custom-host".into(),
                "--workspace".into(),
                "alt-workspace".into(),
            ],
        )
        .expect("web args should parse");

        assert_eq!(parsed.port, 9000);
        assert!(parsed.print_only);
        assert_eq!(parsed.host_path, Some(repo.join("custom-host")));
        assert_eq!(
            parsed.adapter_args,
            vec![
                "--local".to_string(),
                "--workspace".to_string(),
                repo.join("alt-workspace").to_string_lossy().to_string()
            ]
        );

        fs::remove_dir_all(repo).expect("temp dir should be removed");
    }

    #[test]
    fn default_host_path_finds_repo_challenge_host_from_nested_workspace() {
        let root = temp_dir("host-root");
        let repo = root.join("from-zero-to-systems");
        let nested = repo.join(".fzts/workspace");
        let host = repo.join("challenge-host");
        fs::create_dir_all(&nested).expect("nested workspace should exist");
        fs::create_dir_all(&host).expect("host workspace should exist");

        assert_eq!(default_host_path(&nested), host);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn workspace_root_finds_repo_root_from_play_subdir() {
        let root = temp_dir("workspace-root");
        let repo = root.join("repo");
        let play_dir = repo.join("play");

        fs::create_dir_all(repo.join("crates")).expect("crates dir should exist");
        fs::create_dir_all(&play_dir).expect("play dir should exist");
        fs::write(
            repo.join("Cargo.toml"),
            "[workspace]\nmembers = [\"play\"]\n",
        )
        .expect("workspace manifest should exist");
        fs::write(
            play_dir.join("Cargo.toml"),
            "[package]\nname = \"play\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("play manifest should exist");

        assert_eq!(workspace_root(&play_dir), repo);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn ensure_adapter_built_rebuilds_even_when_binary_exists() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let root = temp_dir("build");
        let repo = root.join("repo");
        let bin_dir = repo.join("target/debug");
        let adapter_bin = bin_dir.join("fzts-adapter");
        let fake_bin_dir = root.join("bin");
        let fake_cargo = fake_bin_dir.join("cargo");
        let build_log = root.join("cargo.log");

        fs::create_dir_all(&bin_dir).expect("bin dir should exist");
        fs::create_dir_all(&fake_bin_dir).expect("fake cargo dir should exist");
        fs::write(&adapter_bin, "#!/usr/bin/env bash\n").expect("adapter binary should exist");
        fs::write(
            &fake_cargo,
            format!(
                "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' \"$@\" > \"{}\"\n",
                build_log.display()
            ),
        )
        .expect("fake cargo should be written");

        let mut permissions = fs::metadata(&fake_cargo)
            .expect("fake cargo metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).expect("fake cargo should be executable");

        let old_path = std::env::var_os("PATH");
        let joined_path = match old_path.as_ref() {
            Some(path) => format!("{}:{}", fake_bin_dir.display(), path.to_string_lossy()),
            None => fake_bin_dir.display().to_string(),
        };
        std::env::set_var("PATH", joined_path);

        let result = ensure_adapter_built(&repo);

        match old_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        let built = fs::read_to_string(&build_log);
        fs::remove_dir_all(root).expect("temp dir should be removed");

        result.expect("adapter build should succeed");
        let built = built.expect("cargo build should run even for an existing adapter binary");
        assert!(built.contains("build"));
        assert!(built.contains("fzts-adapter"));
    }

    #[test]
    fn run_host_web_from_play_subdir_uses_workspace_root() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let root = temp_dir("run-web");
        let repo = root.join("repo");
        let play_dir = repo.join("play");
        let host_dir = repo.join("challenge-host");
        let fake_bin_dir = root.join("bin");
        let fake_cargo = fake_bin_dir.join("cargo");
        let cargo_log = root.join("cargo.log");
        let adapter_bin = repo.join("target/debug/fzts-adapter");

        fs::create_dir_all(&play_dir).expect("play dir should exist");
        fs::create_dir_all(&host_dir).expect("host dir should exist");
        fs::create_dir_all(repo.join("crates")).expect("crates dir should exist");
        fs::create_dir_all(&fake_bin_dir).expect("fake cargo dir should exist");
        fs::write(
            repo.join("Cargo.toml"),
            "[workspace]\nmembers = [\"play\"]\n",
        )
        .expect("workspace manifest should exist");
        fs::write(
            play_dir.join("Cargo.toml"),
            "[package]\nname = \"play\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("play manifest should exist");
        fs::write(host_dir.join("Cargo.toml"), "[workspace]\nmembers = []\n")
            .expect("challenge-host manifest should exist");
        fs::write(
            &fake_cargo,
            format!(
                "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s|%s\\n' \"$PWD\" \"$*\" >> \"{}\"\nif [[ \"$1\" == \"build\" ]]; then\n  mkdir -p \"{}\"\n  printf '#!/usr/bin/env bash\\n' > \"{}\"\n  chmod +x \"{}\"\nfi\n",
                cargo_log.display(),
                adapter_bin.parent().expect("adapter parent should exist").display(),
                adapter_bin.display(),
                adapter_bin.display()
            ),
        )
        .expect("fake cargo should be written");

        let mut permissions = fs::metadata(&fake_cargo)
            .expect("fake cargo metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).expect("fake cargo should be executable");

        let old_path = std::env::var_os("PATH");
        let joined_path = match old_path.as_ref() {
            Some(path) => format!("{}:{}", fake_bin_dir.display(), path.to_string_lossy()),
            None => fake_bin_dir.display().to_string(),
        };
        std::env::set_var("PATH", joined_path);

        let result = run_host_web(&play_dir, true, &["--print-only".into()]);

        match old_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        let cargo_log = fs::read_to_string(&cargo_log).expect("cargo log should be written");
        fs::remove_dir_all(root).expect("temp dir should be removed");

        result.expect("web launcher should succeed from the play subdir");
        assert!(
            cargo_log.contains(&format!(
                "{}|build -p play --bin fzts-adapter",
                repo.display()
            )),
            "expected adapter build to run from the workspace root, got:\n{cargo_log}"
        );
        assert!(
            cargo_log.contains(&format!("--adapter-cwd {}", repo.display())),
            "expected host launch to pass the workspace root to the adapter, got:\n{cargo_log}"
        );
    }
}
