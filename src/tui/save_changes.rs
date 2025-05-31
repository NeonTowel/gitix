use crate::app::{AppState, SaveChangesFocus};
use crate::git::{commit, format_file_size, get_git_status, stage_file, unstage_file};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::{Frame, layout::Rect};
use std::path::PathBuf;

pub fn render_save_changes_tab(f: &mut Frame, area: Rect, state: &mut AppState) {
    // Split the area into file list (top) and commit message (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70), // File list
            Constraint::Percentage(30), // Commit message area
        ])
        .split(area);

    render_file_list(f, chunks[0], state);
    render_commit_area(f, chunks[1], state);
}

fn render_file_list(f: &mut Frame, area: Rect, state: &mut AppState) {
    let git_status = match get_git_status() {
        Ok(files) => files,
        Err(e) => {
            let error_paragraph = Paragraph::new(format!("Error reading repository: {}", e))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Save Changes - Error"),
                );
            f.render_widget(error_paragraph, area);
            return;
        }
    };

    if git_status.is_empty() {
        let clean_paragraph =
            Paragraph::new("✓ No changes to commit\n\nYour working directory is clean.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Save Changes"));
        f.render_widget(clean_paragraph, area);
        return;
    }

    // Ensure table state selection is valid
    if !git_status.is_empty() {
        let current_selection = state.save_changes_table_state.selected().unwrap_or(0);
        if current_selection >= git_status.len() {
            state.save_changes_table_state.select(Some(0));
        } else if state.save_changes_table_state.selected().is_none() {
            state.save_changes_table_state.select(Some(0));
        }
    }

    // Create table headers
    let header = Row::new(vec![
        Cell::from("Staged").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("File Path").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Size").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    // Create table rows
    let rows: Vec<Row> = git_status
        .iter()
        .map(|file| {
            let is_staged = state.staged_files.contains(&file.path);

            let staged_cell = Cell::from(if is_staged { "●" } else { "○" })
                .style(Style::default().fg(if is_staged { Color::Green } else { Color::Gray }));

            let path_cell = Cell::from(file.path.display().to_string())
                .style(Style::default().fg(Color::White));

            let status_cell = Cell::from(file.status.as_description()).style(
                Style::default()
                    .fg(file.status.color())
                    .add_modifier(Modifier::BOLD),
            );

            let size_cell = Cell::from(format_file_size(file.file_size))
                .style(Style::default().fg(Color::Gray));

            Row::new(vec![staged_cell, path_cell, status_cell, size_cell])
        })
        .collect();

    // Determine border style based on focus
    let border_style = if state.save_changes_focus == SaveChangesFocus::FileList {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Create the table
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),      // Staged indicator
            Constraint::Percentage(50), // File path
            Constraint::Percentage(25), // Status column
            Constraint::Percentage(15), // Size column
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(
                "Files to Commit ({} total, {} staged) - [Space] to stage/unstage",
                git_status.len(),
                state.staged_files.len()
            )),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("► ");

    f.render_stateful_widget(table, area, &mut state.save_changes_table_state);
}

fn render_commit_area(f: &mut Frame, area: Rect, state: &mut AppState) {
    // Split commit area into message input and buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Commit message input
            Constraint::Length(3), // Buttons/status
        ])
        .split(area);

    // Render commit message input
    let border_style = if state.save_changes_focus == SaveChangesFocus::CommitMessage {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let commit_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Commit Message - [↑↓] to navigate");

    let inner_area = commit_block.inner(chunks[0]);
    f.render_widget(commit_block, chunks[0]);
    f.render_widget(state.commit_message.widget(), inner_area);

    // Render status/buttons area
    let staged_count = state.staged_files.len();
    let status_text = if staged_count > 0 {
        format!(
            "Ready to commit {} file(s) - [Enter] to commit",
            staged_count
        )
    } else {
        "No files staged for commit".to_string()
    };

    let status_style = if staged_count > 0 {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let status_paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status_paragraph, chunks[1]);
}

// Helper functions for handling user input
impl AppState {
    pub fn toggle_file_staging(&mut self) {
        if let Ok(git_status) = get_git_status() {
            if let Some(selected_idx) = self.save_changes_table_state.selected() {
                if selected_idx < git_status.len() {
                    let file_path = &git_status[selected_idx].path;

                    if self.staged_files.contains(file_path) {
                        // Unstage the file
                        self.staged_files.retain(|p| p != file_path);
                        let _ = unstage_file(file_path);
                    } else {
                        // Stage the file
                        self.staged_files.push(file_path.clone());
                        let _ = stage_file(file_path);
                    }
                }
            }
        }
    }

    pub fn commit_staged_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.staged_files.is_empty() {
            return Err("No files staged for commit".into());
        }

        let commit_message = self.commit_message.lines().join("\n");
        if commit_message.trim().is_empty() {
            return Err("Commit message cannot be empty".into());
        }

        // Perform the commit
        commit(&commit_message)?;

        // Clear staged files and commit message
        self.staged_files.clear();
        self.commit_message = tui_textarea::TextArea::new(vec![String::new()]);

        Ok(())
    }

    pub fn switch_save_changes_focus(&mut self) {
        self.save_changes_focus = match self.save_changes_focus {
            SaveChangesFocus::FileList => SaveChangesFocus::CommitMessage,
            SaveChangesFocus::CommitMessage => SaveChangesFocus::FileList,
        };
    }

    /// Navigate down in save changes tab - seamlessly move from file list to commit message
    pub fn save_changes_navigate_down(&mut self) {
        match self.save_changes_focus {
            SaveChangesFocus::FileList => {
                if let Ok(git_status) = get_git_status() {
                    if !git_status.is_empty() {
                        let current = self.save_changes_table_state.selected().unwrap_or(0);
                        if current < git_status.len() - 1 {
                            // Move down in the file list
                            let next = current + 1;
                            self.save_changes_table_state.select(Some(next));
                        } else {
                            // At the end of file list, move to commit message
                            self.save_changes_focus = SaveChangesFocus::CommitMessage;
                        }
                    } else {
                        // No files, move directly to commit message
                        self.save_changes_focus = SaveChangesFocus::CommitMessage;
                    }
                }
            }
            SaveChangesFocus::CommitMessage => {
                // Already at commit message, handle textarea navigation
                self.commit_message
                    .move_cursor(tui_textarea::CursorMove::Down);
            }
        }
    }

    /// Navigate up in save changes tab - seamlessly move from commit message to file list
    pub fn save_changes_navigate_up(&mut self) {
        match self.save_changes_focus {
            SaveChangesFocus::FileList => {
                if let Ok(git_status) = get_git_status() {
                    if !git_status.is_empty() {
                        let current = self.save_changes_table_state.selected().unwrap_or(0);
                        if current > 0 {
                            // Move up in the file list
                            let prev = current - 1;
                            self.save_changes_table_state.select(Some(prev));
                        }
                        // If already at top of file list, stay there
                    }
                }
            }
            SaveChangesFocus::CommitMessage => {
                // Check if we're at the top of the commit message
                let cursor_row = self.commit_message.cursor().0;
                if cursor_row == 0 {
                    // At top of commit message, move back to file list
                    self.save_changes_focus = SaveChangesFocus::FileList;
                    // Select the last item in the file list
                    if let Ok(git_status) = get_git_status() {
                        if !git_status.is_empty() {
                            let last_idx = git_status.len() - 1;
                            self.save_changes_table_state.select(Some(last_idx));
                        }
                    }
                } else {
                    // Move up within the commit message
                    self.commit_message
                        .move_cursor(tui_textarea::CursorMove::Up);
                }
            }
        }
    }
}
