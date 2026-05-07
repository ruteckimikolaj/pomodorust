use ratatui::{prelude::*, widgets::*};

use super::centered_rect;
use crate::app::UiState;
use crate::settings::Theme;

pub fn draw_notes_modal(frame: &mut Frame, ui: &UiState, theme: &Theme) {
    let Some(textarea) = &ui.notes_textarea else { return };

    let area = centered_rect(70, 55, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Notes — [Ctrl+S] Save  [Esc] Cancel ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(theme.accent_color).bg(theme.base_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(textarea, inner);
}
