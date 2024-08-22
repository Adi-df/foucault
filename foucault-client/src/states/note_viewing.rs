use std::{
    env,
    io::stdout,
    sync::{Arc, Mutex},
};

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

use foucault_core::PrettyError;

use crate::{
    helpers::create_help_bar,
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
        State,
    },
    tag::Tag,
    NotebookAPI, APP_DIR_PATH,
};

#[derive(Clone)]
pub struct NoteViewingStateData {
    pub note: Note,
    pub tags: Vec<Tag>,
    pub parsed_content: Arc<Mutex<ParsedMarkdown>>,
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
            parsed_content: Arc::new(Mutex::new(parsed_content)),
            selected: (0, 0),
            help_display: false,
        })
    }
}

impl NoteViewingStateData {
    fn re_parse_content(&mut self) {
        self.parsed_content = Arc::new(Mutex::new(parse(self.note.content())));
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
            info!("Enter notes listing.");
            State::NotesManaging(NotesManagingStateData::empty(notebook).await?)
        }
        KeyCode::Char('d') if notebook.permissions.writable() => {
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
        KeyCode::Up | KeyCode::Char('k') if state_data.selected.1 > 0 => {
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
            if state_data.selected.1
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
    NoteViewingStateData {
        note,
        tags,
        parsed_content,
        selected,
        help_display,
    }: &NoteViewingStateData,
    notebook: &NotebookAPI,
    frame: &mut Frame,
    main_rect: Rect,
) {
    let parsed_content = parsed_content.lock().pretty_unwrap();
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
        .scroll((scroll.try_into().pretty_unwrap(), 0));

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
        let writing_op_color = if notebook.permissions.writable() {
            Color::Blue
        } else {
            Color::Red
        };
        let (commands, commands_area) = create_help_bar(
            &[
                ("e", writing_op_color, "Edit"),
                ("d", writing_op_color, "Delete"),
                ("r", writing_op_color, "Rename"),
                ("s", Color::Blue, "List notes"),
                ("t", Color::Blue, "Tags"),
                ("g", Color::Blue, "Go to note start"),
                ("E", Color::Blue, "Go to note end"),
                ("⏎", Color::Blue, "Open link"),
            ],
            4,
            main_rect,
        );

        frame.render_widget(Clear, commands_area);
        frame.render_widget(commands, commands_area);
    }
}
