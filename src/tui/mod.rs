mod files;
mod overview;
mod save_changes;
mod settings;
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

const TAB_TITLES: [&str; 5] = [
    "Overview",
    "Files",
    "Save Changes",
    "Update",
    "Settings",
];

#[derive(Copy, Clone, Debug)]
enum Tab {
    Overview,
    Files,
    SaveChanges,
    Update,
    Settings,
}

impl Tab {
    fn all() -> &'static [Tab] {
        use Tab::*;
        &[Overview, Files, SaveChanges, Update, Settings]
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
                    2 => save_changes::render_save_changes_tab(f, chunks[1], state),
                    3 => update::render_update_tab(f, chunks[1], state),
                    4 => settings::render_settings_tab(f, chunks[1], state),
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

                // Error popup modal
                if state.show_error_popup {
                    let area = centered_rect(70, 10, size);
                    let error_text = format!("{}\n\nPress [Enter] or [Esc] to close", state.error_popup_message);
                    let modal = Paragraph::new(error_text)
                        .alignment(ratatui::layout::Alignment::Left)
                        .wrap(ratatui::widgets::Wrap { trim: true })
                        .style(theme.text_style())
                        .block(
                            Block::default()
                                .title(state.error_popup_title.as_str())
                                .title_style(theme.title_style())
                                .borders(Borders::ALL)
                                .border_style(theme.error_style()) // Red border for errors
                                .style(theme.secondary_background_style()), // Mantle background
                        );
                    f.render_widget(modal, area);
                }

                // Status bar with key hints (crust background per guidelines)
                let hints = if state.is_loading {
                    // Show loading indicator - simplified
                    "⟳ Loading...".to_string()
                } else {
                    match active_tab {
                        1 => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [↑↓] Navigate  [Enter] Open  [q] Quit",
                        2 if state.git_enabled && state.show_commit_help => "[Enter] OK  [Esc] Close Help",
                        2 if state.git_enabled && state.show_template_popup => "[←→] Navigate  [Enter] Apply  [Esc] Cancel",
                        2 if state.git_enabled => "[Tab] Next Tab  [↑↓] Navigate  [Space] Stage/Unstage  [Enter] Commit  [Shift+?] Help  [Shift+T] Template  [q] Quit",
                        3 if state.git_enabled => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [Shift+R] Refresh  [P] Pull  [U] Push  [q] Quit",
                        _ => "[Tab] Next Tab  [Shift+Tab] Previous Tab  [q] Quit",
                    }.to_string()
                };

