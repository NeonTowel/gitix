use crate::app::{AppState, SaveChangesFocus, TemplatePopupSelection};
use crate::git::{commit, format_file_size, get_git_status, stage_file, unstage_file};
use crate::tui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Table, Wrap,
};
use ratatui::{Frame, layout::Rect};
use std::path::PathBuf;

pub fn render_save_changes_tab(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    // Load git status cache if not already loaded (when tab becomes active)
    state.load_save_changes_git_status();

    // Safety check: ensure focus is on commit message if there are no changes to commit
    if state.save_changes_git_status.is_empty()
        && state.save_changes_focus == SaveChangesFocus::FileList
    {
        state.save_changes_focus = SaveChangesFocus::CommitMessage;
    }

    // Set panel background
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    // Split the area into commit message (top) and file list (bottom)
    // Use responsive layout that ensures status panel is always visible
    let min_status_height = 3; // Status panel minimum
    let min_commit_input_height = 3; // Commit input minimum  
    let min_commit_area_height = min_status_height + min_commit_input_height; // Total minimum for commit area
    let min_file_list_height = 5; // Minimum for file list to be usable

    let commit_area_height = {
        let total_height = area.height;

        // Ensure we always have space for the status panel
        if total_height <= min_commit_area_height + min_file_list_height {
            // Very tight space: prioritize status visibility
            min_commit_area_height
        } else if total_height <= 20 {
            // Very small screens: give commit area what it needs plus a bit more
            std::cmp::max(min_commit_area_height, (total_height * 40) / 100)
        } else if total_height <= 30 {
            // Small screens: give commit area at least 35% but ensure minimums
            std::cmp::max(min_commit_area_height + 2, (total_height * 35) / 100)
        } else if total_height <= 50 {
            // Medium screens: give commit area at least 30% but ensure minimums
            std::cmp::max(min_commit_area_height + 4, (total_height * 30) / 100)
        } else {
            // Large screens: use 25% but with good minimum
            std::cmp::max(min_commit_area_height + 6, (total_height * 25) / 100)
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(commit_area_height), // Responsive commit message area
            Constraint::Min(min_file_list_height),  // File list (minimum for usability)
        ])
        .split(area);

    render_commit_area(f, chunks[0], state);
    render_file_list(f, chunks[1], state);

    // Render help popup if shown
    if state.show_commit_help {
        render_commit_help_popup(f, area, state);
    }

    // Render template popup if shown
    if state.show_template_popup {
        render_template_popup(f, area, state);
    }
}

fn render_file_list(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    if state.save_changes_git_status.is_empty() {
        let clean_paragraph =
            Paragraph::new("✓ No changes to commit\n\nYour working directory is clean.")
                .alignment(Alignment::Center)
                .style(theme.success_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Save Changes")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()),
                );
        f.render_widget(clean_paragraph, area);
        return;
    }

    // Ensure table state selection is valid
    if !state.save_changes_git_status.is_empty() {
        let current_selection = state.save_changes_table_state.selected().unwrap_or(0);
        if current_selection >= state.save_changes_git_status.len() {
            state.save_changes_table_state.select(Some(0));
        } else if state.save_changes_table_state.selected().is_none() {
            state.save_changes_table_state.select(Some(0));
        }
    }

    // Create table headers
    let header = Row::new(vec![
        Cell::from("Staged").style(theme.accent2_style()),
        Cell::from("File Path").style(theme.accent2_style()),
        Cell::from("Status").style(theme.accent2_style()),
        Cell::from("Size").style(theme.accent2_style()),
    ]);

    // Create table rows
    let rows: Vec<Row> = state
        .save_changes_git_status
        .iter()
        .map(|file| {
            let is_staged = file.staged; // Use staging info from git status directly

            let staged_cell = Cell::from(if is_staged { "✔" } else { "○" }).style(if is_staged {
                theme.accent3_style()
            } else {
                Style::default().fg(theme.surface0)
            });

            let path_cell = Cell::from(file.path.display().to_string()).style(if is_staged {
                theme.accent3_style()
            } else {
                Style::default().fg(theme.surface0)
            });

            let status_cell = Cell::from(file.status.as_description()).style(
                Style::default()
                    .fg(file.status.color())
                    .add_modifier(Modifier::BOLD),
            );

            let size_cell =
                Cell::from(format_file_size(file.file_size)).style(theme.secondary_text_style());

            Row::new(vec![staged_cell, path_cell, status_cell, size_cell])
        })
        .collect();

    // Determine border style based on focus
    let border_style = if state.save_changes_focus == SaveChangesFocus::FileList {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    // Count staged files from git status
    let staged_count = state
        .save_changes_git_status
        .iter()
        .filter(|f| f.staged)
        .count();

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
                state.save_changes_git_status.len(),
                staged_count
            ))
            .title_style(theme.title_style())
            .style(theme.secondary_background_style()),
    )
    .row_highlight_style(theme.highlight_style())
    .highlight_symbol("► ");

    f.render_stateful_widget(table, area, &mut state.save_changes_table_state);
}

