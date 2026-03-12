use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envelope<T> {
    pub id: String,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "method")]
pub enum Request {
    #[serde(rename = "handshake")]
    Handshake,
    #[serde(rename = "list_challenges")]
    ListChallenges,
    #[serde(rename = "load_workspace")]
    LoadWorkspace {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
    },
    #[serde(rename = "save_workspace")]
    SaveWorkspace {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    #[serde(rename = "reset_workspace")]
    ResetWorkspace {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
    },
    #[serde(rename = "run_tests")]
    RunTests {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    #[serde(rename = "benchmark")]
    Benchmark {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    #[serde(rename = "load_explain")]
    LoadExplain { challenge_id: String },
    #[serde(rename = "code_quality")]
    CodeQuality {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    #[serde(rename = "grade")]
    Grade {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    #[serde(rename = "load_language_notes")]
    LoadLanguageNotes { challenge_id: String },
    #[serde(rename = "load_progress")]
    LoadProgress,
    #[serde(rename = "reveal_hint")]
    RevealHint {
        challenge_id: String,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        test_name: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseEnvelope {
    pub id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ResponsePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ResponsePayload {
    Handshake {
        game_id: String,
        title: String,
        capabilities: Capabilities,
    },
    ChallengeList(ListChallengesResult),
    Workspace(WorkspaceView),
    Progress(ProgressView),
    TestRun(TestRunResult),
    Benchmark(BenchmarkRunResult),
    Explain(ExplainView),
    CodeQuality(CodeQualityView),
    Grade(GradeView),
    LanguageNotes(LanguageNotesView),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub multi_language: bool,
    pub incremental_hints: bool,
    pub benchmark: bool,
    pub explain: bool,
    pub compare: bool,
    pub idea_tools: bool,
    pub synthesis: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandshakeResult {
    pub game_id: String,
    pub title: String,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListChallengesResult {
    pub current_challenge: Option<String>,
    pub current_language: Option<String>,
    pub challenges: Vec<ChallengeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChallengeSummary {
    pub id: String,
    pub title: String,
    pub track: Option<String>,
    pub difficulty: Option<String>,
    pub status: ChallengeStatus,
    pub available_languages: Vec<String>,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    NotStarted,
    InProgress,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceView {
    pub challenge_id: String,
    pub title: String,
    pub language: Option<String>,
    pub editor: EditorState,
    pub intro: String,
    pub guide: String,
    pub concepts: Vec<String>,
    pub docs: Vec<DocLink>,
    pub hints: Vec<VisibleHint>,
    pub hint_state: HintState,
    pub actions: WorkspaceActions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorState {
    pub file_path: String,
    pub content: String,
    pub can_reset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisibleHint {
    pub label: String,
    pub body: String,
    pub cost: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HintState {
    pub mode: HintMode,
    pub revealed_count: usize,
    pub total_count: usize,
    pub next_cost: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HintMode {
    Incremental,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceActions {
    pub can_save: bool,
    pub can_test: bool,
    pub can_reveal_hint: bool,
    pub can_benchmark: bool,
    pub can_compare: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityDay {
    pub date: String,
    #[serde(default)]
    pub check_ins: u32,
    #[serde(default)]
    pub completed: u32,
    #[serde(default)]
    pub commits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageProgress {
    pub language: String,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgressView {
    pub completed: usize,
    pub total: usize,
    pub streak_days: Option<u32>,
    pub score: Option<i64>,
    #[serde(default)]
    pub activity: Vec<ActivityDay>,
    #[serde(default)]
    pub languages: Vec<LanguageProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestRunResult {
    pub passed: bool,
    pub output: String,
    pub structured_results: Vec<StructuredTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredTestResult {
    pub name: String,
    pub status: TestStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Pass,
    Fail,
    Ignored,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkRunResult {
    pub challenge_id: String,
    pub results: Vec<BenchmarkResultEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkResultEntry {
    pub language: String,
    pub ok: bool,
    pub mean_ns: Option<u64>,
    pub summary: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainView {
    pub levels: Vec<ExplainLevelView>,
    pub use_cases: String,
    pub comparisons: Vec<ExplainComparisonView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainLevelView {
    pub label: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainComparisonView {
    pub challenge_id: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeQualityView {
    pub language: String,
    pub lint_clean: bool,
    pub lint_output: String,
    pub patterns: Vec<CodeQualityPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeQualityPattern {
    pub name: String,
    pub severity: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GradeView {
    pub challenge_id: String,
    pub language: String,
    pub score: u32,
    pub max_points: u32,
    pub used_hint: bool,
    pub first_pass_recorded: bool,
    pub tests_passed: bool,
    pub lint_clean: bool,
    pub benchmark_mean_ns: Option<u64>,
    pub test_output: String,
    pub benchmark_output: String,
    pub lint_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageNotesView {
    pub languages: Vec<LanguageNoteEntry>,
    pub facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageNoteEntry {
    pub language: String,
    pub body: String,
}