                // Create status bar - drop branch info when loading to save space
                if state.git_enabled && !state.is_loading {
                    // Build status line with branch info and hints (only when not loading)
                    let mut status_spans = Vec::new();

                    // Get current branch information when not loading
                    let (current_branch, remote_branch) = if let (Ok(current), Ok(remote)) = 
                        (crate::git::get_current_branch(), crate::git::get_current_remote_branch()) {
                        (Some(current), remote)
                    } else {
                        (None, None)
                    };

                    // Add branch information at the beginning
                    if let Some(branch) = current_branch {
                        // Local branch with parentheses
                        status_spans.push(ratatui::text::Span::styled("(", theme.accent_style()));
                        status_spans.push(ratatui::text::Span::styled(branch, theme.accent_style()));
                        status_spans.push(ratatui::text::Span::styled(")", theme.accent_style()));

                        // Add remote branch if available
                        if let Some(remote) = remote_branch {
                            // Remove redundant "origin/" prefix if present
                            let clean_remote = if remote.starts_with("origin/origin/") {
                                remote.strip_prefix("origin/").unwrap_or(&remote).to_string()
                            } else {
                                remote
                            };
                            
                            status_spans.push(ratatui::text::Span::raw(" "));
                            status_spans.push(ratatui::text::Span::styled("(", theme.accent3_style()));
                            status_spans.push(ratatui::text::Span::styled(clean_remote, theme.accent3_style()));
                            status_spans.push(ratatui::text::Span::styled(")", theme.accent3_style()));
                        }

                        status_spans.push(ratatui::text::Span::raw("  |  "));
                    }

                    // Add the hints
                    status_spans.push(ratatui::text::Span::styled(hints, theme.status_bar_style()));

                    let status_line = ratatui::text::Line::from(status_spans);
                    let hint_paragraph = Paragraph::new(status_line)
                        .alignment(ratatui::layout::Alignment::Center);
                    f.render_widget(hint_paragraph, chunks[2]);
                } else {
                    // No git or loading - just show hints (simplified when loading)
                    let hint_paragraph = Paragraph::new(hints)
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(if state.is_loading { 
                            theme.info_style() 
                        } else { 
                            theme.status_bar_style() 
                        });
                    f.render_widget(hint_paragraph, chunks[2]);
                }
            })
            .unwrap();

        // Perform any pending refresh work immediately after UI is drawn
        // This ensures the loading indicator is visible before the blocking operation
        if state.pending_refresh_work {
            state.perform_refresh_work();
        }

        // Handle input
        let poll_timeout = if state.is_loading { 
            std::time::Duration::from_millis(100) // Reasonable timeout for spinner animation
        } else { 
            std::time::Duration::from_millis(100) // Normal timeout
        };
        
        if event::poll(poll_timeout).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if key_event.kind == KeyEventKind::Press {
                    // If showing error popup, only handle Enter/Esc to close it
                    if state.show_error_popup {
                        match key_event.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                state.hide_error();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // If showing prompt, only handle Y/N
                    if active_tab == 0 && state.show_init_prompt {
                        match key_event.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                if let Err(e) = state.try_init_repo() {
                                    // Show user-friendly error popup
                                    state.show_error(
                                        "Repository Initialization Failed",
                                        &format!("Failed to initialize Git repository:\n\n{}", e)
                                    );
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
                            if active_tab == 2 && next_tab != 2 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Load update tab data when entering update tab
                            if next_tab == 3 && active_tab != 3 {
                                state.load_update_tab();
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 2 && prev_tab != 2 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Load update tab data when entering update tab
                            if prev_tab == 3 && active_tab != 3 {
                                state.load_update_tab();
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
                        (KeyCode::Down, _) if active_tab == 2 => {
                            // Save changes tab navigation - only if no popups are shown
                            if !state.show_commit_help && !state.show_template_popup {
                                state.save_changes_navigate_down();
                            } else if state.show_commit_help {
                                // Scroll down in help popup
                                state.help_popup_scroll_down();
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 2 => {
                            // Save changes tab navigation - only if no popups are shown
                            if !state.show_commit_help && !state.show_template_popup {
                                state.save_changes_navigate_up();
                            } else if state.show_commit_help {
                                // Scroll up in help popup
                                state.help_popup_scroll_up();
                            }
                        }
                        (KeyCode::Char(' '), _) if active_tab == 2 => {
                            // Save changes tab: toggle file staging - only if no popups are shown and focus is on file list
                            if !state.show_commit_help && !state.show_template_popup && state.save_changes_focus == SaveChangesFocus::FileList {
                                state.toggle_file_staging();
                            } else if !state.show_commit_help && !state.show_template_popup && state.save_changes_focus == SaveChangesFocus::CommitMessage {
                                // When focus is on commit message, pass space key to the TextArea input handler
                                state.commit_message.input(Event::Key(key_event));
                            }
                        }
                        (KeyCode::Enter, _) if active_tab == 2 && state.show_commit_help => {
                            // Close help popup when Enter is pressed
                            state.show_commit_help = false;
                        }
                        (KeyCode::Esc, _) if active_tab == 2 && state.show_commit_help => {
                            // Close help popup when Escape is pressed
                            state.show_commit_help = false;
                        }
                        (KeyCode::Enter, _) if active_tab == 2 && state.show_template_popup => {
                            // Template popup: apply selection
                            state.apply_template_selection();
                        }
                        (KeyCode::Esc, _) if active_tab == 2 && state.show_template_popup => {
                            // Template popup: close without applying
                            state.show_template_popup = false;
                        }
                        (KeyCode::Left, _) if active_tab == 2 && state.show_template_popup => {
                            // Template popup: navigate to Yes button
                            state.template_popup_navigate_left();
                        }
                        (KeyCode::Right, _) if active_tab == 2 && state.show_template_popup => {
                            // Template popup: navigate to No button
                            state.template_popup_navigate_right();
                        }
                        (KeyCode::Enter, _) if active_tab == 2 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: commit staged files (only works when in file list and no popups)
                            if state.save_changes_focus == SaveChangesFocus::FileList {
                                if let Err(e) = state.commit_staged_files() {
                                    // Show user-friendly error popup
                                    state.show_error("Commit Failed", &format!("Failed to commit changes:\n\n{}", e));
                                }
                            } else {
                                // In commit message area, add a new line
                                state.commit_message.insert_newline();
                            }
                        }
                        (KeyCode::Char('?'), KeyModifiers::SHIFT) if active_tab == 2 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: show help popup
                            state.show_commit_help = true;
                        }
                        (KeyCode::Char('T'), KeyModifiers::SHIFT) if active_tab == 2 && !state.show_commit_help && !state.show_template_popup => {
                            // Save changes tab: show template popup
                            state.toggle_template_popup();
                        }
                        // Handle commit message input when focused on commit message and no popups are shown
                        _ if active_tab == 2
                            && !state.show_commit_help
                            && !state.show_template_popup
                            && state.save_changes_focus == SaveChangesFocus::CommitMessage =>
                        {
                            // Use TextArea's built-in input handling for full text editing support
                            state.commit_message.input(Event::Key(key_event));
                        }
                        // Settings tab key bindings (tab 4)
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            let mut next_tab = (active_tab + 1) % tab_count;
                            while !state.git_enabled && next_tab > 1 {
                                next_tab = (next_tab + 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 2 && next_tab != 2 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Load update tab data when entering update tab
                            if next_tab == 3 && active_tab != 3 {
                                state.load_update_tab();
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            // Invalidate save changes git status cache when leaving save changes tab
                            if active_tab == 2 && prev_tab != 2 {
                                state.invalidate_save_changes_git_status();
                            }
                            // Load update tab data when entering update tab
                            if prev_tab == 3 && active_tab != 3 {
                                state.load_update_tab();
                            }
                            active_tab = prev_tab;
                        }
                        (KeyCode::Left, KeyModifiers::CONTROL) if active_tab == 4 && state.git_enabled => {
                            // Settings tab: cycle panels backward
                            state.settings_focus = match state.settings_focus {
                                crate::app::SettingsFocus::Author => crate::app::SettingsFocus::Git,
                                crate::app::SettingsFocus::Theme => crate::app::SettingsFocus::Author,
                                crate::app::SettingsFocus::Git => crate::app::SettingsFocus::Theme,
                            };
                        }
                        (KeyCode::Right, KeyModifiers::CONTROL) if active_tab == 4 && state.git_enabled => {
                            // Settings tab: cycle panels forward
                            state.settings_focus = match state.settings_focus {
                                crate::app::SettingsFocus::Author => crate::app::SettingsFocus::Theme,
                                crate::app::SettingsFocus::Theme => crate::app::SettingsFocus::Git,
                                crate::app::SettingsFocus::Git => crate::app::SettingsFocus::Author,
                            };
                        }
                        (KeyCode::Left, _) if active_tab == 4 && state.git_enabled => {
                            // Settings tab: cycle theme colors backward (only works in Theme panel) or toggle Git settings
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
                            } else if state.settings_focus == crate::app::SettingsFocus::Git {
                                // Toggle pull rebase setting
                                state.pull_rebase = !state.pull_rebase;
                                // Clear status message when changing settings
                                if state.settings_status_message.is_some() {
                                    state.settings_status_message = None;
                                }
                            }
                        }
                        (KeyCode::Right, _) if active_tab == 4 && state.git_enabled => {
                            // Settings tab: cycle theme colors forward (only works in Theme panel) or toggle Git settings
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
                            } else if state.settings_focus == crate::app::SettingsFocus::Git {
                                // Toggle pull rebase setting
                                state.pull_rebase = !state.pull_rebase;
                                // Clear status message when changing settings
                                if state.settings_status_message.is_some() {
                                    state.settings_status_message = None;
                                }
                            }
                        }
                        (KeyCode::Up, _) if active_tab == 4 && state.git_enabled => {
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
                                crate::app::SettingsFocus::Git => {
                                    // Only one Git setting for now, so no navigation needed
                                }
                            }
                        }
                        (KeyCode::Down, _) if active_tab == 4 && state.git_enabled => {
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
                                crate::app::SettingsFocus::Git => {
                                    // Only one Git setting for now, so no navigation needed
                                }
                            }
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) if active_tab == 4 && state.git_enabled => {
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
                        _ if active_tab == 4
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
                        // Update tab operations
                        (KeyCode::Char('p'), KeyModifiers::NONE) if active_tab == 3 && state.git_enabled => {
                            // Pull operation
                            state.perform_pull();
                        }
                        (KeyCode::Char('P'), KeyModifiers::NONE) if active_tab == 3 && state.git_enabled => {
                            // Pull operation (uppercase)
                            state.perform_pull();
                        }
                        (KeyCode::Char('u'), KeyModifiers::NONE) if active_tab == 3 && state.git_enabled => {
                            // Push operation
                            state.perform_push();
                        }
                        (KeyCode::Char('U'), KeyModifiers::NONE) if active_tab == 3 && state.git_enabled => {
                            // Push operation (uppercase)
                            state.perform_push();
                        }
                        (KeyCode::Char('r'), KeyModifiers::SHIFT) if active_tab == 3 && state.git_enabled => {
                            // Refresh remote status
                            state.refresh_update_remote_status();
                        }
                        (KeyCode::Char('R'), KeyModifiers::SHIFT) if active_tab == 3 && state.git_enabled => {
                            // Refresh remote status (uppercase)
                            state.refresh_update_remote_status();
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
