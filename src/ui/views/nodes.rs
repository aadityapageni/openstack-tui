use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
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
    let nodes = &state.hypervisors;

    // Apply search filter
    let filtered: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(_, h)| {
            state.search_query.is_empty()
                || h.hypervisor_hostname
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
        })
        .collect();

    let max_idx = filtered.len().saturating_sub(1);
    let sel = state.selected_index.min(max_idx);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_, h))| {
            let is_up = h.state == "up";
            let (state_sym, state_style) = theme::state_symbol(&h.state);

            let mem_pct = if h.memory_mb > 0 {
                (h.memory_mb_used * 100) / h.memory_mb
            } else {
                0
            };

            let row_style = if i == sel {
                theme::selected_style()
            } else {
                theme::default_style()
            };

            Row::new(vec![
                Cell::from(truncate(&h.hypervisor_hostname, 28)),
                Cell::from(Span::styled(state_sym, state_style)),
                Cell::from(format!("{}/{}", h.vcpus_used, h.vcpus)),
                Cell::from(format!(
                    "{}/{}G",
                    h.memory_mb_used / 1024,
                    h.memory_mb / 1024
                )),
                Cell::from(format!("{}%", mem_pct)),
                Cell::from(format!("{}", h.running_vms)),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec!["HOSTNAME", "STATE", "vCPUs", "MEM", "MEM%", "VMs"])
        .style(theme::header_style())
        .height(1);

    let widths = [
        Constraint::Min(28),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(6),
        Constraint::Length(5),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" 󰒋 Compute Nodes  [{}] ", nodes.len()))
                .borders(Borders::ALL)
                .border_style(theme::border_focused_style())
                .style(Style::default().bg(theme::BG)),
        )
        .row_highlight_style(theme::selected_style());

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let nodes = &state.hypervisors;
    let filtered: Vec<_> = nodes
        .iter()
        .filter(|h| {
            state.search_query.is_empty()
                || h.hypervisor_hostname
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
        })
        .collect();

    let sel = state.selected_index.min(filtered.len().saturating_sub(1));

    let content: Vec<Line> = if let Some(h) = filtered.get(sel) {
        let (state_sym, state_style) = theme::state_symbol(&h.state);
        let disk_used_gb = h.local_gb_used;
        let disk_total_gb = h.local_gb;

        vec![
            Line::from(vec![
                Span::styled("  Node: ", theme::muted_style()),
                Span::styled(&h.hypervisor_hostname, theme::header_style()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  State  : ", theme::muted_style()),
                Span::styled(state_sym, state_style),
            ]),
            Line::from(vec![
                Span::styled("  Status : ", theme::muted_style()),
                Span::raw(&h.status),
            ]),
            Line::from(vec![
                Span::styled("  Type   : ", theme::muted_style()),
                Span::raw(h.hypervisor_type.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Compute ─────────────────────", theme::muted_style())),
            Line::from(vec![
                Span::styled("  vCPUs  : ", theme::muted_style()),
                Span::styled(
                    format!("{} used / {} total", h.vcpus_used, h.vcpus),
                    theme::default_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Memory : ", theme::muted_style()),
                Span::raw(format!(
                    "{} GiB used / {} GiB total",
                    h.memory_mb_used / 1024,
                    h.memory_mb / 1024
                )),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Storage ─────────────────────", theme::muted_style())),
            Line::from(vec![
                Span::styled("  Disk   : ", theme::muted_style()),
                Span::raw(format!(
                    "{} GiB used / {} GiB total",
                    disk_used_gb, disk_total_gb
                )),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ── Workload ────────────────────", theme::muted_style())),
            Line::from(vec![
                Span::styled("  VMs    : ", theme::muted_style()),
                Span::styled(
                    format!("{} running", h.running_vms),
                    if h.running_vms > 0 { theme::status_up_style() } else { theme::muted_style() },
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "  No node selected",
            theme::muted_style(),
        ))]
    };

    // Nova service status for selected node
    let node_hostname = filtered
        .get(sel)
        .map(|h| h.hypervisor_hostname.as_str())
        .unwrap_or("");

    let mut lines = content;
    if !node_hostname.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Nova Services ───────────────",
            theme::muted_style(),
        )));

        let svcs: Vec<_> = state
            .nova_services
            .iter()
            .filter(|s| s.host.contains(node_hostname))
            .collect();

        if svcs.is_empty() {
            lines.push(Line::from(Span::styled("  (none)", theme::muted_style())));
        } else {
            for svc in svcs {
                let (sym, sty) = theme::state_symbol(&svc.state);
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:24}", svc.binary), theme::default_style()),
                    Span::styled(sym, sty),
                ]));
            }
        }
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" 󰙀 Node Detail ")
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
