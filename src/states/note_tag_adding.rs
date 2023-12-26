use std::io::Stdout;

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::widgets::Block;
use ratatui::Terminal;

use crate::helpers::draw_text_prompt;
use crate::states::State;
use crate::tags::Tag;
use crate::{notebook::Notebook, states::note_tags_managing::NoteTagsManagingStateData};

use crate::states::note_tags_managing::draw_note_tags_managing;

#[derive(Debug)]
pub struct NoteTagAddingStateData {
    pub tags_managing: NoteTagsManagingStateData,
    pub tag_name: String,
}

pub fn run_note_tag_adding_state(
    NoteTagAddingStateData {
        tags_managing:
            NoteTagsManagingStateData {
                selected,
                mut tags,
                note,
            },
        mut tag_name,
    }: NoteTagAddingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteTagsManaging(NoteTagsManagingStateData {
            selected,
            tags,
            note,
        }),
        KeyCode::Char(c) if !c.is_whitespace() => {
            tag_name.push(c);
            State::NoteTagAdding(NoteTagAddingStateData {
                tags_managing: NoteTagsManagingStateData {
                    selected,
                    tags,
                    note,
                },
                tag_name,
            })
        }
        KeyCode::Backspace => {
            tag_name.pop();
            State::NoteTagAdding(NoteTagAddingStateData {
                tags_managing: NoteTagsManagingStateData {
                    selected,
                    tags,
                    note,
                },
                tag_name,
            })
        }
        KeyCode::Enter => {
            if let Some(tag) = Tag::load_by_name(tag_name.as_str(), notebook.db())? {
                note.add_tag(&tag, notebook.db())?;
                tags.push(tag);
                State::NoteTagsManaging(NoteTagsManagingStateData {
                    selected,
                    tags,
                    note,
                })
            } else {
                State::NoteTagAdding(NoteTagAddingStateData {
                    tags_managing: NoteTagsManagingStateData {
                        selected,
                        tags,
                        note,
                    },
                    tag_name,
                })
            }
        }
        _ => State::NoteTagAdding(NoteTagAddingStateData {
            tags_managing: NoteTagsManagingStateData {
                selected,
                tags,
                note,
            },
            tag_name,
        }),
    })
}

pub fn draw_note_tag_adding_state_data(
    NoteTagAddingStateData {
        tags_managing,
        tag_name,
    }: &NoteTagAddingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    notebook: &Notebook,
    main_frame: Block,
) -> Result<()> {
    let exists = Tag::tag_exists(tag_name, notebook.db())?;

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, tags_managing, main_rect);
            draw_text_prompt(frame, "Tag name", tag_name.as_str(), exists, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map_err(anyhow::Error::from)
        .map(|_| ())
}
