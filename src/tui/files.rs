use crate::app::AppState;
use crate::files::{FileEntry, list_files};
use chrono::{Local, NaiveDateTime};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::{Frame, layout::Rect};

pub fn render_files_tab(f: &mut Frame, area: Rect, state: &AppState) {
    let files = list_files(&state.current_path);
    let header = ["Permissions", "Size", "Modified", "Name"];
    let rows: Vec<Row> = files
        .iter()
        .map(|entry| {
            let perms = format_permissions(entry.permissions, entry.is_dir);
            let size = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                entry.size.to_string()
            };
            let modified = format_time(entry.modified);
            let mut style = Style::default();
            if entry.is_dir {
                style = style.fg(Color::Blue).add_modifier(Modifier::BOLD);
            } else if entry.permissions & 0o111 != 0 {
                style = style.fg(Color::Green);
            }
            Row::new(vec![perms, size, modified, entry.name.clone()]).style(style)
        })
        .collect();
    let widths = [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Min(10),
    ];
    let table = Table::new(rows, widths)
        .header(Row::new(header))
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));
    f.render_widget(table, area);
}

fn format_permissions(perm: u32, is_dir: bool) -> String {
    let mut s = String::new();
    s.push(if is_dir { 'd' } else { '-' });
    for i in (0..3).rev() {
        let shift = i * 3;
        let r = if perm & (0o400 >> (6 - shift)) != 0 {
            'r'
        } else {
            '-'
        };
        let w = if perm & (0o200 >> (6 - shift)) != 0 {
            'w'
        } else {
            '-'
        };
        let x = if perm & (0o100 >> (6 - shift)) != 0 {
            'x'
        } else {
            '-'
        };
        s.push(r);
        s.push(w);
        s.push(x);
    }
    s
}

fn format_time(secs: u64) -> String {
    let dt = NaiveDateTime::from_timestamp_opt(secs as i64, 0)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    let offset = chrono::Local::now().offset().to_owned();
    let dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::from_naive_utc_and_offset(dt, offset);
    dt.format("%Y-%m-%d %H:%M").to_string()
}
