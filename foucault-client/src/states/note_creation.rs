use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::{draw_text_prompt, EdittableText},
    note::Note,
    states::{
        note_viewing::NoteViewingStateData,
        notes_managing::{draw_notes_managing_state, NotesManagingStateData},
        nothing::draw_nothing_state,
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
enum PrecidingState {
    Nothing,
    NotesManaging(NotesManagingStateData),
}

#[derive(Clone)]
pub struct NoteCreationStateData {
    preciding_state: PrecidingState,
    name: EdittableText,
    valid: bool,
}

impl NoteCreationStateData {
    pub fn from_nothing() -> Self {
        NoteCreationStateData {
            preciding_state: PrecidingState::Nothing,
            name: EdittableText::new(String::new()),
            valid: false,
        }
    }

    pub fn from_notes_managing(state_data: NotesManagingStateData) -> Self {
        NoteCreationStateData {
            preciding_state: PrecidingState::NotesManaging(state_data),
            name: EdittableText::new(String::new()),
            valid: false,
        }
    }
}

pub async fn run_note_creation_state(
    mut state_data: NoteCreationStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Cancel the note creation.");
            match state_data.preciding_state {
                PrecidingState::Nothing => State::Nothing,
                PrecidingState::NotesManaging(state) => State::NotesManaging(
                    NotesManagingStateData::from_pattern(state.pattern, notebook).await?,
                ),
            }
        }
        KeyCode::Enter => {
            if Note::validate_name(state_data.name.get_text(), notebook).await? {
                info!("Create note : {}.", state_data.name.get_text());

                let new_note =
                    Note::new(state_data.name.consume(), String::new(), notebook).await?;

                State::NoteViewing(NoteViewingStateData::new(new_note, notebook).await?)
            } else {
                State::NoteCreation(NoteCreationStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Backspace => {
            state_data.name.remove_char();
            state_data.valid = Note::validate_name(state_data.name.get_text(), notebook).await?;
            State::NoteCreation(state_data)
        }
        KeyCode::Delete => {
            state_data.name.del_char();
            state_data.valid = Note::validate_name(state_data.name.get_text(), notebook).await?;
            State::NoteCreation(state_data)
        }
        KeyCode::Left => {
            state_data.name.move_left();
            State::NoteCreation(state_data)
        }
        KeyCode::Right => {
            state_data.name.move_right();
            State::NoteCreation(state_data)
        }
        KeyCode::Char(c) => {
            state_data.name.insert_char(c);
            state_data.valid = Note::validate_name(state_data.name.get_text(), notebook).await?;
            State::NoteCreation(state_data)
        }
        _ => State::NoteCreation(state_data),
    })
}

pub fn draw_note_creation_state(
    NoteCreationStateData {
        preciding_state,
        name,
        valid,
    }: &NoteCreationStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    match preciding_state {
        PrecidingState::Nothing => draw_nothing_state(notebook, frame, main_rect),
        PrecidingState::NotesManaging(state) => {
            draw_notes_managing_state(state, notebook, frame, main_rect)
        }
    }

    draw_text_prompt(frame, "Note name", name, *valid, main_rect);
}
