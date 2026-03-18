use std::path::{Path, PathBuf};

use host_protocol::{
    ActivityDay, BenchmarkResultEntry, BenchmarkRunResult, Capabilities, ChallengeStatus,
    ChallengeSummary, DocLink, EditorState, ExplainComparisonView, ExplainLevelView, ExplainView,
    HintMode, HintState, LanguageNoteEntry, LanguageNotesView, LanguageProgress,
    ListChallengesResult, ProgressView, Request, ResponsePayload, StructuredTestResult,
    TestRunResult, VisibleHint, WorkspaceActions, WorkspaceView,
};

use crate::{lang_runner, meta, progress, workspace};

type DynError = Box<dyn std::error::Error>;

pub(crate) fn resolve_workspace_root(
    current_dir: &Path,
    args: impl Iterator<Item = String>,
) -> Result<PathBuf, DynError> {
    let mut explicit = None;
    let mut local = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => {
                let value = args.next().ok_or("--workspace requires a value")?;
                let path = PathBuf::from(&value);
                explicit = Some(if path.is_absolute() {
                    path
                } else {
                    current_dir.join(path)
                });
            }
            "--local" => local = true,
            "--help" | "-h" => {
                println!("Usage: fzts-adapter [--workspace PATH] [--local]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    match explicit {
        Some(path) => Ok(path),
        None if local => workspace::ensure_local_workspace(current_dir),
        None => Ok(workspace::resolve_workspace_root(current_dir, None)),
    }
}

pub(crate) fn handle_request(
    workspace: &Path,
    request: Request,
) -> Result<ResponsePayload, DynError> {
    match request {
        Request::Handshake => Ok(handshake(workspace)),
        Request::ListChallenges => Ok(ResponsePayload::ChallengeList(list_challenges(workspace)?)),
        Request::LoadWorkspace {
            challenge_id,
            language,
        } => Ok(ResponsePayload::Workspace(load_workspace(
            workspace,
            &challenge_id,
            language.as_deref().unwrap_or("rust"),
        )?)),
        Request::SaveWorkspace {
            challenge_id,
            language,
            content,
        } => Ok(ResponsePayload::Workspace(save_workspace(
            workspace,
            &challenge_id,
            language.as_deref().unwrap_or("rust"),
            &content,
        )?)),
        Request::ResetWorkspace {
            challenge_id,
            language,
        } => Ok(ResponsePayload::Workspace(reset_workspace(
            workspace,
            &challenge_id,
            language.as_deref().unwrap_or("rust"),
        )?)),
        Request::RunTests {
            challenge_id,
            language,
            content,
        } => Ok(ResponsePayload::TestRun(run_tests(
            workspace,
            &challenge_id,
            language.as_deref().unwrap_or("rust"),
            &content,
        )?)),
        Request::LoadProgress => Ok(ResponsePayload::Progress(load_progress(workspace)?)),
        Request::RevealHint { .. } => Err("fzts does not support incremental hint reveals".into()),
        Request::Benchmark {
            challenge_id,
            language,
            content: _,
        } => Ok(ResponsePayload::Benchmark(run_benchmarks(
            workspace,
            &challenge_id,
            language.as_deref(),
        )?)),
        Request::LoadExplain { challenge_id } => {
            Ok(ResponsePayload::Explain(load_explain(workspace, &challenge_id)?))
        }
        Request::CodeQuality { .. } => {
            Err("code quality is not exposed through the shared host yet".into())
        }
        Request::Grade { .. } => Err("grading is not exposed through the shared host yet".into()),
        Request::LoadLanguageNotes { challenge_id } => {
            Ok(ResponsePayload::LanguageNotes(load_language_notes(
                workspace,
                &challenge_id,
            )?))
        }
    }
}

fn handshake(workspace: &Path) -> ResponsePayload {
    let has_multi_language = workspace::discover_challenges(workspace)
        .unwrap_or_default()
        .iter()
        .any(|c| workspace::available_languages(workspace, &c.id).len() > 1);

    ResponsePayload::Handshake {
        game_id: "fzts".into(),
        title: "from-zero-to-systems".into(),
        capabilities: Capabilities {
            multi_language: has_multi_language,
            incremental_hints: false,
            benchmark: has_multi_language,
            explain: has_multi_language,
            compare: has_multi_language,
            idea_tools: false,
            synthesis: false,
        },
    }
}

fn list_challenges(workspace: &Path) -> Result<ListChallengesResult, DynError> {
    let challenges = workspace::discover_challenges(workspace)?
        .into_iter()
        .map(|challenge| {
            let langs = workspace::available_languages(workspace, &challenge.id);
            ChallengeSummary {
                id: challenge.id,
                title: challenge.title,
                track: None,
                difficulty: None,
                status: if challenge.completed {
                    ChallengeStatus::Complete
                } else {
                    ChallengeStatus::NotStarted
                },
                available_languages: langs,
                badges: Vec::new(),
            }
        })
        .collect::<Vec<_>>();

    let current_challenge = challenges
        .iter()
        .find(|challenge| challenge.status != ChallengeStatus::Complete)
        .or_else(|| challenges.first())
        .map(|challenge| challenge.id.clone());

    Ok(ListChallengesResult {
        current_language: current_challenge.as_ref().map(|_| "rust".to_string()),
        current_challenge,
        challenges,
    })
}

fn load_workspace(
    workspace: &Path,
    challenge_id: &str,
    language: &str,
) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(
        workspace::load_workspace_for_language(workspace, challenge_id, language)?,
        language,
    ))
}

