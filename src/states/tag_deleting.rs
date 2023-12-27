use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::draw_yes_no_prompt;
use crate::notebook::Notebook;
use crate::states::tags_managing::{draw_tags_managing, TagsManagingStateData};
use crate::states::{State, Terminal};
use crate::tag::Tag;

#[derive(Debug)]
pub struct TagsDeletingStateData {
    pub tags_managing: TagsManagingStateData,
    pub delete: bool,
}

pub fn run_tag_deleting_state(
    TagsDeletingStateData {
        mut tags_managing,
        delete,
    }: TagsDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::TagsManaging(tags_managing),
        KeyCode::Enter => {
            if delete {
                tags_managing
                    .tags
                    .swap_remove(tags_managing.selected)
                    .delete(notebook.db())?;
            }
            State::TagsManaging(TagsManagingStateData::from_pattern(
                tags_managing.pattern,
                notebook.db(),
            )?)
        }
        KeyCode::Tab => State::TagDeleting(TagsDeletingStateData {
            tags_managing,
            delete: !delete,
        }),
        _ => State::TagDeleting(TagsDeletingStateData {
            tags_managing,
            delete,
        }),
    })
}

pub fn draw_tag_deleting_state(
    TagsDeletingStateData {
        tags_managing,
        delete,
    }: &TagsDeletingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    let Tag { name, .. } = &tags_managing.tags[tags_managing.selected];

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing, main_rect);

            draw_yes_no_prompt(
                frame,
                *delete,
                format!("Delete tag {name} ?").as_str(),
                main_rect,
            );

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
