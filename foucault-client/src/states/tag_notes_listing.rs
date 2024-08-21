use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    prelude::{Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListState, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::{
    note::{Note, NoteSummary},
    states::{note_viewing::NoteViewingStateData, State},
    tag::Tag,
    NotebookAPI,
};

pub struct TagNotesListingStateData {
    pub tag: Tag,
    pub notes: Vec<NoteSummary>,
    pub selected: usize,
}

impl TagNotesListingStateData {
    pub async fn new(tag: Tag, notebook: &NotebookAPI) -> Result<Self> {
        Ok(TagNotesListingStateData {
            notes: tag.get_related_notes(notebook).await?,
            selected: 0,
            tag,
        })
    }
}

pub async fn run_tag_notes_listing_state(
    state_data: TagNotesListingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Cancel tag {} note listing.", state_data.tag.name());
            State::Nothing
        }
        KeyCode::Enter if !state_data.notes.is_empty() => {
            let summary = &state_data.notes[state_data.selected];
            if let Some(note) = Note::load_by_id(summary.id(), notebook).await? {
                info!("Open note {} viewing.", note.name());
                State::NoteViewing(NoteViewingStateData::new(note, notebook).await?)
            } else {
                State::TagNotesListing(state_data)
            }
        }
        KeyCode::Up if state_data.selected > 0 => {
            State::TagNotesListing(TagNotesListingStateData {
                selected: state_data.selected - 1,
                ..state_data
            })
        }
        KeyCode::Down if state_data.selected < state_data.notes.len().saturating_sub(1) => {
            State::TagNotesListing(TagNotesListingStateData {
                selected: state_data.selected + 1,
                ..state_data
            })
        }
        _ => State::TagNotesListing(state_data),
    })
}

pub fn draw_tag_notes_listing_state(
    TagNotesListingStateData {
        tag,
        notes,
        selected,
    }: &TagNotesListingStateData,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let tag_name = Paragraph::new(Line::from(vec![
        Span::raw(tag.name()).style(Style::new().fg(Color::Green))
    ]))
    .block(
        Block::new()
            .title("Tag name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Blue))
            .padding(Padding::uniform(1)),
    );

    let tag_notes = List::new(notes.iter().map(|tag| Span::raw(tag.name())))
        .highlight_symbol(">> ")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::White))
        .block(
            Block::new()
                .title("Tag notes")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Yellow)),
        );

    let notes_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(tag_name, vertical_layout[0]);
    frame.render_stateful_widget(
        tag_notes,
        vertical_layout[1],
        &mut ListState::default().with_selected(Some(*selected)),
    );
    frame.render_stateful_widget(
        notes_scrollbar,
        vertical_layout[1].inner(&Margin::new(0, 1)),
        &mut ScrollbarState::new(notes.len()).position(*selected),
    );
}