fn save_workspace(
    workspace: &Path,
    challenge_id: &str,
    language: &str,
    content: &str,
) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(
        workspace::save_workspace_for_language(workspace, challenge_id, language, content)?,
        language,
    ))
}

fn reset_workspace(
    workspace: &Path,
    challenge_id: &str,
    language: &str,
) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(
        workspace::reset_workspace_for_language(workspace, challenge_id, language)?,
        language,
    ))
}

fn run_tests(
    workspace: &Path,
    challenge_id: &str,
    language: &str,
    content: &str,
) -> Result<TestRunResult, DynError> {
    let result = workspace::run_workspace_tests_for_language(
        workspace,
        challenge_id,
        language,
        content,
    )?;
    Ok(TestRunResult {
        passed: result.passed,
        output: result.output,
        structured_results: Vec::<StructuredTestResult>::new(),
    })
}

fn load_progress(workspace: &Path) -> Result<ProgressView, DynError> {
    let progress = progress::load(workspace);
    let total = workspace::discover_challenges(workspace)?.len();

    let mut languages = vec![LanguageProgress {
        language: "rust".into(),
        completed: progress.completed_in_language("rust"),
        total,
    }];

    for lang in &["c", "python", "haskell"] {
        let count = progress.completed_in_language(lang);
        if count > 0 {
            languages.push(LanguageProgress {
                language: lang.to_string(),
                completed: count,
                total,
            });
        }
    }

    Ok(ProgressView {
        completed: progress.completed_count(),
        total,
        streak_days: Some(progress::streak_days(&progress)),
        score: None,
        activity: progress
            .activity
            .iter()
            .map(|(date, entry)| ActivityDay {
                date: date.clone(),
                check_ins: entry.check_ins,
                completed: entry.completed,
                commits: 0,
            })
            .collect(),
        languages,
    })
}

fn run_benchmarks(
    workspace: &Path,
    challenge_id: &str,
    language: Option<&str>,
) -> Result<BenchmarkRunResult, DynError> {
    let langs: Vec<String> = match language {
        Some(l) => vec![l.to_string()],
        None => workspace::available_languages(workspace, challenge_id),
    };

    let results = langs
        .iter()
        .map(|lang| {
            let runner = lang_runner::runner_for(lang);
            match runner.run_benchmark(workspace, challenge_id) {
                Ok(result) => BenchmarkResultEntry {
                    language: lang.clone(),
                    ok: result.ok,
                    mean_ns: result.mean_ns,
                    summary: result.summary,
                    output: result.output,
                },
                Err(err) => BenchmarkResultEntry {
                    language: lang.clone(),
                    ok: false,
                    mean_ns: None,
                    summary: err.to_string(),
                    output: String::new(),
                },
            }
        })
        .collect();

    Ok(BenchmarkRunResult {
        challenge_id: challenge_id.to_string(),
        results,
    })
}

