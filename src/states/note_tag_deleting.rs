use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::draw_yes_no_prompt;
use crate::notebook::Notebook;
use crate::states::note_tags_managing::{draw_note_tags_managing, NoteTagsManagingStateData};
use crate::states::{State, Terminal};

#[derive(Debug)]
pub struct NoteTagDeletingStateData {
    pub tags_managing: NoteTagsManagingStateData,
    pub delete: bool,
}

pub fn run_note_tag_deleting_state(
    NoteTagDeletingStateData {
        tags_managing:
            NoteTagsManagingStateData {
                selected,
                mut tags,
                note,
            },
        delete,
    }: NoteTagDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteTagsManaging(NoteTagsManagingStateData {
            selected,
            tags,
            note,
        }),
        KeyCode::Enter => {
            if delete {
                let tag = tags.swap_remove(selected);
                note.remove_tag(tag.id, notebook.db())?;

                State::NoteTagsManaging(NoteTagsManagingStateData {
                    selected: 0,
                    tags,
                    note,
                })
            } else {
                State::NoteTagsManaging(NoteTagsManagingStateData {
                    selected,
                    tags,
                    note,
                })
            }
        }
        KeyCode::Tab => State::NoteTagDeleting(NoteTagDeletingStateData {
            tags_managing: NoteTagsManagingStateData {
                selected,
                tags,
                note,
            },
            delete: !delete,
        }),
        _ => State::NoteTagDeleting(NoteTagDeletingStateData {
            tags_managing: NoteTagsManagingStateData {
                selected,
                tags,
                note,
            },
            delete,
        }),
    })
}

pub fn draw_note_tag_deleting_state_data(
    NoteTagDeletingStateData {
        tags_managing,
        delete,
    }: &NoteTagDeletingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, tags_managing, main_rect);
            draw_yes_no_prompt(frame, *delete, "Remove tag ?", main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map_err(anyhow::Error::from)
        .map(|_| ())
}
