use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::draw_text_prompt;
use crate::notebook::Notebook;
use crate::states::tags_managing::{draw_tags_managing, TagsManagingStateData};
use crate::states::{State, Terminal};
use crate::tag::Tag;

#[derive(Debug)]
pub struct TagsCreatingStateData {
    pub tags_search: TagsManagingStateData,
    pub name: String,
    pub valid: bool,
}

impl TagsCreatingStateData {
    pub fn empty(tags_search: TagsManagingStateData) -> Self {
        TagsCreatingStateData {
            tags_search,
            name: String::new(),
            valid: false,
        }
    }
}

pub fn run_tag_creating_state(
    TagsCreatingStateData {
        tags_search,
        mut name,
        valid,
    }: TagsCreatingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::TagsManaging(tags_search),
        KeyCode::Enter if !name.is_empty() => {
            if Tag::tag_exists(name.as_str(), notebook.db())? {
                State::TagCreating(TagsCreatingStateData {
                    tags_search,
                    name,
                    valid: false,
                })
            } else {
                Tag::new(name.as_str(), notebook.db())?;
                State::TagsManaging(TagsManagingStateData::from_pattern(
                    tags_search.pattern,
                    notebook.db(),
                )?)
            }
        }
        KeyCode::Backspace => {
            name.pop();
            State::TagCreating(TagsCreatingStateData {
                tags_search,
                valid: Tag::tag_exists(name.as_str(), notebook.db())? && !name.is_empty(),
                name,
            })
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            name.push(c);
            State::TagCreating(TagsCreatingStateData {
                tags_search,
                valid: Tag::tag_exists(name.as_str(), notebook.db())? && !name.is_empty(),
                name,
            })
        }
        _ => State::TagCreating(TagsCreatingStateData {
            tags_search,
            name,
            valid,
        }),
    })
}

pub fn draw_tag_creating_state(
    TagsCreatingStateData {
        tags_search,
        name,
        valid: taken,
    }: &TagsCreatingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_search, main_rect);
            draw_text_prompt(frame, "Tag name", name, !taken, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
