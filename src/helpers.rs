use anyhow::Result;
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

pub struct OptionalValue<T, E>(Option<T>, E);

impl<T, E> OptionalValue<T, E> {
    pub fn new(val: Option<T>, err: E) -> Self {
        Self(val, err)
    }

    pub fn set(&mut self, val: Option<T>) {
        self.0 = val;
    }
}

impl<T, E> OptionalValue<T, E>
where
    E: Into<anyhow::Error> + Clone,
{
    pub fn by_ref(&self) -> Result<&T> {
        self.0.as_ref().ok_or_else(|| self.1.clone().into())
    }

    pub fn by_ref_mut(&mut self) -> Result<&mut T> {
        self.0.as_mut().ok_or_else(|| self.1.clone().into())
    }
}

impl<'a, T, E> From<&'a OptionalValue<T, E>> for Result<&'a T>
where
    E: Into<anyhow::Error> + Clone,
{
    fn from(value: &'a OptionalValue<T, E>) -> Self {
        value.0.as_ref().ok_or_else(|| value.1.clone().into())
    }
}

impl<'a, T, E> From<&'a mut OptionalValue<T, E>> for Result<&'a mut T>
where
    E: Into<anyhow::Error> + Clone,
{
    fn from(value: &'a mut OptionalValue<T, E>) -> Self {
        value.0.as_mut().ok_or_else(|| value.1.clone().into())
    }
}
