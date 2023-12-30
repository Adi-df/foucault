use std::io::stdout;
use std::process::Command;
use std::{env, fs};

use anyhow::Result;
use log::info;
use rusqlite::Connection;
use scopeguard::defer;

use crossterm::event::KeyCode;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{
    Block, BorderType, Borders, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Table,
};
use ratatui::Frame;

use crate::helpers::{DiscardResult, TryFromDatabase};
use crate::markdown::elements::{InlineElements, SelectableInlineElements};
use crate::markdown::{combine, lines, parse, ParsedMarkdown};
use crate::note::{Note, NoteData};
use crate::notebook::Notebook;
use crate::states::note_deleting::NoteDeletingStateData;
use crate::states::note_renaming::NoteRenamingStateData;
use crate::states::note_tags_managing::NoteTagsManagingStateData;
use crate::states::notes_managing::NotesManagingStateData;
use crate::states::{State, Terminal};

pub struct NoteViewingStateData {
    pub note_data: NoteData,
    pub parsed_content: ParsedMarkdown,
    pub selected: (usize, usize),
}

impl From<NoteData> for NoteViewingStateData {
    fn from(note_data: NoteData) -> Self {
        let mut parsed_content = parse(note_data.note.content.as_str());
        parsed_content.select((0, 0), true);
        NoteViewingStateData {
            note_data,
            parsed_content,
            selected: (0, 0),
        }
    }
}

impl TryFromDatabase<Note> for NoteViewingStateData {
    fn try_from_database(note: Note, db: &Connection) -> Result<Self> {
        Ok(NoteViewingStateData::from(NoteData::try_from_database(
            note, db,
        )?))
    }
}

impl NoteViewingStateData {
    fn re_parse_content(&mut self) {
        self.parsed_content = parse(self.note_data.note.content.as_str());
    }
    fn get_current(&self) -> Option<&SelectableInlineElements> {
        self.parsed_content.get_element(self.selected)
    }
    fn select_current(&mut self, selected: bool) {
        self.parsed_content.select(self.selected, selected);
    }

    fn compute_links(&self, db: &Connection) -> Result<Vec<i64>> {
        self.parsed_content
            .list_links()
            .into_iter()
            .filter_map(|link| Note::get_id_by_name(link, db).transpose())
            .collect::<Result<Vec<i64>>>()
    }
    fn update_links(&mut self, db: &Connection) -> Result<()> {
        let computed_links = self.compute_links(db)?;

        let removed: Vec<i64> = self
            .note_data
            .links
            .iter()
            .copied()
            .filter(|link| !computed_links.contains(link))
            .collect();

        for link in removed {
            self.note_data.remove_link(link, db)?;
        }

        let added: Vec<i64> = computed_links
            .into_iter()
            .filter(|link| !self.note_data.links.contains(link))
            .collect();

        for link in added {
            self.note_data.add_link(link, db)?;
        }

        Ok(())
    }
}

pub fn run_note_viewing_state(
    mut state_data: NoteViewingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
    force_redraw: &mut bool,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Stop viewing of note {}.", state_data.note_data.note.name);
            State::Nothing
        }
        KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Char('e') => {
            info!("Edit note {}", state_data.note_data.note.name);
            edit_note(&mut state_data.note_data.note, notebook)?;

            state_data.re_parse_content();
            state_data.update_links(notebook.db())?;
            state_data.selected = (0, 0);
            state_data.select_current(true);
            *force_redraw = true;

            State::NoteViewing(state_data)
        }
        KeyCode::Char('s') => {
            info!("Enter notes listing.");
            State::NotesManaging(NotesManagingStateData::empty(notebook.db())?)
        }
        KeyCode::Char('d') => {
            info!(
                "Open deleting prompt for note {}.",
                state_data.note_data.note.name
            );
            State::NoteDeleting(NoteDeletingStateData::empty(state_data))
        }
        KeyCode::Char('r') => {
            info!(
                "Open renaming prompt for note {}.",
                state_data.note_data.note.name
            );
            State::NoteRenaming(NoteRenamingStateData::empty(state_data))
        }
        KeyCode::Char('t') => {
            info!(
                "Open tags manager for note {}",
                state_data.note_data.note.name
            );
            State::NoteTagsManaging(NoteTagsManagingStateData::from(state_data.note_data))
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
                        if let Some(note) = Note::load_by_name(dest.as_str(), notebook.db())? {
                            State::NoteViewing(NoteViewingStateData::try_from_database(
                                note,
                                notebook.db(),
                            )?)
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
        KeyCode::Up if state_data.selected.1 > 0 => {
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
        KeyCode::Down
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
        KeyCode::Left if state_data.selected.0 > 0 => {
            state_data.select_current(false);
            state_data.selected.0 -= 1;
            state_data.select_current(true);
            State::NoteViewing(state_data)
        }
        KeyCode::Right
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
        _ => State::NoteViewing(state_data),
    })
}

fn edit_note(note: &mut Note, notebook: &Notebook) -> Result<()> {
    let tmp_file_path = notebook
        .dir()
        .unwrap()
        .join(format!("{}.tmp.md", note.name));
    note.export_content(tmp_file_path.as_path())?;

    let editor = env::var("EDITOR")?;

    stdout()
        .execute(LeaveAlternateScreen)
        .expect("Leave foucault screen");

    defer! {
        stdout().execute(EnterAlternateScreen).expect("Return to foucault");
    }

    Command::new(editor)
        .args([&tmp_file_path])
        .current_dir(notebook.dir().unwrap())
        .status()?;

    note.import_content(tmp_file_path.as_path())?;
    note.update(notebook.db())?;

    fs::remove_file(&tmp_file_path)?;
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
        note_data: NoteData { note, tags, .. },
        parsed_content,
        selected,
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

    let note_title = Paragraph::new(note.name.as_str())
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Title")
                .title_style(Style::default())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .padding(Padding::uniform(1)),
        );
    let note_tags = Table::default()
        .rows([Row::new(tags.iter().map(|el| Text::raw(el.name.as_str())))])
        .widths(
            [if tags.is_empty() {
                Constraint::Min(0)
            } else {
                Constraint::Percentage(100 / u16::try_from(tags.len()).unwrap())
            }]
            .into_iter()
            .cycle()
            .take(tags.len()),
        )
        .column_spacing(1)
        .block(
            Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
                .padding(Padding::uniform(1)),
        );

    let content_block = Block::default()
        .title("Content")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let content_area = content_block.inner(vertical_layout[1]);
    let rendered_content = parsed_content.render_blocks();
    let scroll = lines(&rendered_content[..selected.1], content_area.width);

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
}
