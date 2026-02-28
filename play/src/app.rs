use std::{path::PathBuf, sync::mpsc::{self, Receiver}};

use crate::{
    meta::CRATES,
    progress::Progress,
    runner::{RunnerMsg, TestStatus},
};

#[derive(Debug, Clone, PartialEq)]
pub enum PanelMode {
    Idle,
    Hint(usize),
    Docs,
    Concepts,
}

pub struct TestEntry {
    pub name:   String,
    pub status: TestStatus,
}

pub struct CrateState {
    pub tests:   Vec<TestEntry>,
    pub running: bool,
}

impl CrateState {
    pub fn new() -> Self {
        Self { tests: Vec::new(), running: false }
    }

    pub fn is_all_pass(&self) -> bool {
        !self.tests.is_empty()
            && self.tests.iter().all(|t| matches!(t.status, TestStatus::Pass | TestStatus::Ignored))
    }

    pub fn has_failures(&self) -> bool {
        self.tests.iter().any(|t| matches!(t.status, TestStatus::Fail))
    }
}

pub struct App {
    pub current:       usize,
    pub selected_test: usize,
    pub panel:         PanelMode,
    pub states:        Vec<CrateState>,
    pub tick_count:    u64,
    pub workspace:     PathBuf,
    pub progress:      Progress,
    rx:                Option<Receiver<RunnerMsg>>,
    running_crate:     Option<usize>,
}

impl App {
    pub fn new(workspace: PathBuf, progress: Progress) -> Self {
        Self {
            current:       0,
            selected_test: 0,
            panel:         PanelMode::Idle,
            states:        (0..CRATES.len()).map(|_| CrateState::new()).collect(),
            tick_count:    0,
            workspace,
            progress,
            rx:            None,
            running_crate: None,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.poll_runner();
    }

    fn poll_runner(&mut self) {
        let crate_idx = match self.running_crate {
            Some(i) => i,
            None    => return,
        };

        let mut done = false;
        if let Some(rx) = &self.rx {
            loop {
                match rx.try_recv() {
                    Ok(RunnerMsg::TestResult { name, status }) => {
                        let state = &mut self.states[crate_idx];
                        if let Some(entry) = state.tests.iter_mut().find(|t| t.name == name) {
                            entry.status = status;
                        } else {
                            state.tests.push(TestEntry { name, status });
                        }
                    }
                    Ok(RunnerMsg::Done) => { done = true; break; }
                    Err(_) => break,
                }
            }
        }

        if done {
            self.rx             = None;
            self.running_crate  = None;
            self.states[crate_idx].running = false;

            if self.states[crate_idx].is_all_pass() {
                let pkg = CRATES[crate_idx].package.to_string();
                self.progress.completed.insert(pkg);
                let _ = crate::progress::save(&self.workspace, &self.progress);
            }
        }
    }

    pub fn run_tests(&mut self) {
        if self.running_crate.is_some() { return; }

        let state = &mut self.states[self.current];
        state.running = true;
        state.tests.clear();

        self.panel         = PanelMode::Idle;
        self.selected_test = 0;

        let (tx, rx) = mpsc::channel();
        self.rx           = Some(rx);
        self.running_crate = Some(self.current);

        crate::runner::spawn(CRATES[self.current].package, &self.workspace, tx);
    }

    pub fn next_hint(&mut self) {
        let meta = &CRATES[self.current];

        let test_name = self.states[self.current]
            .tests
            .get(self.selected_test)
            .map(|t| t.name.clone())
            .unwrap_or_default();

        let hints = meta.tests.iter()
            .find(|th| test_name.contains(th.test_name))
            .map(|th| th.hints)
            .unwrap_or(&[]);

        if hints.is_empty() { return; }

        let next_idx = match &self.panel {
            PanelMode::Hint(i) => (*i + 1).min(hints.len() - 1),
            _                  => 0,
        };
        self.panel = PanelMode::Hint(next_idx);
    }

    pub fn go_next(&mut self) {
        if self.current + 1 < CRATES.len() {
            self.current       += 1;
            self.selected_test  = 0;
            self.panel          = PanelMode::Idle;
        }
    }

    pub fn go_prev(&mut self) {
        if self.current > 0 {
            self.current       -= 1;
            self.selected_test  = 0;
            self.panel          = PanelMode::Idle;
        }
    }

    pub fn select_up(&mut self) {
        if self.selected_test > 0 {
            self.selected_test -= 1;
            if matches!(self.panel, PanelMode::Hint(_)) {
                self.panel = PanelMode::Idle;
            }
        }
    }

    pub fn select_down(&mut self) {
        let len = self.states[self.current].tests.len();
        if self.selected_test + 1 < len {
            self.selected_test += 1;
            if matches!(self.panel, PanelMode::Hint(_)) {
                self.panel = PanelMode::Idle;
            }
        }
    }
}
