use ratatui::style::{Color, Modifier, Style};

// ── Color palette ─────────────────────────────────────────────────────────────
// Inspired by catppuccin-mocha / nord dark tones

pub const BG: Color          = Color::Rgb(24, 24, 37);    // dark base
pub const BG_ALT: Color      = Color::Rgb(30, 30, 46);    // slightly lighter panel
pub const BORDER: Color      = Color::Rgb(88, 91, 112);   // subtle grey border
pub const ACCENT: Color      = Color::Rgb(137, 180, 250); // soft blue
pub const HIGHLIGHT: Color   = Color::Rgb(166, 227, 161); // green for active/up
pub const DANGER: Color      = Color::Rgb(243, 139, 168); // red for down/error
pub const WARNING: Color     = Color::Rgb(249, 226, 175); // yellow for warn/shutoff
pub const MUTED: Color       = Color::Rgb(108, 112, 134); // muted secondary text
pub const TEXT: Color        = Color::Rgb(205, 214, 244); // main text
pub const TEXT_BRIGHT: Color = Color::Rgb(255, 255, 255); // labels/headings
pub const SELECTED_BG: Color = Color::Rgb(49, 50, 68);   // selected row bg

// ── Style helpers ─────────────────────────────────────────────────────────────

pub fn default_style() -> Style {
    Style::default().fg(TEXT).bg(BG)
}

pub fn header_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default()
        .fg(TEXT_BRIGHT)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

pub fn border_focused_style() -> Style {
    Style::default().fg(ACCENT)
}

pub fn status_up_style() -> Style {
    Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
}

pub fn status_down_style() -> Style {
    Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
}

pub fn status_warn_style() -> Style {
    Style::default().fg(WARNING)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn tab_active_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_inactive_style() -> Style {
    Style::default().fg(MUTED)
}

/// Format a state string into a colored symbol
pub fn state_symbol(state: &str) -> (&'static str, Style) {
    match state.to_lowercase().as_str() {
        "up" | "active" | "alive" | "enabled" | "running" => {
            ("● UP", status_up_style())
        }
        "down" | "error" | "dead" => {
            ("● DOWN", status_down_style())
        }
        "shutoff" | "stopped" | "suspended" => {
            ("◌ OFF", status_warn_style())
        }
        "build" | "migrating" | "resize" => {
            ("↻ BUSY", Style::default().fg(Color::Cyan))
        }
        _ => ("? UNKN", muted_style()),
    }
}
