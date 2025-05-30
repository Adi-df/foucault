use std::{
    io::stdout,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use log::info;
use scopeguard::defer;

use tokio::fs;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};

use foucault_core::PrettyError;

use crate::{
    helpers::create_left_help_bar,
    links::Link,
    markdown::{
        combine,
        elements::{InlineElements, SelectableInlineElements},
        lines, parse, Header, ParsedMarkdown,
    },
    note::Note,
    states::{
        note_deletion::NoteDeletionStateData, note_renaming::NoteRenamingStateData,
        note_tags_managing::NoteTagsManagingStateData, notes_managing::NotesManagingStateData,
        State,
    },
    tag::Tag,
    NotebookAPI, APP_DIR_PATH,
};

#[derive(Clone)]
pub struct NoteViewingStateData {
    pub note: Note,
    tags: Arc<[Tag]>,
    table_of_content: Arc<[Header]>,
    parsed_content: Arc<Mutex<ParsedMarkdown>>,
    selected: (usize, usize),
    help_display: bool,
    toc_display: bool,
}

impl NoteViewingStateData {
    pub async fn new(note: Note, notebook: &NotebookAPI) -> Result<Self> {
        let mut parsed_content = parse(note.content());
        parsed_content.select((0, 0), true);
        Ok(NoteViewingStateData {
            tags: note.tags(notebook).await?.into(),
            table_of_content: Arc::from(parsed_content.list_headers()),
            parsed_content: Arc::new(Mutex::new(parsed_content)),
            selected: (0, 0),
            help_display: false,
            toc_display: false,
            note,
        })
    }
}

impl NoteViewingStateData {
    fn re_parse_content(&mut self) {
        let content = parse(self.note.content());
        self.table_of_content = Arc::from(content.list_headers());
        self.parsed_content = Arc::new(Mutex::new(content));
    }

    fn get_current(&self) -> Option<SelectableInlineElements> {
        self.parsed_content
            .lock()
            .pretty_unwrap()
            .get_element(self.selected)
            .cloned()
    }

    fn select_current(&mut self, selected: bool) {
        self.parsed_content
            .lock()
            .pretty_unwrap()
            .select(self.selected, selected);
    }

    fn compute_links(&self) -> Vec<Link> {
        self.parsed_content
            .lock()
            .pretty_unwrap()
            .list_links()
            .into_iter()
            .map(|to| Link::new(self.note.id(), to.to_string()))
            .collect()
    }
}

