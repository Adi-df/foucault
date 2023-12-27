use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_yes_no_prompt, DiscardResult};
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::{State, Terminal};

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

pub fn run_note_deleting_state(
    NoteDeletingStateData {
        note_viewing_data,
        delete,
    }: NoteDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel deleting");
            State::NoteViewing(note_viewing_data)
        }
        KeyCode::Tab => State::NoteDeleting(NoteDeletingStateData {
            note_viewing_data,
            delete: !delete,
        }),
        KeyCode::Enter => {
            if delete {
                note_viewing_data.note_data.note.delete(notebook.db())?;
                State::Nothing
            } else {
                State::NoteViewing(note_viewing_data)
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
