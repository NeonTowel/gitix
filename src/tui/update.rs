use crate::app::AppState;
use crate::tui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{layout::Rect, Frame};

// Mock data structures for UI design
#[derive(Debug, Clone)]
struct RemoteStatus {
    name: String,
    url: String,
    ahead: usize,
    behind: usize,
    last_fetch: Option<String>,
}

#[derive(Debug, Clone)]
struct SyncOperation {
    operation_type: SyncOperationType,
    status: OperationStatus,
    message: String,
}

#[derive(Debug, Clone)]
enum SyncOperationType {
    Fetch,
    Pull,
    Push,
    Refresh,
}

#[derive(Debug, Clone)]
enum OperationStatus {
    Pending,
    InProgress,
    Success,
    Error,
}

impl SyncOperationType {
    fn as_str(&self) -> &'static str {
        match self {
            SyncOperationType::Fetch => "Fetch",
            SyncOperationType::Pull => "Download",
            SyncOperationType::Push => "Upload",
            SyncOperationType::Refresh => "Refresh",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            SyncOperationType::Fetch => "Check for remote changes",
            SyncOperationType::Pull => "Download and merge remote changes",
            SyncOperationType::Push => "Upload local changes to remote",
            SyncOperationType::Refresh => "Update remote and local status",
        }
    }
}

impl OperationStatus {
    fn symbol(&self) -> &'static str {
        match self {
            OperationStatus::Pending => "◦",
            OperationStatus::InProgress => "→",
            OperationStatus::Success => "✓",
            OperationStatus::Error => "✗",
        }
    }

    fn style(&self, theme: &Theme) -> Style {
        match self {
            OperationStatus::Pending => theme.muted_text_style(),
            OperationStatus::InProgress => theme.info_style(),
            OperationStatus::Success => theme.success_style(),
            OperationStatus::Error => theme.error_style(),
        }
    }
}

pub fn render_update_tab(f: &mut Frame, area: Rect, state: &AppState) {
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

    // Check if git is enabled
    if !state.git_enabled {
        render_no_git_message(f, area, &theme);
        return;
    }

    // Check for remote origin using real git operations
    let has_remote = crate::git::has_remote_origin().unwrap_or(false);

    if !has_remote {
        render_no_remote_message(f, area, &theme);
        return;
    }

    // Main sync interface
    render_sync_interface(f, area, state, &theme);
}

fn render_no_git_message(f: &mut Frame, area: Rect, theme: &Theme) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠ Not a Git Repository",
            Style::default()
                .fg(theme.yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This directory is not a Git repository."),
        Line::from("Initialize a repository first to sync with remotes."),
        Line::from(""),
        Line::from(Span::styled(
            "• Tip:",
            Style::default().fg(theme.sky).add_modifier(Modifier::BOLD),
        )),
        Line::from("Use the Overview tab to initialize a new repository."),
    ])
    .alignment(Alignment::Center)
    .style(theme.text_style())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Repository Sync")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    );
    f.render_widget(message, area);
}

fn render_no_remote_message(f: &mut Frame, area: Rect, theme: &Theme) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠ No Remote Repository",
            Style::default()
                .fg(theme.yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This repository doesn't have a remote origin configured."),
        Line::from("Add a remote repository to sync your changes."),
        Line::from(""),
        Line::from(Span::styled(
            "• How to add a remote:",
            Style::default().fg(theme.sky).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "git remote add origin <repository-url>",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from("Examples:"),
        Line::from(Span::styled(
            "  ◦ GitHub: git remote add origin https://github.com/user/repo.git",
            theme.muted_text_style(),
        )),
        Line::from(Span::styled(
            "  ◦ GitLab: git remote add origin https://gitlab.com/user/repo.git",
            theme.muted_text_style(),
        )),
        Line::from(Span::styled(
            "  ◦ SSH: git remote add origin git@github.com:user/repo.git",
            theme.muted_text_style(),
        )),
    ])
    .alignment(Alignment::Center)
    .style(theme.text_style())
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Repository Sync")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    );
    f.render_widget(message, area);
}

fn render_sync_interface(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Split into three sections: remote status, sync actions, and recent activity
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Remote status
            Constraint::Length(12), // Sync actions
            Constraint::Min(5),     // Recent activity
        ])
        .split(area);

    render_remote_status(f, chunks[0], state, theme);
    render_sync_actions(f, chunks[1], state, theme);
    render_recent_activity(f, chunks[2], state, theme);
}

