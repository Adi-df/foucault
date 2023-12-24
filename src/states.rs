mod note_creating;
mod note_deleting;
mod note_managing;
mod note_renaming;
mod note_viewing;
mod nothing;
mod tag_creating;
mod tag_deleting;
mod tag_managing;

use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Padding};
use ratatui::Terminal;

use crate::note::Note;
use crate::notebook::Notebook;

use self::note_creating::{
    draw_note_creating_state, run_note_creating_state, NoteCreatingStateData,
};
use self::note_deleting::{
    draw_note_deleting_state, run_note_deleting_state, NoteDeletingStateData,
};
use self::note_managing::{
    draw_note_managing_state, run_note_managing_state, NotesManagingStateData,
};
use self::note_renaming::{
    draw_note_renaming_state, run_note_renaming_state, NoteRenamingStateData,
};
use self::note_viewing::{draw_note_viewing_state, run_note_viewing_state, NoteViewingStateData};
use self::nothing::{draw_nothing_state, run_nothing_state};
use self::tag_creating::{draw_tags_creating_state, run_tag_creating_state, TagsCreatingStateData};
use self::tag_deleting::{draw_tag_deleting_state, run_tag_deleting_state, TagsDeletingStateData};
use self::tag_managing::{
    draw_tags_managing_state, run_tags_managing_state, TagsManagingStateData,
};

#[derive(Debug)]
pub struct NoteData {
    pub note: Note,
    pub tags: Vec<String>,
    pub links: Vec<i64>,
}

#[derive(Debug)]
pub enum State {
    Nothing,
    Exit,
    NotesManaging(NotesManagingStateData),
    NoteViewing(NoteViewingStateData),
    NoteCreating(NoteCreatingStateData),
    NoteDeleting(NoteDeletingStateData),
    NoteRenaming(NoteRenamingStateData),
    TagsManaging(TagsManagingStateData),
    TagCreating(TagsCreatingStateData),
    TagDeleting(TagsDeletingStateData),
}

impl State {
    pub fn run(
        self,
        key_code: KeyCode,
        notebook: &Notebook,
        force_redraw: &mut bool,
    ) -> Result<Self> {
        match self {
            State::Nothing => run_nothing_state(key_code, notebook),
            State::NotesManaging(data) => run_note_managing_state(data, key_code, notebook),
            State::NoteCreating(data) => run_note_creating_state(data, key_code, notebook),
            State::NoteViewing(data) => {
                run_note_viewing_state(data, key_code, notebook, force_redraw)
            }
            State::NoteDeleting(data) => run_note_deleting_state(data, key_code, notebook),
            State::NoteRenaming(data) => run_note_renaming_state(data, key_code, notebook),
            State::TagsManaging(data) => run_tags_managing_state(data, key_code, notebook),
            State::TagCreating(data) => run_tag_creating_state(data, key_code, notebook),
            State::TagDeleting(data) => run_tag_deleting_state(data, key_code, notebook),
            State::Exit => unreachable!(),
        }
    }

    pub fn draw(
        &self,
        notebook: &Notebook,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let main_frame = Block::default()
            .title(notebook.name.as_str())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        match self {
            State::Nothing => draw_nothing_state(terminal, notebook, main_frame),
            State::NotesManaging(data) => draw_note_managing_state(data, terminal, main_frame),
            State::NoteCreating(data) => draw_note_creating_state(data, terminal, main_frame),
            State::NoteViewing(data) => draw_note_viewing_state(data, terminal, main_frame),
            State::NoteDeleting(data) => draw_note_deleting_state(data, terminal, main_frame),
            State::NoteRenaming(data) => draw_note_renaming_state(data, terminal, main_frame),
            State::TagsManaging(data) => draw_tags_managing_state(data, terminal, main_frame),
            State::TagCreating(data) => {
                draw_tags_creating_state(data, terminal, notebook, main_frame)
            }
            State::TagDeleting(data) => draw_tag_deleting_state(data, terminal, main_frame),
            State::Exit => unreachable!(),
        }
    }
}
