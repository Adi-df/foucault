use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use rusqlite::Connection;

use crate::helpers::TryFromDatabase;
use crate::notebook::Notebook;
use crate::states::tag_creating::TagsCreatingStateData;
use crate::states::tag_deleting::TagsDeletingStateData;
use crate::states::tag_notes_listing::TagNotesListingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

#[derive(Debug)]
pub struct TagsManagingStateData {
    pub pattern: String,
    pub pattern_editing: bool,
    pub selected: usize,
    pub tags: Vec<Tag>,
}

impl TagsManagingStateData {
    pub fn from_pattern(pattern: String, db: &Connection) -> Result<Self> {
        Ok(TagsManagingStateData {
            tags: Tag::search_by_name(pattern.as_str(), db)?,
            pattern_editing: false,
            selected: 0,
            pattern,
        })
    }

    pub fn empty(db: &Connection) -> Result<Self> {
        Self::from_pattern(String::new(), db)
    }
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
        KeyCode::Char('c') if !state_data.pattern_editing => {
            State::TagCreating(TagsCreatingStateData::empty(state_data))
        }
        KeyCode::Char('d') if !state_data.pattern_editing && !state_data.tags.is_empty() => {
            State::TagDeleting(TagsDeletingStateData::empty(state_data))
        }
        KeyCode::Enter if !state_data.tags.is_empty() => {
            let tag = state_data.tags.swap_remove(state_data.selected);

            State::TagNotesListing(TagNotesListingStateData::try_from_database(
                tag,
                notebook.db(),
            )?)
        }
        KeyCode::Tab => State::TagsManaging(TagsManagingStateData {
            pattern_editing: !state_data.pattern_editing,
            ..state_data
        }),
        KeyCode::Backspace if state_data.pattern_editing => {
            state_data.pattern.pop();
            state_data.tags = Tag::search_by_name(state_data.pattern.as_str(), notebook.db())?;
            state_data.selected = 0;
            State::TagsManaging(state_data)
        }
        KeyCode::Char(c) if state_data.pattern_editing && !c.is_whitespace() => {
            state_data.pattern.push(c);
            state_data.tags = Tag::search_by_name(state_data.pattern.as_str(), notebook.db())?;
            state_data.selected = 0;
            State::TagsManaging(state_data)
        }
        _ => State::TagsManaging(state_data),
    })
}

pub fn draw_tags_managing_state(
    tags_managing: &TagsManagingStateData,
    terminal: &mut Terminal,
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

    let tags_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(filter_editing_active, filter_bar_layout[0]);
    frame.render_widget(filter_bar, filter_bar_layout[1]);
    frame.render_stateful_widget(
        list_results,
        vertical_layout[1],
        &mut ListState::with_selected(ListState::default(), Some(*selected)),
    );
    frame.render_stateful_widget(
        tags_scrollbar,
        vertical_layout[1].inner(&Margin::new(0, 1)),
        &mut ScrollbarState::new(tags.len()).position(*selected),
    );
}
