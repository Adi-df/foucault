use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Block;

use crate::helpers::draw_yes_no_prompt;
use crate::states::note_tags_managing::{draw_note_tags_managing, NoteTagsManagingStateData};
use crate::states::{State, Terminal};
use crate::NotebookAPI;

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
                "Cancel delting tag {} from note {}",
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
                    .tags
                    .swap_remove(note_tags_managing_data.selected);

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
