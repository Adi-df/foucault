use ratatui::prelude::{Constraint, Direction, Layout, Rect};

pub fn create_popup(proportion: (u16, u16), rect: Rect) -> Rect {
    let vertical = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage((100 - proportion.1) / 2),
            Constraint::Percentage(proportion.1),
            Constraint::Percentage((100 - proportion.1) / 2),
        ],
    )
    .split(rect);
    let horizontal = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage((100 - proportion.0) / 2),
            Constraint::Percentage(proportion.0),
            Constraint::Percentage((100 - proportion.0) / 2),
        ],
    )
    .split(vertical[1]);
    horizontal[1]
}
