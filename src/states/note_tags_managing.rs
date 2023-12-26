use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph};
use ratatui::{Frame, Terminal};

use crate::note::{Note, NoteData};
use crate::notebook::Notebook;
use crate::states::State;
use crate::tags::Tag;

use super::note_tag_deleting::NoteTagDeletingStateData;
use super::note_viewing::NoteViewingStateData;

#[derive(Debug)]
pub struct NoteTagsManagingStateData {
    pub new_tag: String,
    pub selected: usize,
    pub tags: Vec<Tag>,
    pub note: Note,
}

pub fn run_note_tags_managing_state(
    NoteTagsManagingStateData {
        note,
        tags,
        mut new_tag,
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
        KeyCode::Char(c) if !c.is_whitespace() => {
            new_tag.push(c);
            State::NoteTagsManaging(NoteTagsManagingStateData {
                new_tag,
                selected,
                tags,
                note,
            })
        }
        KeyCode::Backspace => {
            new_tag.pop();
            State::NoteTagsManaging(NoteTagsManagingStateData {
                new_tag,
                selected,
                tags,
                note,
            })
        }
        KeyCode::Enter => {
            if let Some(tag) = Tag::load_by_name(new_tag.as_str(), notebook.db())? {
                note.add_tag(&tag, notebook.db())?;
                State::NoteTagsManaging(NoteTagsManagingStateData {
                    new_tag: String::new(),
                    selected: 0,
                    tags: note.get_tags(notebook.db())?,
                    note,
                })
            } else {
                State::NoteTagsManaging(NoteTagsManagingStateData {
                    new_tag,
                    selected,
                    tags,
                    note,
                })
            }
        }
        KeyCode::Delete if !tags.is_empty() => State::NoteTagDeleting(NoteTagDeletingStateData {
            tags_managing: NoteTagsManagingStateData {
                new_tag,
                selected,
                tags,
                note,
            },
            delete: false,
        }),
        KeyCode::Up if selected > 0 => State::NoteTagsManaging(NoteTagsManagingStateData {
            new_tag,
            selected: selected - 1,
            tags,
            note,
        }),
        KeyCode::Down if selected < tags.len() - 1 => {
            State::NoteTagsManaging(NoteTagsManagingStateData {
                new_tag,
                selected: selected + 1,
                tags,
                note,
            })
        }
        _ => State::NoteTagsManaging(NoteTagsManagingStateData {
            new_tag,
            selected,
            tags,
            note,
        }),
    })
}

pub fn draw_note_tags_managing_state(
    data: &NoteTagsManagingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    notebook: &Notebook,
    main_frame: Block,
) -> Result<()> {
    let valid_new_tag_name = Tag::tag_exists(data.new_tag.as_str(), notebook.db())?;

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, data, valid_new_tag_name, main_rect);

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
        new_tag,
        selected,
    }: &NoteTagsManagingStateData,
    valid: bool,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);
    let horizontal_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(30), Constraint::Min(0)],
    )
    .split(vertical_layout[0]);

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
    let new_tag_name = Paragraph::new(Line::from(vec![
        Span::raw(new_tag.as_str()).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::new()
            .title("New tag")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if valid { Color::Green } else { Color::Red }))
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

    frame.render_widget(note_name, horizontal_layout[0]);
    frame.render_widget(new_tag_name, horizontal_layout[1]);
    frame.render_stateful_widget(
        note_tags,
        vertical_layout[1],
        &mut ListState::default().with_selected(Some(*selected)),
    );
}
