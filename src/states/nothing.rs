use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table};

use crate::helpers::{create_popup_proportion, Capitalize, DiscardResult};
use crate::notebook::Notebook;
use crate::states::note_creating::NoteCreatingStateData;
use crate::states::notes_managing::NotesManagingStateData;
use crate::states::tags_managing::TagsManagingStateData;
use crate::states::{State, Terminal};

pub async fn run_nothing_state(key_event: KeyEvent, notebook: &Notebook) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('c') => {
            info!("Open new note prompt.");
            State::NoteCreating(NoteCreatingStateData::empty())
        }
        KeyCode::Char('s') => {
            info!("Open notes listing.");
            State::NotesManaging(NotesManagingStateData::empty(notebook.db()).await?)
        }
        KeyCode::Char('t') => {
            info!("Open tags manager.");
            State::TagsManaging(TagsManagingStateData::empty(notebook.db())?)
        }
        _ => State::Nothing,
    })
}

pub fn draw_nothing_state(
    terminal: &mut Terminal,
    notebook: &Notebook,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            let title = Paragraph::new(Line::from(vec![Span::raw(notebook.name.capitalize())
                .style(
                    Style::new()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]))
            .alignment(Alignment::Center);

            let commands = Table::new(
                [
                    Row::new([Cell::from("q"), Cell::from("Quit Foucault")]),
                    Row::new([Cell::from("c"), Cell::from("Create new note")]),
                    Row::new([Cell::from("s"), Cell::from("Search through notes")]),
                    Row::new([Cell::from("t"), Cell::from("Manage tags")]),
                ],
                [Constraint::Length(3), Constraint::Fill(1)],
            )
            .block(
                Block::new()
                    .padding(Padding::uniform(1))
                    .borders(Borders::all())
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::White)),
            );

            let title_layout = Layout::new(
                Direction::Vertical,
                [Constraint::Length(1), Constraint::Fill(1)],
            )
            .split(create_popup_proportion((40, 30), main_rect));

            frame.render_widget(title, title_layout[0]);
            frame.render_widget(commands, title_layout[1]);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
