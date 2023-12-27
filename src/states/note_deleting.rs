use anyhow::Result;

use crossterm::event::KeyCode;
use log::info;
use ratatui::widgets::Block;

use crate::helpers::draw_yes_no_prompt;
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::{State, Terminal};

#[derive(Debug)]
pub struct NoteDeletingStateData {
    pub viewing_data: NoteViewingStateData,
    pub delete: bool,
}

pub fn run_note_deleting_state(
    NoteDeletingStateData {
        viewing_data: NoteViewingStateData { note_data, scroll },
        delete,
    }: NoteDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel deleting");
            State::NoteViewing(NoteViewingStateData { note_data, scroll })
        }
        KeyCode::Tab => State::NoteDeleting(NoteDeletingStateData {
            viewing_data: NoteViewingStateData { note_data, scroll },
            delete: !delete,
        }),
        KeyCode::Enter => {
            if delete {
                note_data.note.delete(notebook.db())?;
                State::Nothing
            } else {
                State::NoteViewing(NoteViewingStateData { note_data, scroll })
            }
        }
        _ => State::NoteDeleting(NoteDeletingStateData {
            viewing_data: NoteViewingStateData { note_data, scroll },
            delete,
        }),
    })
}

pub fn draw_note_deleting_state(
    NoteDeletingStateData {
        viewing_data,
        delete,
    }: &NoteDeletingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, viewing_data, main_rect);

            draw_yes_no_prompt(frame, *delete, "Delete note ?", main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
