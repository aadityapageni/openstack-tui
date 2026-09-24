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
    let images = &state.images;
    let filtered: Vec<_> = images.iter().enumerate().filter(|(_, img)| {
        state.search_query.is_empty()
            || img.name.as_deref().unwrap_or("").to_lowercase().contains(&state.search_query.to_lowercase())
    }).collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, (_, img))| {
        let (sym, sty) = theme::state_symbol(&img.status);
        let size_str = img.size.map(|b| format!("{:.1} GiB", b as f64 / 1_073_741_824.0))
            .unwrap_or_else(|| "-".to_string());
        let row_style = if i == sel { theme::selected_style() } else { theme::default_style() };
        Row::new(vec![
            Cell::from(truncate(img.name.as_deref().unwrap_or("(unnamed)"), 28)),
            Cell::from(Span::styled(sym, sty)),
            Cell::from(img.disk_format.as_deref().unwrap_or("-")),
            Cell::from(size_str),
            Cell::from(img.visibility.as_deref().unwrap_or("-")),
        ]).style(row_style)
    }).collect();

    let header = Row::new(vec!["NAME", "STATUS", "FORMAT", "SIZE", "VISIBILITY"])
        .style(theme::header_style()).height(1);
    let widths = [
        Constraint::Min(28), Constraint::Length(10),
        Constraint::Length(8), Constraint::Length(12), Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .title(format!("  Glance Images [{}] ", images.len()))
            .borders(Borders::ALL)
            .border_style(theme::border_focused_style())
            .style(Style::default().bg(theme::BG)));

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let images = &state.images;
    let filtered: Vec<_> = images.iter().filter(|img| {
        state.search_query.is_empty()
            || img.name.as_deref().unwrap_or("").to_lowercase().contains(&state.search_query.to_lowercase())
    }).collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let lines: Vec<Line> = if let Some(img) = filtered.get(sel) {
        let size_str = img.size.map(|b| format!("{:.2} GiB", b as f64 / 1_073_741_824.0))
            .unwrap_or_else(|| "unknown".to_string());
        vec![
            Line::from(vec![
                Span::styled("  Image: ", theme::muted_style()),
                Span::styled(img.name.as_deref().unwrap_or("(unnamed)"), theme::header_style()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("  UUID       : ", theme::muted_style()), Span::styled(&img.id, theme::muted_style())]),
            Line::from(vec![Span::styled("  Status     : ", theme::muted_style()), Span::raw(&img.status)]),
            Line::from(vec![Span::styled("  Format     : ", theme::muted_style()), Span::raw(img.disk_format.as_deref().unwrap_or("-"))]),
            Line::from(vec![Span::styled("  Container  : ", theme::muted_style()), Span::raw(img.container_format.as_deref().unwrap_or("-"))]),
            Line::from(vec![Span::styled("  Size       : ", theme::muted_style()), Span::raw(&size_str)]),
            Line::from(vec![Span::styled("  Visibility : ", theme::muted_style()), Span::raw(img.visibility.as_deref().unwrap_or("-"))]),
            Line::from(vec![Span::styled("  Protected  : ", theme::muted_style()), Span::raw(img.protected.map(|p| if p { "yes" } else { "no" }).unwrap_or("-"))]),
            Line::from(vec![Span::styled("  Min Disk   : ", theme::muted_style()), Span::raw(img.min_disk.map(|d| format!("{} GiB", d)).unwrap_or_else(|| "-".to_string()))]),
            Line::from(vec![Span::styled("  Min RAM    : ", theme::muted_style()), Span::raw(img.min_ram.map(|r| format!("{} MiB", r)).unwrap_or_else(|| "-".to_string()))]),
            Line::from(vec![Span::styled("  Created    : ", theme::muted_style()), Span::raw(img.created_at.as_deref().unwrap_or("-"))]),
        ]
    } else {
        vec![Line::from(Span::styled("  No image selected", theme::muted_style()))]
    };

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Image Detail ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)))
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max - 1]) } else { s.to_string() }
}