fn render_remote_status(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Get remote status from app state or fetch it
    let remote_status = match &state.update_remote_status {
        Some(status) => status.clone(),
        None => {
            // Fallback to fetching status if not cached
            match crate::git::get_remote_status() {
                Ok(status) => status,
                Err(_) => {
                    // Show error message if we can't get remote status
                    let error_text = vec![
                        Line::from(vec![
                            Span::styled("Error: ", theme.error_style()),
                            Span::styled("Unable to fetch remote status", theme.text_style()),
                        ]),
                        Line::from(""),
                        Line::from("Press [Shift+R] to refresh"),
                    ];

                    let error_block = Paragraph::new(error_text).style(theme.text_style()).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Remote Repository Status - [Shift+R] Refresh")
                            .title_style(theme.title_style())
                            .border_style(theme.border_style())
                            .style(theme.secondary_background_style()),
                    );

                    f.render_widget(error_block, area);
                    return;
                }
            }
        }
    };

    let url_text = format!("({})", remote_status.url);
    let ahead_behind_text = if remote_status.ahead > 0 && remote_status.behind > 0 {
        format!(
            "{} ahead, {} behind",
            remote_status.ahead, remote_status.behind
        )
    } else if remote_status.ahead > 0 {
        format!("{} ahead", remote_status.ahead)
    } else if remote_status.behind > 0 {
        format!("{} behind", remote_status.behind)
    } else {
        "Up to date".to_string()
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("Remote: ", theme.accent2_style()),
            Span::styled(&remote_status.name, theme.text_style()),
            Span::raw(" "),
            Span::styled(&url_text, theme.muted_text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", theme.accent2_style()),
            if remote_status.ahead > 0 && remote_status.behind > 0 {
                Span::styled(&ahead_behind_text, theme.warning_style())
            } else if remote_status.ahead > 0 {
                Span::styled(&ahead_behind_text, theme.info_style())
            } else if remote_status.behind > 0 {
                Span::styled(&ahead_behind_text, theme.warning_style())
            } else {
                Span::styled(&ahead_behind_text, theme.success_style())
            },
        ]),
        Line::from(vec![
            Span::styled("Last updated: ", theme.accent2_style()),
            Span::styled(
                remote_status.last_fetch.as_deref().unwrap_or("Never"),
                theme.accent3_style(),
            ),
        ]),
    ];

    let status_block = Paragraph::new(status_text).style(theme.text_style()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Remote Repository Status - [Shift+R] Refresh")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    );

    f.render_widget(status_block, area);
}

fn render_sync_actions(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Split into two columns for actions
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_download_section(f, chunks[0], state, theme);
    render_upload_section(f, chunks[1], state, theme);
}

