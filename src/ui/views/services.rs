use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, left: Rect, right: Rect, state: &AppState) {
    render_nova_services(frame, left, state);
    render_neutron_agents(frame, right, state);
}

fn render_nova_services(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("  Nova Services", theme::header_style())),
        Line::from(""),
    ];

    for svc in &state.nova_services {
        let (sym, sty) = theme::state_symbol(&svc.state);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:20}", truncate(&svc.binary, 20)), theme::default_style()),
            Span::styled(format!("{:20}", truncate(&svc.host, 18)), theme::muted_style()),
            Span::styled(sym, sty),
        ]));
    }

    if state.nova_services.is_empty() {
        lines.push(Line::from(Span::styled(
            if state.nova_status.error.is_some() { "  ⚠ Error fetching" } else { "  Loading…" },
            theme::muted_style(),
        )));
    }

    if let Some(err) = &state.nova_status.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Error: {}", err),
            theme::status_down_style(),
        )));
    }

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Nova Service Health ")
            .borders(Borders::ALL)
            .border_style(theme::border_focused_style())
            .style(Style::default().bg(theme::BG)))
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn render_neutron_agents(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("  Neutron Agents", theme::header_style())),
        Line::from(""),
    ];

    for agent in &state.agents {
        let sym = if agent.alive { "● alive" } else { "● dead" };
        let sty = if agent.alive { theme::status_up_style() } else { theme::status_down_style() };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:22}", truncate(&agent.binary, 20)), theme::default_style()),
            Span::styled(format!("{:18}", truncate(&agent.host, 16)), theme::muted_style()),
            Span::styled(sym, sty),
        ]));
    }

    if state.agents.is_empty() {
        lines.push(Line::from(Span::styled("  Loading…", theme::muted_style())));
    }

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Neutron Agent Health ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)))
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max - 1]) } else { s.to_string() }
}