fn render_commit_area(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    // Ensure status area is always visible with minimum height
    let min_status_height = 3; // Always keep at least 3 lines for status
    let min_commit_input_height = 3; // Minimum for commit message input

    // Calculate how much space we need for both components
    let required_height = min_status_height + min_commit_input_height;

    let (commit_input_height, status_height) = if area.height < required_height {
        // If we don't have enough space, prioritize status visibility
        // Give status its minimum and whatever is left to commit input
        let status_h = min_status_height;
        let commit_h = area.height.saturating_sub(status_h).max(2); // At least 2 lines for input
        (commit_h, status_h)
    } else {
        // We have enough space, use normal sizing
        let status_h = min_status_height;
        let commit_h = area.height - status_h;
        (commit_h, status_h)
    };

    // Split commit area into message input and buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(commit_input_height), // Commit message input
            Constraint::Length(status_height),       // Status area (always visible)
        ])
        .split(area);

    // Render commit message input
    let border_style = if state.save_changes_focus == SaveChangesFocus::CommitMessage {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let commit_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Commit Message - [↑↓] to navigate, [Shift+?] for help, [Shift+T] for template")
        .title_style(theme.title_style())
        .style(theme.secondary_background_style());

    let inner_area = commit_block.inner(chunks[0]);
    f.render_widget(commit_block, chunks[0]);

    // Conditionally render TextArea or Paragraph based on focus
    if state.save_changes_focus == SaveChangesFocus::CommitMessage {
        // When focused, render the interactive TextArea with cursor
        f.render_widget(&state.commit_message, inner_area);
    } else {
        // When not focused, render as static Paragraph (no cursor)
        let commit_text = state.commit_message.lines().join("\n");
        let paragraph = Paragraph::new(commit_text)
            .style(theme.text_style())
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(paragraph, inner_area);
    }

    // Render status/buttons area
    let staged_count = state
        .save_changes_git_status
        .iter()
        .filter(|f| f.staged)
        .count();
    let status_text = if staged_count > 0 {
        format!(
            "Ready to commit {} file(s) - [Enter] to commit",
            staged_count
        )
    } else {
        "No files staged for commit".to_string()
    };

    let status_style = if staged_count > 0 {
        theme.success_style()
    } else {
        theme.warning_style()
    };

    let status_paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .style(status_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        );

    f.render_widget(status_paragraph, chunks[1]);
}

/// Helper function to create a centered popup area
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Render the commit message help popup
fn render_commit_help_popup(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    let popup_area = popup_area(area, 70, 70);

    // Clear the background
    f.render_widget(Clear, popup_area);

    // Split popup into content area and button area
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Help content
            Constraint::Length(3), // OK button
        ])
        .split(popup_area);

    let help_text = vec![
        "Conventional Commits Guide",
        "",
        "Format: <type>[optional scope]: <description>",
        "",
        "Types:",
        "• feat:     A new feature",
        "• fix:      A bug fix",
        "• docs:     Documentation only changes",
        "• style:    Code style changes (formatting, etc.)",
        "• refactor: Code change that neither fixes bug nor adds feature",
        "• test:     Adding missing tests or correcting existing tests",
        "• chore:    Changes to build process or auxiliary tools",
        "",
        "Examples:",
        "• feat: add user authentication",
        "• fix(auth): resolve login validation issue",
        "• docs: update API documentation",
        "",
        "Good commit messages are:",
        "• Clear and concise",
        "• Written in imperative mood (\"add\" not \"added\")",
        "• Start with lowercase after the type",
        "• No period at the end of the description",
    ];

    let total_lines = help_text.len();

    // Main help content with margins and Catppuccin Macchiato styling
    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Commit Message Help - [↑↓] to scroll, [Esc] to close")
        .title_style(theme.popup_title_style())
        .border_style(theme.popup_border_style())
        .style(theme.popup_background_style());

    let help_inner_area = help_block.inner(popup_chunks[0]).inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    f.render_widget(help_block, popup_chunks[0]);

    // Calculate visible area height for scrolling
    let visible_height = help_inner_area.height as usize;

    // Calculate the maximum scroll position properly
    let max_scroll = if total_lines > visible_height {
        total_lines - visible_height
    } else {
        0
    };

    // Ensure scroll position doesn't exceed bounds
    let actual_scroll = state.help_popup_scroll.min(max_scroll);

    // Also update the actual scroll position in state to match the clamped value
    // This prevents the internal scroll position from getting out of sync
    if state.help_popup_scroll != actual_scroll {
        state.help_popup_scroll = actual_scroll;
    }

    // Create the paragraph with scroll offset
    let help_paragraph = Paragraph::new(help_text.join("\n"))
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: false })
        .scroll((actual_scroll as u16, 0));

    f.render_widget(help_paragraph, help_inner_area);

    // Render scrollbar if content is longer than visible area
    if total_lines > visible_height {
        // Use standard scrollbar configuration like the official example
        state.help_popup_scrollbar_state = state
            .help_popup_scrollbar_state
            .content_length(total_lines)
            .viewport_content_length(visible_height)
            .position(actual_scroll);

        // Position scrollbar relative to the popup content area
        let scrollbar_area = Rect {
            x: popup_chunks[0].x + popup_chunks[0].width.saturating_sub(1),
            y: popup_chunks[0].y + 1, // Account for top border
            width: 1,
            height: popup_chunks[0].height.saturating_sub(2), // Account for top/bottom borders
        };

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.overlay1));

        f.render_stateful_widget(
            scrollbar,
            scrollbar_area,
            &mut state.help_popup_scrollbar_state,
        );
    }

    // OK Button centered with limited width
    let button_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),     // Left flex
            Constraint::Length(10), // Button width
            Constraint::Min(1),     // Right flex
        ])
        .split(popup_chunks[1]);

    let button_text = "[ OK ]";
    let button_paragraph = Paragraph::new(button_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.base)
                .bg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent()))
                .style(theme.popup_background_style()),
        );

    f.render_widget(button_paragraph, button_area[1]);
}

