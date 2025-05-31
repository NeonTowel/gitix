use crate::app::AppState;
use crate::git::{format_file_size, get_git_status};
use crate::tui::theme::Theme;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState};
use ratatui::{Frame, layout::Constraint, layout::Rect};

pub fn render_status_tab(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = Theme::new();

    // Set panel background
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    let git_status = match get_git_status() {
        Ok(files) => files,
        Err(e) => {
            let error_paragraph = Paragraph::new(format!("Error reading repository: {}", e))
                .alignment(Alignment::Center)
                .style(theme.error_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Repository Status")
                        .title_style(theme.title_style())
                        .border_style(theme.border_style())
                        .style(theme.secondary_background_style()),
                );
            f.render_widget(error_paragraph, area);
            return;
        }
    };

    if git_status.is_empty() {
        let clean_paragraph = Paragraph::new(
            "✓ No changes detected\n\nYour working directory is clean and matches the last commit.",
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

    // Ensure table state selection is valid
    if !git_status.is_empty() {
        let current_selection = state.status_table_state.selected().unwrap_or(0);
        if current_selection >= git_status.len() {
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
    let rows: Vec<Row> = git_status
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
            .title(format!("Repository Changes ({} files)", git_status.len()))
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    )
    .row_highlight_style(theme.highlight_style())
    .highlight_symbol("► ");

    f.render_stateful_widget(table, area, &mut state.status_table_state);
}
