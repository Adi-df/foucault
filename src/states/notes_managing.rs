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

use crate::helpers::TryIntoDatabase;
use crate::note::{Note, NoteSummary};
use crate::notebook::Notebook;
use crate::states::{State, Terminal};

#[derive(Debug)]
pub struct NotesManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub notes: Vec<NoteSummary>,
}

pub fn run_note_managing_state(
    NotesManagingStateData {
        mut pattern,
        selected,
        notes,
    }: NotesManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Stop note searching.");
            State::Nothing
        }
        KeyCode::Enter if !notes.is_empty() => {
            let note_summary = &notes[selected];
            if let Some(note) = Note::load(note_summary.id, notebook.db())? {
                info!("Open note {}", note_summary.name);

                State::NoteViewing(note.try_into_database(notebook.db())?)
            } else {
                State::NotesManaging(NotesManagingStateData {
                    pattern,
                    selected,
                    notes,
                })
            }
        }
        KeyCode::Backspace => {
            pattern.pop();
            State::NotesManaging(NotesManagingStateData {
                notes: Note::search_by_name(pattern.as_str(), notebook.db())?,
                selected: 0,
                pattern,
            })
        }
        KeyCode::Char(c) => {
            pattern.push(c);
            State::NotesManaging(NotesManagingStateData {
                notes: Note::search_by_name(pattern.as_str(), notebook.db())?,
                selected: 0,
                pattern,
            })
        }
        KeyCode::Up if selected > 0 => State::NotesManaging(NotesManagingStateData {
            pattern,
            selected: selected - 1,
            notes,
        }),
        KeyCode::Down if selected < notes.len().saturating_sub(1) => {
            State::NotesManaging(NotesManagingStateData {
                pattern,
                selected: selected + 1,
                notes,
            })
        }
        _ => State::NotesManaging(NotesManagingStateData {
            pattern,
            selected,
            notes,
        }),
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
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
