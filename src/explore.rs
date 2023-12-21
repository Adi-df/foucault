use std::io::stdout;
use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use anyhow::Result;
use log::info;
use scopeguard::defer;
use thiserror::Error;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Table};
use ratatui::{Frame, Terminal};

use crate::helpers::{create_popup, OptionalValue};
use crate::note::Note;
use crate::notebook::Notebook;

#[derive(Clone, Copy, Debug)]
enum State {
    Nothing,
    NoteListing,
    NoteViewing,
    NoteCreating,
}

#[derive(Clone, Debug, Error)]
pub enum ExplorerError {
    #[error("No new note name. Should be unreachable.")]
    NoNewNoteName,
    #[error("No note openend. Should be unreachable.")]
    NoNoteOpened,
}

pub fn explore(notebook: Notebook) -> Result<()> {
    info!("Explore notebook : {}", notebook.name);

    enable_raw_mode().expect("Prepare terminal");
    stdout()
        .execute(EnterAlternateScreen)
        .expect("Prepare terminal");

    defer! {
        stdout().execute(LeaveAlternateScreen).expect("Reset terminal");
        disable_raw_mode().expect("Reset terminal");
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut forced_redraw = false;

    let mut state = State::Nothing;
    let mut openened_note: OptionalValue<Note, ExplorerError> =
        OptionalValue::new(None, ExplorerError::NoNoteOpened);
    let mut new_note_name: OptionalValue<String, ExplorerError> =
        OptionalValue::new(None, ExplorerError::NoNewNoteName);

    loop {
        {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match state {
                            State::Nothing => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('c') => {
                                    info!("Create new note.");
                                    state = State::NoteCreating;
                                    new_note_name.set(Some(String::new()));
                                }
                                _ => {}
                            },
                            State::NoteCreating => match key.code {
                                KeyCode::Enter => {
                                    info!("Complete note creation : {}.", new_note_name.by_ref()?);

                                    let new_note = Note::new(
                                        new_note_name.by_ref()?.clone(),
                                        Vec::new(),
                                        Vec::new(),
                                        String::new(),
                                    );
                                    new_note.insert(notebook.db())?;

                                    state = State::NoteViewing;
                                    openened_note.set(Some(new_note));
                                }
                                KeyCode::Esc => {
                                    info!("Cancel new note.");
                                    state = State::Nothing;
                                    new_note_name.set(None);
                                }
                                KeyCode::Backspace => {
                                    new_note_name.by_ref_mut()?.pop();
                                }
                                KeyCode::Char(c) => {
                                    new_note_name.by_ref_mut()?.push(c);
                                }
                                _ => {}
                            },
                            State::NoteViewing => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note viewing.");
                                    state = State::Nothing;
                                    openened_note.set(None);
                                }
                                KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('e') => {
                                    info!("Edit note {}", openened_note.by_ref()?.name);
                                    edit_note(openened_note.by_ref_mut()?, &notebook)?;
                                    forced_redraw = true;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        {
            if forced_redraw {
                terminal.draw(|frame| frame.render_widget(Clear, frame.size()))?;
            }
            forced_redraw = false;

            let main_frame = Block::default()
                .title(notebook.name.as_str())
                .padding(Padding::uniform(1))
                .borders(Borders::all())
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White));

            match state {
                State::Nothing => {
                    terminal.draw(|mut frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_nothing(&mut frame, main_rect, notebook.name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteCreating => {
                    let entry_name = new_note_name.by_ref()?;

                    terminal.draw(|mut frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_new_note(&mut frame, main_rect, entry_name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteViewing => {
                    let note_name = openened_note.by_ref()?.name.as_str();
                    let note_content = openened_note.by_ref()?.content.as_str();
                    let note_tags = &openened_note.by_ref()?.tags;

                    terminal.draw(|mut frame| {
                        let main_rect = main_frame.inner(frame.size());
                        // TODO : Render Markdown
                        draw_viewed_note(
                            &mut frame,
                            main_rect,
                            note_name,
                            note_tags,
                            Paragraph::new(note_content),
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                _ => {}
            }
        }
    }

    Ok(())
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

fn draw_nothing(frame: &mut Frame, rect: Rect, name: &str) {
    let title = Paragraph::new(name)
        .style(Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD))
        .alignment(Alignment::Center);

    frame.render_widget(title, create_popup((40, 10), rect));
}

fn draw_new_note(frame: &mut Frame, rect: Rect, entry: &str) {
    let new_note_entry = Paragraph::new(entry).style(Style::default()).block(
        Block::default()
            .title("New note name")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(new_note_entry, create_popup((50, 20), rect));
}

fn draw_viewed_note(
    frame: &mut Frame,
    main_rect: Rect,
    note_title: &str,
    note_tags: &[String],
    note_content: Paragraph,
) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);
    let horizontal_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(40), Constraint::Min(0)],
    )
    .split(vertical_layout[0]);

    let note_title = Paragraph::new(note_title)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Title")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .padding(Padding::uniform(1)),
        );
    let note_tags = Table::default()
        .rows([Row::new(note_tags.into_iter().map(|tag| Text::raw(tag)))])
        .block(
            Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded),
        );
    let note_content = note_content.block(
        Block::default()
            .title("Content")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(note_title, horizontal_layout[0]);
    frame.render_widget(note_tags, horizontal_layout[1]);
    frame.render_widget(note_content, vertical_layout[1]);
}
