use std::ops::Deref;

use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn create_popup(proportion: (Constraint, Constraint), rect: Rect) -> Rect {
    let vertical = Layout::new(
        Direction::Vertical,
        match proportion.1 {
            Constraint::Percentage(percent) => [
                Constraint::Percentage((100u16.saturating_sub(percent)) / 2),
                Constraint::Percentage(percent),
                Constraint::Percentage((100u16.saturating_sub(percent)) / 2),
            ],
            Constraint::Length(length) => [
                Constraint::Length((rect.height.saturating_sub(length)) / 2),
                Constraint::Length(length),
                Constraint::Length((rect.height.saturating_sub(length)) / 2),
            ],
            _ => unimplemented!(),
        },
    )
    .split(rect);
    let horizontal = Layout::new(
        Direction::Horizontal,
        match proportion.0 {
            Constraint::Percentage(percent) => [
                Constraint::Percentage((100u16.saturating_sub(percent)) / 2),
                Constraint::Percentage(percent),
                Constraint::Percentage((100u16.saturating_sub(percent)) / 2),
            ],
            Constraint::Length(length) => [
                Constraint::Length((rect.width.saturating_sub(length)) / 2),
                Constraint::Length(length),
                Constraint::Length((rect.width.saturating_sub(length)) / 2),
            ],
            _ => unimplemented!(),
        },
    )
    .split(vertical[1]);
    horizontal[1]
}

pub fn create_bottom_help_bar<'a>(
    help: &[(&'a str, Color, &'a str)],
    max_by_row: usize,
    rect: Rect,
) -> (Table<'a>, Rect) {
    let rows: Vec<_> = help
        .chunks(max_by_row)
        .map(|infos| {
            Row::new(infos.iter().flat_map(|(key, color, def)| {
                [
                    Cell::from(*key).style(Style::new().bg(*color).add_modifier(Modifier::BOLD)),
                    Cell::from(*def).style(Style::new().bg(Color::Black)),
                ]
            }))
        })
        .collect();
    let row_count = rows.len();
    let table = Table::new(
        rows,
        [Constraint::Fill(1), Constraint::Fill(2)]
            .into_iter()
            .cycle()
            .take((help.len() * 2).min(max_by_row * 2)),
    )
    .block(
        Block::new()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(Color::White)),
    );

    let vertical = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(100),
            Constraint::Min(u16::try_from(row_count).unwrap() + 2),
        ],
    )
    .split(rect);
    (table, vertical[1])
}

pub fn create_left_help_bar<'a>(
    help: &[(&'a str, Color, &'a str)],
    rect: Rect,
) -> (Table<'a>, Rect) {
    let rows: Vec<_> = help
        .iter()
        .map(|(key, color, def)| {
            Row::new([
                Cell::from(*key).style(Style::new().bg(*color).add_modifier(Modifier::BOLD)),
                Cell::from(*def).style(Style::new().bg(Color::Black)),
            ])
        })
        .collect();
    let table = Table::new(rows, [Constraint::Fill(1), Constraint::Fill(2)]).block(
        Block::new()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(Color::White)),
    );

    let horrizontal = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(40), Constraint::Min(0)],
    )
    .split(rect);
    (table, horrizontal[0])
}

pub fn draw_yes_no_prompt(frame: &mut Frame, choice: bool, title: &str, main_rect: Rect) {
    let popup_area = create_popup((Constraint::Length(50), Constraint::Length(5)), main_rect);
    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Blue));

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
    .style(Style::new().fg(Color::Green))
    .alignment(Alignment::Center)
    .block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::new().fg(Color::Green)),
    );
    let no = Paragraph::new(Line::from(vec![if choice {
        Span::raw("No")
    } else {
        Span::raw("No").add_modifier(Modifier::UNDERLINED)
    }]))
    .style(Style::new().fg(Color::Red))
    .alignment(Alignment::Center)
    .block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::new().fg(Color::Red)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(yes, layout[0]);
    frame.render_widget(no, layout[1]);
    frame.render_widget(block, popup_area);
}

pub fn draw_text_prompt(
    frame: &mut ratatui::Frame<'_>,
    title: &str,
    text: &EditableText,
    valid: bool,
    main_rect: ratatui::prelude::Rect,
) {
    let popup_area = create_popup((Constraint::Length(30), Constraint::Length(5)), main_rect);

    let new_note_entry = text.build_paragraph().block(
        Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if valid { Color::Green } else { Color::Red }))
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(new_note_entry, popup_area);
}

#[derive(Clone)]
pub struct EditableText {
    text: String,
    cursor: usize,
}

impl EditableText {
    pub fn new(text: String) -> Self {
        Self {
            cursor: text.graphemes(true).count(),
            text,
        }
    }

    fn len(&self) -> usize {
        self.text.graphemes(true).count()
    }

    pub fn consume(self) -> String {
        self.text
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor == 0 {
            self.text.insert(0, c);
        } else {
            self.text.insert(
                self.text
                    .grapheme_indices(true)
                    .map(|(i, g)| i + g.len())
                    .nth(self.cursor - 1)
                    .unwrap(),
                c,
            );
        }
        self.move_right();
    }

    pub fn remove_char(&mut self) {
        if !self.text.is_empty() && self.cursor > 0 {
            let (start, end) = self
                .text
                .grapheme_indices(true)
                .map(|(i, g)| (i, i + g.len()))
                .nth(self.cursor - 1)
                .unwrap();
            self.text.drain(start..end);
            self.move_left();
        }
    }

    pub fn del_char(&mut self) {
        if self.cursor < self.text.len() {
            let (start, end) = self
                .text
                .grapheme_indices(true)
                .map(|(i, g)| (i, i + g.len()))
                .nth(self.cursor)
                .unwrap();
            self.text.drain(start..end);
        }
    }

    pub fn build_paragraph(&self) -> Paragraph {
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(self.text.as_str(), true).collect();

        let before_cursor = Span::raw(graphemes[..self.cursor].concat())
            .style(Style::new().add_modifier(Modifier::UNDERLINED));
        let cursor = if self.cursor == graphemes.len() {
            Span::raw(" ").style(Style::new().bg(Color::Black))
        } else {
            Span::raw(graphemes[self.cursor..=self.cursor].concat()).style(
                Style::new()
                    .bg(Color::Black)
                    .add_modifier(Modifier::UNDERLINED),
            )
        };
        let after_cursor = if self.cursor == graphemes.len() {
            Span::raw("")
        } else {
            Span::raw(graphemes[(self.cursor + 1)..].concat())
                .style(Style::new().add_modifier(Modifier::UNDERLINED))
        };
        Paragraph::new(Line::from(vec![before_cursor, cursor, after_cursor]))
    }
}

impl Deref for EditableText {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
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
        if let Some(b) = inner_str.chars().nth(0) {
            let mut formated_string = b.to_uppercase().to_string();
            formated_string.extend(inner_str.chars().skip(1));
            formated_string
        } else {
            String::new()
        }
    }
}
