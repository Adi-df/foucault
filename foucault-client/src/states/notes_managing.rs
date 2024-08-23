use std::sync::Arc;

use anyhow::Result;
use log::{info, warn};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    prelude::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::{
    helpers::create_help_bar,
    note::{Note, NoteSummary},
    states::{
        note_creating::NoteCreatingStateData, note_deleting::NoteDeletingStateData,
        note_viewing::NoteViewingStateData, State,
    },
    NotebookAPI,
};

use foucault_core::note_repr::NoteError;

#[derive(Clone)]
pub struct NotesManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub notes: Arc<[NoteSummary]>,
    pub help_display: bool,
}

impl NotesManagingStateData {
    pub async fn from_pattern(pattern: String, notebook: &NotebookAPI) -> Result<Self> {
        Ok(NotesManagingStateData {
            notes: NoteSummary::search_by_name(pattern.as_str(), notebook)
                .await?
                .into(),
            pattern,
            selected: 0,
            help_display: false,
        })
    }

    pub async fn empty(notebook: &NotebookAPI) -> Result<Self> {
        Self::from_pattern(String::new(), notebook).await
    }

    fn selected(&self) -> &NoteSummary {
        &self.notes[self.selected]
    }
}

pub async fn run_note_managing_state(
    mut state_data: NotesManagingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Quit the notes manager.");
            State::Nothing
        }
        KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle the help bar.");
            state_data.help_display = !state_data.help_display;
            State::NotesManaging(state_data)
        }
        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Open the note creation prompt.");
            State::NoteCreating(NoteCreatingStateData::empty())
        }
        KeyCode::Char('d')
            if key_event.modifiers == KeyModifiers::CONTROL && !state_data.notes.is_empty() =>
        {
            info!("Open the note deletion prompt.");
            let selected_note = state_data.selected();
            State::NoteDeleting(NoteDeletingStateData::from_notes_managing(
                selected_note.name().to_string(),
                selected_note.id(),
                state_data,
            ))
        }
        KeyCode::Enter if !state_data.notes.is_empty() => {
            let note_summary = state_data.selected();
            info!("Open note {}.", note_summary.name());

            let note = Note::load_by_id(note_summary.id(), notebook)
                .await?
                .ok_or(NoteError::DoesNotExist)?;
            State::NoteViewing(NoteViewingStateData::new(note, notebook).await?)
        }
        KeyCode::Backspace if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.pop();
            state_data.notes = NoteSummary::search_by_name(state_data.pattern.as_str(), notebook)
                .await?
                .into();
            state_data.selected = 0;

            State::NotesManaging(state_data)
        }
        KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.push(c);
            state_data.notes = NoteSummary::search_by_name(state_data.pattern.as_str(), notebook)
                .await?
                .into();
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
        help_display,
    }: &NotesManagingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
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
            .title("Filter")
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
        let mut note_line =
            if let Some(pattern_start) = note.name().to_lowercase().find(&pattern.to_lowercase()) {
                let pattern_end = pattern_start + pattern.len();
                vec![
                    Span::raw(&note.name()[..pattern_start])
                        .style(Style::new().add_modifier(Modifier::BOLD)),
                    Span::raw(&note.name()[pattern_start..pattern_end]).style(
                        Style::new()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::raw(&note.name()[pattern_end..])
                        .style(Style::new().add_modifier(Modifier::BOLD)),
                ]
            } else {
                warn!(
                    "The search pattern '{pattern}' did not match on note {}",
                    note.name()
                );
                vec![Span::raw(note.name())]
            };

        note_line.push(Span::raw("    "));

        for tag in &note.tags() {
            note_line.push(
                Span::raw(tag.name().to_string())
                    .style(Style::new().bg(Color::from_u32(tag.color()))),
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
            .padding(Padding::uniform(1)),
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
        vertical_layout[1].inner(Margin::new(0, 1)),
        &mut ScrollbarState::new(notes.len()).position(*selected),
    );

    if *help_display {
        let writing_op_color = if notebook.permissions.writable() {
            Color::Blue
        } else {
            Color::Red
        };
        let (commands, commands_area) = create_help_bar(
            &[
                ("Ctrl+c", writing_op_color, "Create note"),
                ("⏎", Color::Blue, "Open note"),
            ],
            3,
            main_rect,
        );

        frame.render_widget(Clear, commands_area);
        frame.render_widget(commands, commands_area);
    }
}
