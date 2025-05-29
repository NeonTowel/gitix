use crate::app::AppState;
use chrono::{Duration, NaiveDate, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use gix::bstr::ByteSlice;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline, Tabs};
use std::io;
use time::{Date, Month};

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

// Helper struct for overview stats
struct OverviewStats {
    num_commits: Option<u64>,
    num_branches: Option<u64>,
    latest_author: Option<String>,
    commit_activity: Option<Vec<u64>>, // e.g. commits per week for 6 months
    calendar_events: Option<CalendarEventStore>, // commit days for calendar
}

// Real function to get stats from gix
fn get_overview_stats(repo_path: &std::path::Path) -> OverviewStats {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::widgets::calendar::CalendarEventStore;
    let repo = match gix::open(repo_path) {
        Ok(r) => r,
        Err(_) => {
            return OverviewStats {
                num_commits: None,
                num_branches: None,
                latest_author: None,
                commit_activity: None,
                calendar_events: None,
            };
        }
    };
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
    OverviewStats {
        num_commits,
        num_branches,
        latest_author,
        commit_activity: None,
        calendar_events: None,
    }
}

pub fn start_tui(state: &mut AppState) {
    // Terminal setup
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

                // Tab bar with disabled tabs if not in git repo
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

                // Main area: Overview tab logic
                if active_tab == 0 {
                    if state.git_enabled {
                        // Get stats (replace with real data later)
                        let stats = state.repo_root.as_ref()
                            .map(|p| get_overview_stats(p))
                            .unwrap_or_else(|| OverviewStats {
                                num_commits: None,
                                num_branches: None,
                                latest_author: None,
                                commit_activity: None,
                                calendar_events: None,
                            });
                        let overview_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Length(5), // stats row
                                Constraint::Length(10), // calendar
                                Constraint::Length(8), // sparkline
                                Constraint::Length(1), // combined label (date hints + direction)
                                Constraint::Min(0),    // filler
                            ])
                            .split(chunks[1]);

                        // Stats row
                        let mut stats_lines = vec![];
                        if let Some(n) = stats.num_commits {
                            stats_lines.push(format!("Commits: {}", n));
                        }
                        if let Some(n) = stats.num_branches {
                            stats_lines.push(format!("Branches: {}", n));
                        }
                        if let Some(ref author) = stats.latest_author {
                            stats_lines.push(format!("Latest Author: {}", author));
                        }
                        let stats_text = stats_lines.join("    |    ");
                        let stats_paragraph = Paragraph::new(stats_text)
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL).title("Repository Stats"));
                        f.render_widget(stats_paragraph, overview_chunks[0]);

                        // Calendar for last 3 months
                        if let Some(ref events) = stats.calendar_events {
                            use chrono::{Datelike, Utc};
                            use ratatui::widgets::calendar::Monthly;
                            let today = Utc::now().date_naive();
                            let mut months = vec![];
                            for offset in (0..3).rev() {
                                let month_date = today - chrono::Duration::days(30 * offset);
                                let year = month_date.year();
                                let month = month_date.month();
                                // Convert to time::Date for Monthly::new
                                if let Ok(time_date) = Date::from_calendar_date(year, Month::try_from(month as u8).unwrap(), 1) {
                                    let cal = Monthly::new(
                                        time_date,
                                        events.clone(),
                                    )
                                    .block(Block::default().borders(Borders::ALL).title(format!("{:04}-{:02}", year, month)));
                                    months.push(cal);
                                }
                            }
                            // Layout horizontally for 3 months
                            let cal_chunks = ratatui::layout::Layout::default()
                                .direction(ratatui::layout::Direction::Horizontal)
                                .constraints([
                                    ratatui::layout::Constraint::Percentage(33),
                                    ratatui::layout::Constraint::Percentage(33),
                                    ratatui::layout::Constraint::Percentage(34),
                                ])
                                .split(overview_chunks[1]);
                            for (i, cal) in months.into_iter().enumerate() {
                                f.render_widget(cal, cal_chunks[i]);
                            }
                        }

                        // Sparkline for commit activity (last 6 months)
                        if let Some(ref activity) = stats.commit_activity {
                            let spark = Sparkline::default()
                                .block(Block::default().borders(Borders::ALL).title("Commit Activity (last 6 months)"))
                                .data(activity)
                                .style(Style::default().fg(Color::Cyan))
                                .bar_set(symbols::bar::NINE_LEVELS);
                            f.render_widget(spark, overview_chunks[2]);
                            // Combined label: month/year hints and direction
                            use chrono::Datelike;
                            let today = Utc::now().date_naive();
                            let weeks = activity.len();
                            let start_date = today - chrono::Duration::weeks(weeks as i64 - 1);
                            let mid_date = start_date + chrono::Duration::weeks((weeks as i64 - 1) / 2);
                            let end_date = today;
                            let start_label = start_date.format("%b %Y").to_string();
                            let mid_label = mid_date.format("%b %Y").to_string();
                            let end_label = end_date.format("%b %Y").to_string();
                            let direction = "(oldest [33m→[0m newest)"; // arrow with color hint
                            let width = overview_chunks[2].width as usize;
                            // Compose the combined label
                            let mut label_line = vec![' '; width];
                            // Place start_label at far left
                            for (i, c) in start_label.chars().enumerate() {
                                if i < width {
                                    label_line[i] = c;
                                }
                            }
                            // Place end_label at far right
                            for (i, c) in end_label.chars().rev().enumerate() {
                                if i < width {
                                    label_line[width - 1 - i] = c;
                                }
                            }
                            // Place mid_label and direction at center
                            let mid = width / 2;
                            let mid_label_start = mid.saturating_sub((mid_label.len() + direction.len() + 1) / 2);
                            let dir_start = mid_label_start + mid_label.len() + 1;
                            for (i, c) in mid_label.chars().enumerate() {
                                if mid_label_start + i < width {
                                    label_line[mid_label_start + i] = c;
                                }
                            }
                            for (i, c) in direction.chars().enumerate() {
                                if dir_start + i < width {
                                    label_line[dir_start + i] = c;
                                }
                            }
                            let label_string: String = label_line.into_iter().collect();
                            let date_hint = Paragraph::new(label_string)
                                .alignment(Alignment::Left)
                                .style(Style::default().fg(Color::DarkGray));
                            f.render_widget(date_hint, overview_chunks[3]);
                        }
                    } else {
                        let paragraph = Paragraph::new("Overview: Limited functionality. Not a Git repository.")
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL));
                        f.render_widget(paragraph, chunks[1]);
                    }
                } else {
                    let tab_content = match active_tab {
                        1 => "Files tab (coming soon)",
                        2 => "Status tab (coming soon)",
                        3 => "Save Changes tab (coming soon)",
                        4 => "Update tab (coming soon)",
                        5 => "Settings tab (coming soon)",
                        _ => unreachable!(),
                    };
                    let paragraph = Paragraph::new(tab_content)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                }

                // Modal popup for git init prompt
                if active_tab == 0 && state.show_init_prompt {
                    let area = centered_rect(60, 7, size);
                    let modal = Paragraph::new("This folder is not a Git repository.\n\nInitialize a new Git repository here? (Y/N)")
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .title("Initialize Git Repository")
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        );
                    f.render_widget(modal, area);
                }

                // Key hints
                let hints = "[Tab] Next Tab  [Shift+Tab] Previous Tab  [q] Quit";
                let hint_paragraph = Paragraph::new(hints)
                    .alignment(Alignment::Center)
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
                        continue; // Skip other input handling while prompt is active
                    }

                    // Only allow navigation to enabled tabs
                    let max_enabled_tab = if state.git_enabled { tab_count - 1 } else { 1 };
                    match (key_event.code, key_event.modifiers) {
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            // Find next enabled tab
                            let mut next_tab = (active_tab + 1) % tab_count;
                            while !state.git_enabled && next_tab > 1 {
                                next_tab = (next_tab + 1) % tab_count;
                            }
                            active_tab = next_tab;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            // Find previous enabled tab
                            let mut prev_tab = (active_tab + tab_count - 1) % tab_count;
                            while !state.git_enabled && prev_tab > 1 {
                                prev_tab = (prev_tab + tab_count - 1) % tab_count;
                            }
                            active_tab = prev_tab;
                        }
                        (KeyCode::Char('q'), _) => {
                            break;
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
    let popup_layout = Layout::default()
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
    let horizontal = Layout::default()
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
