use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};

use foucault_core::permissions::Permissions;

use crate::{
    helpers::{create_popup, Capitalize},
    states::{
        note_creation::NoteCreationStateData, notes_managing::NotesManagingStateData,
        tags_managing::TagsManagingStateData, State,
    },
    NotebookAPI,
};

pub async fn run_nothing_state(key_event: KeyEvent, notebook: &NotebookAPI) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('c') if notebook.permissions.writable() => {
            info!("Open the note creation prompt.");
            State::NoteCreation(NoteCreationStateData::from_nothing())
        }
        KeyCode::Char('s') => {
            info!("Open the notes manager.");
            State::NotesManaging(NotesManagingStateData::empty(notebook).await?)
        }
        KeyCode::Char('t') => {
            info!("Open the tags manager.");
            State::TagsManaging(TagsManagingStateData::empty(notebook).await?)
        }
        _ => State::Nothing,
    })
}

pub fn draw_nothing_state(notebook: &NotebookAPI, frame: &mut Frame, main_rect: Rect) {
    let mut tile_text = vec![Line::from(vec![Span::raw(notebook.name.capitalize())
        .style(
            Style::new()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )])];

    if matches!(notebook.permissions, Permissions::ReadOnly) {
        tile_text.push(Line::from(vec![Span::raw(" READ ONLY ").style(
            Style::new()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    let title = Paragraph::new(tile_text).alignment(Alignment::Center);

    let writable_op_color = if notebook.permissions.writable() {
        Color::Blue
    } else {
        Color::Red
    };
    let commands = Table::new(
        [
            Row::new([
                Cell::from("q").set_style(Style::new().fg(Color::White).bg(Color::LightBlue)),
                Cell::from("Quit Foucault")
                    .set_style(Style::new().fg(Color::White).bg(Color::Black)),
            ]),
            Row::new([
                Cell::from("c").set_style(Style::new().fg(Color::White).bg(writable_op_color)),
                Cell::from("Create new note")
                    .set_style(Style::new().fg(Color::White).bg(Color::Black)),
            ]),
            Row::new([
                Cell::from("s").set_style(Style::new().fg(Color::White).bg(Color::LightBlue)),
                Cell::from("Search through notes")
                    .set_style(Style::new().fg(Color::White).bg(Color::Black)),
            ]),
            Row::new([
                Cell::from("t").set_style(Style::new().fg(Color::White).bg(Color::LightBlue)),
                Cell::from("Manage tags").set_style(Style::new().fg(Color::White).bg(Color::Black)),
            ]),
        ],
        [Constraint::Length(3), Constraint::Fill(1)],
    )
    .block(
        Block::new()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(Color::White)),
    );

    let title_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(2), Constraint::Fill(1)],
    )
    .split(create_popup(
        (Constraint::Percentage(40), Constraint::Length(8)),
        main_rect,
    ));

    frame.render_widget(title, title_layout[0]);
    frame.render_widget(commands, title_layout[1]);
}
