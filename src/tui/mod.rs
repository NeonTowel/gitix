mod files;
mod overview;
mod save_changes;
mod settings;
mod status;
pub mod theme;
mod update;

use crate::app::{AppState, SaveChangesFocus};
use crate::git::get_git_status;
use crate::tui::theme::Theme;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    let theme = Theme::new();

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
                
                // Create theme with current settings for live preview
                let theme = Theme::with_accents_and_title(
                    state.current_theme_accent,
                    state.current_theme_accent2,
                    state.current_theme_accent3,
                    state.current_theme_title,
                );
                
                // Set main background
                f.render_widget(
                    Block::default().style(theme.main_background_style()),
                    size
                );
                
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Tab bar
                            Constraint::Min(1),    // Main area
                            Constraint::Length(2), // Key hints (status bar)
                        ]
                        .as_ref(),
                    )
                    .split(size);

                // Tab bar with semantic theme colors
                let tab_titles: Vec<Line> = TAB_TITLES.iter().enumerate().map(|(i, t)| {
                    if !state.git_enabled && i > 1 {
                        Line::styled(*t, theme.disabled_tab_style())
                    } else if active_tab == i {
                        Line::styled(*t, theme.active_tab_style())
                    } else {
                        Line::styled(*t, theme.inactive_tab_style())
                    }
                }).collect();
                let tabs = Tabs::new(tab_titles)
                    .select(active_tab)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("GIT-iX")
                            .title_style(Style::default().fg(theme.maroon))
                            .border_style(theme.border_style())
                            .style(theme.secondary_background_style()) // Mantle background for tab panel
                    )
                    .style(theme.text_style());
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

                // Modal popup for git init prompt with proper semantic styling
                if active_tab == 0 && state.show_init_prompt {
                    let area = centered_rect(60, 7, size);
                    let modal = Paragraph::new("This folder is not a Git repository.\n\nInitialize a new Git repository here? (Y/N)")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(theme.text_style())
                        .block(
                            Block::default()
                                .title("Initialize Git Repository")
                                .title_style(theme.title_style())
                                .borders(Borders::ALL)
                                .border_style(theme.focused_border_style()) // Accent color for focus
                                .style(theme.secondary_background_style()), // Mantle background
                        );
                    f.render_widget(modal, area);
                }

                // Status bar with key hints (crust background per guidelines)
                let hints = match active_tab {
                    1 => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [↑↓] Navigate  [Enter] Open  [q] Quit",
                    2 if state.git_enabled => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [↑↓] Navigate  [q] Quit  (Read Only - Use 'Save Changes' tab to stage/commit)",
                    3 if state.git_enabled && state.show_commit_help => "[Enter] OK  [Esc] Close Help",
                    3 if state.git_enabled && state.show_template_popup => "[←→] Navigate  [Enter] Apply  [Esc] Cancel",
                    3 if state.git_enabled => "[Tab] Next Tab  [↑↓] Navigate  [Space] Stage/Unstage  [Enter] Commit  [Shift+?] Help  [Shift+T] Template  [q] Quit",
                    4 if state.git_enabled => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [Shift+R] Refresh  [P] Pull  [U] Push  [q] Quit",
                    _ => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [q] Quit",
                };
                let hint_paragraph = Paragraph::new(hints)
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(theme.status_bar_style()); // Crust background with text color
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
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 3 && next_tab != 3 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Invalidate status git status cache when leaving status tab
                            if active_tab == 2 && next_tab != 2 {
                                state.invalidate_status_git_status();
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 3 && prev_tab != 3 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Invalidate status git status cache when leaving status tab
                            if active_tab == 2 && prev_tab != 2 {
                                state.invalidate_status_git_status();
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
                            if !state.status_git_status.is_empty() {
                                let current = state.status_table_state.selected().unwrap_or(0);
                                let next = (current + 1).min(state.status_git_status.len() - 1);
                                state.status_table_state.select(Some(next));
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 2 => {
                            // Status tab: move selection up
                            if !state.status_git_status.is_empty() {
                                let current = state.status_table_state.selected().unwrap_or(0);
                                let prev = current.saturating_sub(1);
                                state.status_table_state.select(Some(prev));
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
                            // Save changes tab navigation - only if no popups are shown
                            if !state.show_commit_help && !state.show_template_popup {
                                state.save_changes_navigate_down();
                            } else if state.show_commit_help {
                                // Scroll down in help popup
                                state.help_popup_scroll_down();
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 3 => {
                            // Save changes tab navigation - only if no popups are shown
                            if !state.show_commit_help && !state.show_template_popup {
                                state.save_changes_navigate_up();
                            } else if state.show_commit_help {
                                // Scroll up in help popup
                                state.help_popup_scroll_up();
                            }
                        }
                        (KeyCode::Char(' '), _) if active_tab == 3 => {
                            // Save changes tab: toggle file staging - only if no popups are shown and focus is on file list
                            if !state.show_commit_help && !state.show_template_popup && state.save_changes_focus == SaveChangesFocus::FileList {
                                state.toggle_file_staging();
                            } else if !state.show_commit_help && !state.show_template_popup && state.save_changes_focus == SaveChangesFocus::CommitMessage {
                                // When focus is on commit message, pass space key to the TextArea input handler
                                state.commit_message.input(Event::Key(key_event));
                            }
                        }
                        (KeyCode::Enter, _) if active_tab == 3 && state.show_commit_help => {
                            // Close help popup when Enter is pressed
                            state.show_commit_help = false;
                        }
                        (KeyCode::Esc, _) if active_tab == 3 && state.show_commit_help => {
                            // Close help popup when Escape is pressed
                            state.show_commit_help = false;
                        }
                        (KeyCode::Enter, _) if active_tab == 3 && state.show_template_popup => {
                            // Template popup: apply selection
                            state.apply_template_selection();
                        }
                        (KeyCode::Esc, _) if active_tab == 3 && state.show_template_popup => {
                            // Template popup: close without applying
                            state.show_template_popup = false;
                        }
                        (KeyCode::Left, _) if active_tab == 3 && state.show_template_popup => {
                            // Template popup: navigate to Yes button
                            state.template_popup_navigate_left();
                        }
                        (KeyCode::Right, _) if active_tab == 3 && state.show_template_popup => {
                            // Template popup: navigate to No button
                            state.template_popup_navigate_right();
                        }
                        (KeyCode::Enter, _) if active_tab == 3 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: commit staged files (only works when in file list and no popups)
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
                        (KeyCode::Char('?'), KeyModifiers::SHIFT) if active_tab == 3 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: show help popup
                            state.show_commit_help = true;
                        }
                        (KeyCode::Char('T'), KeyModifiers::SHIFT) if active_tab == 3 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: show template popup
                            state.toggle_template_popup();
                        }
                        // Update tab key bindings
                        (KeyCode::Char('R'), KeyModifiers::SHIFT) if active_tab == 4 && state.git_enabled => {
                            // Update tab: refresh remote and local status
                            // TODO: Implement actual refresh functionality
                            // This would fetch from remote and update local commit count
                            println!("Refreshing repository status...");
                        }
                        (KeyCode::Char('p') | KeyCode::Char('P'), _) if active_tab == 4 && state.git_enabled => {
                            // Update tab: pull changes from remote
                            // TODO: Implement actual pull functionality
                            println!("Pulling changes from remote...");
                        }
                        (KeyCode::Char('u') | KeyCode::Char('U'), _) if active_tab == 4 && state.git_enabled => {
                            // Update tab: push changes to remote
                            // TODO: Implement actual push functionality
                            println!("Pushing changes to remote...");
                        }
                        // Handle commit message input when focused on commit message and no popups are shown
                        _ if active_tab == 3
                            && !state.show_commit_help
                            && !state.show_template_popup
                            && state.save_changes_focus == SaveChangesFocus::CommitMessage =>
                        {
                            // Use TextArea's built-in input handling for full text editing support
                            state.commit_message.input(Event::Key(key_event));
                        }
                        // Settings tab key bindings (tab 5)
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            let mut next_tab = (active_tab + 1) % tab_count;
                            while !state.git_enabled && next_tab > 1 {
                                next_tab = (next_tab + 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 3 && next_tab != 3 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Invalidate status git status cache when leaving status tab
                            if active_tab == 2 && next_tab != 2 {
                                state.invalidate_status_git_status();
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 3 && prev_tab != 3 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Invalidate status git status cache when leaving status tab
                            if active_tab == 2 && prev_tab != 2 {
                                state.invalidate_status_git_status();
                            }
                            active_tab = prev_tab;
                        }
                        (KeyCode::Left, KeyModifiers::CONTROL) if active_tab == 5 && state.git_enabled => {
                            // Settings tab: switch to Author panel
                            state.settings_focus = crate::app::SettingsFocus::Author;
                        }
                        (KeyCode::Right, KeyModifiers::CONTROL) if active_tab == 5 && state.git_enabled => {
                            // Settings tab: switch to Theme panel
                            state.settings_focus = crate::app::SettingsFocus::Theme;
                        }
                        (KeyCode::Left, _) if active_tab == 5 && state.git_enabled => {
                            // Settings tab: cycle theme colors backward (only works in Theme panel)
                            if state.settings_focus == crate::app::SettingsFocus::Theme {
                                use crate::app::ThemeFocus;
                                match state.settings_theme_focus {
                                    ThemeFocus::Accent => {
                                        state.current_theme_accent = cycle_accent_color_backward(state.current_theme_accent);
                                    }
                                    ThemeFocus::Accent2 => {
                                        state.current_theme_accent2 = cycle_accent_color_backward(state.current_theme_accent2);
                                    }
                                    ThemeFocus::Accent3 => {
                                        state.current_theme_accent3 = cycle_accent_color_backward(state.current_theme_accent3);
                                    }
                                    ThemeFocus::Title => {
                                        state.current_theme_title = cycle_title_color_backward(state.current_theme_title);
                                    }
                                }
                            }
                        }
                        (KeyCode::Right, _) if active_tab == 5 && state.git_enabled => {
                            // Settings tab: cycle theme colors forward (only works in Theme panel)
                            if state.settings_focus == crate::app::SettingsFocus::Theme {
                                use crate::app::ThemeFocus;
                                match state.settings_theme_focus {
                                    ThemeFocus::Accent => {
                                        state.current_theme_accent = cycle_accent_color_forward(state.current_theme_accent);
                                    }
                                    ThemeFocus::Accent2 => {
                                        state.current_theme_accent2 = cycle_accent_color_forward(state.current_theme_accent2);
                                    }
                                    ThemeFocus::Accent3 => {
                                        state.current_theme_accent3 = cycle_accent_color_forward(state.current_theme_accent3);
                                    }
                                    ThemeFocus::Title => {
                                        state.current_theme_title = cycle_title_color_forward(state.current_theme_title);
                                    }
                                }
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 5 && state.git_enabled => {
                            match state.settings_focus {
                                crate::app::SettingsFocus::Author => {
                                    state.settings_author_focus = crate::app::AuthorFocus::Name;
                                }
                                crate::app::SettingsFocus::Theme => {
                                    use crate::app::ThemeFocus;
                                    state.settings_theme_focus = match state.settings_theme_focus {
                                        ThemeFocus::Accent2 => ThemeFocus::Accent,
                                        ThemeFocus::Accent3 => ThemeFocus::Accent2,
                                        ThemeFocus::Title => ThemeFocus::Accent3,
                                        ThemeFocus::Accent => ThemeFocus::Title,
                                    };
                                }
                            }
                        }
                        (KeyCode::Down, _) if active_tab == 5 && state.git_enabled => {
                            match state.settings_focus {
                                crate::app::SettingsFocus::Author => {
                                    state.settings_author_focus = crate::app::AuthorFocus::Email;
                                }
                                crate::app::SettingsFocus::Theme => {
                                    use crate::app::ThemeFocus;
                                    state.settings_theme_focus = match state.settings_theme_focus {
                                        ThemeFocus::Accent => ThemeFocus::Accent2,
                                        ThemeFocus::Accent2 => ThemeFocus::Accent3,
                                        ThemeFocus::Accent3 => ThemeFocus::Title,
                                        ThemeFocus::Title => ThemeFocus::Accent,
                                    };
                                }
                            }
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) if active_tab == 5 && state.git_enabled => {
                            // Save settings
                            match state.save_settings() {
                                Ok(()) => {
                                    state.settings_status_message = Some("✓ Settings saved successfully".to_string());
                                }
                                Err(e) => {
                                    state.settings_status_message = Some(format!("✗ Failed to save: {}", e));
                                }
                            }
                        }
                        // Handle author input when in settings tab and author panel
                        _ if active_tab == 5
                            && state.git_enabled
                            && state.settings_focus == crate::app::SettingsFocus::Author =>
                        {
                            match state.settings_author_focus {
                                crate::app::AuthorFocus::Name => {
                                    state.user_name_input.input(Event::Key(key_event));
                                    if state.settings_status_message.is_some() {
                                        state.settings_status_message = None;
                                    }
                                }
                                crate::app::AuthorFocus::Email => {
                                    state.user_email_input.input(Event::Key(key_event));
                                    if state.settings_status_message.is_some() {
                                        state.settings_status_message = None;
                                    }
                                }
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

// Helper functions for cycling theme colors
fn cycle_accent_color_forward(current: crate::tui::theme::AccentColor) -> crate::tui::theme::AccentColor {
    use crate::tui::theme::AccentColor;
    match current {
        AccentColor::Rosewater => AccentColor::Flamingo,
        AccentColor::Flamingo => AccentColor::Pink,
        AccentColor::Pink => AccentColor::Mauve,
        AccentColor::Mauve => AccentColor::Red,
        AccentColor::Red => AccentColor::Maroon,
        AccentColor::Maroon => AccentColor::Peach,
        AccentColor::Peach => AccentColor::Yellow,
        AccentColor::Yellow => AccentColor::Green,
        AccentColor::Green => AccentColor::Teal,
        AccentColor::Teal => AccentColor::Sky,
        AccentColor::Sky => AccentColor::Sapphire,
        AccentColor::Sapphire => AccentColor::Blue,
        AccentColor::Blue => AccentColor::Lavender,
        AccentColor::Lavender => AccentColor::Rosewater,
    }
}

fn cycle_accent_color_backward(current: crate::tui::theme::AccentColor) -> crate::tui::theme::AccentColor {
    use crate::tui::theme::AccentColor;
    match current {
        AccentColor::Rosewater => AccentColor::Lavender,
        AccentColor::Flamingo => AccentColor::Rosewater,
        AccentColor::Pink => AccentColor::Flamingo,
        AccentColor::Mauve => AccentColor::Pink,
        AccentColor::Red => AccentColor::Mauve,
        AccentColor::Maroon => AccentColor::Red,
        AccentColor::Peach => AccentColor::Maroon,
        AccentColor::Yellow => AccentColor::Peach,
        AccentColor::Green => AccentColor::Yellow,
        AccentColor::Teal => AccentColor::Green,
        AccentColor::Sky => AccentColor::Teal,
        AccentColor::Sapphire => AccentColor::Sky,
        AccentColor::Blue => AccentColor::Sapphire,
        AccentColor::Lavender => AccentColor::Blue,
    }
}

fn cycle_title_color_forward(current: crate::tui::theme::TitleColor) -> crate::tui::theme::TitleColor {
    use crate::tui::theme::{TitleColor, AccentColor};
    match current {
        TitleColor::Overlay0 => TitleColor::Overlay1,
        TitleColor::Overlay1 => TitleColor::Overlay2,
        TitleColor::Overlay2 => TitleColor::Text,
        TitleColor::Text => TitleColor::Subtext0,
        TitleColor::Subtext0 => TitleColor::Subtext1,
        TitleColor::Subtext1 => TitleColor::Accent(AccentColor::Rosewater),
        TitleColor::Accent(accent) => {
            let next_accent = cycle_accent_color_forward(accent);
            if next_accent == AccentColor::Rosewater {
                TitleColor::Overlay0 // Wrap around to start
            } else {
                TitleColor::Accent(next_accent)
            }
        }
    }
}

fn cycle_title_color_backward(current: crate::tui::theme::TitleColor) -> crate::tui::theme::TitleColor {
    use crate::tui::theme::{TitleColor, AccentColor};
    match current {
        TitleColor::Overlay0 => TitleColor::Accent(AccentColor::Lavender),
        TitleColor::Overlay1 => TitleColor::Overlay0,
        TitleColor::Overlay2 => TitleColor::Overlay1,
        TitleColor::Text => TitleColor::Overlay2,
        TitleColor::Subtext0 => TitleColor::Text,
        TitleColor::Subtext1 => TitleColor::Subtext0,
        TitleColor::Accent(accent) => {
            if accent == AccentColor::Rosewater {
                TitleColor::Subtext1
            } else {
                TitleColor::Accent(cycle_accent_color_backward(accent))
            }
        }
    }
}
