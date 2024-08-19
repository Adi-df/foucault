use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};

use crate::helpers::DiscardResult;
use crate::note::{Note, NoteSummary};
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};
use crate::NotebookAPI;

pub struct NotesManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub notes: Vec<NoteSummary>,
}

impl NotesManagingStateData {
    pub async fn from_pattern(pattern: String, notebook: &NotebookAPI) -> Result<Self> {
        Ok(NotesManagingStateData {
            notes: NoteSummary::search_by_name(pattern.as_str(), notebook).await?,
            selected: 0,
            pattern,
        })
    }

    pub async fn empty(notebook: &NotebookAPI) -> Result<Self> {
        Self::from_pattern(String::new(), notebook).await
    }
}

pub async fn run_note_managing_state(
    mut state_data: NotesManagingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Stop notes managing.");
            State::Nothing
        }
        KeyCode::Enter if !state_data.notes.is_empty() => {
            let note_summary = &state_data.notes[state_data.selected];
            info!("Open note {}.", note_summary.name());

            let note = Note::load_from_summary(note_summary, notebook).await?;
            State::NoteViewing(NoteViewingStateData::new(note, notebook).await?)
        }
        KeyCode::Backspace if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.pop();
            state_data.notes =
                NoteSummary::search_by_name(state_data.pattern.as_str(), notebook).await?;
            state_data.selected = 0;

            State::NotesManaging(state_data)
        }
        KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.push(c);
            state_data.notes =
                NoteSummary::search_by_name(state_data.pattern.as_str(), notebook).await?;
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
                Span::raw(pattern).style(Style::new().add_modifier(Modifier::UNDERLINED))
            ]))
            .block(
                Block::new()
                    .title("Searching")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(if notes.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    }))
                    .padding(Padding::uniform(1)),
            );

            let list_results = List::new(notes.iter().map(|note| {
                let pattern_start = note
                    .name()
                    .to_lowercase()
                    .find(&pattern.to_lowercase())
                    .expect("The search pattern should have matched");
                let pattern_end = pattern_start + pattern.len();

                let mut note_line = vec![
                    Span::raw(&note.name()[..pattern_start])
                        .style(Style::new().add_modifier(Modifier::BOLD)),
                    Span::raw(&note.name()[pattern_start..pattern_end]).style(
                        Style::new()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::raw(&note.name()[pattern_end..])
                        .style(Style::new().add_modifier(Modifier::BOLD)),
                    Span::raw("    "),
                ];

                for tag in note.tags() {
                    note_line.push(
                        Span::raw(tag.name()).style(Style::new().bg(Color::from_u32(tag.color()))),
                    );
                    note_line.push(Span::raw(", "));
                }
                if !note.tags().is_empty() {
                    note_line.pop();
                }

                Line::from(note_line)
            }))
            .highlight_symbol(">> ")
            .highlight_style(Style::new().bg(Color::White).fg(Color::Black))
            .block(
                Block::new()
                    .title("Results")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Yellow))
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
