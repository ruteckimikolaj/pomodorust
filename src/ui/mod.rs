pub mod details;
pub mod notes_modal;
pub mod settings;
pub mod statistics;
pub mod task_list;
pub mod timer;

pub use details::draw_task_details;
pub use notes_modal::draw_notes_modal;
pub use settings::draw_settings;
pub use statistics::draw_statistics;
pub use task_list::draw_task_list;
pub use timer::draw_timer;

use ratatui::prelude::*;

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
