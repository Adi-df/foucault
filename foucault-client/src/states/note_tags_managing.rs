use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph},
    Frame,
};

use crate::{
    helpers::create_help_bar,
    note::Note,
    states::{
        note_tag_adding::NoteTagAddingStateData, note_tag_deleting::NoteTagDeletingStateData,
        note_viewing::NoteViewingStateData, tag_notes_listing::TagNotesListingStateData, State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct NoteTagsManagingStateData {
    pub note: Note,
    pub tags: Vec<Tag>,
    pub selected: usize,
    pub help_display: bool,
}

impl NoteTagsManagingStateData {
    pub async fn new(note: Note, notebook: &NotebookAPI) -> Result<Self> {
        Ok(NoteTagsManagingStateData {
            tags: note.tags(notebook).await?,
            note,
            selected: 0,
            help_display: false,
        })
    }

    pub fn get_selected(&self) -> Option<&Tag> {
        self.tags.get(self.selected)
    }
}

pub async fn run_note_tags_managing_state(
    mut state_data: NoteTagsManagingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Quit note {} tags managing.", state_data.note.name());
            State::NoteViewing(NoteViewingStateData::new(state_data.note, notebook).await?)
        }
        KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle the help display.");
            state_data.help_display = !state_data.help_display;

            State::NoteTagsManaging(state_data)
        }
        KeyCode::Char('d') if !state_data.tags.is_empty() && notebook.permissions.writable() => {
            info!(
                "Open note {} tag {} deletion prompt.",
                state_data.note.name(),
                state_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name()
            );
            State::NoteTagDeleting(NoteTagDeletingStateData::empty(state_data))
        }
        KeyCode::Char('a') if notebook.permissions.writable() => {
            info!("Open note {} tag adding prompt.", state_data.note.name());
            State::NoteTagAdding(NoteTagAddingStateData::empty(state_data))
        }
        KeyCode::Enter if !state_data.tags.is_empty() => {
            info!(
                "Open the listing of notes related to tag {}.",
                state_data
                    .get_selected()
                    .expect("A tag should be selected.")
                    .name()
            );
            State::TagNotesListing(
                TagNotesListingStateData::new(
                    state_data.tags.swap_remove(state_data.selected),
                    notebook,
                )
                .await?,
            )
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
    NoteTagsManagingStateData {
        note,
        tags,
        selected,
        help_display,
    }: &NoteTagsManagingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let note_name = Paragraph::new(Line::from(vec![
        Span::raw(note.name()).style(Style::new().fg(Color::Green))
    ]))
    .block(
        Block::new()
            .title("Note name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Blue))
            .padding(Padding::uniform(1)),
    );

    let note_tags = List::new(tags.iter().map(|tag| Span::raw(tag.name())))
        .highlight_symbol(">> ")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::White))
        .block(
            Block::new()
                .title("Note Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Yellow))
                .padding(Padding::uniform(1)),
        );

    frame.render_widget(note_name, vertical_layout[0]);
    frame.render_stateful_widget(
        note_tags,
        vertical_layout[1],
        &mut ListState::default().with_selected(Some(*selected)),
    );

    if *help_display {
        let writing_op_color = if notebook.permissions.writable() {
            Color::Blue
        } else {
            Color::Red
        };
        let (commands, commands_area) = create_help_bar(
            &[
                ("a", writing_op_color, "Add tag"),
                ("d", writing_op_color, "Delete tag"),
                ("⏎", Color::Blue, "List related notes"),
            ],
            3,
            main_rect,
        );

        frame.render_widget(Clear, commands_area);
        frame.render_widget(commands, commands_area);
    }
}
