use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_yes_no_prompt,
    note::Note,
    states::{
        note_viewing::{draw_note_viewing_state, NoteViewingStateData},
        notes_managing::{draw_note_managing_state, NotesManagingStateData},
        State,
    },
    NotebookAPI,
};

use foucault_core::note_repr::NoteError;

#[derive(Clone)]
enum PrecidingState {
    NoteViewingState(NoteViewingStateData),
    NotesManagingState(NotesManagingStateData),
}

#[derive(Clone)]
pub struct NoteDeletingStateData {
    preciding_state: PrecidingState,
    note_name: String,
    note_id: i64,
    delete: bool,
}

impl NoteDeletingStateData {
    pub fn from_note_viewing(state: NoteViewingStateData) -> Self {
        NoteDeletingStateData {
            note_name: state.note.name().to_string(),
            note_id: state.note.id(),
            delete: false,
            preciding_state: PrecidingState::NoteViewingState(state),
        }
    }
    pub fn from_notes_managing(
        note_name: String,
        note_id: i64,
        state: NotesManagingStateData,
    ) -> Self {
        NoteDeletingStateData {
            note_name,
            note_id,
            delete: false,
            preciding_state: PrecidingState::NotesManagingState(state),
        }
    }
}

pub async fn run_note_deleting_state(
    state_data: NoteDeletingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Cancel the deletion of note {}.", &state_data.note_name);
            match state_data.preciding_state {
                PrecidingState::NoteViewingState(_) => State::NoteViewing(
                    NoteViewingStateData::new(
                        Note::load_by_id(state_data.note_id, notebook)
                            .await?
                            .ok_or(NoteError::DoesNotExist)?,
                        notebook,
                    )
                    .await?,
                ),
                PrecidingState::NotesManagingState(state) => State::NotesManaging(
                    NotesManagingStateData::from_pattern(state.pattern, notebook).await?,
                ),
            }
        }
        KeyCode::Tab => State::NoteDeleting(NoteDeletingStateData {
            delete: !state_data.delete,
            ..state_data
        }),
        KeyCode::Enter => {
            if state_data.delete {
                info!("Delete note {}.", &state_data.note_name);
                Note::delete(state_data.note_id, notebook).await?;
                match state_data.preciding_state {
                    PrecidingState::NoteViewingState(_) => State::Nothing,
                    PrecidingState::NotesManagingState(state) => State::NotesManaging(
                        NotesManagingStateData::from_pattern(state.pattern, notebook).await?,
                    ),
                }
            } else {
                info!("Cancel the deletion of note {}.", &state_data.note_name);
                match state_data.preciding_state {
                    PrecidingState::NoteViewingState(_) => State::NoteViewing(
                        NoteViewingStateData::new(
                            Note::load_by_id(state_data.note_id, notebook)
                                .await?
                                .ok_or(NoteError::DoesNotExist)?,
                            notebook,
                        )
                        .await?,
                    ),
                    PrecidingState::NotesManagingState(state) => State::NotesManaging(
                        NotesManagingStateData::from_pattern(state.pattern, notebook).await?,
                    ),
                }
            }
        }
        _ => State::NoteDeleting(state_data),
    })
}

pub fn draw_note_deleting_state(
    NoteDeletingStateData {
        preciding_state,
        delete,
        ..
    }: &NoteDeletingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    match preciding_state {
        PrecidingState::NoteViewingState(state) => {
            draw_note_viewing_state(state, notebook, frame, main_rect);
        }
        PrecidingState::NotesManagingState(state) => {
            draw_note_managing_state(state, notebook, frame, main_rect);
        }
    }
    draw_yes_no_prompt(frame, *delete, "Delete note ?", main_rect);
}
