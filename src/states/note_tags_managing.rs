use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph};
use ratatui::Frame;

use crate::helpers::{DiscardResult, TryFromDatabase};
use crate::note::Note;
use crate::notebook::Notebook;
use crate::states::note_tag_adding::NoteTagAddingStateData;
use crate::states::note_tag_deleting::NoteTagDeletingStateData;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

use super::tag_notes_listing::TagNotesListingStateData;

pub struct NoteTagsManagingStateData {
    pub selected: usize,
    pub tags: Vec<Tag>,
    pub note: Note,
}

impl TryFromDatabase<Note> for NoteTagsManagingStateData {
    fn try_from_database(note: Note, db: &rusqlite::Connection) -> Result<Self> {
        Ok(NoteTagsManagingStateData {
            selected: 0,
            tags: note.get_tags(db)?,
            note,
        })
    }
}

pub fn run_note_tags_managing_state(
    mut state_data: NoteTagsManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteViewing(NoteViewingStateData::try_from_database(
            state_data.note,
            notebook.db(),
        )?),
        KeyCode::Char('d') if !state_data.tags.is_empty() => {
            State::NoteTagDeleting(NoteTagDeletingStateData::empty(state_data))
        }
        KeyCode::Char('a') => State::NoteTagAdding(NoteTagAddingStateData::empty(state_data)),
        KeyCode::Enter if !state_data.tags.is_empty() => {
            State::TagNotesListing(TagNotesListingStateData::try_from_database(
                state_data.tags.swap_remove(state_data.selected),
                notebook.db(),
            )?)
        }
        KeyCode::Up if state_data.selected > 0 => {
            State::NoteTagsManaging(NoteTagsManagingStateData {
                selected: state_data.selected - 1,
                ..state_data
            })
        }
        KeyCode::Down if state_data.selected < state_data.tags.len().saturating_sub(1) => {
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
        note,
        tags,
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
        Span::raw(note.name.as_str()).style(Style::default().fg(Color::Green))
    ]))
    .block(
        Block::new()
            .title("Note name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .padding(Padding::uniform(1)),
    );

    let note_tags = List::new(tags.iter().map(|tag| Span::raw(tag.name.as_str())))
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
