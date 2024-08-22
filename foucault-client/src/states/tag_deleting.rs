use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::draw_yes_no_prompt,
    states::{
        tags_managing::{draw_tags_managing_state, TagsManagingStateData},
        State,
    },
    NotebookAPI,
};

#[derive(Clone)]
pub struct TagsDeletingStateData {
    pub tags_managing_data: TagsManagingStateData,
    pub delete: bool,
}

impl TagsDeletingStateData {
    pub fn empty(tags_managing_data: TagsManagingStateData) -> Self {
        TagsDeletingStateData {
            tags_managing_data,
            delete: false,
        }
    }
}

pub async fn run_tag_deleting_state(
    TagsDeletingStateData {
        mut tags_managing_data,
        delete,
    }: TagsDeletingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the deletion of tag {}.",
                tags_managing_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name()
            );
            State::TagsManaging(
                TagsManagingStateData::from_pattern(tags_managing_data.pattern, notebook).await?,
            )
        }
        KeyCode::Enter => {
            if delete {
                info!(
                    "Delete tag {}.",
                    tags_managing_data
                        .get_selected()
                        .expect("A tag should be selected.")
                        .name()
                );

                tags_managing_data
                    .tags
                    .swap_remove(tags_managing_data.selected)
                    .delete(notebook)
                    .await?;
            } else {
                info!(
                    "Cancel the deletion of tag {}.",
                    tags_managing_data
                        .get_selected()
                        .expect("A tag should be selected.")
                        .name()
                );
            }
            State::TagsManaging(
                TagsManagingStateData::from_pattern(tags_managing_data.pattern, notebook).await?,
            )
        }
        KeyCode::Tab => State::TagDeleting(TagsDeletingStateData {
            tags_managing_data,
            delete: !delete,
        }),
        _ => State::TagDeleting(TagsDeletingStateData {
            tags_managing_data,
            delete,
        }),
    })
}

pub fn draw_tag_deleting_state(
    TagsDeletingStateData {
        tags_managing_data,
        delete,
    }: &TagsDeletingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let selected_tag = &tags_managing_data.tags[tags_managing_data.selected];

    draw_tags_managing_state(tags_managing_data, notebook, frame, main_rect);

    draw_yes_no_prompt(
        frame,
        *delete,
        format!("Delete tag {} ?", selected_tag.name()).as_str(),
        main_rect,
    );
}
