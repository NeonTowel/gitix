use crate::app::AppState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, layout::Rect};

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
    let (num_commits, num_branches, latest_author) = if state.git_enabled {
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
                    (num_commits, num_branches, latest_author)
                }
                Err(_) => (None, None, None),
            }
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
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

    // Calendar for last 3 months (placeholder)
    let calendar_paragraph = Paragraph::new("Calendar (last 3 months): [placeholder]")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Calendar"));
    f.render_widget(calendar_paragraph, overview_chunks[1]);

    // Sparkline for commit activity (placeholder)
    let sparkline_paragraph = Paragraph::new("Commit Activity Sparkline: [placeholder]")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Commit Activity"),
        );
    f.render_widget(sparkline_paragraph, overview_chunks[2]);

    // Combined label (placeholder)
    let label_paragraph = Paragraph::new("Date hints and direction: [placeholder]")
        .alignment(Alignment::Left)
        .block(Block::default());
    f.render_widget(label_paragraph, overview_chunks[3]);
}
