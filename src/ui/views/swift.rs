use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    style::Style,
};
use crate::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, left: Rect, right: Rect, state: &AppState) {
    render_stats(frame, left, state);
    render_info(frame, right, state);
}

fn render_stats(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![
        Line::from(Span::styled("  Swift Object Storage", theme::header_style())),
        Line::from(""),
    ];

    if let Some(stats) = &state.swift_stats {
        let bytes_gb = stats.bytes_used as f64 / 1_073_741_824.0;
        lines.push(Line::from(vec![
            Span::styled("  Containers  : ", theme::muted_style()),
            Span::styled(format!("{}", stats.container_count), theme::default_style()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Objects     : ", theme::muted_style()),
            Span::styled(format!("{}", stats.object_count), theme::default_style()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Bytes Used  : ", theme::muted_style()),
            Span::styled(format!("{:.2} GiB", bytes_gb), theme::status_up_style()),
        ]));
    } else if let Some(err) = &state.swift_status.error {
        lines.push(Line::from(Span::styled(
            format!("  Error: {}", err),
            theme::status_down_style(),
        )));
    } else {
        lines.push(Line::from(Span::styled("  Loading Swift data…", theme::muted_style())));
    }

    if let Some(last) = state.swift_status.last_updated {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Last updated: {}", last.format("%H:%M:%S")),
            theme::muted_style(),
        )));
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title("  Swift Status ")
                .borders(Borders::ALL)
                .border_style(theme::border_focused_style())
                .style(Style::default().bg(theme::BG)),
        );

    frame.render_widget(para, area);
}

fn render_info(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![
        Line::from(Span::styled("  Cluster Info", theme::header_style())),
        Line::from(""),
    ];

    if let Some(info) = &state.swift_info {
        if let Some(core) = &info.swift {
            lines.push(Line::from(vec![
                Span::styled("  Version     : ", theme::muted_style()),
                Span::raw(core.version.as_deref().unwrap_or("unknown")),
            ]));
            if let Some(max_size) = core.max_file_size {
                lines.push(Line::from(vec![
                    Span::styled("  Max Object  : ", theme::muted_style()),
                    Span::raw(format!("{} bytes", max_size)),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled("  No cluster info yet", theme::muted_style())));
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title("  Swift Cluster ")
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::BG)),
        );

    frame.render_widget(para, area);
}
