use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult};
use crate::note::Note;
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::{State, Terminal};

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
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel renaming note {}",
                state_data.note_viewing_data.note.name()
            );
            State::NoteViewing(
                NoteViewingStateData::new(state_data.note_viewing_data.note, notebook.db()).await?,
            )
        }
        KeyCode::Enter => {
            if Note::validate_new_name(state_data.new_name.as_str(), notebook.db())
                .await
                .is_err()
            {
                State::NoteRenaming(NoteRenamingStateData {
                    valid: false,
                    ..state_data
                })
            } else {
                info!(
                    "Renaming note {} to {}.",
                    state_data.note_viewing_data.note.name(),
                    state_data.new_name
                );
                state_data
                    .note_viewing_data
                    .note
                    .rename(state_data.new_name, notebook.db())
                    .await?;
                State::NoteViewing(
                    NoteViewingStateData::new(state_data.note_viewing_data.note, notebook.db())
                        .await?,
                )
            }
        }
        KeyCode::Backspace => {
            state_data.new_name.pop();
            state_data.valid = Note::validate_new_name(state_data.new_name.as_str(), notebook.db())
                .await
                .is_ok();
            State::NoteRenaming(state_data)
        }
        KeyCode::Char(c) => {
            state_data.new_name.push(c);
            state_data.valid = Note::validate_new_name(state_data.new_name.as_str(), notebook.db())
                .await
                .is_ok();
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
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, note_viewing_data, main_rect);
            draw_text_prompt(frame, "Rename note", new_name, *valid, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