fn load_explain(workspace: &Path, challenge_id: &str) -> Result<ExplainView, DynError> {
    let toml_meta = meta::load_challenge_meta(workspace, challenge_id)
        .ok_or_else(|| format!("no metadata found for {challenge_id}"))?;

    let levels = vec![
        ExplainLevelView {
            label: "ELI5".to_string(),
            body: format!(
                "Imagine you flip a biased coin thousands of times and write down how much you lose each flip. Then you sort your losses and find the number at the 95% mark. That number tells you \"in 95% of cases, you will not lose more than this.\" That is what this challenge builds."
            ),
        },
        ExplainLevelView {
            label: "Educated".to_string(),
            body: toml_meta.shared.intro.clone(),
        },
    ];

    let comparisons = if let Some(ref comp) = toml_meta.comparison {
        comp.trade_offs
            .iter()
            .map(|t| ExplainComparisonView {
                challenge_id: t.dimension.clone(),
                body: format!(
                    "Rust: {}\nC: {}\nPython: {}\nHaskell: {}",
                    t.rust, t.c, t.python, t.haskell
                ),
            })
            .collect()
    } else {
        Vec::new()
    };

    let use_cases = toml_meta
        .comparison
        .as_ref()
        .map(|c| c.summary.clone())
        .unwrap_or_default();

    Ok(ExplainView {
        levels,
        use_cases,
        comparisons,
    })
}

fn load_language_notes(
    workspace: &Path,
    challenge_id: &str,
) -> Result<LanguageNotesView, DynError> {
    let toml_meta = meta::load_challenge_meta(workspace, challenge_id);

    let mut languages = Vec::new();
    let mut facts = Vec::new();

    if let Some(ref tm) = toml_meta {
        for (lang_key, lang_meta) in [
            ("rust", &tm.rust),
            ("c", &tm.c),
            ("python", &tm.python),
            ("haskell", &tm.haskell),
        ] {
            if let Some(lm) = lang_meta {
                let mut body = String::new();
                if !lm.concepts.is_empty() {
                    body.push_str("Concepts:\n");
                    for c in &lm.concepts {
                        body.push_str(&format!("  - {c}\n"));
                    }
                }
                if !lm.tools.is_empty() {
                    body.push_str("\nTools:\n");
                    for t in &lm.tools {
                        body.push_str(&format!("  - {} ({}): {}\n", t.name, t.url, t.description));
                    }
                }
                languages.push(LanguageNoteEntry {
                    language: lang_key.to_string(),
                    body,
                });
            }
        }

        if let Some(ref comp) = tm.comparison {
            facts.push(comp.summary.clone());
        }
    }

    Ok(LanguageNotesView { languages, facts })
}

