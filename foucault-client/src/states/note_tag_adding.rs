use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    states::{
        note_tags_managing::{draw_note_tags_managing_state, NoteTagsManagingStateData},
        State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteTagAddingStateData {
    pub note_tags_managing_data: NoteTagsManagingStateData,
    pub tag_name: String,
    pub valid: bool,
}

impl NoteTagAddingStateData {
    pub fn empty(note_tags_managing_data: NoteTagsManagingStateData) -> Self {
        NoteTagAddingStateData {
            note_tags_managing_data,
            tag_name: String::new(),
            valid: false,
        }
    }
}

pub async fn run_note_tag_adding_state(
    mut state_data: NoteTagAddingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the tag addition to note {}.",
                state_data.note_tags_managing_data.note.name()
            );

            State::NoteTagsManaging(
                NoteTagsManagingStateData::new(state_data.note_tags_managing_data.note, notebook)
                    .await?,
            )
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.tag_name.push(c);
            state_data.valid = {
                if let Some(tag) = Tag::load_by_name(state_data.tag_name.as_str(), notebook).await?
                {
                    state_data
                        .note_tags_managing_data
                        .note
                        .validate_tag(tag.id(), notebook)
                        .await?
                } else {
                    false
                }
            };

            State::NoteTagAdding(state_data)
        }
        KeyCode::Backspace => {
            state_data.tag_name.pop();
            state_data.valid = if let Some(tag) =
                Tag::load_by_name(state_data.tag_name.as_str(), notebook).await?
            {
                state_data
                    .note_tags_managing_data
                    .note
                    .validate_tag(tag.id(), notebook)
                    .await?
            } else {
                false
            };

            State::NoteTagAdding(state_data)
        }
        KeyCode::Enter => match Tag::load_by_name(state_data.tag_name.as_str(), notebook).await? {
            Some(tag)
                if state_data
                    .note_tags_managing_data
                    .note
                    .validate_tag(tag.id(), notebook)
                    .await? =>
            {
                info!(
                    "Add tag {} to note {}.",
                    tag.name(),
                    state_data.note_tags_managing_data.note.name()
                );
                state_data
                    .note_tags_managing_data
                    .note
                    .add_tag(tag.id(), notebook)
                    .await?;

                State::NoteTagsManaging(
                    NoteTagsManagingStateData::new(
                        state_data.note_tags_managing_data.note,
                        notebook,
                    )
                    .await?,
                )
            }
            _ => {
                state_data.valid = false;

                State::NoteTagAdding(state_data)
            }
        },
        _ => State::NoteTagAdding(state_data),
    })
}

pub fn draw_note_tag_adding_state(
    NoteTagAddingStateData {
        note_tags_managing_data,
        tag_name,
        valid,
    }: &NoteTagAddingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_note_tags_managing_state(note_tags_managing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "Tag name", tag_name.as_str(), *valid, main_rect);
}
