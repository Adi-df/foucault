use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    note::Note,
    states::{note_viewing::NoteViewingStateData, notes_managing::NotesManagingStateData, State},
    NotebookAPI,
};

use super::{notes_managing::draw_note_managing_state, nothing::draw_nothing_state};

#[derive(Clone)]
enum PrecidingState {
    Nothing,
    NotesManaging(NotesManagingStateData),
}

#[derive(Clone)]
pub struct NoteCreatingStateData {
    preciding_state: PrecidingState,
    pub name: String,
    pub valid: bool,
}

impl NoteCreatingStateData {
    pub fn from_nothing() -> Self {
        NoteCreatingStateData {
            preciding_state: PrecidingState::Nothing,
            name: String::new(),
            valid: false,
        }
    }

    pub fn from_notes_managing(state_data: NotesManagingStateData) -> Self {
        NoteCreatingStateData {
            preciding_state: PrecidingState::NotesManaging(state_data),
            name: String::new(),
            valid: false,
        }
    }
}

pub async fn run_note_creating_state(
    mut state_data: NoteCreatingStateData,
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
            if Note::validate_name(state_data.name.as_str(), notebook).await? {
                info!("Create note : {}.", state_data.name.as_str());

                let new_note = Note::new(state_data.name, String::new(), notebook).await?;

                State::NoteViewing(NoteViewingStateData::new(new_note, notebook).await?)
            } else {
                State::NoteCreating(NoteCreatingStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            let valid = Note::validate_name(state_data.name.as_str(), notebook).await?;
            State::NoteCreating(NoteCreatingStateData {
                valid,
                ..state_data
            })
        }
        KeyCode::Char(c) => {
            state_data.name.push(c);
            let valid = Note::validate_name(state_data.name.as_str(), notebook).await?;
            State::NoteCreating(NoteCreatingStateData {
                valid,
                ..state_data
            })
        }
        _ => State::NoteCreating(state_data),
    })
}

pub fn draw_note_creating_state(
    NoteCreatingStateData {
        preciding_state,
        name,
        valid,
    }: &NoteCreatingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    match preciding_state {
        PrecidingState::Nothing => draw_nothing_state(notebook, frame, main_rect),
        PrecidingState::NotesManaging(state) => {
            draw_note_managing_state(state, notebook, frame, main_rect)
        }
    }

    draw_text_prompt(frame, "Note name", name, *valid, main_rect);
}
