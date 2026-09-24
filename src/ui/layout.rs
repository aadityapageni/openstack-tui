use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::state::{ActiveView, AppState};
use super::theme;
use super::views;

const TABS: &[&str] = &[
    "F1 Overview",
    "F2 Nodes",
    "F3 VMs",
    "F4 Networks",
    "F5 Swift",
    "F6 Volumes",
    "F7 Images",
    "F8 Services",
];

/// Top-level layout compositor
pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.area();

    // ── Outer vertical split: tab bar | content | status bar ──────────────
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // tab bar
            Constraint::Min(0),      // main content
            Constraint::Length(2),   // status bar
        ])
        .split(size);

    draw_tabs(frame, outer[0], state);
    draw_content(frame, outer[1], state);
    draw_status_bar(frame, outer[2], state);
}

fn draw_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let active_idx = view_to_tab_index(&state.active_view);

    let tab_titles: Vec<Line> = TABS
        .iter()
        .enumerate()
        .map(|(i, &title)| {
            if i == active_idx {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    theme::tab_active_style(),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    theme::tab_inactive_style(),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(active_idx)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::BG)),
        )
        .highlight_style(theme::tab_active_style())
        .divider(Span::styled("│", theme::muted_style()));

    frame.render_widget(tabs, area);
}

fn draw_content(frame: &mut Frame, area: Rect, state: &AppState) {
    // ── Horizontal split: left list | right detail ─────────────────────────
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let left = split[0];
    let right = split[1];

    match state.active_view {
        ActiveView::Overview  => views::overview::render(frame, left, right, state),
        ActiveView::Nodes     => views::nodes::render(frame, left, right, state),
        ActiveView::Servers   => views::servers::render(frame, left, right, state),
        ActiveView::Networks  => views::networks::render(frame, left, right, state),
        ActiveView::Swift     => views::swift::render(frame, left, right, state),
        ActiveView::Volumes   => views::volumes::render(frame, left, right, state),
        ActiveView::Images    => views::images::render(frame, left, right, state),
        ActiveView::Services  => views::services::render(frame, left, right, state),
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    // Left section: keymap hints
    let keymap = " /:search  ↑↓/jk:nav  Enter:detail  d:scroll  r:refresh  q:quit";

    // Right section: cluster health + token
    let nodes_up = state.nodes_up();
    let nodes_total = state.hypervisors.len();
    let health = if state.nodes_down() == 0 && nodes_total > 0 {
        format!("✓ {}/{} nodes UP", nodes_up, nodes_total)
    } else if nodes_total == 0 {
        "Loading...".to_string()
    } else {
        format!("⚠ {}/{} nodes UP", nodes_up, nodes_total)
    };

    let token_info = format!("Token: {}", state.token_remaining_display());
    let cloud_info = format!(" Cloud: {}@{} ", state.cloud_name, state.cloud_config.region_name);

    let right_text = format!("{}  │  {}  │  {}", health, token_info, cloud_info);
    let spacer = " ".repeat(
        area.width.saturating_sub((keymap.len() + right_text.len()) as u16) as usize,
    );

    let status_line = Line::from(vec![
        Span::styled(keymap, theme::muted_style()),
        Span::raw(spacer),
        Span::styled(right_text, theme::header_style()),
    ]);

    let bar = Paragraph::new(status_line)
        .style(Style::default().bg(theme::BG_ALT))
        .block(Block::default().borders(Borders::TOP).border_style(theme::border_style()));

    // Search bar override
    if state.search_active {
        let search = Paragraph::new(Line::from(vec![
            Span::styled(" / ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(&state.search_query, Style::default().fg(theme::TEXT_BRIGHT)),
            Span::styled("█", Style::default().fg(theme::ACCENT)), // cursor
        ]))
        .style(Style::default().bg(theme::BG_ALT))
        .block(Block::default().borders(Borders::TOP).border_style(
            Style::default().fg(theme::ACCENT),
        ));
        frame.render_widget(search, area);
        return;
    }

    frame.render_widget(bar, area);
}

fn view_to_tab_index(view: &ActiveView) -> usize {
    match view {
        ActiveView::Overview => 0,
        ActiveView::Nodes    => 1,
        ActiveView::Servers  => 2,
        ActiveView::Networks => 3,
        ActiveView::Swift    => 4,
        ActiveView::Volumes  => 5,
        ActiveView::Images   => 6,
        ActiveView::Services => 7,
    }
}
