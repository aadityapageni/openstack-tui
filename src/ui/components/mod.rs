// Components are render helpers shared across views.
// Currently views render inline for clarity.
// This module will house:
//   - table.rs     — reusable sortable Table widget wrapper
//   - detail_pane.rs — standardized right pane renderer
//   - status_bar.rs  — extracted status bar component
//   - search.rs      — fuzzy search bar widget
//   - popup.rs       — modal popup (help, confirm, error)

pub mod table;
