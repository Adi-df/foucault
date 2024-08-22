use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_text_prompt,
    states::{
        tags_managing::{draw_tags_managing_state, TagsManagingStateData},
        State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct TagsCreatingStateData {
    pub tags_managing_data: TagsManagingStateData,
    pub name: String,
    pub valid: bool,
}

impl TagsCreatingStateData {
    pub fn empty(tags_managing_data: TagsManagingStateData) -> Self {
        TagsCreatingStateData {
            tags_managing_data,
            name: String::new(),
            valid: false,
        }
    }
}

pub async fn run_tag_creating_state(
    mut state_data: TagsCreatingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Cancel tag creation.");
            State::TagsManaging(
                TagsManagingStateData::from_pattern(
                    state_data.tags_managing_data.pattern,
                    notebook,
                )
                .await?,
            )
        }
        KeyCode::Enter => {
            if Tag::validate_name(state_data.name.as_str(), notebook).await? {
                info!("Create tag {}.", state_data.name);
                Tag::new(state_data.name.as_str(), notebook).await?;
                State::TagsManaging(
                    TagsManagingStateData::from_pattern(
                        state_data.tags_managing_data.pattern,
                        notebook,
                    )
                    .await?,
                )
            } else {
                State::TagCreating(TagsCreatingStateData {
                    valid: false,
                    ..state_data
                })
            }
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            state_data.valid = Tag::validate_name(state_data.name.as_str(), notebook).await?;
            State::TagCreating(state_data)
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.name.push(c);
            state_data.valid = Tag::validate_name(state_data.name.as_str(), notebook).await?;
            State::TagCreating(state_data)
        }
        _ => State::TagCreating(state_data),
    })
}

pub fn draw_tag_creating_state(
    TagsCreatingStateData {
        tags_managing_data,
        name,
        valid,
    }: &TagsCreatingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_tags_managing_state(tags_managing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "Tag name", name, *valid, main_rect);
}
