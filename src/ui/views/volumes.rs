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
    let volumes = &state.volumes;
    let filtered: Vec<_> = volumes.iter().enumerate().filter(|(_, v)| {
        state.search_query.is_empty()
            || v.name.as_deref().unwrap_or("").to_lowercase().contains(&state.search_query.to_lowercase())
    }).collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, (_, vol))| {
        let (sym, sty) = theme::state_symbol(&vol.status);
        let attached = if vol.attachments.as_ref().map(|a| !a.is_empty()).unwrap_or(false) {
            "attached"
        } else {
            "free"
        };
        let row_style = if i == sel { theme::selected_style() } else { theme::default_style() };
        Row::new(vec![
            Cell::from(truncate(vol.name.as_deref().unwrap_or("(unnamed)"), 24)),
            Cell::from(Span::styled(sym, sty)),
            Cell::from(format!("{} GiB", vol.size)),
            Cell::from(vol.volume_type.as_deref().unwrap_or("-")),
            Cell::from(attached),
        ]).style(row_style)
    }).collect();

    let header = Row::new(vec!["NAME", "STATUS", "SIZE", "TYPE", "ATTACHED"])
        .style(theme::header_style()).height(1);
    let widths = [
        Constraint::Min(24), Constraint::Length(12),
        Constraint::Length(10), Constraint::Length(14), Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .title(format!("  Block Volumes [{}] ", volumes.len()))
            .borders(Borders::ALL)
            .border_style(theme::border_focused_style())
            .style(Style::default().bg(theme::BG)));

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("  Cinder Services", theme::header_style())),
        Line::from(""),
    ];

    for svc in &state.cinder_services {
        let (sym, sty) = theme::state_symbol(&svc.state);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:20}", truncate(&svc.binary, 20)), theme::default_style()),
            Span::styled(format!("{:20}", truncate(&svc.host, 18)), theme::muted_style()),
            Span::styled(sym, sty),
        ]));
    }

    if state.cinder_services.is_empty() {
        lines.push(Line::from(Span::styled("  Loading…", theme::muted_style())));
    }

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Cinder Services ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)))
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max - 1]) } else { s.to_string() }
}
