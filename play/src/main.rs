mod app;
mod meta;
mod progress;
mod runner;
mod ui;

use std::{io, time::Duration};

use app::{App, PanelMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = std::env::current_dir()?;
    let prog      = progress::load(&workspace);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut app    = App::new(workspace, prog);
    let result     = event_loop(&mut term, &mut app);

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    result
}

fn event_loop<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    app:  &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        app.on_tick();
        term.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('r') | KeyCode::Char('R') => app.run_tests(),
                    KeyCode::Char('h') | KeyCode::Char('H') => app.next_hint(),
                    KeyCode::Char('d') | KeyCode::Char('D') => app.panel = PanelMode::Docs,
                    KeyCode::Char('c') | KeyCode::Char('C') => app.panel = PanelMode::Concepts,
                    KeyCode::Char('n') | KeyCode::Right      => app.go_next(),
                    KeyCode::Char('p') | KeyCode::Left       => app.go_prev(),
                    KeyCode::Char('j') | KeyCode::Down       => app.select_down(),
                    KeyCode::Char('k') | KeyCode::Up         => app.select_up(),
                    KeyCode::Esc                             => app.panel = PanelMode::Idle,
                    _                                        => {}
                }
            }
        }
    }
    Ok(())
}
