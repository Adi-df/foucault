use std::{env, io::stdout};

use anyhow::Result;
use log::info;
use scopeguard::defer;

use tokio::{fs, process::Command};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};

use crate::{
    helpers::{create_bottom_line, create_row_help_layout, DiscardResult},
    links::Link,
    markdown::{
        combine,
        elements::{InlineElements, SelectableInlineElements},
        lines, parse, ParsedMarkdown,
    },
    note::Note,
    states::{
        note_deleting::NoteDeletingStateData, note_renaming::NoteRenamingStateData,
        note_tags_managing::NoteTagsManagingStateData, notes_managing::NotesManagingStateData,
        State, Terminal,
    },
    tag::Tag,
    NotebookAPI, APP_DIR_PATH,
};

pub struct NoteViewingStateData {
    pub note: Note,
    pub tags: Vec<Tag>,
    pub parsed_content: ParsedMarkdown,
    pub selected: (usize, usize),
    pub help_display: bool,
}

impl NoteViewingStateData {
    pub async fn new(note: Note, notebook: &NotebookAPI) -> Result<Self> {
        let mut parsed_content = parse(note.content());
        parsed_content.select((0, 0), true);
        Ok(NoteViewingStateData {
            tags: note.tags(notebook).await?,
            note,
            parsed_content,
            selected: (0, 0),
            help_display: false,
        })
    }
}

impl NoteViewingStateData {
    fn re_parse_content(&mut self) {
        self.parsed_content = parse(self.note.content());
    }
    fn get_current(&self) -> Option<&SelectableInlineElements> {
        self.parsed_content.get_element(self.selected)
    }
    fn select_current(&mut self, selected: bool) {
        self.parsed_content.select(self.selected, selected);
    }

    fn compute_links(&self) -> Vec<Link> {
        self.parsed_content
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
            info!("Stop viewing of note {}.", state_data.note.name());
            State::Nothing
        }
        KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
            info!("Toogle help display.");
            state_data.help_display = !state_data.help_display;

            State::NoteViewing(state_data)
        }
        KeyCode::Char('e') => {
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
            info!("Enter notes listing.");
            State::NotesManaging(NotesManagingStateData::empty(notebook).await?)
        }
        KeyCode::Char('d') => {
            info!("Open deleting prompt for note {}.", state_data.note.name());
            State::NoteDeleting(NoteDeletingStateData::empty(state_data))
        }
        KeyCode::Char('r') => {
            info!("Open renaming prompt for note {}.", state_data.note.name());
            State::NoteRenaming(NoteRenamingStateData::empty(state_data))
        }
        KeyCode::Char('t') => {
            info!("Open tags manager for note {}", state_data.note.name());
            State::NoteTagsManaging(
                NoteTagsManagingStateData::new(state_data.note, notebook).await?,
            )
        }
        KeyCode::Enter => {
            info!("Try to trigger element action.");
            if let Some(element) = state_data.get_current() {
                match <&InlineElements>::from(element) {
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
        KeyCode::Up | KeyCode::Char('k') if state_data.selected.1 > 0 => {
            state_data.select_current(false);
            state_data.selected.1 -= 1;
            state_data.selected.0 = state_data.selected.0.min(
                state_data
                    .parsed_content
                    .block_length(state_data.selected.1)
                    .saturating_sub(1),
            );
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Down | KeyCode::Char('j')
            if state_data.selected.1
                < state_data.parsed_content.block_count().saturating_sub(1) =>
        {
            state_data.select_current(false);
            state_data.selected.1 += 1;
            state_data.selected.0 = state_data.selected.0.min(
                state_data
                    .parsed_content
                    .block_length(state_data.selected.1)
                    .saturating_sub(1),
            );
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Left | KeyCode::Char('h') if state_data.selected.0 > 0 => {
            state_data.select_current(false);
            state_data.selected.0 -= 1;
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Right | KeyCode::Char('l')
            if state_data.selected.0
                < state_data
                    .parsed_content
                    .block_length(state_data.selected.1)
                    .saturating_sub(1) =>
        {
            state_data.select_current(false);
            state_data.selected.0 += 1;
            state_data.select_current(true);
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
            state_data.selected.1 = state_data.parsed_content.block_count().saturating_sub(1);
            state_data.selected.0 = state_data
                .parsed_content
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

    let editor = env::var("EDITOR")?;

    stdout()
        .execute(LeaveAlternateScreen)
        .expect("Leave foucault screen.");

    defer! {
        stdout().execute(EnterAlternateScreen).expect("Return to foucault.");
    }

    Command::new(editor)
        .args([&tmp_file_path])
        .current_dir(&*APP_DIR_PATH)
        .status()
        .await?;

    note.import_content(tmp_file_path.as_path(), notebook)
        .await?;

    fs::remove_file(&tmp_file_path).await?;
    Ok(())
}

pub fn draw_note_viewing_state(
    state_data: &NoteViewingStateData,
    terminal: &mut Terminal,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, state_data, main_rect);

            frame.render_widget(main_frame, frame.size());
        })
        .discard_result()
}

pub fn draw_viewed_note(
    frame: &mut Frame,
    NoteViewingStateData {
        note,
        tags,
        parsed_content,
        selected,
        help_display,
    }: &NoteViewingStateData,
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

    let content_block = Block::new()
        .title("Content")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let content_area = content_block.inner(vertical_layout[1]);
    let rendered_content = parsed_content.render_blocks(content_area.width as usize);
    let scroll = lines(&rendered_content[..selected.1]);

    let note_content = combine(&rendered_content)
        .build_paragraph()
        .scroll((scroll.try_into().unwrap(), 0));

    let content_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(note_title, horizontal_layout[0]);
    frame.render_widget(note_tags, horizontal_layout[1]);
    frame.render_widget(note_content, content_block.inner(vertical_layout[1]));
    frame.render_widget(content_block, vertical_layout[1]);
    frame.render_stateful_widget(
        content_scrollbar,
        vertical_layout[1].inner(&Margin::new(0, 1)),
        &mut ScrollbarState::default()
            .content_length(parsed_content.block_count().saturating_sub(1))
            .viewport_content_length(1)
            .position(selected.1),
    );

    if *help_display {
        let command_area = create_bottom_line(main_rect);
        let commands = create_row_help_layout(&[
            ("e", "Edit"),
            ("s", "List notes"),
            ("d", "Delete"),
            ("t", "Tags"),
            ("r", "Rename"),
            ("⏎", "Open link"),
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
