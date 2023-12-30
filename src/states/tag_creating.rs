use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult};
use crate::notebook::Notebook;
use crate::states::tags_managing::{draw_tags_managing, TagsManagingStateData};
use crate::states::{State, Terminal};
use crate::tag::Tag;

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

pub fn run_tag_creating_state(
    mut state_data: TagsCreatingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel tag creation.");
            State::TagsManaging(state_data.tags_managing_data)
        }
        KeyCode::Enter if !state_data.name.is_empty() => {
            if Tag::tag_exists(state_data.name.as_str(), notebook.db())? {
                State::TagCreating(TagsCreatingStateData {
                    valid: false,
                    ..state_data
                })
            } else {
                info!("Create tag {}.", state_data.name);
                Tag::new(state_data.name.as_str(), notebook.db())?;
                State::TagsManaging(TagsManagingStateData::from_pattern(
                    state_data.tags_managing_data.pattern,
                    notebook.db(),
                )?)
            }
        }
        KeyCode::Backspace => {
            state_data.name.pop();
            state_data.valid = Tag::tag_exists(state_data.name.as_str(), notebook.db())?
                && !state_data.name.is_empty();
            State::TagCreating(state_data)
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.name.push(c);
            state_data.valid = Tag::tag_exists(state_data.name.as_str(), notebook.db())?;
            State::TagCreating(state_data)
        }
        _ => State::TagCreating(state_data),
    })
}

pub fn draw_tag_creating_state(
    TagsCreatingStateData {
        tags_managing_data,
        name,
        valid: taken,
    }: &TagsCreatingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing_data, main_rect);
            draw_text_prompt(frame, "Tag name", name, !taken, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
