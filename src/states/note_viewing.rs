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

use crate::helpers::{TryFromDatabase, TryIntoDatabase};
use crate::markdown::{lines, parse, render};
use crate::note::{Note, NoteData};
use crate::notebook::Notebook;
use crate::states::note_deleting::NoteDeletingStateData;
use crate::states::note_renaming::NoteRenamingStateData;
use crate::states::note_tags_managing::NoteTagsManagingStateData;
use crate::states::notes_managing::NotesManagingStateData;
use crate::states::{State, Terminal};

#[derive(Debug)]
pub struct NoteViewingStateData {
    pub note_data: NoteData,
    pub scroll: usize,
}

impl TryFromDatabase<Note> for NoteViewingStateData {
    fn try_from_database(note: Note, db: &Connection) -> Result<Self> {
        Ok(NoteViewingStateData {
            note_data: note.try_into_database(db)?,
            scroll: 0,
        })
    }
}

pub fn run_note_viewing_state(
    NoteViewingStateData {
        mut note_data,
        scroll,
    }: NoteViewingStateData,
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
            info!("Edit note {}", note_data.note.name);
            edit_note(&mut note_data.note, notebook)?;
            *force_redraw = true;
            State::NoteViewing(NoteViewingStateData { note_data, scroll })
        }
        KeyCode::Char('s') => {
            info!("List notes.");
            State::NotesManaging(NotesManagingStateData {
                pattern: String::new(),
                selected: 0,
                notes: Note::search_by_name("", notebook.db())?,
            })
        }
        KeyCode::Char('d') => {
            info!("Not deleting prompt.");
            State::NoteDeleting(NoteDeletingStateData {
                viewing_data: NoteViewingStateData { note_data, scroll },
                delete: false,
            })
        }
        KeyCode::Char('r') => {
            info!("Prompt note new name");
            State::NoteRenaming(NoteRenamingStateData {
                viewing_data: NoteViewingStateData { note_data, scroll },
                new_name: String::new(),
            })
        }
        KeyCode::Char('t') => {
            info!("Manage tags of note {}", note_data.note.name);
            State::NoteTagsManaging(NoteTagsManagingStateData {
                selected: 0,
                tags: note_data.note.get_tags(notebook.db())?,
                note: note_data.note,
            })
        }
        KeyCode::Up => State::NoteViewing(NoteViewingStateData {
            note_data,
            scroll: scroll.saturating_sub(1),
        }),
        KeyCode::Down => State::NoteViewing(NoteViewingStateData {
            note_data,
            scroll: scroll.saturating_add(1),
        }),
        _ => State::NoteViewing(NoteViewingStateData { note_data, scroll }),
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
        .map_err(anyhow::Error::from)
        .map(|_| ())
}

pub fn draw_viewed_note(
    frame: &mut Frame,
    NoteViewingStateData {
        note_data: NoteData { note, tags, .. },
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

    let parsed_content = parse(note.content.as_str());
    let content_len = lines(
        &parsed_content,
        content_block.inner(vertical_layout[1]).width,
    );
    let scroll = if content_len == 0 {
        0
    } else {
        scroll.rem_euclid(content_len)
    };

    let note_content = render(&parsed_content).scroll((scroll.try_into().unwrap(), 0));

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
