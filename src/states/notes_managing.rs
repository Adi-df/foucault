use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};

use rusqlite::Connection;

use crate::helpers::{DiscardResult, TryFromDatabase};
use crate::note::{Note, NoteSummary};
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};

pub struct NotesManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub notes: Vec<NoteSummary>,
}

impl NotesManagingStateData {
    pub fn from_pattern(pattern: String, db: &Connection) -> Result<Self> {
        Ok(NotesManagingStateData {
            notes: NoteSummary::search_by_name(pattern.as_str(), db)?,
            selected: 0,
            pattern,
        })
    }

    pub fn empty(db: &Connection) -> Result<Self> {
        Self::from_pattern(String::new(), db)
    }
}

pub fn run_note_managing_state(
    mut state_data: NotesManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Stop notes managing.");
            State::Nothing
        }
        KeyCode::Enter if !state_data.notes.is_empty() => {
            let note_summary = &state_data.notes[state_data.selected];
            if let Some(note) = Note::load_by_id(note_summary.id, notebook.db())? {
                info!("Open note {}.", note_summary.name);
                State::NoteViewing(NoteViewingStateData::try_from_database(
                    note,
                    notebook.db(),
                )?)
            } else {
                State::NotesManaging(state_data)
            }
        }
        KeyCode::Backspace => {
            state_data.pattern.pop();
            state_data.notes =
                NoteSummary::search_by_name(state_data.pattern.as_str(), notebook.db())?;
            state_data.selected = 0;

            State::NotesManaging(state_data)
        }
        KeyCode::Char(c) => {
            state_data.pattern.push(c);
            state_data.notes =
                NoteSummary::search_by_name(state_data.pattern.as_str(), notebook.db())?;
            state_data.selected = 0;

            State::NotesManaging(state_data)
        }
        KeyCode::Up if state_data.selected > 0 => State::NotesManaging(NotesManagingStateData {
            selected: state_data.selected - 1,
            ..state_data
        }),
        KeyCode::Down if state_data.selected < state_data.notes.len().saturating_sub(1) => {
            State::NotesManaging(NotesManagingStateData {
                selected: state_data.selected + 1,
                ..state_data
            })
        }
        _ => State::NotesManaging(state_data),
    })
}

pub fn draw_note_managing_state(
    NotesManagingStateData {
        pattern,
        selected,
        notes,
    }: &NotesManagingStateData,
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

            let search_bar = Paragraph::new(Line::from(vec![
                Span::raw(pattern).style(Style::default().add_modifier(Modifier::UNDERLINED))
            ]))
            .block(
                Block::new()
                    .title("Searching")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(if notes.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    }))
                    .padding(Padding::uniform(1)),
            );

            let list_results = List::new(notes.iter().map(|note| Span::raw(note.name.as_str())))
                .highlight_symbol(">> ")
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
                .block(
                    Block::new()
                        .title("Results")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Yellow))
                        .padding(Padding::uniform(2)),
                );

            let notes_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            frame.render_widget(search_bar, vertical_layout[0]);
            frame.render_stateful_widget(
                list_results,
                vertical_layout[1],
                &mut ListState::with_selected(ListState::default(), Some(*selected)),
            );
            frame.render_stateful_widget(
                notes_scrollbar,
                vertical_layout[1].inner(&Margin::new(0, 1)),
                &mut ScrollbarState::new(notes.len()).position(*selected),
            );

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}
