use anyhow::Result;

use ratatui::prelude::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};
use ratatui::Frame;

use rusqlite::Connection;

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

pub fn draw_yes_no_prompt(frame: &mut Frame, choice: bool, title: &str, main_rect: Rect) {
    let popup_area = create_popup_size((50, 5), main_rect);
    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(block.inner(popup_area));

    let yes = Paragraph::new(Line::from(vec![if choice {
        Span::raw("Yes").add_modifier(Modifier::UNDERLINED)
    } else {
        Span::raw("Yes")
    }]))
    .style(Style::default().fg(Color::Green))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Green)),
    );
    let no = Paragraph::new(Line::from(vec![if choice {
        Span::raw("No")
    } else {
        Span::raw("No").add_modifier(Modifier::UNDERLINED)
    }]))
    .style(Style::default().fg(Color::Red))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(yes, layout[0]);
    frame.render_widget(no, layout[1]);
    frame.render_widget(block, popup_area);
}

pub fn draw_text_prompt(
    frame: &mut ratatui::Frame<'_>,
    title: &str,
    text: &str,
    valid: bool,
    main_rect: ratatui::prelude::Rect,
) {
    let popup_area = create_popup_size((30, 5), main_rect);

    let new_note_entry = Paragraph::new(Line::from(vec![
        Span::raw(text).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if valid { Color::Green } else { Color::Red }))
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(new_note_entry, popup_area);
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

pub trait TryFromDatabase<T>: Sized {
    fn try_from_database(value: T, db: &Connection) -> Result<Self>;
}

pub trait DiscardResult {
    fn discard_result(self) -> Result<()>;
}

impl<T, E> DiscardResult for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn discard_result(self) -> Result<()> {
        self.map_err(Into::<anyhow::Error>::into).map(|_| ())
    }
}
