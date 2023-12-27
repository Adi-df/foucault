use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult};
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::{State, Terminal};

#[derive(Debug)]
pub struct NoteRenamingStateData {
    pub note_viewing_data: NoteViewingStateData,
    pub new_name: String,
}

impl NoteRenamingStateData {
    pub fn empty(note_viewing_data: NoteViewingStateData) -> Self {
        NoteRenamingStateData {
            note_viewing_data,
            new_name: String::new(),
        }
    }
}

pub fn run_note_renaming_state(
    NoteRenamingStateData {
        mut note_viewing_data,
        mut new_name,
    }: NoteRenamingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel renaming");
            State::NoteViewing(note_viewing_data)
        }
        KeyCode::Enter => {
            note_viewing_data.note_data.note.name = new_name;
            note_viewing_data.note_data.note.update(notebook.db())?;
            State::NoteViewing(note_viewing_data)
        }

        KeyCode::Backspace => {
            new_name.pop();
            State::NoteRenaming(NoteRenamingStateData {
                note_viewing_data,
                new_name,
            })
        }
        KeyCode::Char(c) => {
            new_name.push(c);
            State::NoteRenaming(NoteRenamingStateData {
                note_viewing_data,
                new_name,
            })
        }
        _ => State::NoteRenaming(NoteRenamingStateData {
            note_viewing_data,
            new_name,
        }),
    })
}

pub fn draw_note_renaming_state(
    NoteRenamingStateData {
        note_viewing_data: viewing_data,
        new_name,
    }: &NoteRenamingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, viewing_data, main_rect);
            draw_text_prompt(frame, "Rename note", new_name, true, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
