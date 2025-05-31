use crate::app::AppState;
use crate::tui::theme::Theme;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, layout::Rect};

pub fn render_update_tab(f: &mut Frame, area: Rect, _state: &AppState) {
    let theme = Theme::new();

    let paragraph = Paragraph::new("Update tab (coming soon)")
        .alignment(Alignment::Center)
        .style(theme.text_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Update")
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        );
    f.render_widget(paragraph, area);
}
