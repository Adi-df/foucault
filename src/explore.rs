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
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Row, Table, Wrap,
};
use ratatui::{Frame, Terminal};

use crate::helpers::{create_popup_proportion, create_popup_size, Capitalize};
use crate::note::{Note, NoteSummary};
use crate::notebook::Notebook;

#[derive(Debug)]
enum State {
    Nothing,
    NoteListing {
        pattern: String,
        selected: usize,
        results: Vec<NoteSummary>,
    },
    NoteViewing {
        note: Note,
    },
    NoteCreating {
        name: String,
    },
    NoteDeleting {
        note: Note,
        delete: bool,
    },
}

#[derive(Clone, Debug, Error)]
pub enum ExplorerError {
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

    loop {
        {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        state = match state {
                            State::Nothing => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('c') => {
                                    info!("Create new note.");
                                    State::NoteCreating {
                                        name: String::new(),
                                    }
                                }
                                KeyCode::Char('s') => {
                                    info!("List notes.");
                                    State::NoteListing {
                                        pattern: String::new(),
                                        selected: 0,
                                        results: notebook.search_name("")?,
                                    }
                                }
                                _ => state,
                            },
                            State::NoteCreating { mut name } => match key.code {
                                KeyCode::Enter => {
                                    info!("Complete note creation : {}.", name.as_str());

                                    let new_note = Note::new(
                                        name.clone(),
                                        Vec::new(),
                                        Vec::new(),
                                        String::new(),
                                    );
                                    new_note.insert(notebook.db())?;

                                    State::NoteViewing { note: new_note }
                                }
                                KeyCode::Esc => {
                                    info!("Cancel new note.");
                                    State::Nothing
                                }
                                KeyCode::Backspace => {
                                    name.pop();
                                    State::NoteCreating { name }
                                }
                                KeyCode::Char(c) => {
                                    name.push(c);
                                    State::NoteCreating { name }
                                }
                                _ => State::NoteCreating { name },
                            },
                            State::NoteDeleting { note, delete } => match key.code {
                                KeyCode::Esc => {
                                    info!("Cancel deleting");
                                    State::NoteViewing { note }
                                }
                                KeyCode::Tab => State::NoteDeleting {
                                    note,
                                    delete: !delete,
                                },
                                KeyCode::Enter => {
                                    if delete {
                                        note.delete(notebook.db())?;
                                        State::Nothing
                                    } else {
                                        State::NoteViewing { note }
                                    }
                                }
                                _ => State::NoteDeleting { note, delete },
                            },
                            State::NoteViewing { mut note } => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note viewing.");
                                    State::Nothing
                                }
                                KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('e') => {
                                    info!("Edit note {}", note.name);
                                    edit_note(&mut note, notebook)?;
                                    forced_redraw = true;
                                    State::NoteViewing { note }
                                }
                                KeyCode::Char('s') => {
                                    info!("List notes.");
                                    State::NoteListing {
                                        pattern: String::new(),
                                        selected: 0,
                                        results: notebook.search_name("")?,
                                    }
                                }
                                KeyCode::Char('d') => {
                                    info!("Not deleting prompt.");
                                    State::NoteDeleting {
                                        note,
                                        delete: false,
                                    }
                                }
                                _ => State::NoteViewing { note },
                            },
                            State::NoteListing {
                                pattern: mut search,
                                mut selected,
                                mut results,
                            } => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note searching.");
                                    State::Nothing
                                }
                                KeyCode::Up if selected > 0 => State::NoteListing {
                                    pattern: search,
                                    selected: selected - 1,
                                    results,
                                },
                                KeyCode::Down if selected < results.len() - 1 => {
                                    State::NoteListing {
                                        pattern: search,
                                        selected: selected + 1,
                                        results,
                                    }
                                }
                                KeyCode::Enter if !results.is_empty() => {
                                    let note_summary = results
                                        .get(selected)
                                        .ok_or(ExplorerError::NoNoteSelected)?;

                                    info!("Open note {}", note_summary.name);

                                    State::NoteViewing {
                                        note: Note::load(note_summary.id, notebook.db())?,
                                    }
                                }
                                KeyCode::Backspace => {
                                    search.pop();
                                    State::NoteListing {
                                        pattern: search,
                                        selected,
                                        results,
                                    }
                                }
                                KeyCode::Char(c) => {
                                    search.push(c);
                                    selected = 0;
                                    results = notebook.search_name(search.as_str())?;
                                    State::NoteListing {
                                        pattern: search,
                                        selected,
                                        results,
                                    }
                                }
                                _ => State::NoteListing {
                                    pattern: search,
                                    selected,
                                    results,
                                },
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
                State::NoteCreating { ref name } => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_new_note(frame, main_rect, name.as_str());
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteViewing { ref note } => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        // TODO : Render Markdown
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note.name.as_str(),
                            note.tags.as_slice(),
                            Paragraph::new(note.content.as_str()).wrap(Wrap { trim: true }),
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteDeleting { ref note, delete } => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        // TODO : Render Markdown
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note.name.as_str(),
                            note.tags.as_slice(),
                            Paragraph::new(note.content.as_str()).wrap(Wrap { trim: true }),
                        );
                        draw_deleting_popup(frame, main_rect, delete);

                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteListing {
                    pattern: ref search,
                    selected,
                    ref results,
                } => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_note_listing(
                            frame,
                            main_rect,
                            search.as_str(),
                            results.as_slice(),
                            selected,
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
            }
        }
    }

    Ok(())
}

fn draw_nothing(frame: &mut Frame, rect: Rect, name: &str) {
    let title = Paragraph::new(Line::from(vec![Span::raw(name.capitalize()).style(
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]))
    .alignment(Alignment::Center);

    frame.render_widget(title, create_popup_proportion((40, 10), rect));
}

fn draw_new_note(frame: &mut Frame, rect: Rect, entry: &str) {
    let new_note_entry = Paragraph::new(Line::from(vec![
        Span::raw(entry).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::default()
            .title("New note name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(new_note_entry, create_popup_size((30, 5), rect));
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
                .title_style(Style::default())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .padding(Padding::uniform(1)),
        );
    let note_tags = Table::default()
        .rows([Row::new(note_tags.iter().map(Text::raw))])
        .block(
            Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red)),
        );
    let note_content = note_content.block(
        Block::default()
            .title("Content")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(note_title, horizontal_layout[0]);
    frame.render_widget(note_tags, horizontal_layout[1]);
    frame.render_widget(note_content, vertical_layout[1]);
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
            .border_style(Style::default().fg(if search_note_result.is_empty() {
                Color::Red
            } else {
                Color::Green
            }))
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
            .border_style(Style::default().fg(Color::Yellow))
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

fn draw_deleting_popup(frame: &mut Frame, main_rect: Rect, note_delete_selected: bool) {
    let popup_area = create_popup_size((50, 5), main_rect);
    let block = Block::new()
        .title("Delete note ?")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(block.inner(popup_area));

    let yes = Paragraph::new(Line::from(vec![if note_delete_selected {
        Span::raw("Yes").add_modifier(Modifier::UNDERLINED)
    } else {
        Span::raw("Yes")
    }]))
    .style(Style::default().fg(Color::Green))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Green)),
    );
    let no = Paragraph::new(Line::from(vec![if note_delete_selected {
        Span::raw("No")
    } else {
        Span::raw("No").add_modifier(Modifier::UNDERLINED)
    }]))
    .style(Style::default().fg(Color::Red))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(yes, layout[0]);
    frame.render_widget(no, layout[1]);
    frame.render_widget(block, popup_area);
}