pub async fn run_note_viewing_state(
    mut state_data: NoteViewingStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
    force_redraw: &mut bool,
) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Esc => {
            info!("Close note {}.", state_data.note.name());
            State::Nothing
        }
        KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle the help display.");
            state_data.help_display = !state_data.help_display;

            State::NoteViewing(state_data)
        }
        KeyCode::Char('t') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle the table of content display.");
            state_data.toc_display = !state_data.toc_display;

            State::NoteViewing(state_data)
        }
        KeyCode::Char('e') if notebook.permissions.writable() => {
            info!("Edit note {}.", state_data.note.name());
            edit_note(&mut state_data.note, notebook).await?;

            state_data.re_parse_content();
            state_data
                .note
                .update_links(&state_data.compute_links(), notebook)
                .await?;
            state_data.selected = (0, 0);
            state_data.select_current(true);
            *force_redraw = true;

            State::NoteViewing(state_data)
        }
        KeyCode::Char('s') => {
            info!("Open the notes manager.");
            State::NotesManaging(NotesManagingStateData::empty(notebook).await?)
        }
        KeyCode::Char('d') if notebook.permissions.writable() => {
            info!(
                "Open the deletion prompt for note {}.",
                state_data.note.name()
            );
            State::NoteDeletion(NoteDeletionStateData::from_note_viewing(state_data))
        }
        KeyCode::Char('r') => {
            info!(
                "Open the renaming prompt for note {}.",
                state_data.note.name()
            );
            State::NoteRenaming(NoteRenamingStateData::empty(state_data))
        }
        KeyCode::Char('t') => {
            info!("Open the tags manager for note {}", state_data.note.name());
            State::NoteTagsManaging(
                NoteTagsManagingStateData::new(state_data.note, notebook).await?,
            )
        }
        KeyCode::Enter => {
            info!("Try to trigger the selected element action.");
            if let Some(element) = state_data.get_current() {
                match <&InlineElements>::from(&element) {
                    InlineElements::HyperLink { dest, .. } => {
                        opener::open(dest.as_str())?;
                        State::NoteViewing(state_data)
                    }
                    InlineElements::CrossRef { dest, .. } => {
                        if let Some(note) = Note::load_by_name(dest.as_str(), notebook).await? {
                            State::NoteViewing(NoteViewingStateData::new(note, notebook).await?)
                        } else {
                            State::NoteViewing(state_data)
                        }
                    }
                    _ => State::NoteViewing(state_data),
                }
            } else {
                State::NoteViewing(state_data)
            }
        }
        KeyCode::Up | KeyCode::Char('k')
            if key_event.modifiers == KeyModifiers::NONE && state_data.selected.1 > 0 =>
        {
            state_data.select_current(false);
            state_data.selected.1 -= 1;
            state_data.selected.0 = state_data.selected.0.min(
                state_data
                    .parsed_content
                    .lock()
                    .pretty_unwrap()
                    .block_length(state_data.selected.1)
                    .saturating_sub(1),
            );
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Down | KeyCode::Char('j')
            if key_event.modifiers == KeyModifiers::NONE
                && state_data.selected.1
                    < state_data
                        .parsed_content
                        .lock()
                        .pretty_unwrap()
                        .block_count()
                        .saturating_sub(1) =>
        {
            state_data.select_current(false);
            state_data.selected.1 += 1;
            state_data.selected.0 = state_data.selected.0.min(
                state_data
                    .parsed_content
                    .lock()
                    .pretty_unwrap()
                    .block_length(state_data.selected.1)
                    .saturating_sub(1),
            );
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Left | KeyCode::Char('h')
            if key_event.modifiers == KeyModifiers::NONE && state_data.selected.0 > 0 =>
        {
            state_data.select_current(false);
            state_data.selected.0 -= 1;
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Right | KeyCode::Char('l')
            if key_event.modifiers == KeyModifiers::NONE
                && state_data.selected.0
                    < state_data
                        .parsed_content
                        .lock()
                        .pretty_unwrap()
                        .block_length(state_data.selected.1)
                        .saturating_sub(1) =>
        {
            state_data.select_current(false);
            state_data.selected.0 += 1;
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Up if key_event.modifiers == KeyModifiers::CONTROL => {
            let mut parsed_content = state_data.parsed_content.lock().pretty_unwrap();
            let current = parsed_content.related_header(state_data.selected.1);

            parsed_content.select(state_data.selected, false);

            if let Some(header) = current {
                let current_header_index = parsed_content.header_index(header).pretty_unwrap();
                let up_header = if current_header_index == state_data.selected.1 {
                    header.saturating_sub(1)
                } else {
                    header
                };
                if let Some(block) = parsed_content.header_index(up_header) {
                    state_data.selected = (0, block);
                }
            }

            parsed_content.select(state_data.selected, true);

            drop(parsed_content);
            State::NoteViewing(state_data)
        }
        KeyCode::Down if key_event.modifiers == KeyModifiers::CONTROL => {
            let mut parsed_content = state_data.parsed_content.lock().pretty_unwrap();
            let current = parsed_content.related_header(state_data.selected.1);

            parsed_content.select(state_data.selected, false);

            if let Some(header) = current {
                if let Some(block) = parsed_content.header_index(header + 1) {
                    state_data.selected = (0, block);
                }
            } else if let Some(block) = parsed_content.header_index(0) {
                state_data.selected = (0, block);
            }

            parsed_content.select(state_data.selected, true);

            drop(parsed_content);
            State::NoteViewing(state_data)
        }
        KeyCode::Char('g') => {
            state_data.select_current(false);
            state_data.selected = (0, 0);
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Char('E') => {
            state_data.select_current(false);
            state_data.selected.1 = state_data
                .parsed_content
                .lock()
                .pretty_unwrap()
                .block_count()
                .saturating_sub(1);
            state_data.selected.0 = state_data
                .parsed_content
                .lock()
                .pretty_unwrap()
                .block_length(state_data.selected.1)
                .saturating_sub(1);
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        _ => State::NoteViewing(state_data),
    })
}

async fn edit_note(note: &mut Note, notebook: &NotebookAPI) -> Result<()> {
    let tmp_file_path = APP_DIR_PATH.join(format!("{}.tmp.md", note.name()));
    note.export_content(tmp_file_path.as_path()).await?;

    stdout()
        .execute(LeaveAlternateScreen)
        .expect("Leave the foucault screen.");

    defer! {
        stdout().execute(EnterAlternateScreen).expect("Return to the foucault screen.");
    }

    let tmp_file_path_clone = tmp_file_path.clone();
    tokio::task::spawn_blocking(move || edit::edit_file(&tmp_file_path_clone)).await??;

    note.import_content(tmp_file_path.as_path(), notebook)
        .await?;

    fs::remove_file(&tmp_file_path).await?;
    Ok(())
}

pub fn draw_note_viewing_state(
    NoteViewingStateData {
        note,
        tags,
        table_of_content,
        parsed_content,
        selected,
        help_display,
        toc_display,
    }: &NoteViewingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let parsed_content = parsed_content.lock().pretty_unwrap();
    let [top_bar_rect, note_display_rect] = *TryInto::<&[Rect; 2]>::try_into(
        Layout::new(
            Direction::Vertical,
            [Constraint::Length(5), Constraint::Min(0)],
        )
        .split(main_rect)
        .as_ref(),
    )
    .pretty_unwrap();
    let [note_title_rect, note_tags_rect] = *TryInto::<&[Rect; 2]>::try_into(
        Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(30), Constraint::Min(0)],
        )
        .split(top_bar_rect)
        .as_ref(),
    )
    .pretty_unwrap();

    let (content_rect, toc_rect) = if *toc_display {
        let [toc_rect, content_rect] = *TryInto::<&[Rect; 2]>::try_into(
            Layout::new(
                Direction::Horizontal,
                [Constraint::Percentage(30), Constraint::Min(0)],
            )
            .split(note_display_rect)
            .as_ref(),
        )
        .pretty_unwrap();
        (content_rect, Some(toc_rect))
    } else {
        (note_display_rect, None)
    };

    let note_title = Paragraph::new(note.name())
        .style(Style::new().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(
            Block::new()
                .title("Title")
                .title_style(Style::new())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Green))
                .padding(Padding::uniform(1)),
        );
    let note_tags = Table::default()
        .rows([Row::new(tags.iter().map(|tag| {
            Text::raw(tag.name()).style(
                Style::new()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::from_u32(tag.color())),
            )
        }))])
        .widths([Constraint::Fill(1)].into_iter().cycle().take(tags.len()))
        .column_spacing(1)
        .block(
            Block::new()
                .title("Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Red))
                .padding(Padding::uniform(1)),
        );

    let mut headers = table_of_content
        .iter()
        .map(|header| Line::from(vec![header.build_span()]))
        .collect::<Vec<_>>();
    if let Some(selected) = parsed_content.related_header(selected.1) {
        headers[selected].style = headers[selected].style.bg(Color::Black);
    }
    let toc_widget = Paragraph::new(headers)
        .style(Style::new().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(
            Block::new()
                .title("Table of Content")
                .title_style(Style::new())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Blue))
                .padding(Padding::uniform(1)),
        );

    let content_block = Block::new()
        .title("Content")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let content_area = content_block.inner(content_rect);
    let rendered_content = parsed_content.render_blocks(content_area.width as usize);
    let scroll = lines(&rendered_content[..selected.1]);

    let note_content = combine(&rendered_content)
        .build_paragraph()
        .scroll((scroll.try_into().pretty_unwrap(), 0));

    let content_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(note_title, note_title_rect);
    frame.render_widget(note_tags, note_tags_rect);
    frame.render_widget(note_content, content_area);
    frame.render_widget(content_block, content_rect);
    frame.render_stateful_widget(
        content_scrollbar,
        content_rect.inner(Margin::new(0, 1)),
        &mut ScrollbarState::default()
            .content_length(parsed_content.block_count().saturating_sub(1))
            .viewport_content_length(1)
            .position(selected.1),
    );

    if *toc_display {
        frame.render_widget(toc_widget, toc_rect.pretty_unwrap());
    }

    if *help_display {
        let writing_op_color = if notebook.permissions.writable() {
            Color::Blue
        } else {
            Color::Red
        };
        let (commands, commands_area) = create_left_help_bar(
            &[
                ("e", writing_op_color, "Edit"),
                ("r", writing_op_color, "Rename"),
                ("d", writing_op_color, "Delete"),
                ("g", Color::Blue, "Go to note start"),
                ("E", Color::Blue, "Go to note end"),
                ("CTRL+t", Color::Blue, "Table of Content"),
                ("CTRL+UP", Color::Blue, "Previous heading"),
                ("CTRL+DOWN", Color::Blue, "Next heading"),
                ("t", Color::Blue, "Related Tags"),
                ("s", Color::Blue, "Manage notes"),
                ("⏎", Color::Blue, "Open link"),
            ],
            main_rect,
        );

        frame.render_widget(Clear, commands_area);
        frame.render_widget(commands, commands_area);
    }
}
