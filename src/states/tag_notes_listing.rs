use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};

use rusqlite::Connection;

use crate::helpers::TryFromDatabase;
use crate::note::{Note, NoteSummary};
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

#[derive(Debug)]
pub struct TagNotesListingStateData {
    pub tag: Tag,
    pub notes: Vec<NoteSummary>,
    pub selected: usize,
}

impl TryFromDatabase<Tag> for TagNotesListingStateData {
    fn try_from_database(tag: Tag, db: &Connection) -> Result<Self> {
        Ok(TagNotesListingStateData {
            notes: tag.get_notes(db)?,
            selected: 0,
            tag,
        })
    }
}

pub fn run_tag_notes_listing_state(
    TagNotesListingStateData {
        tag,
        notes,
        selected,
    }: TagNotesListingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::Nothing,
        KeyCode::Enter if !notes.is_empty() => {
            let summary = &notes[selected];
            if let Some(note) = Note::load(summary.id, notebook.db())? {
                State::NoteViewing(NoteViewingStateData::try_from_database(
                    note,
                    notebook.db(),
                )?)
            } else {
                State::TagNotesListing(TagNotesListingStateData {
                    tag,
                    notes,
                    selected,
                })
            }
        }
        KeyCode::Up if selected > 0 => State::TagNotesListing(TagNotesListingStateData {
            tag,
            notes,
            selected: selected - 1,
        }),
        KeyCode::Down if selected < notes.len().saturating_sub(1) => {
            State::TagNotesListing(TagNotesListingStateData {
                tag,
                notes,
                selected: selected + 1,
            })
        }
        _ => State::TagNotesListing(TagNotesListingStateData {
            tag,
            notes,
            selected,
        }),
    })
}

pub fn draw_tag_notes_listing_state(
    TagNotesListingStateData {
        tag,
        notes,
        selected,
    }: &TagNotesListingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            let vertical_layout = Layout::new(
                Direction::Vertical,
                [Constraint::Length(5), Constraint::Min(0)],
            )
            .split(main_rect);

            let tag_name = Paragraph::new(Line::from(vec![
                Span::raw(tag.name.as_str()).style(Style::default().fg(Color::Green))
            ]))
            .block(
                Block::new()
                    .title("Tag name")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .padding(Padding::uniform(1)),
            );

            let tag_notes = List::new(notes.iter().map(|tag| Span::raw(tag.name.as_str())))
                .highlight_symbol(">> ")
                .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
                .block(
                    Block::new()
                        .title("Tag notes")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Yellow)),
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
        })
        .map_err(anyhow::Error::from)
        .map(|_| ())
}
