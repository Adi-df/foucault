use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult, TryFromDatabase};
use crate::note::Note;
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};

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

pub fn run_note_creating_state(
    NoteCreatingStateData { mut name, valid }: NoteCreatingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Enter if !name.is_empty() => {
            if Note::note_exists(name.as_str(), notebook.db())? {
                State::NoteCreating(NoteCreatingStateData { name, valid: false })
            } else {
                info!("Create note : {}.", name.as_str());

                let new_note = Note::new(name.clone(), String::new(), notebook.db())?;

                State::NoteViewing(NoteViewingStateData::try_from_database(
                    new_note,
                    notebook.db(),
                )?)
            }
        }
        KeyCode::Esc => {
            info!("Cancel note creation.");
            State::Nothing
        }
        KeyCode::Backspace => {
            name.pop();
            State::NoteCreating(NoteCreatingStateData {
                valid: !Note::note_exists(name.as_str(), notebook.db())?,
                name,
            })
        }
        KeyCode::Char(c) => {
            name.push(c);
            State::NoteCreating(NoteCreatingStateData {
                valid: !Note::note_exists(name.as_str(), notebook.db())?,
                name,
            })
        }
        _ => State::NoteCreating(NoteCreatingStateData { name, valid }),
    })
}

pub fn draw_note_creating_state(
    NoteCreatingStateData { name, valid }: &NoteCreatingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_text_prompt(frame, "Note name", name, *valid, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
