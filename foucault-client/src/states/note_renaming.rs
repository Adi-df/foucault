use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    note::Note,
    states::{
        note_viewing::{draw_note_viewing_state, NoteViewingStateData},
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteRenamingStateData {
    pub note_viewing_data: NoteViewingStateData,
    pub new_name: String,
    pub valid: bool,
}

impl NoteRenamingStateData {
    pub fn empty(note_viewing_data: NoteViewingStateData) -> Self {
        NoteRenamingStateData {
            note_viewing_data,
            new_name: String::new(),
            valid: false,
        }
    }
}

pub async fn run_note_renaming_state(
    mut state_data: NoteRenamingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the renaming note {}",
                state_data.note_viewing_data.note.name()
            );
            State::NoteViewing(
                NoteViewingStateData::new(state_data.note_viewing_data.note, notebook).await?,
            )
        }
        KeyCode::Enter => {
            if Note::validate_name(state_data.new_name.as_str(), notebook).await? {
                info!(
                    "Rename note {} to {}.",
                    state_data.note_viewing_data.note.name(),
                    state_data.new_name
                );
                state_data
                    .note_viewing_data
                    .note
                    .rename(state_data.new_name, notebook)
                    .await?;
                State::NoteViewing(
                    NoteViewingStateData::new(state_data.note_viewing_data.note, notebook).await?,
                )
            } else {
                State::NoteRenaming(NoteRenamingStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Backspace => {
            state_data.new_name.pop();
            state_data.valid = Note::validate_name(state_data.new_name.as_str(), notebook).await?;
            State::NoteRenaming(state_data)
        }
        KeyCode::Char(c) => {
            state_data.new_name.push(c);
            state_data.valid = Note::validate_name(state_data.new_name.as_str(), notebook).await?;
            State::NoteRenaming(state_data)
        }
        _ => State::NoteRenaming(state_data),
    })
}

pub fn draw_note_renaming_state(
    NoteRenamingStateData {
        note_viewing_data,
        new_name,
        valid,
    }: &NoteRenamingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_note_viewing_state(note_viewing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "New note name", new_name, *valid, main_rect);
}