fn render_download_section(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let remote_status = match &state.update_remote_status {
        Some(status) => status,
        None => {
            // Show loading or error state
            let loading_text = vec![
                Line::from(vec![Span::styled(
                    "↓ Download Changes",
                    Style::default()
                        .fg(theme.green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from("Loading status..."),
                Line::from(""),
                Line::from("Press [Shift+R] to refresh"),
            ];

            let loading_block = Paragraph::new(loading_text)
                .style(theme.text_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Download from Remote")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()),
                );

            f.render_widget(loading_block, area);
            return;
        }
    };

    let available_text = if remote_status.behind > 0 {
        format!("{} new changes", remote_status.behind)
    } else {
        "No new changes".to_string()
    };

    let pull_mode = if state.pull_rebase { "rebase" } else { "merge" };

    let download_text = vec![
        Line::from(vec![Span::styled(
            "↓ Download Changes",
            Style::default()
                .fg(theme.green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        if remote_status.behind > 0 {
            Line::from(vec![
                Span::styled("Available: ", theme.accent2_style()),
                Span::styled(&available_text, theme.info_style()),
            ])
        } else {
            Line::from(vec![
                Span::styled("Status: ", theme.accent2_style()),
                Span::styled(&available_text, theme.success_style()),
            ])
        },
        Line::from(vec![
            Span::styled("Mode: ", theme.accent2_style()),
            Span::styled(pull_mode, theme.accent3_style()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Actions:", theme.accent2_style())]),
        if remote_status.behind > 0 {
            Line::from(vec![
                Span::raw("  ◦ "),
                Span::styled("[P] Pull", theme.accent_style()),
                Span::raw(" - Download changes"),
            ])
        } else {
            Line::from(vec![
                Span::raw("  ◦ "),
                Span::styled("[P] Pull", theme.muted_text_style()),
                Span::raw(" - Nothing to download"),
            ])
        },
    ];

    let download_block = Paragraph::new(download_text)
        .style(theme.text_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Download from Remote")
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        );

    f.render_widget(download_block, area);
}

fn render_upload_section(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let remote_status = match &state.update_remote_status {
        Some(status) => status,
        None => {
            // Show loading or error state
            let loading_text = vec![
                Line::from(vec![Span::styled(
                    "↑ Upload Changes",
                    Style::default().fg(theme.blue).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from("Loading status..."),
                Line::from(""),
                Line::from("Press [Shift+R] to refresh"),
            ];

            let loading_block = Paragraph::new(loading_text)
                .style(theme.text_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Upload to Remote")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()),
                );

            f.render_widget(loading_block, area);
            return;
        }
    };

    let ready_text = if remote_status.ahead > 0 {
        format!("{} local changes", remote_status.ahead)
    } else {
        "Nothing to upload".to_string()
    };

    let upload_text = vec![
        Line::from(vec![Span::styled(
            "↑ Upload Changes",
            Style::default().fg(theme.blue).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        if remote_status.ahead > 0 {
            Line::from(vec![
                Span::styled("Ready: ", theme.accent2_style()),
                Span::styled(&ready_text, theme.info_style()),
            ])
        } else {
            Line::from(vec![
                Span::styled("Status: ", theme.accent2_style()),
                Span::styled(&ready_text, theme.success_style()),
            ])
        },
        Line::from(""),
        Line::from(vec![Span::styled("Actions:", theme.accent2_style())]),
        if remote_status.ahead > 0 {
            Line::from(vec![
                Span::raw("  ◦ "),
                Span::styled("[U] Push", theme.accent_style()),
                Span::raw(" - Upload changes"),
            ])
        } else {
            Line::from(vec![
                Span::raw("  ◦ "),
                Span::styled("[U] Push", theme.muted_text_style()),
                Span::raw(" - Nothing to upload"),
            ])
        },
    ];

    let upload_block = Paragraph::new(upload_text).style(theme.text_style()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Upload to Remote")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    );

    f.render_widget(upload_block, area);
}

fn render_recent_activity(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let operations = &state.update_recent_operations;

    let activity_items: Vec<ListItem> = if operations.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "No recent activity",
            theme.muted_text_style(),
        )]))]
    } else {
        operations
            .iter()
            .map(|op| {
                let status_style = match op.status {
                    crate::git::OperationStatus::Pending => theme.muted_text_style(),
                    crate::git::OperationStatus::InProgress => theme.info_style(),
                    crate::git::OperationStatus::Success => theme.success_style(),
                    crate::git::OperationStatus::Error => theme.error_style(),
                };

                let status_symbol = match op.status {
                    crate::git::OperationStatus::Pending => "◦",
                    crate::git::OperationStatus::InProgress => "→",
                    crate::git::OperationStatus::Success => "✓",
                    crate::git::OperationStatus::Error => "✗",
                };

                let operation_name = match op.operation_type {
                    crate::git::SyncOperationType::Fetch => "Fetch",
                    crate::git::SyncOperationType::Pull => "Download",
                    crate::git::SyncOperationType::Push => "Upload",
                    crate::git::SyncOperationType::Refresh => "Refresh",
                };

                // Format the timestamp as relative time
                let relative_time = crate::git::format_system_time_relative(op.timestamp);

                ListItem::new(Line::from(vec![
                    Span::styled(status_symbol, status_style),
                    Span::raw(" "),
                    Span::styled(operation_name, theme.accent2_style()),
                    Span::raw(" - "),
                    Span::styled(&op.message, theme.text_style()),
                    Span::raw(" "),
                    Span::styled(format!("({})", relative_time), theme.muted_text_style()),
                ]))
            })
            .collect()
    };

    let activity_list = List::new(activity_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Sync Activity")
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        )
        .style(theme.text_style());

    f.render_widget(activity_list, area);
}
