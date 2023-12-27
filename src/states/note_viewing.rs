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

use crate::helpers::{DiscardResult, TryFromDatabase, TryIntoDatabase};
use crate::markdown::{lines, parse, render, ParsedMarkdown};
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
    pub scroll: usize,
}

impl TryFromDatabase<Note> for NoteViewingStateData {
    fn try_from_database(note: Note, db: &Connection) -> Result<Self> {
        Ok(NoteViewingStateData {
            parsed_content: parse(note.content.as_str()),
            note_data: note.try_into_database(db)?,
            scroll: 0,
        })
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
            info!("Stop note viewing.");
            State::Nothing
        }
        KeyCode::Char('q') => {
            info!("Quit notebook.");
            State::Exit
        }
        KeyCode::Char('e') => {
            info!("Edit note {}", state_data.note_data.note.name);
            edit_note(&mut state_data.note_data.note, notebook)?;
            state_data.parsed_content = parse(state_data.note_data.note.content.as_str());
            *force_redraw = true;
            State::NoteViewing(state_data)
        }
        KeyCode::Char('s') => {
            info!("List notes.");
            State::NotesManaging(NotesManagingStateData::empty(notebook.db())?)
        }
        KeyCode::Char('d') => {
            info!("Not deleting prompt.");
            State::NoteDeleting(NoteDeletingStateData::empty(state_data))
        }
        KeyCode::Char('r') => {
            info!("Prompt note new name");
            State::NoteRenaming(NoteRenamingStateData::empty(state_data))
        }
        KeyCode::Char('t') => {
            info!("Manage tags of note {}", state_data.note_data.note.name);
            State::NoteTagsManaging(NoteTagsManagingStateData::try_from_database(
                state_data.note_data.note,
                notebook.db(),
            )?)
        }
        KeyCode::Up => {
            state_data.scroll = state_data.scroll.saturating_sub(1);
            State::NoteViewing(state_data)
        }
        KeyCode::Down => {
            state_data.scroll = state_data.scroll.saturating_add(1);
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
        scroll,
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

    let content_len = lines(
        parsed_content,
        content_block.inner(vertical_layout[1]).width,
    );
    let scroll = if content_len == 0 {
        0
    } else {
        scroll.rem_euclid(content_len)
    };

    let note_content = render(parsed_content).scroll((scroll.try_into().unwrap(), 0));

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
        &mut ScrollbarState::new(content_len).position(scroll),
    );
}
