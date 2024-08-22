use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Margin, Rect},
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
    states::{
        tag_creating::TagsCreatingStateData, tag_deleting::TagsDeletingStateData,
        tag_notes_listing::TagNotesListingStateData, State,
    },
    tag::Tag,
    NotebookAPI,
};

#[derive(Clone)]
pub struct TagsManagingStateData {
    pub pattern: String,
    pub selected: usize,
    pub tags: Vec<Tag>,
    pub help_display: bool,
}

impl TagsManagingStateData {
    pub async fn from_pattern(pattern: String, notebook: &NotebookAPI) -> Result<Self> {
        Ok(TagsManagingStateData {
            tags: Tag::search_by_name(pattern.as_str(), notebook).await?,
            selected: 0,
            pattern,
            help_display: false,
        })
    }

    pub async fn empty(notebook: &NotebookAPI) -> Result<Self> {
        Self::from_pattern(String::new(), notebook).await
    }

    pub fn get_selected(&self) -> Option<&Tag> {
        self.tags.get(self.selected)
    }
}

pub async fn run_tags_managing_state(
    mut state_data: TagsManagingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
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
        KeyCode::Char('c')
            if key_event.modifiers == KeyModifiers::CONTROL && notebook.permissions.writable() =>
        {
            info!("Open tag creating prompt.");
            State::TagCreating(TagsCreatingStateData::empty(state_data))
        }
        KeyCode::Char('d')
            if key_event.modifiers == KeyModifiers::CONTROL
                && !state_data.tags.is_empty()
                && notebook.permissions.writable() =>
        {
            info!("Open tag deleting prompt.");
            State::TagDeleting(TagsDeletingStateData::empty(state_data))
        }
        KeyCode::Up if state_data.selected > 0 => State::TagsManaging(TagsManagingStateData {
            selected: state_data.selected - 1,
            ..state_data
        }),
        KeyCode::Down if state_data.selected < state_data.tags.len().saturating_sub(1) => {
            State::TagsManaging(TagsManagingStateData {
                selected: state_data.selected + 1,
                ..state_data
            })
        }
        KeyCode::Enter if !state_data.tags.is_empty() => {
            info!("Open tag notes listing.");
            let tag = state_data.tags.swap_remove(state_data.selected);

            State::TagNotesListing(TagNotesListingStateData::new(tag, notebook).await?)
        }
        KeyCode::Backspace if key_event.modifiers == KeyModifiers::NONE => {
            state_data.pattern.pop();
            state_data.tags = Tag::search_by_name(state_data.pattern.as_str(), notebook).await?;
            state_data.selected = 0;
            State::TagsManaging(state_data)
        }
        KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE && !c.is_whitespace() => {
            state_data.pattern.push(c);
            state_data.tags = Tag::search_by_name(state_data.pattern.as_str(), notebook).await?;
            state_data.selected = 0;
            State::TagsManaging(state_data)
        }
        _ => State::TagsManaging(state_data),
    })
}

pub fn draw_tags_managing_state(
    TagsManagingStateData {
        pattern,
        selected,
        tags,
        help_display,
    }: &TagsManagingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
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
        let writing_op_color = if notebook.permissions.writable() {
            Color::Blue
        } else {
            Color::Red
        };
        let (commands, commands_area) = create_help_bar(
            &[
                ("Ctrl+c", writing_op_color, "Create tag"),
                ("Ctrl+d", writing_op_color, "Delete tag"),
                ("⏎", Color::Blue, "List related notes"),
            ],
            3,
            main_rect,
        );

        frame.render_widget(Clear, commands_area);
        frame.render_widget(commands, commands_area);
    }
}
