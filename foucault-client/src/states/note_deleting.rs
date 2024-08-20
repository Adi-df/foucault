use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Block;

use crate::{
    helpers::{draw_yes_no_prompt, DiscardResult},
    states::{
        note_viewing::{draw_viewed_note, NoteViewingStateData},
        State, Terminal,
    },
    NotebookAPI,
};

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
            info!("Cancel deleting note {}.", note_viewing_data.note.name());
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
                info!("Cancel deleting note {}.", note_viewing_data.note.name());
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
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, note_viewing_data, main_rect);

            draw_yes_no_prompt(frame, *delete, "Delete note ?", main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
