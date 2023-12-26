use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use log::info;
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListState, Padding, Paragraph},
    Frame, Terminal,
};

use crate::{notebook::Notebook, tags::Tag};

use super::{tag_creating::TagsCreatingStateData, tag_deleting::TagsDeletingStateData, State};

#[derive(Debug)]
pub struct TagsManagingStateData {
    pub pattern: String,
    pub pattern_editing: bool,
    pub selected: usize,
    pub tags: Vec<Tag>,
}

pub fn run_tags_managing_state(
    mut state_data: TagsManagingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Stop tags managing.");
            State::Nothing
        }
        KeyCode::Up if state_data.selected > 0 => State::TagsManaging(TagsManagingStateData {
            selected: state_data.selected - 1,
            ..state_data
        }),
        KeyCode::Down if state_data.selected < state_data.tags.len() - 1 => {
            State::TagsManaging(TagsManagingStateData {
                selected: state_data.selected + 1,
                ..state_data
            })
        }
        KeyCode::Tab => State::TagsManaging(TagsManagingStateData {
            pattern_editing: !state_data.pattern_editing,
            ..state_data
        }),
        KeyCode::Backspace if state_data.pattern_editing => {
            state_data.pattern.pop();
            State::TagsManaging(TagsManagingStateData {
                tags: Tag::search_by_name(state_data.pattern.as_str(), notebook.db())?,
                pattern: state_data.pattern,
                selected: 0,
                ..state_data
            })
        }
        KeyCode::Char(c) if state_data.pattern_editing && !c.is_whitespace() => {
            state_data.pattern.push(c);
            State::TagsManaging(TagsManagingStateData {
                selected: 0,
                tags: Tag::search_by_name(state_data.pattern.as_str(), notebook.db())?,
                pattern: state_data.pattern,
                ..state_data
            })
        }
        KeyCode::Char('c') if !state_data.pattern_editing => {
            State::TagCreating(TagsCreatingStateData {
                tags_search: state_data,
                name: String::new(),
                valid: false,
            })
        }
        KeyCode::Char('d') if !state_data.pattern_editing && !state_data.tags.is_empty() => {
            State::TagDeleting(TagsDeletingStateData {
                delete: false,
                tags_managing: state_data,
            })
        }
        _ => State::TagsManaging(state_data),
    })
}

pub fn draw_tags_managing_state(
    tags_managing: &TagsManagingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}

pub fn draw_tags_managing(
    frame: &mut Frame,
    TagsManagingStateData {
        pattern,
        pattern_editing,
        selected,
        tags,
    }: &TagsManagingStateData,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let filter_bar_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Length(10), Constraint::Min(0)],
    )
    .split(vertical_layout[0]);

    let filter_editing_active = Paragraph::new(Line::from(vec![Span::raw(if *pattern_editing {
        "Yes"
    } else {
        "No"
    })
    .style(
        Style::default()
            .fg(if *pattern_editing {
                Color::Green
            } else {
                Color::Red
            })
            .add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::new()
            .title("Editing")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .padding(Padding::uniform(1)),
    );

    let filter_bar = Paragraph::new(Line::from(vec![
        Span::raw(pattern).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::new()
            .title("Filter")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if tags.is_empty() {
                Color::Red
            } else {
                Color::Green
            }))
            .padding(Padding::uniform(1)),
    );

    let list_results = List::new(tags.iter().map(|tag| Span::raw(tag.name.as_str())))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
        .block(
            Block::new()
                .title("Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .padding(Padding::uniform(2)),
        );

    frame.render_widget(filter_editing_active, filter_bar_layout[0]);
    frame.render_widget(filter_bar, filter_bar_layout[1]);
    frame.render_stateful_widget(
        list_results,
        vertical_layout[1],
        &mut ListState::with_selected(ListState::default(), Some(*selected)),
    );
}
