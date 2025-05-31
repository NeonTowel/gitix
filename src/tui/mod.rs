mod files;
mod overview;
mod save_changes;
mod settings;
mod status;
mod update;

use crate::app::{AppState, SaveChangesFocus};
use crate::git::get_git_status;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use std::io;

const TAB_TITLES: [&str; 6] = [
    "Overview",
    "Files",
    "Status",
    "Save Changes",
    "Update",
    "Settings",
];

#[derive(Copy, Clone, Debug)]
enum Tab {
    Overview,
    Files,
    Status,
    SaveChanges,
    Update,
    Settings,
}

impl Tab {
    fn all() -> &'static [Tab] {
        use Tab::*;
        &[Overview, Files, Status, SaveChanges, Update, Settings]
    }
    fn as_usize(self) -> usize {
        self as usize
    }
}

pub fn start_tui(state: &mut AppState) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut active_tab = 0;
    let tab_count = TAB_TITLES.len();

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Tab bar
                            Constraint::Min(1),    // Main area
                            Constraint::Length(2), // Key hints
                        ]
                        .as_ref(),
                    )
                    .split(size);

                // Tab bar
                let tab_titles: Vec<Line> = TAB_TITLES.iter().enumerate().map(|(i, t)| {
                    if !state.git_enabled && i > 1 {
                        Line::styled(*t, Style::default().fg(Color::DarkGray))
                    } else if active_tab == i {
                        Line::styled(*t, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    } else {
                        Line::raw(*t)
                    }
                }).collect();
                let tabs = Tabs::new(tab_titles)
                    .select(active_tab)
                    .block(Block::default().borders(Borders::ALL).title("Git TUI"))
                    .style(Style::default().fg(Color::White));
                f.render_widget(tabs, chunks[0]);

                // Main area: delegate to tab modules
                match active_tab {
                    0 => overview::render_overview_tab(f, chunks[1], state),
                    1 => files::render_files_tab(f, chunks[1], state),
                    2 => status::render_status_tab(f, chunks[1], state),
                    3 => save_changes::render_save_changes_tab(f, chunks[1], state),
                    4 => update::render_update_tab(f, chunks[1], state),
                    5 => settings::render_settings_tab(f, chunks[1], state),
                    _ => {}
                }

                // Modal popup for git init prompt
                if active_tab == 0 && state.show_init_prompt {
                    let area = centered_rect(60, 7, size);
                    let modal = Paragraph::new("This folder is not a Git repository.\n\nInitialize a new Git repository here? (Y/N)")
                        .alignment(ratatui::layout::Alignment::Center)
                        .block(
                            Block::default()
                                .title("Initialize Git Repository")
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        );
                    f.render_widget(modal, area);
                }

                // Key hints
                let hints = match active_tab {
                    1 => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [↑↓] Navigate  [Enter] Open  [q] Quit",
                    2 if state.git_enabled => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [↑↓] Navigate Files  [q] Quit",
                    3 if state.git_enabled => "[Tab] Next Tab  [↑↓] Navigate  [Space] Stage/Unstage  [Enter] Commit  [q] Quit",
                    _ => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [q] Quit",
                };
                let hint_paragraph = Paragraph::new(hints)
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(hint_paragraph, chunks[2]);
            })
            .unwrap();

        // Handle input
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if key_event.kind == KeyEventKind::Press {
                    // If showing prompt, only handle Y/N
                    if active_tab == 0 && state.show_init_prompt {
                        match key_event.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                if state.try_init_repo().is_err() {
                                    // Optionally: show error message
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                state.decline_init_repo();
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                        continue;
                    }

                    // Only allow navigation to enabled tabs
                    let max_enabled_tab = if state.git_enabled { tab_count - 1 } else { 1 };
                    match (key_event.code, key_event.modifiers) {
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            let mut next_tab = (active_tab + 1) % tab_count;
                            while !state.git_enabled && next_tab > 1 {
                                next_tab = (next_tab + 1) % tab_count;
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            active_tab = prev_tab;
                        }
                        (KeyCode::Char('q'), _) => {
                            break;
                        }
                        (KeyCode::Down, _) if active_tab == 1 => {
                            // Files tab: move selection down
                            let add_parent = state.current_dir != state.root_dir;
                            let files = crate::files::list_files(&state.current_dir, add_parent);
                            if !files.is_empty() {
                                state.files_selected_row =
                                    (state.files_selected_row + 1).min(files.len() - 1);
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 1 => {
                            // Files tab: move selection up
                            let add_parent = state.current_dir != state.root_dir;
                            let files = crate::files::list_files(&state.current_dir, add_parent);
                            if !files.is_empty() {
                                state.files_selected_row =
                                    state.files_selected_row.saturating_sub(1);
                            }
                        }
                        (KeyCode::Down, _) if active_tab == 2 => {
                            // Status tab: move selection down
                            if let Ok(git_status) = get_git_status() {
                                if !git_status.is_empty() {
                                    let current = state.status_table_state.selected().unwrap_or(0);
                                    let next = (current + 1).min(git_status.len() - 1);
                                    state.status_table_state.select(Some(next));
                                }
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 2 => {
                            // Status tab: move selection up
                            if let Ok(git_status) = get_git_status() {
                                if !git_status.is_empty() {
                                    let current = state.status_table_state.selected().unwrap_or(0);
                                    let prev = current.saturating_sub(1);
                                    state.status_table_state.select(Some(prev));
                                }
                            }
                        }
                        (KeyCode::Enter, _) if active_tab == 1 => {
                            let add_parent = state.current_dir != state.root_dir;
                            let files = crate::files::list_files(&state.current_dir, add_parent);
                            if files.is_empty() {
                                return;
                            }
                            let idx = state.files_selected_row.min(files.len() - 1);
                            let entry = &files[idx];
                            if entry.name == ".." && add_parent {
                                // Go up a directory
                                if let Some(parent) = state.current_dir.parent() {
                                    if parent.starts_with(&state.root_dir) {
                                        state.current_dir = parent.to_path_buf();
                                        state.files_selected_row = 0;
                                    }
                                }
                            } else if entry.is_dir {
                                // Go into directory
                                let mut new_dir = state.current_dir.clone();
                                new_dir.push(&entry.name);
                                if new_dir.starts_with(&state.root_dir) && new_dir.is_dir() {
                                    state.current_dir = new_dir;
                                    state.files_selected_row = 0;
                                }
                            } else {
                                // Open file in $EDITOR
                                let mut file_path = state.current_dir.clone();
                                file_path.push(&entry.name);
                                if let Ok(editor) = std::env::var("EDITOR") {
                                    let mut cmd = std::process::Command::new(&editor);
                                    // Add --wait for VSCode
                                    if editor.contains("code") {
                                        cmd.arg("--wait");
                                    }
                                    let _ = cmd.arg(&file_path).status();
                                } else {
                                    // Fallback to vi
                                    let _ =
                                        std::process::Command::new("vi").arg(&file_path).status();
                                }
                            }
                        }
                        (KeyCode::Down, _) if active_tab == 3 => {
                            // Save changes tab: seamless navigation down
                            state.save_changes_navigate_down();
                        }
                        (KeyCode::Up, _) if active_tab == 3 => {
                            // Save changes tab: seamless navigation up
                            state.save_changes_navigate_up();
                        }
                        (KeyCode::Char(' '), _) if active_tab == 3 => {
                            // Save changes tab: toggle file staging (only works when in file list)
                            if state.save_changes_focus == SaveChangesFocus::FileList {
                                state.toggle_file_staging();
                            }
                        }
                        (KeyCode::Enter, _) if active_tab == 3 => {
                            // Save changes tab: commit staged files (only works when in file list)
                            if state.save_changes_focus == SaveChangesFocus::FileList {
                                if let Err(e) = state.commit_staged_files() {
                                    // TODO: Show error message in UI
                                    eprintln!("Commit failed: {}", e);
                                }
                            } else {
                                // In commit message area, add a new line
                                state.commit_message.insert_newline();
                            }
                        }
                        // Handle commit message input when focused on commit message
                        _ if active_tab == 3
                            && state.save_changes_focus == SaveChangesFocus::CommitMessage =>
                        {
                            // For now, basic character input - more complex input handling can be added later
                            match key_event.code {
                                KeyCode::Char(c) => {
                                    state.commit_message.insert_char(c);
                                }
                                KeyCode::Backspace => {
                                    state.commit_message.delete_char();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode().unwrap();
    crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
}

// Helper function to create a centered rect for the modal
fn centered_rect(percent_x: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50 - (height / 2)),
                Constraint::Length(height),
                Constraint::Percentage(50 - (height / 2)),
            ]
            .as_ref(),
        )
        .split(r);
    let horizontal = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50 - (percent_x / 2)),
                Constraint::Percentage(percent_x),
                Constraint::Percentage(50 - (percent_x / 2)),
            ]
            .as_ref(),
        )
        .split(popup_layout[1]);
    horizontal[1]
}