/// Render the template selection popup
fn render_template_popup(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = Theme::new();

    let popup_area = popup_area(area, 60, 40);

    // Clear the background
    f.render_widget(Clear, popup_area);

    // Split popup into content area and button area
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    // Main content
    let content_text = "Apply Conventional Commits template to commit message?\n\nThis will replace your current message with a template that follows best practices.";

    let content_block = Block::default()
        .borders(Borders::ALL)
        .title("Apply Template")
        .title_style(theme.popup_title_style())
        .border_style(theme.popup_border_style())
        .style(theme.popup_background_style());

    let content_inner_area = content_block.inner(popup_chunks[0]).inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    f.render_widget(content_block, popup_chunks[0]);

    let content_paragraph = Paragraph::new(content_text)
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);

    f.render_widget(content_paragraph, content_inner_area);

    // Buttons area - center both buttons properly
    let total_button_width = 12 + 10; // Yes (12) + No (10)
    let gap_between = 4; // Space between buttons
    let total_used = total_button_width + gap_between;

    let button_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),     // Left flex
            Constraint::Length(12), // Yes button (wider)
            Constraint::Length(4),  // Gap between buttons
            Constraint::Length(10), // No button
            Constraint::Min(1),     // Right flex
        ])
        .split(popup_chunks[1]);

    // Yes button
    let yes_style = if state.template_popup_selection == TemplatePopupSelection::Yes {
        Style::default()
            .fg(theme.base)
            .bg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.overlay2)
            .add_modifier(Modifier::BOLD)
    };

    let yes_border_style = if state.template_popup_selection == TemplatePopupSelection::Yes {
        Style::default().fg(theme.accent())
    } else {
        Style::default().fg(theme.overlay2)
    };

    let yes_button = Paragraph::new("[ Yes ]")
        .alignment(Alignment::Center)
        .style(yes_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(yes_border_style)
                .style(theme.popup_background_style()),
        );

    f.render_widget(yes_button, button_area[1]);

    // No button
    let no_style = if state.template_popup_selection == TemplatePopupSelection::No {
        Style::default()
            .fg(theme.base)
            .bg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.overlay2)
            .add_modifier(Modifier::BOLD)
    };

    let no_border_style = if state.template_popup_selection == TemplatePopupSelection::No {
        Style::default().fg(theme.accent())
    } else {
        Style::default().fg(theme.overlay2)
    };

    let no_button = Paragraph::new("[ No ]")
        .alignment(Alignment::Center)
        .style(no_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(no_border_style)
                .style(theme.popup_background_style()),
        );

    f.render_widget(no_button, button_area[3]);
}

