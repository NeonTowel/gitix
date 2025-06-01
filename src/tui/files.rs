use crate::app::AppState;
use crate::files::{list_files, list_files_with_git_status, FileEntry};
use crate::git::format_file_size;
use crate::tui::theme::Theme;
use chrono::{Local, NaiveDateTime};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::{layout::Rect, Frame};

pub fn render_files_tab(f: &mut Frame, area: Rect, state: &mut AppState) {
    // Use configured theme from app state
    let theme = Theme::with_accents_and_title(
        state.current_theme_accent,
        state.current_theme_accent2,
        state.current_theme_accent3,
        state.current_theme_title,
    );

    // Set panel background
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    let add_parent = state.current_dir != state.root_dir;

    // Load git status if git is enabled and not already loaded
    if state.git_enabled {
        state.load_status_git_status();
    }

    // Use enhanced file listing with git status if git is enabled
    let files = if state.git_enabled {
        list_files_with_git_status(&state.current_dir, add_parent, &state.status_git_status)
    } else {
        list_files(&state.current_dir, add_parent)
    };

    // Update header to include Tracked and Status columns
    let header = if state.git_enabled {
        [
            "Permissions",
            "Size",
            "Modified",
            "Tracked",
            "Status",
            "Name",
        ]
    } else {
        ["Permissions", "Size", "Modified", "", "", "Name"]
    };

    let rows: Vec<Row> = files
        .iter()
        .map(|entry| {
            let perms = format_permissions(entry.permissions, entry.is_dir);

            // Use format_file_size function like in status tab
            let size = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                format_file_size(Some(entry.size))
            };

            let modified = format_time(entry.modified);

            // Format tracked indicator (checkmark for tracked files)
            let tracked = if state.git_enabled {
                match &entry.git_status {
                    Some(crate::git::FileStatusType::Untracked) => "", // Untracked files get no checkmark
                    Some(_) => "✓", // Files with any other status are tracked
                    None => {
                        if entry.is_dir || entry.name == ".." {
                            "" // Directories and parent dir get no indicator
                        } else {
                            "✓" // Clean files (no git status) are tracked
                        }
                    }
                }
            } else {
                ""
            };

            // Format git status description (only show for files with actual changes)
            let status_description = if state.git_enabled {
                match &entry.git_status {
                    Some(git_status) => git_status.as_description(),
                    None => "", // Clean tracked files show no status
                }
            } else {
                ""
            };

            let mut style = theme.text_style();
            if entry.is_dir {
                style = theme.accent3_style().add_modifier(Modifier::BOLD);
            } else if entry.permissions & 0o111 != 0 {
                style = theme.success_style();
            }

            // Create cells with appropriate styling
            let perm_cell = Cell::from(perms).style(style);
            let size_cell = Cell::from(size).style(style);
            let modified_cell = Cell::from(modified).style(style);

            // Tracked cell with tertiary accent color for checkmarks
            let tracked_cell = if tracked == "✓" {
                Cell::from(tracked).style(theme.accent3_style())
            } else {
                Cell::from(tracked).style(style)
            };

            let name_cell = Cell::from(entry.name.clone()).style(style);

            // Status cell with git status coloring
            let status_cell = if let Some(git_status) = &entry.git_status {
                Cell::from(status_description).style(
                    Style::default()
                        .fg(git_status.color())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Cell::from(status_description).style(style)
            };

            Row::new(vec![
                perm_cell,
                size_cell,
                modified_cell,
                tracked_cell,
                status_cell,
                name_cell,
            ])
        })
        .collect();

    // Update column widths to accommodate Tracked and Status columns
    let widths = if state.git_enabled {
        [
            Constraint::Length(12), // Permissions
            Constraint::Length(10), // Size
            Constraint::Length(20), // Modified
            Constraint::Length(8),  // Tracked
            Constraint::Length(12), // Status
            Constraint::Min(15),    // Name
        ]
    } else {
        [
            Constraint::Length(12), // Permissions
            Constraint::Length(10), // Size
            Constraint::Length(20), // Modified
            Constraint::Length(0),  // Tracked (hidden)
            Constraint::Length(0),  // Status (hidden)
            Constraint::Min(10),    // Name
        ]
    };

    let mut table_state = TableState::default();
    if !files.is_empty() {
        table_state.select(Some(state.files_selected_row.min(files.len() - 1)));
    }

    // Update title to reflect git integration
    let title = "Files".to_string();

    let table = Table::new(rows, widths)
        .header(Row::new(header).style(theme.accent2_style()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        )
        .column_spacing(1)
        .row_highlight_style(theme.highlight_style())
        .highlight_symbol("► ");
    f.render_stateful_widget(table, area, &mut table_state);
}

fn format_permissions(perm: u32, is_dir: bool) -> String {
    let mut s = String::new();
    s.push(if is_dir { 'd' } else { '-' });
    for i in (0..3).rev() {
        let shift = i * 3;
        let r = if perm & (0o400 >> (6 - shift)) != 0 {
            'r'
        } else {
            '-'
        };
        let w = if perm & (0o200 >> (6 - shift)) != 0 {
            'w'
        } else {
            '-'
        };
        let x = if perm & (0o100 >> (6 - shift)) != 0 {
            'x'
        } else {
            '-'
        };
        s.push(r);
        s.push(w);
        s.push(x);
    }
    s
}

fn format_time(secs: u64) -> String {
    let dt = NaiveDateTime::from_timestamp_opt(secs as i64, 0)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    let offset = chrono::Local::now().offset().to_owned();
    let dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::from_naive_utc_and_offset(dt, offset);
    dt.format("%Y-%m-%d %H:%M").to_string()
}
