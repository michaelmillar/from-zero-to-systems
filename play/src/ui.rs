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

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer = Block::new()
        .borders(Borders::ALL)
        .title(" from-zero-to-systems ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    render_strip(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_keys(frame, chunks[2]);
}

fn spinner(tick: u64) -> &'static str {
    SPINNERS[tick as usize % SPINNERS.len()]
}

fn render_strip(frame: &mut Frame, app: &App, area: Rect) {
    let spin = spinner(app.tick_count);
    let mut nums: Vec<Span> = Vec::new();
    let mut icons: Vec<Span> = Vec::new();

    for (i, _meta) in CRATES.iter().enumerate() {
        let is_cur = i == app.current;
        let state  = &app.states[i];

        let num_style = if is_cur {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let (icon, icon_style) = if state.running {
            (spin, Style::default().fg(Color::Yellow))
        } else if state.is_all_pass() {
            ("✓", Style::default().fg(Color::Green))
        } else if state.has_failures() {
            ("✗", Style::default().fg(Color::Red))
        } else {
            ("·", Style::default().fg(Color::DarkGray))
        };

        nums.push(Span::styled(format!("{:02}", i + 1), num_style));
        nums.push(Span::raw(" "));
        icons.push(Span::styled(icon.to_string(), icon_style));
        icons.push(Span::raw("  "));
    }

    let text = Text::from(vec![Line::from(nums), Line::from(icons)]);
    frame.render_widget(Paragraph::new(text), area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(58),
        Constraint::Percentage(42),
    ])
    .split(area);

    render_test_list(frame, app, chunks[0]);
    render_context(frame, app, chunks[1]);
}

fn render_test_list(frame: &mut Frame, app: &App, area: Rect) {
    let meta  = &CRATES[app.current];
    let state = &app.states[app.current];
    let spin  = spinner(app.tick_count);
    let title = format!(" {} ", meta.display);
    let block = Block::new().borders(Borders::ALL).title(title);

    if state.tests.is_empty() {
        let msg = if state.running {
            Line::from(vec![
                Span::styled(spin.to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(format!("  running {}...", meta.package)),
            ])
        } else {
            Line::from(Span::styled(
                "  Press [r] to run tests",
                Style::default().fg(Color::DarkGray),
            ))
        };
        frame.render_widget(Paragraph::new(msg).block(block), area);
        return;
    }

    let items: Vec<ListItem> = state.tests.iter().map(|t| {
        let (icon, sty) = match t.status {
            TestStatus::Pass    => ("✓", Style::default().fg(Color::Green)),
            TestStatus::Fail    => ("✗", Style::default().fg(Color::Red)),
            TestStatus::Ignored => ("~", Style::default().fg(Color::DarkGray)),
        };
        ListItem::new(Line::from(vec![
            Span::styled(icon.to_string(), sty),
            Span::raw("  "),
            Span::raw(t.name.clone()),
        ]))
    })
    .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let mut ls = ListState::default();
    if !state.tests.is_empty() {
        ls.select(Some(app.selected_test.min(state.tests.len() - 1)));
    }

    frame.render_stateful_widget(list, area, &mut ls);
}

fn render_context(frame: &mut Frame, app: &App, area: Rect) {
    let meta  = &CRATES[app.current];
    let state = &app.states[app.current];

    let (title, body): (String, String) = match &app.panel {
        PanelMode::Idle => (
            " Info ".to_string(),
            format!(
                "Crate: {}\n\nCompleted: {}/{}\n\n[r] run tests\n[h] next hint\n[d] docs\n[c] concepts\n[jk / ↑↓] select test\n[np / ←→] prev / next crate\n[q] quit",
                meta.display,
                app.progress.completed.len(),
                CRATES.len(),
            ),
        ),

        PanelMode::Hint(idx) => {
            let test_name = state.tests
                .get(app.selected_test)
                .map(|t| t.name.as_str())
                .unwrap_or("");

            let hints = meta.tests.iter()
                .find(|th| test_name.contains(th.test_name))
                .map(|th| th.hints)
                .unwrap_or(&[]);

            let text  = hints.get(*idx).copied().unwrap_or("No hint recorded for this test.");
            let total = hints.len().max(1);
            (
                format!(" Hint {}/{total} ", idx + 1),
                format!("Test: {test_name}\n\n{text}"),
            )
        }

        PanelMode::Docs => {
            let body = if meta.docs.is_empty() {
                "No docs listed.".to_string()
            } else {
                meta.docs.iter()
                    .map(|d| format!("• {}\n  {}", d.label, d.url))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            };
            (" Docs ".to_string(), body)
        }

        PanelMode::Concepts => {
            let body = if meta.concepts.is_empty() {
                "No concepts listed.".to_string()
            } else {
                meta.concepts.iter()
                    .map(|c| format!("• {c}"))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            };
            (" Concepts ".to_string(), body)
        }
    };

    let block = Block::new().borders(Borders::ALL).title(title);
    frame.render_widget(Paragraph::new(body).block(block).wrap(Wrap { trim: false }), area);
}

fn render_keys(frame: &mut Frame, area: Rect) {
    let bar = "[r]un  [h]int  [d]ocs  [c]oncepts  [←/p]prev  [→/n]next  [q]uit";
    frame.render_widget(
        Paragraph::new(bar).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
