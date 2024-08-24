use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    helpers::{draw_text_prompt, EdittableText},
    states::{
        note_tags_managing::{draw_note_tags_managing_state, NoteTagsManagingStateData},
        State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteTagAdditionStateData {
    note_tags_managing_data: NoteTagsManagingStateData,
    tag_name: EdittableText,
    valid: bool,
}

impl NoteTagAdditionStateData {
    pub fn empty(note_tags_managing_data: NoteTagsManagingStateData) -> Self {
        NoteTagAdditionStateData {
            note_tags_managing_data,
            tag_name: EdittableText::new(String::new()),
            valid: false,
        }
    }

    async fn validate(&mut self, notebook: &NotebookAPI) -> Result<()> {
        // TODO : Better error handling possible here
        self.valid =
            if let Some(tag) = Tag::load_by_name(self.tag_name.get_text(), notebook).await? {
                self.note_tags_managing_data
                    .note
                    .validate_tag(tag.id(), notebook)
                    .await?
            } else {
                false
            };
        Ok(())
    }
}

pub async fn run_note_tag_addition_state(
    mut state_data: NoteTagAdditionStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!(
                "Cancel the tag addition to note {}.",
                state_data.note_tags_managing_data.note.name()
            );

            State::NoteTagsManaging(
                NoteTagsManagingStateData::new(state_data.note_tags_managing_data.note, notebook)
                    .await?,
            )
        }
        KeyCode::Enter => {
            match Tag::load_by_name(state_data.tag_name.get_text(), notebook).await? {
                Some(tag)
                    if state_data
                        .note_tags_managing_data
                        .note
                        .validate_tag(tag.id(), notebook)
                        .await? =>
                {
                    info!(
                        "Add tag {} to note {}.",
                        tag.name(),
                        state_data.note_tags_managing_data.note.name()
                    );
                    state_data
                        .note_tags_managing_data
                        .note
                        .add_tag(tag.id(), notebook)
                        .await?;

                    State::NoteTagsManaging(
                        NoteTagsManagingStateData::new(
                            state_data.note_tags_managing_data.note,
                            notebook,
                        )
                        .await?,
                    )
                }
                _ => {
                    state_data.valid = false;

                    State::NoteTagAddition(state_data)
                }
            }
        }
        KeyCode::Backspace => {
            state_data.tag_name.remove_char();
            state_data.validate(notebook).await?;

            State::NoteTagAddition(state_data)
        }
        KeyCode::Delete => {
            state_data.tag_name.del_char();
            state_data.validate(notebook).await?;

            State::NoteTagAddition(state_data)
        }
        KeyCode::Left => {
            state_data.tag_name.move_left();

            State::NoteTagAddition(state_data)
        }
        KeyCode::Right => {
            state_data.tag_name.move_right();

            State::NoteTagAddition(state_data)
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.tag_name.insert_char(c);
            state_data.validate(notebook).await?;

            State::NoteTagAddition(state_data)
        }
        _ => State::NoteTagAddition(state_data),
    })
}

pub fn draw_note_tag_addition_state(
    NoteTagAdditionStateData {
        note_tags_managing_data,
        tag_name,
        valid,
    }: &NoteTagAdditionStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    draw_note_tags_managing_state(note_tags_managing_data, notebook, frame, main_rect);
    draw_text_prompt(frame, "Tag name", tag_name, *valid, main_rect);
}
