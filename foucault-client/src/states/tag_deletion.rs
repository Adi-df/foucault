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
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct TagsDeletionStateData {
    tags_managing_data: TagsManagingStateData,
    delete: bool,
}

impl TagsDeletionStateData {
    pub fn empty(tags_managing_data: TagsManagingStateData) -> Self {
        TagsDeletionStateData {
            tags_managing_data,
            delete: false,
        }
    }
}

pub async fn run_tag_deletion_state(
    TagsDeletionStateData {
        tags_managing_data,
        delete,
    }: TagsDeletionStateData,
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

                Tag::delete(
                    tags_managing_data
                        .get_selected()
                        .expect("A tag to be selected")
                        .id(),
                    notebook,
                )
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
        KeyCode::Tab => State::TagDeletion(TagsDeletionStateData {
            tags_managing_data,
            delete: !delete,
        }),
        _ => State::TagDeletion(TagsDeletionStateData {
            tags_managing_data,
            delete,
        }),
    })
}

pub fn draw_tag_deletion_state(
    TagsDeletionStateData {
        tags_managing_data,
        delete,
    }: &TagsDeletionStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let selected_tag = tags_managing_data
        .get_selected()
        .expect("A tag to be selected");

    draw_tags_managing_state(tags_managing_data, notebook, frame, main_rect);

    draw_yes_no_prompt(
        frame,
        *delete,
        format!("Delete tag {} ?", selected_tag.name()).as_str(),
        main_rect,
    );
}
