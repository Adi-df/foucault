use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::helpers::{create_popup_proportion, Capitalize, DiscardResult};
use crate::notebook::Notebook;
use crate::states::note_creating::NoteCreatingStateData;
use crate::states::notes_managing::NotesManagingStateData;
use crate::states::tags_managing::TagsManagingStateData;
use crate::states::{State, Terminal};

pub fn run_nothing_state(key_code: KeyCode, notebook: &Notebook) -> Result<State> {
    Ok(match key_code {
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
            State::NotesManaging(NotesManagingStateData::empty(notebook.db())?)
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
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]))
            .alignment(Alignment::Center);

            frame.render_widget(title, create_popup_proportion((40, 10), main_rect));

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
