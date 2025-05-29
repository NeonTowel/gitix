use crate::app::AppState;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, layout::Rect};

pub fn render_update_tab(f: &mut Frame, area: Rect, _state: &AppState) {
    let paragraph = Paragraph::new("Update tab (coming soon)")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
