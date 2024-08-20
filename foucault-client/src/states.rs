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

use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyEvent;
use ratatui::{
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Padding},
    Terminal as UITerminal,
};

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

pub type Terminal = UITerminal<CrosstermBackend<Stdout>>;

pub enum State {
    Nothing,
    Exit,
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

    pub fn draw(&self, notebook: &NotebookAPI, terminal: &mut Terminal) -> Result<()> {
        let main_frame = Block::new()
            .title(notebook.name.as_str())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::White));

        match self {
            State::Nothing => draw_nothing_state(terminal, notebook, main_frame),
            State::NotesManaging(data) => draw_note_managing_state(data, terminal, main_frame),
            State::NoteCreating(data) => draw_note_creating_state(data, terminal, main_frame),
            State::NoteViewing(data) => draw_note_viewing_state(data, terminal, main_frame),
            State::NoteDeleting(data) => draw_note_deleting_state(data, terminal, main_frame),
            State::NoteRenaming(data) => draw_note_renaming_state(data, terminal, main_frame),
            State::NoteTagsManaging(data) => {
                draw_note_tags_managing_state(data, terminal, main_frame)
            }
            State::NoteTagAdding(data) => {
                draw_note_tag_adding_state_data(data, terminal, main_frame)
            }
            State::NoteTagDeleting(data) => {
                draw_note_tag_deleting_state_data(data, terminal, main_frame)
            }
            State::TagsManaging(data) => draw_tags_managing_state(data, terminal, main_frame),
            State::TagCreating(data) => draw_tag_creating_state(data, terminal, main_frame),
            State::TagDeleting(data) => draw_tag_deleting_state(data, terminal, main_frame),
            State::TagNotesListing(data) => {
                draw_tag_notes_listing_state(data, terminal, main_frame)
            }
            State::Exit => unreachable!(),
        }
    }
}
