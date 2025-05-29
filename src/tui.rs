use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
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

pub fn start_tui() {
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

                // Tab bar
                let tab_titles: Vec<Line> = TAB_TITLES.iter().map(|t| Line::raw(*t)).collect();
                let tabs = Tabs::new(tab_titles)
                    .select(active_tab)
                    .block(Block::default().borders(Borders::ALL).title("Git TUI"))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().fg(Color::White));
                f.render_widget(tabs, chunks[0]);

                // Main area: placeholder for each tab
                let tab_content = match active_tab {
                    0 => "Overview tab (coming soon)",
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
                    match (key_event.code, key_event.modifiers) {
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            active_tab = (active_tab + 1) % tab_count;
                        }
                        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                            active_tab = (active_tab + tab_count - 1) % tab_count;
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
