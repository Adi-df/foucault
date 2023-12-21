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
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Row, Table,
};
use ratatui::{Frame, Terminal};

use crate::helpers::{create_popup, OptionalValue};
use crate::note::{Note, NoteSummary};
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
    #[error("No note openend. Should be unreachable.")]
    NoNoteOpened,
    #[error("No note selected. Should be unreachable.")]
    NoNoteSelected,
}

pub fn explore(notebook: &Notebook) -> Result<()> {
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
    let mut openened_note = OptionalValue::new(None, ExplorerError::NoNoteOpened);
    let mut new_note_name = String::new();
    let mut search_note_name = String::new();
    let mut search_note_selected: usize = 0;
    let mut search_note_result: Vec<NoteSummary> = Vec::new();

    loop {
        {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        info!(
                            "Available rows {}, selected {}",
                            search_note_result.len(),
                            search_note_selected
                        );
                        match state {
                            State::Nothing => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('c') => {
                                    info!("Create new note.");
                                    state = State::NoteCreating;
                                    new_note_name = String::new();
                                }
                                KeyCode::Char('s') => {
                                    info!("List notes.");
                                    state = State::NoteListing;
                                    search_note_name = String::new();
                                    search_note_selected = 0;
                                    search_note_result = notebook.search_name("")?;
                                }
                                _ => {}
                            },
                            State::NoteCreating => match key.code {
                                KeyCode::Enter => {
                                    info!("Complete note creation : {}.", new_note_name.as_str());

                                    let new_note = Note::new(
                                        new_note_name.clone(),
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
                                    new_note_name = String::new();
                                }
                                KeyCode::Backspace => {
                                    new_note_name.pop();
                                }
                                KeyCode::Char(c) => {
                                    new_note_name.push(c);
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
                                    edit_note(openened_note.by_ref_mut()?, notebook)?;
                                    forced_redraw = true;
                                }
                                _ => {}
                            },
                            State::NoteListing => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note searching.");
                                    state = State::Nothing;
                                    search_note_name = String::new();
                                    search_note_selected = 0;
                                }
                                KeyCode::Up if search_note_selected > 0 => {
                                    search_note_selected -= 1;
                                }
                                KeyCode::Down
                                    if search_note_selected < search_note_result.len() - 1 =>
                                {
                                    search_note_selected += 1;
                                }
                                KeyCode::Enter if !search_note_result.is_empty() => {
                                    let note_summary = search_note_result
                                        .get(search_note_selected)
                                        .ok_or(ExplorerError::NoNoteSelected)?;

                                    info!("Open note {}", note_summary.name);

                                    state = State::NoteViewing;
                                    openened_note
                                        .set(Some(Note::load(note_summary.id, notebook.db())?));
                                    search_note_selected = 0;
                                    search_note_name = String::new();
                                    search_note_result = Vec::new();
                                }
                                KeyCode::Backspace => {
                                    search_note_selected = 0;
                                    search_note_name.pop();
                                    search_note_result =
                                        notebook.search_name(search_note_name.as_str())?;
                                }
                                KeyCode::Char(c) => {
                                    search_note_selected = 0;
                                    search_note_name.push(c);
                                    search_note_result =
                                        notebook.search_name(search_note_name.as_str())?;
                                }
                                _ => {}
                            },
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
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_nothing(frame, main_rect, notebook.name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteCreating => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_new_note(frame, main_rect, new_note_name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteViewing => {
                    let note_name = openened_note.by_ref()?.name.as_str();
                    let note_content = openened_note.by_ref()?.content.as_str();
                    let note_tags = &openened_note.by_ref()?.tags;

                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        // TODO : Render Markdown
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note_name,
                            note_tags,
                            Paragraph::new(note_content),
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteListing => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_note_listing(
                            frame,
                            main_rect,
                            &search_note_name,
                            &search_note_result,
                            search_note_selected,
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
            }
        }
    }

    Ok(())
}

fn draw_note_listing(
    frame: &mut Frame,
    main_rect: Rect,
    search_note_name: &str,
    search_note_result: &[NoteSummary],
    search_note_selected: usize,
) {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let search_bar = Paragraph::new(Line::from(vec![
        Span::raw(search_note_name).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::new()
            .title("Searching")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1)),
    );

    let list_results = List::new(
        search_note_result
            .iter()
            .map(|note| Span::raw(note.name.as_str())),
    )
    .highlight_symbol(">> ")
    .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
    .block(
        Block::new()
            .title("Results")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(2)),
    );

    frame.render_widget(search_bar, layout[0]);
    frame.render_stateful_widget(
        list_results,
        layout[1],
        &mut ListState::with_selected(ListState::default(), Some(search_note_selected)),
    );
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
        .rows([Row::new(note_tags.iter().map(Text::raw))])
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