// Helper functions for handling user input
impl AppState {
    pub fn toggle_file_staging(&mut self) {
        if !self.save_changes_git_status.is_empty() {
            if let Some(selected_idx) = self.save_changes_table_state.selected() {
                if selected_idx < self.save_changes_git_status.len() {
                    let file_path = &self.save_changes_git_status[selected_idx].path;
                    let is_currently_staged = self.save_changes_git_status[selected_idx].staged;

                    if is_currently_staged {
                        // Unstage the file
                        let _ = unstage_file(file_path);
                    } else {
                        // Stage the file
                        let _ = stage_file(file_path);
                    }

                    // Refresh git status cache after staging/unstaging
                    self.refresh_save_changes_git_status();
                }
            }
        }
    }

    pub fn commit_staged_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if there are any staged files from cached git status
        let staged_count = self
            .save_changes_git_status
            .iter()
            .filter(|f| f.staged)
            .count();

        if staged_count == 0 {
            return Err("No files staged for commit".into());
        }

        let commit_message = self.commit_message.lines().join("\n");
        if commit_message.trim().is_empty() {
            return Err("Commit message cannot be empty".into());
        }

        // Perform the commit
        commit(&commit_message)?;

        // Clear commit message
        self.commit_message = tui_textarea::TextArea::new(vec![String::new()]);

        // Refresh git status cache after commit
        self.refresh_save_changes_git_status();

        Ok(())
    }

    pub fn switch_save_changes_focus(&mut self) {
        // Only allow focus switching if there are changes to commit
        if self.save_changes_git_status.is_empty() {
            // No changes to commit, keep focus on commit message
            self.save_changes_focus = SaveChangesFocus::CommitMessage;
            return;
        }

        self.save_changes_focus = match self.save_changes_focus {
            SaveChangesFocus::FileList => SaveChangesFocus::CommitMessage,
            SaveChangesFocus::CommitMessage => SaveChangesFocus::FileList,
        };
    }

    /// Navigate down in save changes tab - move from commit message to file list
    pub fn save_changes_navigate_down(&mut self) {
        match self.save_changes_focus {
            SaveChangesFocus::CommitMessage => {
                // Check if we're at the bottom of the commit message
                let cursor_row = self.commit_message.cursor().0;
                let total_lines = self.commit_message.lines().len();
                if cursor_row >= total_lines.saturating_sub(1) {
                    // At bottom of commit message, only move to file list if there are changes
                    if !self.save_changes_git_status.is_empty() {
                        self.save_changes_focus = SaveChangesFocus::FileList;
                        // Select the first item in the file list
                        self.save_changes_table_state.select(Some(0));
                    }
                    // If no changes, stay in commit message (do nothing)
                } else {
                    // Move down within the commit message
                    self.commit_message
                        .move_cursor(tui_textarea::CursorMove::Down);
                }
            }
            SaveChangesFocus::FileList => {
                if !self.save_changes_git_status.is_empty() {
                    let current = self.save_changes_table_state.selected().unwrap_or(0);
                    if current < self.save_changes_git_status.len() - 1 {
                        // Move down in the file list
                        let next = current + 1;
                        self.save_changes_table_state.select(Some(next));
                    }
                    // If at the last item, stay there (no wrapping to commit message)
                }
            }
        }
    }

    /// Navigate up in save changes tab - move from file list to commit message
    pub fn save_changes_navigate_up(&mut self) {
        match self.save_changes_focus {
            SaveChangesFocus::FileList => {
                if !self.save_changes_git_status.is_empty() {
                    let current = self.save_changes_table_state.selected().unwrap_or(0);
                    if current > 0 {
                        // Move up in the file list
                        let prev = current - 1;
                        self.save_changes_table_state.select(Some(prev));
                    } else {
                        // At first item in file list, move back to commit message
                        self.save_changes_focus = SaveChangesFocus::CommitMessage;
                        // Move cursor to the end of the commit message
                        self.commit_message
                            .move_cursor(tui_textarea::CursorMove::End);
                    }
                } else {
                    // No files, move to commit message
                    self.save_changes_focus = SaveChangesFocus::CommitMessage;
                }
            }
            SaveChangesFocus::CommitMessage => {
                // Move up within the commit message
                self.commit_message
                    .move_cursor(tui_textarea::CursorMove::Up);
            }
        }
    }

    /// Scroll up in the help popup
    pub fn help_popup_scroll_up(&mut self) {
        self.help_popup_scroll = self.help_popup_scroll.saturating_sub(1);
    }

    /// Scroll down in the help popup
    pub fn help_popup_scroll_down(&mut self) {
        self.help_popup_scroll = self.help_popup_scroll.saturating_add(1);
    }

    /// Reset help popup scroll position
    pub fn reset_help_popup_scroll(&mut self) {
        self.help_popup_scroll = 0;
    }
}
