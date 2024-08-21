use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    note::Note,
    states::{error::ErrorStateData, note_viewing::NoteViewingStateData, State},
    try_err, NotebookAPI,
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
        KeyCode::Enter => {
            if try_err!(
                Note::validate_name(state_data.name.as_str(), notebook).await,
                State::NoteCreating(state_data)
            ) {
                info!("Create note : {}.", state_data.name.as_str());

                let new_note = try_err!(
                    Note::new(state_data.name.clone(), String::new(), notebook).await,
                    State::NoteCreating(state_data)
                );

                let data = try_err!(
                    NoteViewingStateData::new(new_note, notebook).await,
                    State::NoteCreating(state_data)
                );

                State::NoteViewing(data)
            } else {
                State::NoteCreating(NoteCreatingStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Esc => {
            info!("Cancel note creation.");
            State::Nothing
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            let valid = try_err!(
                Note::validate_name(state_data.name.as_str(), notebook).await,
                State::NoteCreating(state_data)
            );
            State::NoteCreating(NoteCreatingStateData {
                valid,
                ..state_data
            })
        }
        KeyCode::Char(c) => {
            state_data.name.push(c);
            let valid = try_err!(
                Note::validate_name(state_data.name.as_str(), notebook).await,
                State::NoteCreating(state_data)
            );
            State::NoteCreating(NoteCreatingStateData {
                valid,
                ..state_data
            })
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
