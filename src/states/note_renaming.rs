use std::io::Stdout;

use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::widgets::Block;
use ratatui::Terminal;

use crate::helpers::draw_text_prompt;
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::State;

#[derive(Debug)]
pub struct NoteRenamingStateData {
    pub viewing_data: NoteViewingStateData,
    pub new_name: String,
}

pub fn run_note_renaming_state(
    NoteRenamingStateData {
        viewing_data:
            NoteViewingStateData {
                mut note_data,
                scroll,
            },
        mut new_name,
    }: NoteRenamingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel renaming");
            State::NoteViewing(NoteViewingStateData { note_data, scroll })
        }
        KeyCode::Enter => {
            note_data.note.name = new_name;
            note_data.note.update(notebook.db())?;
            State::NoteViewing(NoteViewingStateData { note_data, scroll })
        }

        KeyCode::Backspace => {
            new_name.pop();
            State::NoteRenaming(NoteRenamingStateData {
                viewing_data: NoteViewingStateData { note_data, scroll },
                new_name,
            })
        }
        KeyCode::Char(c) => {
            new_name.push(c);
            State::NoteRenaming(NoteRenamingStateData {
                viewing_data: NoteViewingStateData { note_data, scroll },
                new_name,
            })
        }
        _ => State::NoteRenaming(NoteRenamingStateData {
            viewing_data: NoteViewingStateData { note_data, scroll },
            new_name,
        }),
    })
}

pub fn draw_note_renaming_state(
    NoteRenamingStateData {
        viewing_data,
        new_name,
    }: &NoteRenamingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, viewing_data, main_rect);
            draw_text_prompt(frame, "Rename note", new_name, true, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
