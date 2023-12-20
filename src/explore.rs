use std::io::{stdout, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use log::info;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::{Frame, Terminal};
use thiserror::Error;

use rusqlite::Connection;

use crate::note::Note;
use crate::notebook::Notebook;

#[derive(Clone, Copy, Debug)]
enum State {
    Nothing,
    NoteListing,
    NoteViewing,
    NoteCreating,
    NoteEditing,
}

#[derive(Clone, Debug, Error)]
pub enum ExplorerError {
    #[error("No new note name. Should be unreachable.")]
    NoNewNoteName,
}

pub fn explore(notebook: Notebook) -> Result<()> {
    info!("Explore notebook : {}", notebook.name);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let res = run(notebook, terminal);

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    res
}

fn run(notebook: Notebook, mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut state = State::Nothing;
    let mut openened_note: Option<Note> = None;
    let mut new_note_name: Option<String> = None;

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
                                    new_note_name = Some(String::new());
                                }
                                _ => {}
                            },
                            State::NoteCreating => match key.code {
                                KeyCode::Enter => {
                                    info!(
                                        "Complete note creation : {}.",
                                        new_note_name
                                            .as_ref()
                                            .ok_or(ExplorerError::NoNewNoteName)?
                                    );

                                    let new_note = Note::new(
                                        new_note_name
                                            .as_ref()
                                            .ok_or(ExplorerError::NoNewNoteName)?
                                            .clone(),
                                        Vec::new(),
                                        Vec::new(),
                                        String::new(),
                                    );
                                    new_note.insert(notebook.db())?;

                                    state = State::NoteViewing;
                                    openened_note = Some(new_note);
                                }
                                KeyCode::Esc => {
                                    info!("Cancel new note.");
                                    state = State::Nothing;
                                    new_note_name = None;
                                }
                                KeyCode::Backspace => {
                                    new_note_name
                                        .as_mut()
                                        .ok_or(ExplorerError::NoNewNoteName)?
                                        .pop();
                                }
                                KeyCode::Char(c) => {
                                    new_note_name
                                        .as_mut()
                                        .ok_or(ExplorerError::NoNewNoteName)?
                                        .push(c);
                                }
                                _ => {}
                            },
                            State::NoteViewing => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note viewing.");
                                    state = State::Nothing;
                                    openened_note = None;
                                }
                                KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
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
                    let entry_name = new_note_name.as_ref().ok_or(ExplorerError::NoNewNoteName)?;

                    terminal.draw(|mut frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_new_note(&mut frame, main_rect, entry_name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw_nothing(frame: &mut Frame, rect: Rect, name: &str) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(10),
            Constraint::Percentage(45),
        ],
    )
    .split(rect);

    let title = Paragraph::new(name)
        .style(Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD))
        .alignment(Alignment::Center);

    frame.render_widget(title, vertical_layout[1]);
}

fn draw_new_note(frame: &mut Frame, rect: Rect, entry: &str) {
    let vertical_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(10),
            Constraint::Percentage(45),
        ],
    )
    .split(rect);

    let new_note_entry =
        Paragraph::new(entry).style(Style::default().add_modifier(Modifier::UNDERLINED));

    frame.render_widget(new_note_entry, vertical_layout[1]);
}
