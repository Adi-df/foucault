use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::widgets::Block;

use crate::helpers::{draw_text_prompt, DiscardResult};
use crate::notebook::Notebook;
use crate::states::note_tags_managing::{draw_note_tags_managing, NoteTagsManagingStateData};
use crate::states::{State, Terminal};
use crate::tag::Tag;

pub struct NoteTagAddingStateData {
    pub note_tags_managing_data: NoteTagsManagingStateData,
    pub tag_name: String,
    pub valid: bool,
}

impl NoteTagAddingStateData {
    pub fn empty(note_tags_managing_data: NoteTagsManagingStateData) -> Self {
        NoteTagAddingStateData {
            note_tags_managing_data,
            tag_name: String::new(),
            valid: false,
        }
    }
}

pub fn run_note_tag_adding_state(
    mut state_data: NoteTagAddingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteTagsManaging(state_data.note_tags_managing_data),
        KeyCode::Char(c) if !c.is_whitespace() => {
            state_data.tag_name.push(c);
            state_data.valid = Tag::tag_exists(state_data.tag_name.as_str(), notebook.db())?;

            State::NoteTagAdding(state_data)
        }
        KeyCode::Backspace => {
            state_data.tag_name.pop();
            state_data.valid = Tag::tag_exists(state_data.tag_name.as_str(), notebook.db())?;

            State::NoteTagAdding(state_data)
        }
        KeyCode::Enter => {
            if let Some(tag) = Tag::load_by_name(state_data.tag_name.as_str(), notebook.db())? {
                state_data
                    .note_tags_managing_data
                    .note
                    .add_tag(&tag, notebook.db())?;
                state_data.note_tags_managing_data.tags.push(tag);
                State::NoteTagsManaging(state_data.note_tags_managing_data)
            } else {
                state_data.valid = false;

                State::NoteTagAdding(state_data)
            }
        }
        _ => State::NoteTagAdding(state_data),
    })
}

pub fn draw_note_tag_adding_state_data(
    NoteTagAddingStateData {
        note_tags_managing_data,
        tag_name,
        valid,
    }: &NoteTagAddingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, note_tags_managing_data, main_rect);
            draw_text_prompt(frame, "Tag name", tag_name.as_str(), *valid, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
