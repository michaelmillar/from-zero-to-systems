mod app;
mod host_launcher;
#[allow(dead_code)]
mod lang_runner;
mod launch;
#[allow(dead_code)]
mod meta;
mod progress;
mod runner;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use app::{App, PanelMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use launch::Command as LaunchCommand;
use ratatui::{backend::CrosstermBackend, Terminal};

// Tick at 10 Hz for smooth spinner animation without hammering the CPU.
const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = std::env::current_dir()?;
    let program_name = std::env::args()
        .next()
        .unwrap_or_else(|| "play".to_string());
    let args: Vec<String> = std::env::args().skip(1).collect();

    match launch::parse_invocation(&program_name, &args)? {
        LaunchCommand::Tui => run_tui(workspace),
        LaunchCommand::Web { local, passthrough } => {
            host_launcher::run_host_web(&workspace, local, &passthrough)
        }
    }
}

fn run_tui(workspace: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let prog = progress::load(&workspace);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut app = App::new(workspace, prog);
    let result = event_loop(&mut term, &mut app);

    // Kill any in-flight cargo process before tearing down the terminal.
    app.cancel();

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    result
}

fn event_loop<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_tick = Instant::now();

    loop {
        // Wait for an event, but no longer than the remaining tick interval.
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('r') | KeyCode::Char('R') => app.run_tests(),
                        KeyCode::Char('h') | KeyCode::Char('H') => app.next_hint(),
                        KeyCode::Char('d') | KeyCode::Char('D') => app.panel = PanelMode::Docs,
                        KeyCode::Char('c') | KeyCode::Char('C') => app.panel = PanelMode::Concepts,
                        KeyCode::Char('n') | KeyCode::Right => app.go_next(),
                        KeyCode::Char('p') | KeyCode::Left => app.go_prev(),
                        KeyCode::Char('j') | KeyCode::Down => app.select_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.select_up(),
                        KeyCode::Esc => app.panel = PanelMode::Idle,
                        _ => {}
                    }
                }
            }
            // Redraw immediately on any user input for responsiveness.
            term.draw(|f| ui::render(f, app))?;
        }

        // Tick at a fixed rate regardless of how many events arrived.
        if last_tick.elapsed() >= TICK_RATE {
            app.on_tick();
            term.draw(|f| ui::render(f, app))?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}
