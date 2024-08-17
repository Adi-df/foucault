use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use crate::helpers::{create_bottom_line, create_row_help_layout, DiscardResult};
use crate::notebook::Notebook;
use crate::states::tag_creating::TagsCreatingStateData;
use crate::states::tag_deleting::TagsDeletingStateData;
use crate::states::tag_notes_listing::TagNotesListingStateData;
use crate::states::{State, Terminal};
use crate::tag::Tag;

pub struct TagsManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub tags: Vec<Tag>,
    pub help_display: bool,
}

impl TagsManagingStateData {
    pub fn from_pattern(pattern: String, db: &Connection) -> Result<Self> {
        Ok(TagsManagingStateData {
            tags: Tag::search_by_name(pattern.as_str(), db)?,
            selected: 0,
            pattern,
            help_display: false,
        })
    }

    pub fn empty(db: &Connection) -> Result<Self> {
        Self::from_pattern(String::new(), db)
    }

    pub fn get_selected(&self) -> Option<&Tag> {
        self.tags.get(self.selected)
    }
}

pub async fn run_tags_managing_state(
    mut state_data: TagsManagingStateData,
    key_event: KeyEvent,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Stop tags managing.");
            State::Nothing
        }
        KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle help display.");
            state_data.help_display = !state_data.help_display;

            State::TagsManaging(state_data)
        }
        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Open tag creating prompt.");
            State::TagCreating(TagsCreatingStateData::empty(state_data))
        }
        KeyCode::Char('d')
            if key_event.modifiers == KeyModifiers::CONTROL && !state_data.tags.is_empty() =>
        {
            info!("Open tag deleting prompt.");
            State::TagDeleting(TagsDeletingStateData::empty(state_data))
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
        KeyCode::Enter if !state_data.tags.is_empty() => {
            info!("Open tag notes listing.");
            let tag = state_data.tags.swap_remove(state_data.selected);

            State::TagNotesListing(TagNotesListingStateData::new(tag, notebook.db())?)
        }
        KeyCode::Backspace if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.pop();
            state_data.tags = Tag::search_by_name(state_data.pattern.as_str(), notebook.db())?;
            state_data.selected = 0;
            State::TagsManaging(state_data)
        }
        KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE && !c.is_whitespace() => {
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
        .discard_result()
}

pub fn draw_tags_managing(
    frame: &mut Frame,
    TagsManagingStateData {
        pattern,
        selected,
        tags,
        help_display,
    }: &TagsManagingStateData,
    main_rect: Rect,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let filter_bar = Paragraph::new(Line::from(vec![
        Span::raw(pattern).style(Style::new().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::new()
            .title("Filter")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if tags.is_empty() {
                Color::Red
            } else {
                Color::Green
            }))
            .padding(Padding::uniform(1)),
    );

    let list_results = List::new(tags.iter().map(|tag| {
        let pattern_start = tag
            .name()
            .to_lowercase()
            .find(pattern)
            .expect("The pattern should match listed tags");
        let pattern_end = pattern_start + pattern.len();
        Line::from(vec![
            Span::raw(&tag.name()[..pattern_start]),
            Span::raw(&tag.name()[pattern_start..pattern_end])
                .style(Style::new().add_modifier(Modifier::UNDERLINED)),
            Span::raw(&tag.name()[pattern_end..]),
        ])
    }))
    .highlight_symbol(">> ")
    .highlight_style(Style::new().bg(Color::White).fg(Color::Black))
    .block(
        Block::new()
            .title("Tags")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Yellow))
            .padding(Padding::uniform(2)),
    );

    let tags_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(filter_bar, vertical_layout[0]);
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

    if *help_display {
        let command_area = create_bottom_line(main_rect);
        let commands = create_row_help_layout(&[
            ("Ctrl+c", "Create tag"),
            ("Ctrl+d", "Delete tag"),
            ("⏎", "List related notes"),
        ])
        .block(
            Block::new()
                .padding(Padding::uniform(1))
                .borders(Borders::all())
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::White)),
        );

        frame.render_widget(Clear, command_area);
        frame.render_widget(commands, command_area);
    }
}
