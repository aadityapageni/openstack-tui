use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    layout::Constraint,
};

use crate::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, left: Rect, right: Rect, state: &AppState) {
    render_list(frame, left, state);
    render_detail(frame, right, state);
}

fn render_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let servers = &state.servers;

    let filtered: Vec<_> = servers
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            state.search_query.is_empty()
                || s.name.to_lowercase().contains(&state.search_query.to_lowercase())
                || s.id.starts_with(&state.search_query)
        })
        .collect();

    let max_idx = filtered.len().saturating_sub(1);
    let sel = state.selected_index.min(max_idx);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_, srv))| {
            let (state_sym, state_style) = theme::state_symbol(&srv.status);
            let host = srv.host.as_deref().unwrap_or("-");
            let az = srv.availability_zone.as_deref().unwrap_or("-");

            let row_style = if i == sel {
                theme::selected_style()
            } else {
                theme::default_style()
            };

            Row::new(vec![
                Cell::from(truncate(&srv.name, 26)),
                Cell::from(Span::styled(state_sym, state_style)),
                Cell::from(truncate(host, 20)),
                Cell::from(az),
                Cell::from(truncate(&srv.id[..8.min(srv.id.len())], 10)),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec!["NAME", "STATUS", "HOST", "AZ", "UUID"])
        .style(theme::header_style())
        .height(1);

    let widths = [
        Constraint::Min(26),
        Constraint::Length(10),
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!("  Running VMs  [{}] ", servers.len()))
                .borders(Borders::ALL)
                .border_style(theme::border_focused_style())
                .style(Style::default().bg(theme::BG)),
        );

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let servers = &state.servers;

    let filtered: Vec<_> = servers
        .iter()
        .filter(|s| {
            state.search_query.is_empty()
                || s.name.to_lowercase().contains(&state.search_query.to_lowercase())
        })
        .collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let lines: Vec<Line> = if let Some(srv) = filtered.get(sel) {
        let (state_sym, state_style) = theme::state_symbol(&srv.status);
        let power = match srv.power_state.unwrap_or(0) {
            1 => "Running",
            3 => "Paused",
            4 => "Shutdown",
            6 => "Crashed",
            7 => "Suspended",
            _ => "Unknown",
        };

        let mut base = vec![
            Line::from(vec![
                Span::styled("  VM: ", theme::muted_style()),
                Span::styled(&srv.name, theme::header_style()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status      : ", theme::muted_style()),
                Span::styled(state_sym, state_style),
            ]),
            Line::from(vec![
                Span::styled("  Power State : ", theme::muted_style()),
                Span::raw(power),
            ]),
            Line::from(vec![
                Span::styled("  UUID        : ", theme::muted_style()),
                Span::styled(&srv.id, theme::muted_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Placement ──────────────────", theme::muted_style())),
            Line::from(vec![
                Span::styled("  Host        : ", theme::muted_style()),
                Span::raw(srv.host.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(vec![
                Span::styled("  Hypervisor  : ", theme::muted_style()),
                Span::raw(srv.hypervisor_hostname.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(vec![
                Span::styled("  Avail Zone  : ", theme::muted_style()),
                Span::raw(srv.availability_zone.as_deref().unwrap_or("-")),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Timestamps ─────────────────", theme::muted_style())),
            Line::from(vec![
                Span::styled("  Created     : ", theme::muted_style()),
                Span::raw(srv.created.as_deref().unwrap_or("-")),
            ]),
            Line::from(vec![
                Span::styled("  Updated     : ", theme::muted_style()),
                Span::raw(srv.updated.as_deref().unwrap_or("-")),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Flavor ─────────────────────", theme::muted_style())),
        ];

        if let Some(f) = &srv.flavor {
            base.push(Line::from(vec![
                Span::styled("  Original Name: ", theme::muted_style()),
                Span::raw(f.original_name.clone().unwrap_or_else(|| "-".to_string())),
            ]));
            base.push(Line::from(vec![
                Span::styled("  vCPUs        : ", theme::muted_style()),
                Span::raw(format!("{}", f.vcpus.unwrap_or(0))),
            ]));
            base.push(Line::from(vec![
                Span::styled("  RAM          : ", theme::muted_style()),
                Span::raw(format!("{} MiB", f.ram.unwrap_or(0))),
            ]));
            base.push(Line::from(vec![
                Span::styled("  Disk         : ", theme::muted_style()),
                Span::raw(format!("{} GiB", f.disk.unwrap_or(0))),
            ]));
        } else {
            base.push(Line::from(Span::styled("  (flavor data unavailable)", theme::muted_style())));
        }
        
        base
    } else {
        vec![Line::from(Span::styled("  No VM selected", theme::muted_style()))]
    };

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title("  VM Detail ")
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::BG)),
        )
        .scroll((state.detail_scroll, 0));

    frame.render_widget(para, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}
