use ratatui::prelude::{Constraint, Direction, Layout, Rect};

pub fn create_popup_proportion(proportion: (u16, u16), rect: Rect) -> Rect {
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

pub fn create_popup_size(size: (u16, u16), rect: Rect) -> Rect {
    let vertical = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length((rect.height - size.1) / 2),
            Constraint::Length(size.1),
            Constraint::Length((rect.height - size.1) / 2),
        ],
    )
    .split(rect);
    let horizontal = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Length((rect.width - size.0) / 2),
            Constraint::Length(size.0),
            Constraint::Length((rect.width - size.0) / 2),
        ],
    )
    .split(vertical[1]);
    horizontal[1]
}

pub trait Capitalize<'a> {
    fn capitalize(&'a self) -> String;
}

impl<'a, T: 'a + ?Sized> Capitalize<'a> for T
where
    &'a T: Into<&'a str>,
{
    fn capitalize(&'a self) -> String {
        let inner_str: &'a str = self.into();
        let mut formated_string = inner_str[0..1].to_uppercase();
        formated_string.push_str(&inner_str[1..]);
        formated_string
    }
}
