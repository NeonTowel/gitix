use crate::app::AppState;
use crate::tui::theme::Theme;
use chrono::{Datelike, NaiveDate, Utc};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::{Frame, layout::Rect};
use time::{Date, Month};

// Helper struct for commit information
#[derive(Debug, Clone)]
struct CommitInfo {
    message: String,
    author: String,
    timestamp: i64,
}

// Helper function to get recent commits from repository
fn get_recent_commits(repo_root: &std::path::Path, limit: usize) -> Vec<CommitInfo> {
    let mut commits = Vec::new();

    if let Ok(repo) = gix::open(repo_root) {
        if let Ok(head) = repo.head_ref() {
            if let Some(head) = head {
                if let Some(oid) = head.target().try_id() {
                    if let Ok(obj) = repo.find_object(oid) {
                        if let Ok(commit) = obj.try_into_commit() {
                            if let Ok(walk) = commit.ancestors().all() {
                                for info in walk.filter_map(Result::ok).take(limit) {
                                    let oid = info.id();
                                    if let Ok(obj) = repo.find_object(oid) {
                                        if let Ok(commit_obj) = obj.try_into_commit() {
                                            if let (Ok(message), Ok(author), Ok(time)) = (
                                                commit_obj.message(),
                                                commit_obj.author(),
                                                commit_obj.time(),
                                            ) {
                                                let message_str = message.title.to_string();
                                                let author_str = format!("{}", author.name);

                                                commits.push(CommitInfo {
                                                    message: message_str,
                                                    author: author_str,
                                                    timestamp: time.seconds,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    commits
}

// Helper function to format relative time
fn format_relative_time(timestamp: i64) -> String {
    let now = Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let minutes = diff / 60;
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if diff < 86400 {
        let hours = diff / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else {
        // For commits older than a day, show the date
        if let Some(naive) = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0) {
            naive.format("%Y-%m-%d").to_string()
        } else {
            "unknown date".to_string()
        }
    }
}

pub fn render_overview_tab(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = Theme::new();

    // Set panel background (mantle per guidelines)
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    // Define minimum heights for each component
    const STATS_HEIGHT: u16 = 10;
    const CALENDAR_HEIGHT: u16 = 7;
    const SPARKLINE_HEIGHT: u16 = 6;
    const LABEL_HEIGHT: u16 = 1;

    // Calculate minimum total height needed for all components
    let min_height_all = STATS_HEIGHT + CALENDAR_HEIGHT + SPARKLINE_HEIGHT + LABEL_HEIGHT;
    let min_height_with_sparkline = STATS_HEIGHT + SPARKLINE_HEIGHT + LABEL_HEIGHT;
    let min_height_stats_only = STATS_HEIGHT + LABEL_HEIGHT;

    // Determine what components to show based on available height
    let show_calendar = area.height >= min_height_all;
    let show_sparkline = area.height >= min_height_with_sparkline;
    let show_stats = area.height >= min_height_stats_only;

    // Build constraints based on what we can show
    let mut constraints = Vec::new();

    if show_stats {
        constraints.push(Constraint::Length(STATS_HEIGHT));
    }

    if show_calendar {
        constraints.push(Constraint::Length(CALENDAR_HEIGHT));
    }

    if show_sparkline {
        constraints.push(Constraint::Length(SPARKLINE_HEIGHT));
    }

    if show_stats || show_calendar || show_sparkline {
        constraints.push(Constraint::Length(LABEL_HEIGHT));
        constraints.push(Constraint::Min(0)); // filler
    }

    // If we can't show anything meaningful, just show a message
    if !show_stats {
        let too_small_paragraph = Paragraph::new("Terminal too small - resize to view overview")
            .alignment(Alignment::Center)
            .style(theme.text_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Overview")
                    .title_style(theme.title_style())
                    .border_style(theme.border_style())
                    .style(theme.secondary_background_style()), // Mantle background
            );
        f.render_widget(too_small_paragraph, area);
        return;
    }

    let overview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // --- Repo stats logic ---
    let (num_commits, num_branches, latest_author, commit_dates): (
        Option<u64>,
        Option<u64>,
        Option<String>,
        Vec<NaiveDate>,
    ) = if state.git_enabled {
        if let Some(repo_root) = &state.repo_root {
            match gix::open(repo_root) {
                Ok(repo) => {
                    // Commit count
                    let num_commits = repo.head_ref().ok().and_then(|opt_head| {
                        opt_head.and_then(|head| {
                            let target = head.target();
                            let oid = target.try_id()?;
                            let commit = repo.find_object(oid).ok()?.try_into_commit().ok()?;
                            let walk = commit.ancestors().all().ok()?;
                            Some(walk.count() as u64)
                        })
                    });
                    // Branch count
                    let num_branches = repo.references().ok().and_then(|refs| {
                        refs.all().ok().map(|iter| {
                            iter.filter_map(Result::ok)
                                .filter(|r| r.name().as_bstr().starts_with(b"refs/heads/"))
                                .count() as u64
                        })
                    });
                    // Latest author
                    let latest_author = repo.head_ref().ok().and_then(|opt_head| {
                        opt_head.and_then(|head| {
                            let target = head.target();
                            let oid = target.try_id()?;
                            let commit = repo.find_object(oid).ok()?.try_into_commit().ok()?;
                            let sig = commit.author().ok()?;
                            let name = sig.name.to_string();
                            let email = sig.email.to_string();
                            Some(format!("{} <{}>", name, email))
                        })
                    });
                    // Gather commit dates for calendar
                    let mut commit_dates: Vec<NaiveDate> = Vec::new();
                    if let Ok(head) = repo.head_ref() {
                        if let Some(head) = head {
                            if let Some(oid) = head.target().try_id() {
                                if let Ok(obj) = repo.find_object(oid) {
                                    if let Ok(commit) = obj.try_into_commit() {
                                        if let Ok(walk) = commit.ancestors().all() {
                                            for info in walk.filter_map(Result::ok) {
                                                // Get the commit id from Info
                                                let oid = info.id();
                                                if let Ok(obj) = repo.find_object(oid) {
                                                    if let Ok(commit_obj) = obj.try_into_commit() {
                                                        if let Ok(time) = commit_obj.time() {
                                                            let timestamp = time.seconds;
                                                            let naive =
                                                                chrono::NaiveDateTime::from_timestamp_opt(
                                                                    timestamp, 0,
                                                                );
                                                            if let Some(naive) = naive {
                                                                let date = NaiveDate::from_ymd_opt(
                                                                    naive.year(),
                                                                    naive.month(),
                                                                    naive.day(),
                                                                );
                                                                if let Some(date) = date {
                                                                    commit_dates.push(date);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    (num_commits, num_branches, latest_author, commit_dates)
                }
                Err(_) => (None, None, None, Vec::new()),
            }
        } else {
            (None, None, None, Vec::new())
        }
    } else {
        (None, None, None, Vec::new())
    };

    // Stats row (always shown if we have minimum height)
    if show_stats {
        // Split the stats area into two parts: stats line and commit history
        let stats_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Stats line (with borders)
                Constraint::Min(0),    // Commit history (remaining space)
            ])
            .split(overview_chunks[chunk_idx]);

        // Repository stats line with highlighted labels and values
        let mut stats_spans = Vec::new();

        if let Some(n) = num_commits {
            stats_spans.push(Span::styled("Commits: ", theme.stats_label_style()));
            stats_spans.push(Span::styled(n.to_string(), theme.text_style()));
        }

        if let Some(n) = num_branches {
            if !stats_spans.is_empty() {
                stats_spans.push(Span::styled("    |    ", theme.secondary_text_style()));
            }
            stats_spans.push(Span::styled("Branches: ", theme.stats_label_style()));
            stats_spans.push(Span::styled(n.to_string(), theme.text_style()));
        }

        if let Some(ref author) = latest_author {
            if !stats_spans.is_empty() {
                stats_spans.push(Span::styled("    |    ", theme.secondary_text_style()));
            }
            stats_spans.push(Span::styled("Latest Author: ", theme.stats_label_style()));
            stats_spans.push(Span::styled(author.clone(), theme.text_style()));
        }

        let stats_line = if stats_spans.is_empty() {
            Line::from(Span::styled(
                "No repository stats available",
                theme.muted_text_style(),
            ))
        } else {
            Line::from(stats_spans)
        };

        let stats_paragraph = Paragraph::new(stats_line)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Repository Stats")
                    .title_style(theme.title_style())
                    .border_style(theme.border_style())
                    .style(theme.secondary_background_style()), // Mantle background
            );
        f.render_widget(stats_paragraph, stats_chunks[0]);

        // Get real commit history data
        let recent_commits = if state.git_enabled {
            if let Some(repo_root) = &state.repo_root {
                get_recent_commits(repo_root, 5)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Build commit history with colored spans
        let mut commit_lines = Vec::new();

        if recent_commits.is_empty() {
            commit_lines.push(Line::from(Span::styled(
                "No recent commits found",
                theme.muted_text_style(),
            )));
        } else {
            for commit in &recent_commits {
                let relative_time = format_relative_time(commit.timestamp);

                let line = Line::from(vec![
                    Span::raw("• "),
                    Span::styled(&commit.message, theme.commit_message_style()),
                    Span::styled(" - ", theme.secondary_text_style()),
                    Span::styled(&commit.author, theme.author_style()),
                    Span::styled(" (", theme.secondary_text_style()),
                    Span::styled(relative_time, theme.timestamp_style()),
                    Span::styled(")", theme.secondary_text_style()),
                ]);
                commit_lines.push(line);
            }
        }

        let commit_paragraph = Paragraph::new(commit_lines)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Changes")
                    .title_style(theme.title_style())
                    .border_style(theme.border_style())
                    .style(theme.secondary_background_style()), // Mantle background
            );
        f.render_widget(commit_paragraph, stats_chunks[1]);

        chunk_idx += 1;
    }

    // --- Calendar for last 3 months (only if we have enough space) ---
    if show_calendar {
        if state.git_enabled && !commit_dates.is_empty() {
            // Build event store for last 3 months
            let today = Utc::now().date_naive();
            let three_months_ago = today - chrono::Duration::days(90);
            let mut event_store = CalendarEventStore::default();

            // Count commits per day to determine activity level
            let mut commits_per_day = std::collections::HashMap::new();
            for date in &commit_dates {
                if *date >= three_months_ago {
                    *commits_per_day.entry(*date).or_insert(0) += 1;
                }
            }

            // Style for non-commit days (surface color)
            let default_style = Style::default().fg(theme.surface1).bg(theme.mantle); // Mantle background

            // Add commit days to event store with different styles based on activity
            for (date, commit_count) in commits_per_day {
                let month = Month::try_from(date.month() as u8).ok();
                let day = u8::try_from(date.day()).ok();
                if let (Some(month), Some(day)) = (month, day) {
                    if let Ok(time_date) = Date::from_calendar_date(date.year(), month, day) {
                        // Use accent2 (rosewater) with different styling based on commit count
                        let commit_style = if commit_count >= 3 {
                            // High activity: bold rosewater
                            Style::default()
                                .fg(theme.accent2())
                                .bg(theme.mantle)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            // Some activity: normal rosewater
                            Style::default().fg(theme.accent2()).bg(theme.mantle)
                        };
                        event_store.add(time_date, commit_style);
                    }
                }
            }

            // Render 3 months horizontally
            let mut months = vec![];
            for offset in (0..3).rev() {
                let month_date = today - chrono::Duration::days(30 * offset);
                let year = month_date.year();
                let month = month_date.month();
                if let Ok(month_enum) = Month::try_from(month as u8) {
                    if let Ok(time_date) = Date::from_calendar_date(year, month_enum, 1) {
                        let cal = Monthly::new(time_date, &event_store)
                            .default_style(default_style)
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(format!("{:04}-{:02}", year, month))
                                    .title_style(theme.title_style())
                                    .border_style(theme.border_style())
                                    .style(theme.secondary_background_style()), // Mantle background
                            );
                        months.push(cal);
                    }
                }
            }
            let cal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(overview_chunks[chunk_idx]);
            for (i, cal) in months.into_iter().enumerate() {
                f.render_widget(cal, cal_chunks[i]);
            }
        } else {
            let calendar_paragraph = Paragraph::new("Calendar (last 3 months): [no data]")
                .alignment(Alignment::Center)
                .style(theme.muted_text_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Calendar")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()), // Mantle background
                );
            f.render_widget(calendar_paragraph, overview_chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Sparkline for commit activity (only if we have enough space)
    if show_sparkline {
        if state.git_enabled && !commit_dates.is_empty() {
            let sparkline_area = overview_chunks[chunk_idx];
            let width = sparkline_area.width.saturating_sub(2); // account for borders
            let num_days = 90;
            let today = Utc::now().date_naive();
            let start_date = today - chrono::Duration::days(num_days - 1);
            let bars = width as usize;
            let days_per_bar = (num_days as f32 / bars as f32).ceil() as usize;
            let mut buckets = vec![0u64; bars];
            for date in &commit_dates {
                if *date >= start_date && *date <= today {
                    let days_since_start = (*date - start_date).num_days() as usize;
                    let bar_idx = (days_since_start / days_per_bar).min(bars - 1);
                    buckets[bar_idx] += 1;
                }
            }
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Recent Activity (last 3 months)")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()), // Mantle background
                )
                .data(&buckets)
                .style(theme.accent2_style());
            f.render_widget(sparkline, sparkline_area);
        } else {
            let sparkline_paragraph = Paragraph::new("Recent Activity: [no data]")
                .alignment(Alignment::Center)
                .style(theme.muted_text_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Recent Activity")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()), // Mantle background
                );
            f.render_widget(sparkline_paragraph, overview_chunks[chunk_idx]);
        }
    }
}
