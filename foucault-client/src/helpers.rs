use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table},
    Frame,
};

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

pub fn create_bottom_line(rect: Rect) -> Rect {
    let vertical = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(100), Constraint::Min(5)],
    )
    .split(rect);
    vertical[1]
}

pub fn create_row_help_layout<'a>(help: &[(&'a str, &'a str)]) -> Table<'a> {
    Table::new(
        [Row::new(help.iter().flat_map(|(key, def)| {
            [
                Cell::from(*key).style(Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD)),
                Cell::from(*def).style(Style::new().bg(Color::Black)),
            ]
        }))],
        [Constraint::Fill(1), Constraint::Fill(2)]
            .into_iter()
            .cycle()
            .take(help.len() * 2),
    )
}

pub fn draw_yes_no_prompt(frame: &mut Frame, choice: bool, title: &str, main_rect: Rect) {
    let popup_area = create_popup_size((50, 5), main_rect);
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
    text: &str,
    valid: bool,
    main_rect: ratatui::prelude::Rect,
) {
    let popup_area = create_popup_size((30, 5), main_rect);

    let new_note_entry = Paragraph::new(Line::from(vec![
        Span::raw(text).style(Style::new().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
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
