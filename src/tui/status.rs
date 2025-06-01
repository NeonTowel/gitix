use crate::app::AppState;
use crate::git::{format_file_size, get_git_status};
use crate::tui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState};
use ratatui::{layout::Rect, Frame};

pub fn render_status_tab(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    // Set panel background
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    // Load git status cache if not already loaded (when tab becomes active)
    state.load_status_git_status();

    if state.status_git_status.is_empty() {
        let clean_paragraph = Paragraph::new(
            "✓ No changes detected\n\nYour working directory is clean and matches the last commit.\n\nTo stage files and commit changes, use the 'Save Changes' tab.",
        )
        .alignment(Alignment::Center)
        .style(theme.success_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Repository Status")
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        );
        f.render_widget(clean_paragraph, area);
        return;
    }

    // Split area for table and info text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
        .split(area);

    // Ensure table state selection is valid
    if !state.status_git_status.is_empty() {
        let current_selection = state.status_table_state.selected().unwrap_or(0);
        if current_selection >= state.status_git_status.len() {
            state.status_table_state.select(Some(0));
        } else if state.status_table_state.selected().is_none() {
            // Initialize selection to first item if nothing is selected
            state.status_table_state.select(Some(0));
        }
    }

    // Create table headers
    let header = Row::new(vec![
        Cell::from("File Path").style(theme.accent2_style()),
        Cell::from("Status").style(theme.accent2_style()),
        Cell::from("Size").style(theme.accent2_style()),
    ]);

    // Create table rows
    let rows: Vec<Row> = state
        .status_git_status
        .iter()
        .map(|file| {
            let path_cell = Cell::from(file.path.display().to_string()).style(theme.text_style());

            let status_cell = Cell::from(file.status.as_description()).style(
                Style::default()
                    .fg(file.status.color())
                    .add_modifier(Modifier::BOLD),
            );

            let size_cell =
                Cell::from(format_file_size(file.file_size)).style(theme.secondary_text_style());

            Row::new(vec![path_cell, status_cell, size_cell])
        })
        .collect();

    // Create the table
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(60), // File path takes most space
            Constraint::Percentage(25), // Status column
            Constraint::Percentage(15), // Size column
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Repository Status ({} files) - Read Only",
                state.status_git_status.len()
            ))
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    )
    .row_highlight_style(theme.highlight_style())
    .highlight_symbol("► ");

    f.render_stateful_widget(table, chunks[0], &mut state.status_table_state);

    // Info text
    let info_text = Paragraph::new(vec![Line::from(vec![
        Span::raw("This tab shows the current repository status. "),
        Span::styled(
            "To stage files and commit changes, use the ",
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            "'Save Changes'",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" tab.", Style::default().fg(Color::Gray)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Information")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    )
    .style(theme.text_style());

    f.render_widget(info_text, chunks[1]);
}
