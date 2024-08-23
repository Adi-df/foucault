use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_yes_no_prompt,
    states::{
        note_tags_managing::{draw_note_tags_managing_state, NoteTagsManagingStateData},
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteTagDeletingStateData {
    note_tags_managing_data: NoteTagsManagingStateData,
    delete: bool,
}

impl NoteTagDeletingStateData {
    pub fn empty(note_tags_managing_data: NoteTagsManagingStateData) -> Self {
        NoteTagDeletingStateData {
            note_tags_managing_data,
            delete: false,
        }
    }
}

pub async fn run_note_tag_deleting_state(
    NoteTagDeletingStateData {
        mut note_tags_managing_data,
        delete,
    }: NoteTagDeletingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the removal of tag {} from note {}",
                note_tags_managing_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name(),
                note_tags_managing_data.note.name()
            );
            State::NoteTagsManaging(
                NoteTagsManagingStateData::new(note_tags_managing_data.note, notebook).await?,
            )
        }
        KeyCode::Enter => {
            if delete {
                let tag = note_tags_managing_data
                    .get_selected()
                    .expect("A tag to be selected");

                info!(
                    "Remove tag {} from note {}.",
                    tag.name(),
                    note_tags_managing_data.note.name()
                );

                note_tags_managing_data
                    .note
                    .remove_tag(tag.id(), notebook)
                    .await?;

                State::NoteTagsManaging(
                    NoteTagsManagingStateData::new(note_tags_managing_data.note, notebook).await?,
                )
            } else {
                State::NoteTagsManaging(
                    NoteTagsManagingStateData::new(note_tags_managing_data.note, notebook).await?,
                )
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

pub fn draw_note_tag_deleting_state(
    NoteTagDeletingStateData {
        note_tags_managing_data,
        delete,
    }: &NoteTagDeletingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_note_tags_managing_state(note_tags_managing_data, notebook, frame, main_rect);
    draw_yes_no_prompt(frame, *delete, "Remove tag ?", main_rect);
}
