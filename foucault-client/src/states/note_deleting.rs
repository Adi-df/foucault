use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_yes_no_prompt,
    states::{
        note_viewing::{draw_note_viewing_state, NoteViewingStateData},
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteDeletingStateData {
    pub note_viewing_data: NoteViewingStateData,
    pub delete: bool,
}

impl NoteDeletingStateData {
    pub fn empty(note_viewing_data: NoteViewingStateData) -> Self {
        NoteDeletingStateData {
            note_viewing_data,
            delete: false,
        }
    }
}

pub async fn run_note_deleting_state(
    NoteDeletingStateData {
        note_viewing_data,
        delete,
    }: NoteDeletingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the deletion of note {}.",
                note_viewing_data.note.name()
            );
            State::NoteViewing(NoteViewingStateData::new(note_viewing_data.note, notebook).await?)
        }
        KeyCode::Tab => State::NoteDeleting(NoteDeletingStateData {
            note_viewing_data,
            delete: !delete,
        }),
        KeyCode::Enter => {
            if delete {
                info!("Delete note {}.", note_viewing_data.note.name());
                note_viewing_data.note.delete(notebook).await?;
                State::Nothing
            } else {
                info!(
                    "Cancel the deletion of note {}.",
                    note_viewing_data.note.name()
                );
                State::NoteViewing(
                    NoteViewingStateData::new(note_viewing_data.note, notebook).await?,
                )
            }
        }
        _ => State::NoteDeleting(NoteDeletingStateData {
            note_viewing_data,
            delete,
        }),
    })
}

pub fn draw_note_deleting_state(
    NoteDeletingStateData {
        note_viewing_data,
        delete,
    }: &NoteDeletingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_note_viewing_state(note_viewing_data, notebook, frame, main_rect);
    draw_yes_no_prompt(frame, *delete, "Delete note ?", main_rect);
}
