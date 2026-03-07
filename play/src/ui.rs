use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, PanelMode},
    meta::CRATES,
    runner::TestStatus,
};

const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ── style helpers ────────────────────────────────────────────────────────────

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn white_bold() -> Style {
    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
}

fn colored(c: Color) -> Style {
    Style::default().fg(c)
}

fn spinner(app: &App) -> &'static str {
    SPINNERS[app.tick_count as usize % SPINNERS.len()]
}

// ── entry point ──────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let rows = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // separator
        Constraint::Length(2), // crate strip
        Constraint::Length(1), // separator
        Constraint::Min(0),    // main content
        Constraint::Length(1), // separator
        Constraint::Length(1), // key bar
    ])
    .split(area);

    render_header(frame, app, rows[0]);
    render_sep(frame, rows[1]);
    render_strip(frame, app, rows[2]);
    render_sep(frame, rows[3]);
    render_main(frame, app, rows[4]);
    render_sep(frame, rows[5]);
    render_keys(frame, rows[6]);
}

// ── sections ─────────────────────────────────────────────────────────────────

fn render_sep(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(Paragraph::new(line).style(dim()), area);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let left  = "  from-zero-to-systems".to_string();
    let right = format!("{:02} / {}  ", app.current + 1, CRATES.len());
    let pad   = (area.width as usize).saturating_sub(left.len() + right.len());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(left,            white_bold()),
            Span::raw(" ".repeat(pad)),
            Span::styled(right,           dim()),
        ])),
        area,
    );
}

fn render_strip(frame: &mut Frame, app: &App, area: Rect) {
    let spin = spinner(app);
    let mut nums:  Vec<Span> = vec![Span::raw("  ")];
    let mut icons: Vec<Span> = vec![Span::raw("  ")];

    for (i, _) in CRATES.iter().enumerate() {
        let is_cur = i == app.current;
        let state  = &app.states[i];

        let num_sty = if is_cur {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            dim()
        };

        let (icon, icon_sty) = if state.running {
            (spin, colored(Color::Yellow))
        } else if state.is_all_pass() {
            ("✓", colored(Color::Green))
        } else if state.has_failures() {
            ("✗", colored(Color::Red))
        } else {
            ("·", dim())
        };

        nums.push(Span::styled(format!("{:02}", i + 1), num_sty));
        nums.push(Span::raw("  "));
        icons.push(Span::styled(icon.to_string(), icon_sty));
        icons.push(Span::raw("   "));
    }

    frame.render_widget(
        Paragraph::new(Text::from(vec![Line::from(nums), Line::from(icons)])),
        area,
    );
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Percentage(58),
        Constraint::Percentage(42),
    ])
    .split(area);

    render_tests(frame, app, cols[0]);
    render_context(frame, app, cols[1]);
}

fn render_tests(frame: &mut Frame, app: &App, area: Rect) {
    let meta   = &CRATES[app.current];
    let state  = &app.states[app.current];
    let spin   = spinner(app);

    let parts  = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
    let (title_a, body_a) = (parts[0], parts[1]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(meta.display, white_bold()),
        ])),
        title_a,
    );

    if state.tests.is_empty() {
        let msg = if state.running {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{spin}  running {}…", meta.package),
                    colored(Color::Yellow),
                ),
            ])
        } else if state.build_failed {
            Line::from(vec![
                Span::raw("  "),
                Span::styled("build failed  —  fix the compile error and press r", colored(Color::Red)),
            ])
        } else {
            Line::from(vec![
                Span::raw("  "),
                Span::styled("press r to run tests", dim()),
            ])
        };
        frame.render_widget(Paragraph::new(msg), body_a);
        return;
    }

    let items: Vec<ListItem> = state
        .tests
        .iter()
        .map(|t| {
            let (icon, icon_sty, name_sty) = match t.status {
                TestStatus::Pass    => ("✓", colored(Color::Green), colored(Color::White)),
                TestStatus::Fail    => ("✗", colored(Color::Red),   colored(Color::White)),
                TestStatus::Ignored => ("·", dim(),                  dim()),
            };
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(icon.to_string(), icon_sty),
                Span::raw("  "),
                Span::styled(t.name.clone(), name_sty),
            ]))
        })
        .collect();

    let mut ls = ListState::default();
    if !state.tests.is_empty() {
        ls.select(Some(app.selected_test.min(state.tests.len() - 1)));
    }

    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(28, 32, 40))
                    .add_modifier(Modifier::BOLD),
            ),
        body_a,
        &mut ls,
    );
}

fn render_context(frame: &mut Frame, app: &App, area: Rect) {
    let meta  = &CRATES[app.current];
    let state = &app.states[app.current];

    let block = Block::new()
        .borders(Borders::LEFT)
        .border_style(dim());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let parts  = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
    let (label_a, body_a) = (parts[0], parts[1]);

    let (label, body): (&str, String) = match &app.panel {
        PanelMode::Idle => (
            "info",
            format!(
                "{}\n\nCompleted  {} / {}",
                meta.intro,
                app.progress.completed.len(),
                CRATES.len(),
            ),
        ),
        PanelMode::Hint(idx) => {
            let test_name = state
                .tests
                .get(app.selected_test)
                .map(|t| t.name.as_str())
                .unwrap_or("");
            let hints = meta
                .tests
                .iter()
                .find(|th| test_name.contains(th.test_name))
                .map(|th| th.hints)
                .unwrap_or(&[]);
            let text  = hints.get(*idx).copied().unwrap_or("No hint for this test.");
            let total = hints.len().max(1);
            (
                "hint",
                format!("{test_name}\n\nhint {} of {total}\n\n{text}", idx + 1),
            )
        }
        PanelMode::Docs => (
            "docs",
            if meta.docs.is_empty() {
                "No docs listed.".into()
            } else {
                meta.docs
                    .iter()
                    .map(|d| format!("{}  {}", d.label, d.url))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            },
        ),
        PanelMode::Concepts => (
            "concepts",
            if meta.concepts.is_empty() {
                "No concepts listed.".into()
            } else {
                meta.concepts
                    .iter()
                    .map(|c| format!("·  {c}"))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            },
        ),
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(label, dim()),
        ])),
        label_a,
    );

    let body_padded = format!(" {body}");
    frame.render_widget(
        Paragraph::new(body_padded.as_str())
            .style(colored(Color::Gray))
            .wrap(Wrap { trim: false }),
        body_a,
    );
}

fn render_keys(frame: &mut Frame, area: Rect) {
    let bar = "  r run  ·  h hint  ·  d docs  ·  c concepts  ·  ← → navigate  ·  q quit";
    frame.render_widget(Paragraph::new(bar).style(dim()), area);
}
