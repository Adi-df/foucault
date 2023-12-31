use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph};
use ratatui::Frame;

use crate::helpers::{DiscardResult, TryFromDatabase};
use crate::note::NoteData;
use crate::notebook::Notebook;
use crate::states::note_tag_adding::NoteTagAddingStateData;
use crate::states::note_tag_deleting::NoteTagDeletingStateData;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

use super::tag_notes_listing::TagNotesListingStateData;

pub struct NoteTagsManagingStateData {
    pub note_data: NoteData,
    pub selected: usize,
}

impl From<NoteData> for NoteTagsManagingStateData {
    fn from(note_data: NoteData) -> Self {
        NoteTagsManagingStateData {
            note_data,
            selected: 0,
        }
    }
}

impl NoteTagsManagingStateData {
    pub fn get_selected(&self) -> Option<&Tag> {
        self.note_data.tags.get(self.selected)
    }
}

pub fn run_note_tags_managing_state(
    mut state_data: NoteTagsManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!(
                "Cancel note {} tags managing.",
                state_data.note_data.note.name
            );
            State::NoteViewing(NoteViewingStateData::from(state_data.note_data))
        }
        KeyCode::Char('d') if !state_data.note_data.tags.is_empty() => {
            info!(
                "Open note {} tag {} deleting prompt.",
                state_data.note_data.note.name,
                state_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name
            );
            State::NoteTagDeleting(NoteTagDeletingStateData::empty(state_data))
        }
        KeyCode::Char('a') => {
            info!(
                "Open note {} tag adding prompt.",
                state_data.note_data.note.name
            );
            State::NoteTagAdding(NoteTagAddingStateData::empty(state_data))
        }
        KeyCode::Enter if !state_data.note_data.tags.is_empty() => {
            info!(
                "Open tag {} notes listing.",
                state_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name
            );
            State::TagNotesListing(TagNotesListingStateData::try_from_database(
                state_data.note_data.tags.swap_remove(state_data.selected),
                notebook.db(),
            )?)
        }
        KeyCode::Up if state_data.selected > 0 => {
            State::NoteTagsManaging(NoteTagsManagingStateData {
                selected: state_data.selected - 1,
                ..state_data
            })
        }
        KeyCode::Down
            if state_data.selected < state_data.note_data.tags.len().saturating_sub(1) =>
        {
            State::NoteTagsManaging(NoteTagsManagingStateData {
                selected: state_data.selected + 1,
                ..state_data
            })
        }
        _ => State::NoteTagsManaging(state_data),
    })
}

pub fn draw_note_tags_managing_state(
    data: &NoteTagsManagingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, data, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}

pub fn draw_note_tags_managing(
    frame: &mut Frame,
    NoteTagsManagingStateData {
        note_data,
        selected,
    }: &NoteTagsManagingStateData,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let note_name = Paragraph::new(Line::from(vec![
        Span::raw(note_data.note.name.as_str()).style(Style::default().fg(Color::Green))
    ]))
    .block(
        Block::new()
            .title("Note name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .padding(Padding::uniform(1)),
    );

    let note_tags = List::new(
        note_data
            .tags
            .iter()
            .map(|tag| Span::raw(tag.name.as_str())),
    )
    .highlight_symbol(">> ")
    .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
    .block(
        Block::new()
            .title("Note Tags")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(note_name, vertical_layout[0]);
    frame.render_stateful_widget(
        note_tags,
        vertical_layout[1],
        &mut ListState::default().with_selected(Some(*selected)),
    );
}
