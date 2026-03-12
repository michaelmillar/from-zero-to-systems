use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use host_protocol::{
    BenchmarkRunResult, CodeQualityView, Envelope, ExplainView, GradeView, HandshakeResult,
    LanguageNotesView, ListChallengesResult, ProgressView, Request, ResponseEnvelope,
    ResponsePayload, TestRunResult, WorkspaceView,
};

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

pub struct AdapterClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl AdapterClient {
    pub fn spawn(program: &str, args: &[String]) -> Result<Self, DynError> {
        Self::spawn_in(program, args, None::<&Path>)
    }

    pub fn spawn_in(
        program: &str,
        args: &[String],
        cwd: Option<impl AsRef<Path>>,
    ) -> Result<Self, DynError> {
        let mut command = Command::new(program);
        command.args(args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd.as_ref());
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("adapter stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("adapter stdout unavailable")?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    pub fn request(&mut self, payload: Request) -> Result<ResponseEnvelope, DynError> {
        let id = self.next_id.to_string();
        self.next_id += 1;

        let request = Envelope {
            id: id.clone(),
            payload,
        };
        let line = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        let read = self.stdout.read_line(&mut response_line)?;
        if read == 0 {
            return Err("adapter closed stdout before responding".into());
        }

        let response = serde_json::from_str::<ResponseEnvelope>(&response_line)?;
        if response.id != id {
            return Err(format!("protocol id mismatch: expected {id}, got {}", response.id).into());
        }
        if !response.ok {
            let message = response
                .error
                .as_ref()
                .map(|error| error.message.as_str())
                .unwrap_or("adapter request failed");
            return Err(message.to_string().into());
        }

        Ok(response)
    }

    pub fn kill(&mut self) -> Result<(), DynError> {
        self.child.kill()?;
        Ok(())
    }
}

impl Drop for AdapterClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct HostSession {
    adapter: AdapterClient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostOverview {
    pub handshake: HandshakeResult,
    pub challenges: ListChallengesResult,
    pub workspace: WorkspaceView,
    pub progress: ProgressView,
}

impl HostSession {
    pub fn connect(program: &str, args: &[String]) -> Result<Self, DynError> {
        Self::connect_in(program, args, None::<&Path>)
    }

    pub fn connect_spec(spec: &AdapterSpec) -> Result<Self, DynError> {
        Self::connect_in(&spec.program, &spec.args, spec.cwd.as_deref())
    }

    pub fn connect_in(
        program: &str,
        args: &[String],
        cwd: Option<impl AsRef<Path>>,
    ) -> Result<Self, DynError> {
        Ok(Self {
            adapter: AdapterClient::spawn_in(program, args, cwd)?,
        })
    }

    pub fn handshake(&mut self) -> Result<HandshakeResult, DynError> {
        match self.take_result(Request::Handshake, "handshake")? {
            ResponsePayload::Handshake {
                game_id,
                title,
                capabilities,
            } => Ok(HandshakeResult {
                game_id,
                title,
                capabilities,
            }),
            other => Err(format!("unexpected handshake payload: {other:?}").into()),
        }
    }

    pub fn list_challenges(&mut self) -> Result<ListChallengesResult, DynError> {
        match self.take_result(Request::ListChallenges, "challenge list")? {
            ResponsePayload::ChallengeList(result) => Ok(result),
            other => Err(format!("unexpected challenge list payload: {other:?}").into()),
        }
    }

    pub fn load_workspace(
        &mut self,
        challenge_id: String,
        language: Option<String>,
    ) -> Result<WorkspaceView, DynError> {
        match self.take_result(
            Request::LoadWorkspace {
                challenge_id,
                language,
            },
            "workspace",
        )? {
            ResponsePayload::Workspace(result) => Ok(result),
            other => Err(format!("unexpected workspace payload: {other:?}").into()),
        }
    }

    pub fn save_workspace(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        content: String,
    ) -> Result<WorkspaceView, DynError> {
        match self.take_result(
            Request::SaveWorkspace {
                challenge_id,
                language,
                content,
            },
            "workspace",
        )? {
            ResponsePayload::Workspace(result) => Ok(result),
            other => Err(format!("unexpected save payload: {other:?}").into()),
        }
    }

