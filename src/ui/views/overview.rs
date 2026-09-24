use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use crate::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, left: Rect, right: Rect, state: &AppState) {
    // Overview uses the full width split into two column sections
    render_left(frame, left, state);
    render_right(frame, right, state);
}

fn render_left(frame: &mut Frame, area: Rect, state: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Compute summary
            Constraint::Length(8),   // Network summary
            Constraint::Min(0),      // Storage summary
        ])
        .split(area);

    render_compute_summary(frame, sections[0], state);
    render_network_summary(frame, sections[1], state);
    render_storage_summary(frame, sections[2], state);
}

fn render_right(frame: &mut Frame, area: Rect, state: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_vm_summary(frame, sections[0], state);
    render_service_health(frame, sections[1], state);
}

fn render_compute_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let total_vcpu = state.total_vcpus();
    let used_vcpu = state.used_vcpus();
    let total_mem = state.total_memory_mb();
    let used_mem = state.used_memory_mb();

    let vcpu_pct = if total_vcpu > 0 { (used_vcpu * 100 / total_vcpu) as u16 } else { 0 };
    let mem_pct = if total_mem > 0 { (used_mem * 100 / total_mem) as u16 } else { 0 };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Nodes    : ", theme::muted_style()),
            Span::styled(format!("{} up", state.nodes_up()), theme::status_up_style()),
            Span::raw("  /  "),
            Span::styled(format!("{} down", state.nodes_down()), if state.nodes_down() > 0 { theme::status_down_style() } else { theme::muted_style() }),
        ]),
        Line::from(vec![
            Span::styled("  vCPUs    : ", theme::muted_style()),
            Span::raw(format!("{}/{} ({vcpu_pct}%)", used_vcpu, total_vcpu)),
        ]),
        Line::from(vec![
            Span::styled("  Memory   : ", theme::muted_style()),
            Span::raw(format!("{}/{} GiB ({mem_pct}%)", used_mem / 1024, total_mem / 1024)),
        ]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Compute ")
            .borders(Borders::ALL)
            .border_style(theme::border_focused_style())
            .style(Style::default().bg(theme::BG)));
    frame.render_widget(para, area);
}

fn render_network_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let alive = state.agents_alive();
    let dead = state.agents_dead();

    let lines = vec![
        Line::from(vec![
            Span::styled("  Networks  : ", theme::muted_style()),
            Span::raw(format!("{}", state.networks.len())),
        ]),
        Line::from(vec![
            Span::styled("  Routers   : ", theme::muted_style()),
            Span::raw(format!("{}", state.routers.len())),
        ]),
        Line::from(vec![
            Span::styled("  Agents    : ", theme::muted_style()),
            Span::styled(format!("{} alive", alive), theme::status_up_style()),
            Span::raw("  /  "),
            Span::styled(format!("{} dead", dead), if dead > 0 { theme::status_down_style() } else { theme::muted_style() }),
        ]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Network ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)));
    frame.render_widget(para, area);
}

fn render_storage_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let total_vol_gib: u64 = state.volumes.iter().map(|v| v.size).sum();
    let attached: usize = state.volumes.iter()
        .filter(|v| v.attachments.as_ref().map(|a| !a.is_empty()).unwrap_or(false))
        .count();

    let lines = vec![
        Line::from(vec![
            Span::styled("  Volumes   : ", theme::muted_style()),
            Span::raw(format!("{} ({} attached)", state.volumes.len(), attached)),
        ]),
        Line::from(vec![
            Span::styled("  Total Cap : ", theme::muted_style()),
            Span::raw(format!("{} GiB", total_vol_gib)),
        ]),
        Line::from(vec![
            Span::styled("  Images    : ", theme::muted_style()),
            Span::raw(format!("{}", state.images.len())),
        ]),
        Line::from(if let Some(stats) = &state.swift_stats {
            vec![
                Span::styled("  Swift Obj : ", theme::muted_style()),
                Span::raw(format!("{} objects / {:.1} GiB",
                    stats.object_count,
                    stats.bytes_used as f64 / 1_073_741_824.0
                )),
            ]
        } else {
            vec![Span::styled("  Swift     : loading…", theme::muted_style())]
        }),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Storage ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)));
    frame.render_widget(para, area);
}

fn render_vm_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let total = state.servers.len();
    let active = state.active_servers();
    let error = state.servers.iter().filter(|s| s.status == "ERROR").count();

    let lines = vec![
        Line::from(vec![
            Span::styled("  Total     : ", theme::muted_style()),
            Span::raw(format!("{}", total)),
        ]),
        Line::from(vec![
            Span::styled("  Active    : ", theme::muted_style()),
            Span::styled(format!("{}", active), theme::status_up_style()),
        ]),
        Line::from(vec![
            Span::styled("  Stopped   : ", theme::muted_style()),
            Span::raw(format!("{}", state.servers.iter().filter(|s| s.status == "SHUTOFF").count())),
        ]),
        Line::from(vec![
            Span::styled("  Error     : ", theme::muted_style()),
            Span::styled(format!("{}", error), if error > 0 { theme::status_down_style() } else { theme::muted_style() }),
        ]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  Virtual Machines ")
            .borders(Borders::ALL)
            .border_style(theme::border_focused_style())
            .style(Style::default().bg(theme::BG)));
    frame.render_widget(para, area);
}

fn render_service_health(frame: &mut Frame, area: Rect, state: &AppState) {
    let nova_ok = state.nova_status.error.is_none() && !state.hypervisors.is_empty();
    let neutron_ok = state.neutron_status.error.is_none() && !state.agents.is_empty();
    let swift_ok = state.swift_status.error.is_none();
    let cinder_ok = state.cinder_status.error.is_none();
    let glance_ok = state.glance_status.error.is_none() && !state.images.is_empty();

    let mk_line = |name: &str, ok: bool| -> Line {
        let (sym, sty) = if ok {
            ("● healthy", theme::status_up_style())
        } else {
            ("● degraded", theme::status_down_style())
        };
        Line::from(vec![
            Span::styled(format!("  {:12}", name), theme::default_style()),
            Span::styled(sym, sty),
        ])
    };

    let lines = vec![
        Line::from(Span::styled("  Service Health", theme::header_style())),
        Line::from(""),
        mk_line("Nova", nova_ok),
        mk_line("Neutron", neutron_ok),
        mk_line("Swift", swift_ok),
        mk_line("Cinder", cinder_ok),
        mk_line("Glance", glance_ok),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default()
            .title("  API Services ")
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)));
    frame.render_widget(para, area);
}
