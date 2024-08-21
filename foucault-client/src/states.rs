mod error;
mod note_creating;
mod note_deleting;
mod note_renaming;
mod note_tag_adding;
mod note_tag_deleting;
mod note_tags_managing;
mod note_viewing;
mod notes_managing;
mod nothing;
mod tag_creating;
mod tag_deleting;
mod tag_notes_listing;
mod tags_managing;

use anyhow::Result;

use crossterm::event::KeyEvent;
use error::{draw_error_state, run_error_state, ErrorStateData};
use ratatui::{layout::Rect, Frame};

use crate::{
    states::{
        note_creating::{draw_note_creating_state, run_note_creating_state, NoteCreatingStateData},
        note_deleting::{draw_note_deleting_state, run_note_deleting_state, NoteDeletingStateData},
        note_renaming::{draw_note_renaming_state, run_note_renaming_state, NoteRenamingStateData},
        note_tag_adding::{
            draw_note_tag_adding_state_data, run_note_tag_adding_state, NoteTagAddingStateData,
        },
        note_tag_deleting::{
            draw_note_tag_deleting_state_data, run_note_tag_deleting_state,
            NoteTagDeletingStateData,
        },
        note_tags_managing::{
            draw_note_tags_managing_state, run_note_tags_managing_state, NoteTagsManagingStateData,
        },
        note_viewing::{draw_note_viewing_state, run_note_viewing_state, NoteViewingStateData},
        notes_managing::{
            draw_note_managing_state, run_note_managing_state, NotesManagingStateData,
        },
        nothing::{draw_nothing_state, run_nothing_state},
        tag_creating::{draw_tag_creating_state, run_tag_creating_state, TagsCreatingStateData},
        tag_deleting::{draw_tag_deleting_state, run_tag_deleting_state, TagsDeletingStateData},
        tag_notes_listing::{
            draw_tag_notes_listing_state, run_tag_notes_listing_state, TagNotesListingStateData,
        },
        tags_managing::{draw_tags_managing_state, run_tags_managing_state, TagsManagingStateData},
    },
    NotebookAPI,
};

pub enum State {
    Nothing,
    Exit,
    Error(ErrorStateData),
    NotesManaging(NotesManagingStateData),
    NoteViewing(NoteViewingStateData),
    NoteCreating(NoteCreatingStateData),
    NoteDeleting(NoteDeletingStateData),
    NoteRenaming(NoteRenamingStateData),
    NoteTagsManaging(NoteTagsManagingStateData),
    NoteTagDeleting(NoteTagDeletingStateData),
    NoteTagAdding(NoteTagAddingStateData),
    TagsManaging(TagsManagingStateData),
    TagCreating(TagsCreatingStateData),
    TagDeleting(TagsDeletingStateData),
    TagNotesListing(TagNotesListingStateData),
}

impl State {
    pub async fn run(
        self,
        key_event: KeyEvent,
        notebook: &NotebookAPI,
        force_redraw: &mut bool,
    ) -> Result<Self> {
        match self {
            State::Nothing => run_nothing_state(key_event, notebook).await,
            State::Error(data) => run_error_state(data, key_event).await,
            State::NotesManaging(data) => run_note_managing_state(data, key_event, notebook).await,
            State::NoteCreating(data) => run_note_creating_state(data, key_event, notebook).await,
            State::NoteViewing(data) => {
                run_note_viewing_state(data, key_event, notebook, force_redraw).await
            }
            State::NoteDeleting(data) => run_note_deleting_state(data, key_event, notebook).await,
            State::NoteRenaming(data) => run_note_renaming_state(data, key_event, notebook).await,
            State::NoteTagsManaging(data) => {
                run_note_tags_managing_state(data, key_event, notebook).await
            }
            State::NoteTagAdding(data) => {
                run_note_tag_adding_state(data, key_event, notebook).await
            }
            State::NoteTagDeleting(data) => {
                run_note_tag_deleting_state(data, key_event, notebook).await
            }
            State::TagsManaging(data) => run_tags_managing_state(data, key_event, notebook).await,
            State::TagCreating(data) => run_tag_creating_state(data, key_event, notebook).await,
            State::TagDeleting(data) => run_tag_deleting_state(data, key_event, notebook).await,
            State::TagNotesListing(data) => {
                run_tag_notes_listing_state(data, key_event, notebook).await
            }
            State::Exit => unreachable!(),
        }
    }

    pub fn draw(&self, notebook: &NotebookAPI, frame: &mut Frame, main_rect: Rect) {
        match self {
            State::Nothing => draw_nothing_state(notebook, frame, main_rect),
            State::Error(data) => draw_error_state(notebook, data, frame, main_rect),
            State::NotesManaging(data) => draw_note_managing_state(data, frame, main_rect),
            State::NoteCreating(data) => draw_note_creating_state(data, frame, main_rect),
            State::NoteViewing(data) => draw_note_viewing_state(data, frame, main_rect),
            State::NoteDeleting(data) => draw_note_deleting_state(data, frame, main_rect),
            State::NoteRenaming(data) => draw_note_renaming_state(data, frame, main_rect),
            State::NoteTagsManaging(data) => draw_note_tags_managing_state(data, frame, main_rect),
            State::NoteTagAdding(data) => draw_note_tag_adding_state_data(data, frame, main_rect),
            State::NoteTagDeleting(data) => {
                draw_note_tag_deleting_state_data(data, frame, main_rect)
            }
            State::TagsManaging(data) => draw_tags_managing_state(data, frame, main_rect),
            State::TagCreating(data) => draw_tag_creating_state(data, frame, main_rect),
            State::TagDeleting(data) => draw_tag_deleting_state(data, frame, main_rect),
            State::TagNotesListing(data) => draw_tag_notes_listing_state(data, frame, main_rect),
            State::Exit => unreachable!(),
        }
    }
}