    pub fn reset_workspace(
        &mut self,
        challenge_id: String,
        language: Option<String>,
    ) -> Result<WorkspaceView, DynError> {
        match self.take_result(
            Request::ResetWorkspace {
                challenge_id,
                language,
            },
            "workspace",
        )? {
            ResponsePayload::Workspace(result) => Ok(result),
            other => Err(format!("unexpected reset payload: {other:?}").into()),
        }
    }

    pub fn run_tests(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        content: String,
    ) -> Result<TestRunResult, DynError> {
        match self.take_result(
            Request::RunTests {
                challenge_id,
                language,
                content,
            },
            "test run",
        )? {
            ResponsePayload::TestRun(result) => Ok(result),
            other => Err(format!("unexpected test payload: {other:?}").into()),
        }
    }

    pub fn benchmark(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        content: String,
    ) -> Result<BenchmarkRunResult, DynError> {
        match self.take_result(
            Request::Benchmark {
                challenge_id,
                language,
                content,
            },
            "benchmark",
        )? {
            ResponsePayload::Benchmark(result) => Ok(result),
            other => Err(format!("unexpected benchmark payload: {other:?}").into()),
        }
    }

    pub fn load_explain(&mut self, challenge_id: String) -> Result<ExplainView, DynError> {
        match self.take_result(Request::LoadExplain { challenge_id }, "explain")? {
            ResponsePayload::Explain(result) => Ok(result),
            other => Err(format!("unexpected explain payload: {other:?}").into()),
        }
    }

    pub fn code_quality(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        content: String,
    ) -> Result<CodeQualityView, DynError> {
        match self.take_result(
            Request::CodeQuality {
                challenge_id,
                language,
                content,
            },
            "code quality",
        )? {
            ResponsePayload::CodeQuality(result) => Ok(result),
            other => Err(format!("unexpected code quality payload: {other:?}").into()),
        }
    }

    pub fn grade(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        content: String,
    ) -> Result<GradeView, DynError> {
        match self.take_result(
            Request::Grade {
                challenge_id,
                language,
                content,
            },
            "grade",
        )? {
            ResponsePayload::Grade(result) => Ok(result),
            other => Err(format!("unexpected grade payload: {other:?}").into()),
        }
    }

    pub fn load_language_notes(
        &mut self,
        challenge_id: String,
    ) -> Result<LanguageNotesView, DynError> {
        match self.take_result(
            Request::LoadLanguageNotes { challenge_id },
            "language notes",
        )? {
            ResponsePayload::LanguageNotes(result) => Ok(result),
            other => Err(format!("unexpected language notes payload: {other:?}").into()),
        }
    }

    pub fn load_progress(&mut self) -> Result<ProgressView, DynError> {
        match self.take_result(Request::LoadProgress, "progress")? {
            ResponsePayload::Progress(result) => Ok(result),
            other => Err(format!("unexpected progress payload: {other:?}").into()),
        }
    }

    pub fn reveal_hint(
        &mut self,
        challenge_id: String,
        language: Option<String>,
        test_name: Option<String>,
    ) -> Result<WorkspaceView, DynError> {
        match self.take_result(
            Request::RevealHint {
                challenge_id,
                language,
                test_name,
            },
            "workspace",
        )? {
            ResponsePayload::Workspace(result) => Ok(result),
            other => Err(format!("unexpected reveal hint payload: {other:?}").into()),
        }
    }

    pub fn load_overview(&mut self) -> Result<HostOverview, DynError> {
        let handshake = self.handshake()?;
        let challenges = self.list_challenges()?;

        let selected_id = challenges
            .current_challenge
            .clone()
            .or_else(|| {
                challenges
                    .challenges
                    .first()
                    .map(|challenge| challenge.id.clone())
            })
            .ok_or("adapter returned no challenges")?;
        let selected_language = challenges.current_language.clone().or_else(|| {
            challenges
                .challenges
                .iter()
                .find(|challenge| challenge.id == selected_id)
                .and_then(|challenge| challenge.available_languages.first().cloned())
        });

        let workspace = self.load_workspace(selected_id, selected_language)?;
        let progress = self.load_progress()?;

        Ok(HostOverview {
            handshake,
            challenges,
            workspace,
            progress,
        })
    }

    fn take_result(&mut self, request: Request, label: &str) -> Result<ResponsePayload, DynError> {
        self.adapter
            .request(request)?
            .result
            .ok_or_else(|| format!("missing {label} result").into())
    }
}
