use crate::app::AppState;
use chrono::{Datelike, NaiveDate, Utc};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::{Frame, layout::Rect};
use time::{Date, Month};

pub fn render_overview_tab(f: &mut Frame, area: Rect, state: &AppState) {
    let overview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // stats row
            Constraint::Length(10), // calendar
            Constraint::Length(8),  // sparkline
            Constraint::Length(1),  // combined label (date hints + direction)
            Constraint::Min(0),     // filler
        ])
        .split(area);

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

    // Stats row
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
    f.render_widget(stats_paragraph, overview_chunks[0]);

    // --- Calendar for last 3 months ---
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
            .split(overview_chunks[1]);
        for (i, cal) in months.into_iter().enumerate() {
            f.render_widget(cal, cal_chunks[i]);
        }
    } else {
        let calendar_paragraph = Paragraph::new("Calendar (last 3 months): [no data]")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Calendar"));
        f.render_widget(calendar_paragraph, overview_chunks[1]);
    }

    // Sparkline for commit activity (histogram by week)
    if state.git_enabled && !commit_dates.is_empty() {
        // Calculate the number of weeks to show (e.g., 12 weeks)
        let num_weeks = 12;
        let today = Utc::now().date_naive();
        let start_date = today - chrono::Duration::weeks(num_weeks as i64 - 1);
        // Map each commit date to a week index (0 = oldest week, num_weeks-1 = this week)
        let mut week_buckets = vec![0u64; num_weeks];
        for date in &commit_dates {
            if *date >= start_date && *date <= today {
                let week_idx = ((*date - start_date).num_days() / 7) as usize;
                if week_idx < num_weeks {
                    week_buckets[week_idx] += 1;
                }
            }
        }
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Commit Activity (weekly)"),
            )
            .data(&week_buckets)
            .style(Style::default().fg(Color::Green));
        f.render_widget(sparkline, overview_chunks[2]);
    } else {
        let sparkline_paragraph = Paragraph::new("Commit Activity Sparkline: [no data]")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Commit Activity"),
            );
        f.render_widget(sparkline_paragraph, overview_chunks[2]);
    }

    // Combined label (placeholder)
    let label_paragraph = Paragraph::new("Date hints and direction: [placeholder]")
        .alignment(Alignment::Left)
        .block(Block::default());
    f.render_widget(label_paragraph, overview_chunks[3]);
}