fn to_workspace_view(workspace: workspace::WorkspaceRecord, language: &str) -> WorkspaceView {
    let hint_count = workspace.hints.len();

    WorkspaceView {
        challenge_id: workspace.challenge,
        title: workspace.title,
        language: Some(language.to_string()),
        editor: EditorState {
            file_path: workspace.file_path,
            content: workspace.content,
            can_reset: workspace.can_reset,
        },
        intro: workspace.intro,
        guide: workspace.guide,
        concepts: workspace.concepts,
        docs: workspace
            .docs
            .into_iter()
            .map(|doc| DocLink {
                label: doc.label,
                url: doc.url,
            })
            .collect(),
        hints: workspace
            .hints
            .into_iter()
            .map(|hint| VisibleHint {
                label: hint.test_name,
                body: hint.hints.join("\n"),
                cost: None,
            })
            .collect(),
        hint_state: HintState {
            mode: HintMode::Full,
            revealed_count: hint_count,
            total_count: hint_count,
            next_cost: None,
        },
        actions: WorkspaceActions {
            can_save: true,
            can_test: true,
            can_reveal_hint: false,
            can_benchmark: false,
            can_compare: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "play-adapter-{prefix}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn ensure_dir(path: &Path) {
        fs::create_dir_all(path).expect("directory should be created");
    }

    fn write_workspace(root: &Path) {
        ensure_dir(&root.join("crates/01-alpha/src"));
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/01-alpha\"]\n",
        )
        .expect("workspace manifest should be created");
        fs::write(
            root.join("crates/01-alpha/Cargo.toml"),
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("crate manifest should be created");
        fs::write(
            root.join("crates/01-alpha/src/lib.rs"),
            "pub fn solve() -> u32 { 7 }\n",
        )
        .expect("lib should be created");
        fs::write(
            root.join("crates/01-alpha/README.md"),
            "# Alpha\n\nFirst crate.\n",
        )
        .expect("README should be created");
    }

    #[test]
    fn adapter_reports_fzts_handshake() {
        let root = temp_dir("handshake");
        write_workspace(&root);

        let response = handle_request(&root, Request::Handshake).expect("handshake should succeed");

        match response {
            ResponsePayload::Handshake {
                game_id,
                title,
                capabilities,
            } => {
                assert_eq!(game_id, "fzts");
                assert_eq!(title, "from-zero-to-systems");
                assert!(!capabilities.multi_language);
                assert!(!capabilities.compare);
            }
            other => panic!("unexpected response: {other:?}"),
        }

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn adapter_lists_challenges_and_loads_workspace_in_host_shape() {
        let root = temp_dir("workspace");
        write_workspace(&root);

        let list =
            handle_request(&root, Request::ListChallenges).expect("challenge list should succeed");
        match list {
            ResponsePayload::ChallengeList(payload) => {
                assert_eq!(payload.current_challenge.as_deref(), Some("01-alpha"));
                assert_eq!(payload.current_language.as_deref(), Some("rust"));
                assert_eq!(payload.challenges.len(), 1);
                assert_eq!(payload.challenges[0].available_languages, vec!["rust"]);
            }
            other => panic!("unexpected response: {other:?}"),
        }

        let workspace = handle_request(
            &root,
            Request::LoadWorkspace {
                challenge_id: "01-alpha".into(),
                language: None,
            },
        )
        .expect("workspace load should succeed");
        match workspace {
            ResponsePayload::Workspace(payload) => {
                assert_eq!(payload.challenge_id, "01-alpha");
                assert_eq!(payload.language.as_deref(), Some("rust"));
                assert_eq!(payload.editor.file_path, "crates/01-alpha/src/lib.rs");
                assert!(payload.editor.content.contains("pub fn solve"));
                assert!(!payload.actions.can_reveal_hint);
            }
            other => panic!("unexpected response: {other:?}"),
        }

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn adapter_reports_progress_history_for_heatmap() {
        let root = temp_dir("progress");
        write_workspace(&root);

        progress::record_check_in_for_day(&root, "2026-03-10");
        progress::record_check_in_for_day(&root, "2026-03-10");
        progress::record_completion_for_day(&root, "alpha", "rust", "2026-03-10");

        let response = handle_request(&root, Request::LoadProgress).expect("progress should load");

        match response {
            ResponsePayload::Progress(payload) => {
                assert_eq!(payload.completed, 1);
                assert_eq!(payload.total, 1);
                assert_eq!(payload.languages.len(), 1);
                assert_eq!(payload.languages[0].language, "rust");
                assert_eq!(payload.languages[0].completed, 1);
                assert_eq!(payload.languages[0].total, 1);
                assert_eq!(payload.activity.len(), 1);
                assert_eq!(payload.activity[0].date, "2026-03-10");
                assert_eq!(payload.activity[0].check_ins, 2);
                assert_eq!(payload.activity[0].completed, 1);
                assert_eq!(payload.activity[0].commits, 0);
            }
            other => panic!("unexpected response: {other:?}"),
        }

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }
}
