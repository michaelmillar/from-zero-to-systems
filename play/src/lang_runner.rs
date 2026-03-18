use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub struct TestResponse {
    pub passed: bool,
    pub output: String,
}

pub trait LanguageRunner {
    fn id(&self) -> &str;
    #[allow(dead_code)]
    fn source_file(&self, challenge_dir: &Path) -> PathBuf;
    fn source_rel_path(&self, challenge_id: &str) -> String;
    fn is_available(&self, challenge_dir: &Path) -> bool;
    fn toolchain_present(&self) -> bool;
    fn run_tests(&self, workspace: &Path, challenge_id: &str, package: &str) -> Result<TestResponse, Box<dyn std::error::Error>>;
    fn run_benchmark(&self, workspace: &Path, challenge_id: &str) -> Result<BenchmarkOutput, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct BenchmarkOutput {
    pub ok: bool,
    pub mean_ns: Option<u64>,
    pub summary: String,
    pub output: String,
}

pub fn runner_for(language: &str) -> Box<dyn LanguageRunner> {
    match language {
        "c" => Box::new(CRunner),
        "python" => Box::new(PythonRunner),
        "haskell" => Box::new(HaskellRunner),
        _ => Box::new(RustRunner),
    }
}

fn package_from_id(id: &str) -> String {
    id.split_once('-')
        .map(|(_, pkg)| pkg.to_string())
        .unwrap_or_else(|| id.to_string())
}

fn pascal_case(package: &str) -> String {
    package
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn truncate_output(output: String) -> String {
    if output.len() <= MAX_OUTPUT_BYTES {
        return output;
    }
    let mut truncated = output;
    truncated.truncate(MAX_OUTPUT_BYTES);
    truncated.push_str("\n\n[output truncated]");
    truncated
}

fn render_command_output(command_label: &str, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.trim().is_empty() {
        stdout.to_string()
    } else if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };
    let body = truncate_output(if combined.trim().is_empty() {
        "(no output)".to_string()
    } else {
        combined
    });
    format!("{command_label}\n\n{body}")
}

fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Rust
// ---------------------------------------------------------------------------

pub struct RustRunner;

impl LanguageRunner for RustRunner {
    fn id(&self) -> &str { "rust" }

    fn source_file(&self, challenge_dir: &Path) -> PathBuf {
        challenge_dir.join("src/lib.rs")
    }

    fn source_rel_path(&self, challenge_id: &str) -> String {
        format!("crates/{challenge_id}/src/lib.rs")
    }

    fn is_available(&self, challenge_dir: &Path) -> bool {
        challenge_dir.join("src/lib.rs").exists()
    }

    fn toolchain_present(&self) -> bool { true }

    fn run_tests(&self, workspace: &Path, _challenge_id: &str, package: &str) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let label = format!("$ cargo test -p {package} --color never");
        let output = Command::new("cargo")
            .args(["test", "-p", package, "--color", "never"])
            .current_dir(workspace)
            .output()
            .map_err(|e| format!("failed to run cargo test: {e}"))?;

        Ok(TestResponse {
            passed: output.status.success(),
            output: render_command_output(&label, &output),
        })
    }

    fn run_benchmark(&self, workspace: &Path, challenge_id: &str) -> Result<BenchmarkOutput, Box<dyn std::error::Error>> {
        let package = package_from_id(challenge_id);
        let bench_path = workspace.join("crates").join(challenge_id).join("src/bench.rs");
        if !bench_path.exists() {
            return Err("no Rust benchmark found".into());
        }
        let label = format!("$ cargo run -p {package} --release --bin bench-{package}");
        let output = Command::new("cargo")
            .args(["run", "-p", &package, "--release", "--bin", &format!("bench-{package}")])
            .current_dir(workspace)
            .output()
            .map_err(|e| format!("failed to run Rust benchmark: {e}"))?;

        parse_benchmark_output(&label, &output)
    }
}

// ---------------------------------------------------------------------------
// C
// ---------------------------------------------------------------------------

pub struct CRunner;

impl LanguageRunner for CRunner {
    fn id(&self) -> &str { "c" }

    fn source_file(&self, challenge_dir: &Path) -> PathBuf {
        let package = package_from_id(
            challenge_dir.file_name().unwrap_or_default().to_str().unwrap_or(""),
        );
        challenge_dir.join("c").join(format!("{package}.c"))
    }

    fn source_rel_path(&self, challenge_id: &str) -> String {
        let package = package_from_id(challenge_id);
        format!("crates/{challenge_id}/c/{package}.c")
    }

    fn is_available(&self, challenge_dir: &Path) -> bool {
        challenge_dir.join("c").is_dir()
    }

    fn toolchain_present(&self) -> bool { which("gcc") }

    fn run_tests(&self, workspace: &Path, challenge_id: &str, _package: &str) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let c_dir = workspace.join("crates").join(challenge_id).join("c");
        let label = format!("$ cd crates/{challenge_id}/c && make test");
        let output = Command::new("make")
            .arg("test")
            .current_dir(&c_dir)
            .output()
            .map_err(|e| format!("failed to run make test: {e}"))?;

        Ok(TestResponse {
            passed: output.status.success(),
            output: render_command_output(&label, &output),
        })
    }

    fn run_benchmark(&self, workspace: &Path, challenge_id: &str) -> Result<BenchmarkOutput, Box<dyn std::error::Error>> {
        let c_dir = workspace.join("crates").join(challenge_id).join("c");
        let label = format!("$ cd crates/{challenge_id}/c && make bench");
        let output = Command::new("make")
            .arg("bench")
            .current_dir(&c_dir)
            .output()
            .map_err(|e| format!("failed to run C benchmark: {e}"))?;

        parse_benchmark_output(&label, &output)
    }
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

