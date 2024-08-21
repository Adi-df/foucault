use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    note::Note,
    states::{error::ErrorStateData, note_viewing::NoteViewingStateData, State},
    NotebookAPI,
};

pub struct NoteCreatingStateData {
    pub name: String,
    pub valid: bool,
}

impl NoteCreatingStateData {
    pub fn empty() -> Self {
        NoteCreatingStateData {
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
        KeyCode::Enter => match Note::validate_name(state_data.name.as_str(), notebook).await {
            Ok(true) => {
                info!("Create note : {}.", state_data.name.as_str());

                match Note::new(state_data.name.clone(), String::new(), notebook).await {
                    Ok(new_note) => {
                        State::NoteViewing(NoteViewingStateData::new(new_note, notebook).await?)
                    }
                    Err(err) => State::Error(ErrorStateData {
                        inner_state: Box::new(State::NoteCreating(state_data)),
                        error_message: err.to_string(),
                    }),
                }
            }
            Ok(false) => State::NoteCreating(NoteCreatingStateData {
                valid: false,
                ..state_data
            }),
            Err(err) => State::Error(ErrorStateData {
                inner_state: Box::new(State::NoteCreating(state_data)),
                error_message: err.to_string(),
            }),
        },
        KeyCode::Esc => {
            info!("Cancel note creation.");
            State::Nothing
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            match Note::validate_name(state_data.name.as_str(), notebook).await {
                Ok(valid) => State::NoteCreating(NoteCreatingStateData {
                    valid,
                    ..state_data
                }),
                Err(err) => State::Error(ErrorStateData {
                    inner_state: Box::new(State::NoteCreating(state_data)),
                    error_message: err.to_string(),
                }),
            }
        }
        KeyCode::Char(c) => {
            state_data.name.push(c);
            match Note::validate_name(state_data.name.as_str(), notebook).await {
                Ok(valid) => State::NoteCreating(NoteCreatingStateData {
                    valid,
                    ..state_data
                }),
                Err(err) => State::Error(ErrorStateData {
                    inner_state: Box::new(State::NoteCreating(state_data)),
                    error_message: err.to_string(),
                }),
            }
        }
        _ => State::NoteCreating(state_data),
    })
}

pub fn draw_note_creating_state(
    NoteCreatingStateData { name, valid }: &NoteCreatingStateData,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_text_prompt(frame, "Note name", name, *valid, main_rect);
}
