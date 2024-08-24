pub mod error;
mod note_creation;
mod note_deletion;
mod note_renaming;
mod note_tag_addition;
mod note_tag_deletion;
mod note_tags_managing;
mod note_viewing;
mod notes_managing;
mod nothing;
mod tag_creation;
mod tag_deletion;
mod tag_notes_listing;
mod tags_managing;

use anyhow::Result;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::{
    states::{
        error::{draw_error_state, run_error_state, ErrorStateData},
        note_creation::{draw_note_creation_state, run_note_creation_state, NoteCreationStateData},
        note_deletion::{draw_note_deletion_state, run_note_deletion_state, NoteDeletionStateData},
        note_renaming::{draw_note_renaming_state, run_note_renaming_state, NoteRenamingStateData},
        note_tag_addition::{
            draw_note_tag_addition_state, run_note_tag_addition_state, NoteTagAdditionStateData,
        },
        note_tag_deletion::{
            draw_note_tag_deletion_state, run_note_tag_deletion_state, NoteTagDeletionStateData,
        },
        note_tags_managing::{
            draw_note_tags_managing_state, run_note_tags_managing_state, NoteTagsManagingStateData,
        },
        note_viewing::{draw_note_viewing_state, run_note_viewing_state, NoteViewingStateData},
        notes_managing::{
            draw_notes_managing_state, run_note_managing_state, NotesManagingStateData,
        },
        nothing::{draw_nothing_state, run_nothing_state},
        tag_creation::{draw_tag_creation_state, run_tag_creation_state, TagsCreationStateData},
        tag_deletion::{draw_tag_deletion_state, run_tag_deletion_state, TagsDeletionStateData},
        tag_notes_listing::{
            draw_tag_notes_listing_state, run_tag_notes_listing_state, TagNotesListingStateData,
        },
        tags_managing::{draw_tags_managing_state, run_tags_managing_state, TagsManagingStateData},
    },
    NotebookAPI,
};

#[derive(Clone)]
pub enum State {
    Nothing,
    Exit,
    Error(ErrorStateData),
    NotesManaging(NotesManagingStateData),
    NoteViewing(NoteViewingStateData),
    NoteCreation(NoteCreationStateData),
    NoteDeletion(NoteDeletionStateData),
    NoteRenaming(NoteRenamingStateData),
    NoteTagsManaging(NoteTagsManagingStateData),
    NoteTagDeletion(NoteTagDeletionStateData),
    NoteTagAddition(NoteTagAdditionStateData),
    TagsManaging(TagsManagingStateData),
    TagCreation(TagsCreationStateData),
    TagDeletion(TagsDeletionStateData),
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
            State::NoteCreation(data) => run_note_creation_state(data, key_event, notebook).await,
            State::NoteViewing(data) => {
                run_note_viewing_state(data, key_event, notebook, force_redraw).await
            }
            State::NoteDeletion(data) => run_note_deletion_state(data, key_event, notebook).await,
            State::NoteRenaming(data) => run_note_renaming_state(data, key_event, notebook).await,
            State::NoteTagsManaging(data) => {
                run_note_tags_managing_state(data, key_event, notebook).await
            }
            State::NoteTagAddition(data) => {
                run_note_tag_addition_state(data, key_event, notebook).await
            }
            State::NoteTagDeletion(data) => {
                run_note_tag_deletion_state(data, key_event, notebook).await
            }
            State::TagsManaging(data) => run_tags_managing_state(data, key_event, notebook).await,
            State::TagCreation(data) => run_tag_creation_state(data, key_event, notebook).await,
            State::TagDeletion(data) => run_tag_deletion_state(data, key_event, notebook).await,
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
            State::NotesManaging(data) => {
                draw_notes_managing_state(data, notebook, frame, main_rect);
            }
            State::NoteCreation(data) => draw_note_creation_state(data, notebook, frame, main_rect),
            State::NoteViewing(data) => draw_note_viewing_state(data, notebook, frame, main_rect),
            State::NoteDeletion(data) => draw_note_deletion_state(data, notebook, frame, main_rect),
            State::NoteRenaming(data) => draw_note_renaming_state(data, notebook, frame, main_rect),
            State::NoteTagsManaging(data) => {
                draw_note_tags_managing_state(data, notebook, frame, main_rect);
            }
            State::NoteTagAddition(data) => {
                draw_note_tag_addition_state(data, notebook, frame, main_rect);
            }
            State::NoteTagDeletion(data) => {
                draw_note_tag_deletion_state(data, notebook, frame, main_rect);
            }
            State::TagsManaging(data) => draw_tags_managing_state(data, notebook, frame, main_rect),
            State::TagCreation(data) => draw_tag_creation_state(data, notebook, frame, main_rect),
            State::TagDeletion(data) => draw_tag_deletion_state(data, notebook, frame, main_rect),
            State::TagNotesListing(data) => draw_tag_notes_listing_state(data, frame, main_rect),
            State::Exit => unreachable!(),
        }
    }
}
