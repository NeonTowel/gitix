use crate::app::AppState;
use chrono::{Datelike, NaiveDate, Utc};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::{Frame, layout::Rect};
use time::{Date, Month};

pub fn render_overview_tab(f: &mut Frame, area: Rect, state: &AppState) {
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
            .block(Block::default().borders(Borders::ALL).title("Overview"));
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

        // Repository stats line (centered)
        let mut stats_lines = vec![];
        if let Some(n) = num_commits {
            stats_lines.push(format!("Commits: {}", n));
        }
        if let Some(n) = num_branches {
            stats_lines.push(format!("Branches: {}", n));
        }
        if let Some(ref author) = latest_author {
            stats_lines.push(format!("Latest Author: {}", author));
        }
        let stats_text = if stats_lines.is_empty() {
            "No repository stats available".to_string()
        } else {
            stats_lines.join("    |    ")
        };

        let stats_paragraph = Paragraph::new(stats_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Repository Stats"),
            );
        f.render_widget(stats_paragraph, stats_chunks[0]);

        // Mock commit history data (TODO: replace with real data)
        let mock_commits = vec![
            (
                "feat: add responsive overview layout",
                "John Doe",
                "2 minutes ago",
            ),
            (
                "fix: calendar widget height adjustment",
                "Jane Smith",
                "15 minutes ago",
            ),
            (
                "docs: update README with new features",
                "Bob Wilson",
                "1 hour ago",
            ),
            (
                "refactor: improve git repository handling",
                "Alice Brown",
                "3 hours ago",
            ),
            ("chore: update dependencies", "Charlie Davis", "2024-01-15"),
        ];

        // Build commit history text (left-aligned within centered block)
        let mut commit_text = String::new();
        for (message, author, time) in &mock_commits {
            // Truncate commit message if too long
            let truncated_message = if message.len() > 50 {
                format!("{}...", &message[..47])
            } else {
                message.to_string()
            };

            if !commit_text.is_empty() {
                commit_text.push('\n');
            }
            commit_text.push_str(&format!("• {} - {} ({})", truncated_message, author, time));
        }

        let commit_paragraph = Paragraph::new(commit_text)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Changes"),
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
            // Style for commit days (bright green)
            let commit_style = Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD);
            // Style for non-commit days (dimmed almost black)
            let default_style = Style::default()
                .fg(Color::Rgb(20, 20, 20))
                .bg(Color::Black)
                .add_modifier(Modifier::DIM);
            // Add commit days to event store
            for date in &commit_dates {
                if *date >= three_months_ago {
                    let month = Month::try_from(date.month() as u8).ok();
                    let day = u8::try_from(date.day()).ok();
                    if let (Some(month), Some(day)) = (month, day) {
                        if let Ok(time_date) = Date::from_calendar_date(date.year(), month, day) {
                            event_store.add(time_date, commit_style);
                        }
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
                                    .title(format!("{:04}-{:02}", year, month)),
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
                .block(Block::default().borders(Borders::ALL).title("Calendar"));
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
                        .title("Recent Activity (last 3 months)"),
                )
                .data(&buckets)
                .style(Style::default().fg(Color::Green));
            f.render_widget(sparkline, sparkline_area);
        } else {
            let sparkline_paragraph = Paragraph::new("Recent Activity: [no data]")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Recent Activity"),
                );
            f.render_widget(sparkline_paragraph, overview_chunks[chunk_idx]);
        }
    }
}
