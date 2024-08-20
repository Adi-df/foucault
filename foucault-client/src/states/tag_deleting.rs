use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Block;

use crate::{
    helpers::{draw_yes_no_prompt, DiscardResult},
    states::{
        tags_managing::{draw_tags_managing, TagsManagingStateData},
        State, Terminal,
    },
    NotebookAPI,
};

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
                "Cancel deleting of tag {}.",
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
                    "Cancel deleting of tag {}.",
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
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    let selected_tag = &tags_managing_data.tags[tags_managing_data.selected];

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing_data, main_rect);

            draw_yes_no_prompt(
                frame,
                *delete,
                format!("Delete tag {} ?", selected_tag.name()).as_str(),
                main_rect,
            );

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