pub struct PythonRunner;

impl LanguageRunner for PythonRunner {
    fn id(&self) -> &str { "python" }

    fn source_file(&self, challenge_dir: &Path) -> PathBuf {
        let package = package_from_id(
            challenge_dir.file_name().unwrap_or_default().to_str().unwrap_or(""),
        );
        challenge_dir.join("python").join(format!("{}.py", package.replace('-', "_")))
    }

    fn source_rel_path(&self, challenge_id: &str) -> String {
        let package = package_from_id(challenge_id);
        format!("crates/{challenge_id}/python/{}.py", package.replace('-', "_"))
    }

    fn is_available(&self, challenge_dir: &Path) -> bool {
        challenge_dir.join("python").is_dir()
    }

    fn toolchain_present(&self) -> bool { which("python3") }

    fn run_tests(&self, workspace: &Path, challenge_id: &str, _package: &str) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let py_dir = workspace.join("crates").join(challenge_id).join("python");
        let package = package_from_id(challenge_id);
        let test_file = format!("test_{}.py", package.replace('-', "_"));
        let label = format!("$ cd crates/{challenge_id}/python && python3 -m pytest {test_file} -v --tb=short");
        let output = Command::new("python3")
            .args(["-m", "pytest", &test_file, "-v", "--tb=short"])
            .current_dir(&py_dir)
            .output()
            .map_err(|e| format!("failed to run pytest: {e}"))?;

        Ok(TestResponse {
            passed: output.status.success(),
            output: render_command_output(&label, &output),
        })
    }

    fn run_benchmark(&self, workspace: &Path, challenge_id: &str) -> Result<BenchmarkOutput, Box<dyn std::error::Error>> {
        let py_dir = workspace.join("crates").join(challenge_id).join("python");
        let package = package_from_id(challenge_id);
        let bench_file = format!("bench_{}.py", package.replace('-', "_"));
        let label = format!("$ cd crates/{challenge_id}/python && python3 {bench_file}");
        let output = Command::new("python3")
            .arg(&bench_file)
            .current_dir(&py_dir)
            .output()
            .map_err(|e| format!("failed to run Python benchmark: {e}"))?;

        parse_benchmark_output(&label, &output)
    }
}

// ---------------------------------------------------------------------------
// Haskell
// ---------------------------------------------------------------------------

pub struct HaskellRunner;

impl LanguageRunner for HaskellRunner {
    fn id(&self) -> &str { "haskell" }

    fn source_file(&self, challenge_dir: &Path) -> PathBuf {
        let package = package_from_id(
            challenge_dir.file_name().unwrap_or_default().to_str().unwrap_or(""),
        );
        challenge_dir.join("haskell").join(format!("{}.hs", pascal_case(&package)))
    }

    fn source_rel_path(&self, challenge_id: &str) -> String {
        let package = package_from_id(challenge_id);
        format!("crates/{challenge_id}/haskell/{}.hs", pascal_case(&package))
    }

    fn is_available(&self, challenge_dir: &Path) -> bool {
        challenge_dir.join("haskell").is_dir()
    }

    fn toolchain_present(&self) -> bool { which("cabal") }

    fn run_tests(&self, workspace: &Path, challenge_id: &str, _package: &str) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let hs_dir = workspace.join("crates").join(challenge_id).join("haskell");
        let label = format!("$ cd crates/{challenge_id}/haskell && cabal test");
        let output = Command::new("cabal")
            .arg("test")
            .current_dir(&hs_dir)
            .output()
            .map_err(|e| format!("failed to run cabal test: {e}"))?;

        Ok(TestResponse {
            passed: output.status.success(),
            output: render_command_output(&label, &output),
        })
    }

    fn run_benchmark(&self, workspace: &Path, challenge_id: &str) -> Result<BenchmarkOutput, Box<dyn std::error::Error>> {
        let hs_dir = workspace.join("crates").join(challenge_id).join("haskell");
        let package = package_from_id(challenge_id);
        let label = format!("$ cd crates/{challenge_id}/haskell && cabal run bench-{package}");
        let output = Command::new("cabal")
            .args(["run", &format!("bench-{package}")])
            .current_dir(&hs_dir)
            .output()
            .map_err(|e| format!("failed to run Haskell benchmark: {e}"))?;

        parse_benchmark_output(&label, &output)
    }
}

// ---------------------------------------------------------------------------
// Shared benchmark JSON parser
// ---------------------------------------------------------------------------

fn parse_benchmark_output(
    label: &str,
    output: &std::process::Output,
) -> Result<BenchmarkOutput, Box<dyn std::error::Error>> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let full_output = render_command_output(label, output);

    if !output.status.success() {
        return Ok(BenchmarkOutput {
            ok: false,
            mean_ns: None,
            summary: "benchmark failed".to_string(),
            output: full_output,
        });
    }

    // Look for a JSON line: {"ok": true, "mean_ns": ..., "summary": "..."}
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.contains("mean_ns") {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                return Ok(BenchmarkOutput {
                    ok: parsed["ok"].as_bool().unwrap_or(true),
                    mean_ns: parsed["mean_ns"].as_u64(),
                    summary: parsed["summary"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    output: full_output,
                });
            }
        }
    }

    Ok(BenchmarkOutput {
        ok: true,
        mean_ns: None,
        summary: "completed (no timing data)".to_string(),
        output: full_output,
    })
}
