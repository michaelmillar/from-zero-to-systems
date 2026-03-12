use std::path::{Path, PathBuf};

use host_protocol::{
    ActivityDay, Capabilities, ChallengeStatus, ChallengeSummary, DocLink, EditorState, HintMode,
    HintState, LanguageNotesView, LanguageProgress, ListChallengesResult, ProgressView, Request,
    ResponsePayload, StructuredTestResult, TestRunResult, VisibleHint, WorkspaceActions,
    WorkspaceView,
};

use crate::{progress, workspace};

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
        Request::Handshake => Ok(handshake()),
        Request::ListChallenges => Ok(ResponsePayload::ChallengeList(list_challenges(workspace)?)),
        Request::LoadWorkspace {
            challenge_id,
            language: _,
        } => Ok(ResponsePayload::Workspace(load_workspace(
            workspace,
            &challenge_id,
        )?)),
        Request::SaveWorkspace {
            challenge_id,
            language: _,
            content,
        } => Ok(ResponsePayload::Workspace(save_workspace(
            workspace,
            &challenge_id,
            &content,
        )?)),
        Request::ResetWorkspace {
            challenge_id,
            language: _,
        } => Ok(ResponsePayload::Workspace(reset_workspace(
            workspace,
            &challenge_id,
        )?)),
        Request::RunTests {
            challenge_id,
            language: _,
            content,
        } => Ok(ResponsePayload::TestRun(run_tests(
            workspace,
            &challenge_id,
            &content,
        )?)),
        Request::LoadProgress => Ok(ResponsePayload::Progress(load_progress(workspace)?)),
        Request::RevealHint { .. } => Err("fzts does not support incremental hint reveals".into()),
        Request::Benchmark { .. } => {
            Err("benchmarking is not exposed through the shared host yet".into())
        }
        Request::LoadExplain { .. } => {
            Err("explain is not exposed through the shared host yet".into())
        }
        Request::CodeQuality { .. } => {
            Err("code quality is not exposed through the shared host yet".into())
        }
        Request::Grade { .. } => Err("grading is not exposed through the shared host yet".into()),
        Request::LoadLanguageNotes { .. } => {
            Ok(ResponsePayload::LanguageNotes(LanguageNotesView {
                languages: Vec::new(),
                facts: Vec::new(),
            }))
        }
    }
}

fn handshake() -> ResponsePayload {
    ResponsePayload::Handshake {
        game_id: "fzts".into(),
        title: "from-zero-to-systems".into(),
        capabilities: Capabilities {
            multi_language: false,
            incremental_hints: false,
            benchmark: false,
            explain: false,
            compare: false,
            idea_tools: false,
            synthesis: false,
        },
    }
}

fn list_challenges(workspace: &Path) -> Result<ListChallengesResult, DynError> {
    let challenges = workspace::discover_challenges(workspace)?
        .into_iter()
        .map(|challenge| ChallengeSummary {
            id: challenge.id,
            title: challenge.title,
            track: None,
            difficulty: None,
            status: if challenge.completed {
                ChallengeStatus::Complete
            } else {
                ChallengeStatus::NotStarted
            },
            available_languages: vec!["rust".into()],
            badges: Vec::new(),
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

fn load_workspace(workspace: &Path, challenge_id: &str) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(workspace::load_workspace(
        workspace,
        challenge_id,
    )?))
}

fn save_workspace(
    workspace: &Path,
    challenge_id: &str,
    content: &str,
) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(workspace::save_workspace(
        workspace,
        challenge_id,
        content,
    )?))
}

fn reset_workspace(workspace: &Path, challenge_id: &str) -> Result<WorkspaceView, DynError> {
    Ok(to_workspace_view(workspace::reset_workspace(
        workspace,
        challenge_id,
    )?))
}

fn run_tests(
    workspace: &Path,
    challenge_id: &str,
    content: &str,
) -> Result<TestRunResult, DynError> {
    let result = workspace::run_workspace_tests(workspace, challenge_id, content)?;
    Ok(TestRunResult {
        passed: result.passed,
        output: result.output,
        structured_results: Vec::<StructuredTestResult>::new(),
    })
}

fn load_progress(workspace: &Path) -> Result<ProgressView, DynError> {
    let progress = progress::load(workspace);
    let total = workspace::discover_challenges(workspace)?.len();

    Ok(ProgressView {
        completed: progress.completed.len(),
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
        languages: vec![LanguageProgress {
            language: "rust".into(),
            completed: progress.completed.len(),
            total,
        }],
    })
}

fn to_workspace_view(workspace: workspace::WorkspaceRecord) -> WorkspaceView {
    let hint_count = workspace.hints.len();

    WorkspaceView {
        challenge_id: workspace.challenge,
        title: workspace.title,
        language: Some("rust".into()),
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
        progress::record_completion_for_day(&root, "alpha", "2026-03-10");

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
