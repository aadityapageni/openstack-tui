use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, left: Rect, right: Rect, state: &AppState) {
    render_list(frame, left, state);
    render_detail(frame, right, state);
}

fn render_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let agents = &state.agents;

    let filtered: Vec<_> = agents
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            state.search_query.is_empty()
                || a.host.to_lowercase().contains(&state.search_query.to_lowercase())
                || a.agent_type.to_lowercase().contains(&state.search_query.to_lowercase())
        })
        .collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_, agent))| {
            let alive_sym = if agent.alive { "● alive" } else { "● dead" };
            let alive_style = if agent.alive {
                theme::status_up_style()
            } else {
                theme::status_down_style()
            };

            let row_style = if i == sel {
                theme::selected_style()
            } else {
                theme::default_style()
            };

            Row::new(vec![
                Cell::from(truncate(&agent.host, 24)),
                Cell::from(truncate(&agent.agent_type, 26)),
                Cell::from(Span::styled(alive_sym, alive_style)),
                Cell::from(if agent.admin_state_up { "enabled" } else { "disabled" }),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec!["HOST", "AGENT TYPE", "ALIVE", "ADMIN"])
        .style(theme::header_style())
        .height(1);

    let widths = [
        Constraint::Min(24),
        Constraint::Min(26),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let title = format!(
        "  Network Agents [{}]  Dead: {}",
        agents.len(),
        state.agents_dead()
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(theme::border_focused_style())
                .style(Style::default().bg(theme::BG)),
        );

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let networks = &state.networks;
    let routers = &state.routers;

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!("  Networks: {}  Routers: {}", networks.len(), routers.len()),
            theme::header_style(),
        )),
        Line::from(""),
        Line::from(Span::styled("  ── Networks ────────────────────", theme::muted_style())),
    ];

    for net in networks.iter().take(15) {
        let net_type = net.network_type.as_deref().unwrap_or("?");
        let admin = if net.admin_state_up { "UP" } else { "DOWN" };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:26}", truncate(&net.name, 24)), theme::default_style()),
            Span::styled(format!("{:8}", net_type), theme::muted_style()),
            Span::styled(admin, if net.admin_state_up { theme::status_up_style() } else { theme::status_down_style() }),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  ── Routers ─────────────────────", theme::muted_style())));

    for router in routers.iter().take(10) {
        let (sym, sty) = theme::state_symbol(&router.status);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:26}", truncate(&router.name, 24)), theme::default_style()),
            Span::styled(sym, sty),
            if router.ha.unwrap_or(false) {
                Span::styled(" HA", theme::status_up_style())
            } else {
                Span::raw("")
            },
        ]));
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title("  Network Detail ")
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::BG)),
        )
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max - 1]) } else { s.to_string() }
}
