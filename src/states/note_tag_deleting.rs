use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_yes_no_prompt, TryFromDatabase};
use crate::notebook::Notebook;
use crate::states::note_tags_managing::{draw_note_tags_managing, NoteTagsManagingStateData};
use crate::states::{State, Terminal};

pub struct NoteTagDeletingStateData {
    pub note_tags_managing_data: NoteTagsManagingStateData,
    pub delete: bool,
}

impl NoteTagDeletingStateData {
    pub fn empty(note_tags_managing_data: NoteTagsManagingStateData) -> Self {
        NoteTagDeletingStateData {
            note_tags_managing_data,
            delete: false,
        }
    }
}

pub fn run_note_tag_deleting_state(
    NoteTagDeletingStateData {
        mut note_tags_managing_data,
        delete,
    }: NoteTagDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteTagsManaging(note_tags_managing_data),
        KeyCode::Enter => {
            if delete {
                let tag = note_tags_managing_data
                    .tags
                    .swap_remove(note_tags_managing_data.selected);
                note_tags_managing_data
                    .note
                    .remove_tag(tag.id, notebook.db())?;

                State::NoteTagsManaging(NoteTagsManagingStateData::try_from_database(
                    note_tags_managing_data.note,
                    notebook.db(),
                )?)
            } else {
                State::NoteTagsManaging(note_tags_managing_data)
            }
        }
        KeyCode::Tab => State::NoteTagDeleting(NoteTagDeletingStateData {
            note_tags_managing_data,
            delete: !delete,
        }),
        _ => State::NoteTagDeleting(NoteTagDeletingStateData {
            note_tags_managing_data,
            delete,
        }),
    })
}

pub fn draw_note_tag_deleting_state_data(
    NoteTagDeletingStateData {
        note_tags_managing_data,
        delete,
    }: &NoteTagDeletingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, note_tags_managing_data, main_rect);
            draw_yes_no_prompt(frame, *delete, "Remove tag ?", main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map_err(anyhow::Error::from)
        .map(|_| ())
}
