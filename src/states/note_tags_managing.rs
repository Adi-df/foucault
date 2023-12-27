use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph};
use ratatui::Frame;

use crate::note::{Note, NoteData};
use crate::notebook::Notebook;
use crate::states::note_tag_adding::NoteTagAddingStateData;
use crate::states::note_tag_deleting::NoteTagDeletingStateData;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

#[derive(Debug)]
pub struct NoteTagsManagingStateData {
    pub selected: usize,
    pub tags: Vec<Tag>,
    pub note: Note,
}

pub fn run_note_tags_managing_state(
    NoteTagsManagingStateData {
        note,
        tags,
        selected,
    }: NoteTagsManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            let tags = note.get_tags(notebook.db())?;
            let links = note.get_links(notebook.db())?;

            State::NoteViewing(NoteViewingStateData {
                note_data: NoteData { note, tags, links },
                scroll: 0,
            })
        }
        KeyCode::Char('d') if !tags.is_empty() => {
            State::NoteTagDeleting(NoteTagDeletingStateData {
                tags_managing: NoteTagsManagingStateData {
                    selected,
                    tags,
                    note,
                },
                delete: false,
            })
        }
        KeyCode::Char('a') => State::NoteTagAdding(NoteTagAddingStateData {
            tags_managing: NoteTagsManagingStateData {
                selected,
                tags,
                note,
            },
            tag_name: String::new(),
            valid: false,
        }),
        KeyCode::Up if selected > 0 => State::NoteTagsManaging(NoteTagsManagingStateData {
            selected: selected - 1,
            tags,
            note,
        }),
        KeyCode::Down if selected < tags.len().saturating_sub(1) => {
            State::NoteTagsManaging(NoteTagsManagingStateData {
                selected: selected + 1,
                tags,
                note,
            })
        }
        _ => State::NoteTagsManaging(NoteTagsManagingStateData {
            selected,
            tags,
            note,
        }),
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
        .map_err(anyhow::Error::from)
        .map(|_| ())
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
