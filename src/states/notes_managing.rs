use std::io::Stdout;

use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph};
use ratatui::Terminal;

use crate::note::{Note, NoteSummary};
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{NoteData, State};

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
        mut notes,
    }: NotesManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Stop note searching.");
            State::Nothing
        }
        KeyCode::Up if selected > 0 => State::NotesManaging(NotesManagingStateData {
            pattern,
            selected: selected - 1,
            notes,
        }),
        KeyCode::Down if selected < notes.len() - 1 => {
            State::NotesManaging(NotesManagingStateData {
                pattern,
                selected: selected + 1,
                notes,
            })
        }
        KeyCode::Enter if !notes.is_empty() => {
            let note_summary = &notes[selected];
            let note = Note::load(note_summary.id, notebook.db())?;
            let tags = note
                .get_tags(notebook.db())?
                .into_iter()
                .map(|tag| tag.name.clone())
                .collect();
            let links = note.get_links(notebook.db())?;

            info!("Open note {}", note_summary.name);

            State::NoteViewing(NoteViewingStateData {
                note_data: NoteData { note, tags, links },
                scroll: 0,
            })
        }
        KeyCode::Backspace => {
            pattern.pop();
            notes = Note::search_by_name(pattern.as_str(), notebook.db())?;
            State::NotesManaging(NotesManagingStateData {
                pattern,
                selected: 0,
                notes,
            })
        }
        KeyCode::Char(c) => {
            pattern.push(c);
            notes = Note::search_by_name(pattern.as_str(), notebook.db())?;
            State::NotesManaging(NotesManagingStateData {
                pattern,
                selected: 0,
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
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            let layout = Layout::new(
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

            frame.render_widget(search_bar, layout[0]);
            frame.render_stateful_widget(
                list_results,
                layout[1],
                &mut ListState::with_selected(ListState::default(), Some(*selected)),
            );

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
