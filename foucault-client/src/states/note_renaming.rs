use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::{draw_text_prompt, EditableText},
    note::Note,
    states::{
        note_viewing::{draw_note_viewing_state, NoteViewingStateData},
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteRenamingStateData {
    note_viewing_data: NoteViewingStateData,
    new_name: EditableText,
    valid: bool,
}

impl NoteRenamingStateData {
    pub fn empty(note_viewing_data: NoteViewingStateData) -> Self {
        NoteRenamingStateData {
            note_viewing_data,
            new_name: EditableText::new(String::new()),
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
            if Note::validate_name(&state_data.new_name, notebook).await? {
                info!(
                    "Rename note {} to {}.",
                    state_data.note_viewing_data.note.name(),
                    &*state_data.new_name
                );
                state_data
                    .note_viewing_data
                    .note
                    .rename(state_data.new_name.consume(), notebook)
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
            state_data.new_name.remove_char();
            state_data.valid = Note::validate_name(&state_data.new_name, notebook).await?;
            State::NoteRenaming(state_data)
        }
        KeyCode::Delete => {
            state_data.new_name.del_char();
            state_data.valid = Note::validate_name(&state_data.new_name, notebook).await?;
            State::NoteRenaming(state_data)
        }
        KeyCode::Left => {
            state_data.new_name.move_left();
            State::NoteRenaming(state_data)
        }
        KeyCode::Right => {
            state_data.new_name.move_right();
            State::NoteRenaming(state_data)
        }
        KeyCode::Char(c) => {
            state_data.new_name.insert_char(c);
            state_data.valid = Note::validate_name(&state_data.new_name, notebook).await?;
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
