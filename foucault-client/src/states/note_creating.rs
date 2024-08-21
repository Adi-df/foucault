use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    note::Note,
    states::{note_viewing::NoteViewingStateData, State},
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
    NoteCreatingStateData { mut name, valid }: NoteCreatingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Enter => {
            if Note::validate_name(name.as_str(), notebook).await? {
                info!("Create note : {}.", name.as_str());

                let new_note = Note::new(name.clone(), String::new(), notebook).await?;

                State::NoteViewing(NoteViewingStateData::new(new_note, notebook).await?)
            } else {
                State::NoteCreating(NoteCreatingStateData { name, valid: false })
            }
        }
        KeyCode::Esc => {
            info!("Cancel note creation.");
            State::Nothing
        }
        KeyCode::Backspace => {
            name.pop();
            State::NoteCreating(NoteCreatingStateData {
                valid: Note::validate_name(name.as_str(), notebook).await?,
                name,
            })
        }
        KeyCode::Char(c) => {
            name.push(c);
            State::NoteCreating(NoteCreatingStateData {
                valid: Note::validate_name(name.as_str(), notebook).await?,

                name,
            })
        }
        _ => State::NoteCreating(NoteCreatingStateData { name, valid }),
    })
}

pub fn draw_note_creating_state(
    NoteCreatingStateData { name, valid }: &NoteCreatingStateData,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_text_prompt(frame, "Note name", name, *valid, main_rect);
}
