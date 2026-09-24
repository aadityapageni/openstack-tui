use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::state::{ActiveView, SharedState};
use super::layout;

/// Main TUI run loop.
///
/// Sets up the crossterm terminal, runs the render+event loop,
/// and tears down cleanly on exit.
pub async fn run(state: SharedState) -> Result<()> {
    // ── Terminal setup ─────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(&mut terminal, state).await;

    // ── Teardown ───────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: SharedState,
) -> Result<()> {
    let tick = Duration::from_millis(250);

    loop {
        // ── Render ─────────────────────────────────────────────────────────
        {
            let s = state.lock().await;
            terminal.draw(|frame| layout::draw(frame, &s))?;
        }

        // ── Events ─────────────────────────────────────────────────────────
        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                // Quit
                if matches!(key.code, KeyCode::Char('q'))
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers == KeyModifiers::CONTROL)
                {
                    return Ok(());
                }

                let mut s = state.lock().await;
                handle_key(&mut s, key.code, key.modifiers);
            }
        }
    }
}

fn handle_key(
    state: &mut crate::state::AppState,
    code: KeyCode,
    _modifiers: KeyModifiers,
) {
    // Search mode
    if state.search_active {
        match code {
            KeyCode::Esc => {
                state.search_active = false;
                state.search_query.clear();
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
                state.selected_index = 0;
            }
            KeyCode::Enter => {
                state.search_active = false;
            }
            _ => {}
        }
        return;
    }

    match code {
        // ── View switching (F-keys) ────────────────────────────────────────
        KeyCode::F(1) => state.active_view = ActiveView::Overview,
        KeyCode::F(2) => { state.active_view = ActiveView::Nodes; state.selected_index = 0; }
        KeyCode::F(3) => { state.active_view = ActiveView::Servers; state.selected_index = 0; }
        KeyCode::F(4) => { state.active_view = ActiveView::Networks; state.selected_index = 0; }
        KeyCode::F(5) => { state.active_view = ActiveView::Swift; state.selected_index = 0; }
        KeyCode::F(6) => { state.active_view = ActiveView::Volumes; state.selected_index = 0; }
        KeyCode::F(7) => { state.active_view = ActiveView::Images; state.selected_index = 0; }
        KeyCode::F(8) => { state.active_view = ActiveView::Services; state.selected_index = 0; }

        // ── Navigation ────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') => {
            state.selected_index = state.selected_index.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.selected_index = state.selected_index.saturating_sub(1);
        }
        KeyCode::PageDown => {
            state.selected_index = state.selected_index.saturating_add(10);
        }
        KeyCode::PageUp => {
            state.selected_index = state.selected_index.saturating_sub(10);
        }
        KeyCode::Char('g') => {
            state.selected_index = 0;
        }
        KeyCode::Char('G') => {
            // Jump to end — view handlers clamp this
            state.selected_index = usize::MAX;
        }

        // ── Search ─────────────────────────────────────────────────────────
        KeyCode::Char('/') => {
            state.search_active = true;
            state.search_query.clear();
        }
        KeyCode::Esc => {
            state.search_query.clear();
        }

        // ── Detail pane scroll ─────────────────────────────────────────────
        KeyCode::Char('u') => {
            state.detail_scroll = state.detail_scroll.saturating_sub(5);
        }
        KeyCode::Char('d') => {
            state.detail_scroll = state.detail_scroll.saturating_add(5);
        }

        _ => {}
    }
}
