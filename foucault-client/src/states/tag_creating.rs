use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult};
use crate::states::tags_managing::{draw_tags_managing, TagsManagingStateData};
use crate::states::{State, Terminal};
use crate::tag::Tag;
use crate::NotebookAPI;

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
            if !Tag::validate_new_tag(state_data.name.as_str(), notebook).await? {
                State::TagCreating(TagsCreatingStateData {
                    valid: false,
                    ..state_data
                })
            } else {
                info!("Create tag {}.", state_data.name);
                Tag::new(state_data.name.as_str(), notebook).await?;
                State::TagsManaging(
                    TagsManagingStateData::from_pattern(
                        state_data.tags_managing_data.pattern,
                        notebook,
                    )
                    .await?,
                )
            }
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            state_data.valid = Tag::validate_new_tag(state_data.name.as_str(), notebook).await?;
            State::TagCreating(state_data)
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.name.push(c);
            state_data.valid = Tag::validate_new_tag(state_data.name.as_str(), notebook).await?;
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
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing_data, main_rect);
            draw_text_prompt(frame, "Tag name", name, *valid, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
