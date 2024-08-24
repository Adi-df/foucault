use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::{draw_text_prompt, EdittableText},
    states::{
        tags_managing::{draw_tags_managing_state, TagsManagingStateData},
        State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct TagsCreationStateData {
    tags_managing_data: TagsManagingStateData,
    name: EdittableText,
    valid: bool,
}

impl TagsCreationStateData {
    pub fn empty(tags_managing_data: TagsManagingStateData) -> Self {
        TagsCreationStateData {
            tags_managing_data,
            name: EdittableText::new(String::new()),
            valid: false,
        }
    }
}

pub async fn run_tag_creation_state(
    mut state_data: TagsCreationStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Cancel the tag creation.");
            State::TagsManaging(
                TagsManagingStateData::from_pattern(
                    state_data.tags_managing_data.pattern,
                    notebook,
                )
                .await?,
            )
        }
        KeyCode::Enter => {
            if Tag::validate_name(state_data.name.get_text(), notebook).await? {
                info!("Create tag {}.", state_data.name.get_text());
                Tag::new(state_data.name.consume(), notebook).await?;
                State::TagsManaging(
                    TagsManagingStateData::from_pattern(
                        state_data.tags_managing_data.pattern,
                        notebook,
                    )
                    .await?,
                )
            } else {
                State::TagCreation(TagsCreationStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Backspace => {
            state_data.name.remove_char();
            state_data.valid = Tag::validate_name(state_data.name.get_text(), notebook).await?;
            State::TagCreation(state_data)
        }
        KeyCode::Delete => {
            state_data.name.del_char();
            state_data.valid = Tag::validate_name(state_data.name.get_text(), notebook).await?;
            State::TagCreation(state_data)
        }
        KeyCode::Left => {
            state_data.name.move_left();
            State::TagCreation(state_data)
        }
        KeyCode::Right => {
            state_data.name.move_right();
            State::TagCreation(state_data)
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.name.insert_char(c);
            state_data.valid = Tag::validate_name(state_data.name.get_text(), notebook).await?;
            State::TagCreation(state_data)
        }
        _ => State::TagCreation(state_data),
    })
}

pub fn draw_tag_creation_state(
    TagsCreationStateData {
        tags_managing_data,
        name,
        valid,
    }: &TagsCreationStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_tags_managing_state(tags_managing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "Tag name", name, *valid, main_rect);
}
