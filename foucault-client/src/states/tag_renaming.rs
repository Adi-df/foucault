use anyhow::Result;

use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::{draw_text_prompt, EditableText},
    states::{tags_managing::TagsManagingStateData, State},
    tag::Tag,
    NotebookAPI,
};

use super::tags_managing::draw_tags_managing_state;

#[derive(Clone)]
pub struct TagRenamingStateData {
    tags_managing_data: TagsManagingStateData,
    new_name: EditableText,
    valid: bool,
}

impl TagRenamingStateData {
    pub fn empty(tags_managing_data: TagsManagingStateData) -> Self {
        Self {
            tags_managing_data,
            new_name: EditableText::new(String::new()),
            valid: false,
        }
    }
}

pub async fn run_tag_renaming_state(
    mut state_data: TagRenamingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the renaming of tag {}",
                state_data
                    .tags_managing_data
                    .get_selected()
                    .expect("A tag to be selected")
                    .name()
            );

            State::TagsManaging(
                TagsManagingStateData::from_pattern(
                    state_data.tags_managing_data.pattern,
                    notebook,
                )
                .await?,
            )
        }
        KeyCode::Enter => {
            if Tag::validate_name(&state_data.new_name, notebook).await? {
                let tag = state_data
                    .tags_managing_data
                    .get_selected()
                    .expect("A tag to be selected.");

                info!("Rename tag {} to {}.", tag.name(), &*state_data.new_name);

                Tag::rename(tag.id(), state_data.new_name.consume(), notebook).await?;

                State::TagsManaging(
                    TagsManagingStateData::from_pattern(
                        state_data.tags_managing_data.pattern,
                        notebook,
                    )
                    .await?,
                )
            } else {
                state_data.valid = false;
                State::TagRenaming(state_data)
            }
        }
        KeyCode::Backspace => {
            state_data.new_name.remove_char();
            state_data.valid = Tag::validate_name(&state_data.new_name, notebook).await?;
            State::TagRenaming(state_data)
        }
        KeyCode::Delete => {
            state_data.new_name.del_char();
            state_data.valid = Tag::validate_name(&state_data.new_name, notebook).await?;
            State::TagRenaming(state_data)
        }
        KeyCode::Left => {
            state_data.new_name.move_left();
            State::TagRenaming(state_data)
        }
        KeyCode::Right => {
            state_data.new_name.move_right();
            State::TagRenaming(state_data)
        }
        KeyCode::Char(c) => {
            state_data.new_name.insert_char(c);
            state_data.valid = Tag::validate_name(&state_data.new_name, notebook).await?;
            State::TagRenaming(state_data)
        }
        _ => State::TagRenaming(state_data),
    })
}

pub fn draw_tag_renaming_state(
    TagRenamingStateData {
        tags_managing_data,
        new_name,
        valid,
    }: &TagRenamingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_tags_managing_state(tags_managing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "New tag name", new_name, *valid, main_rect);
}
