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
    Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Row, Table,
};
use ratatui::{Frame, Terminal};

use crate::helpers::{create_popup_proportion, create_popup_size, Capitalize};
use crate::markdown::render;
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
        note_data: NoteData,
    },
    NoteCreating {
        name: String,
    },
    NoteDeleting {
        note_data: NoteData,
        delete: bool,
    },
    NoteRenaming {
        note_data: NoteData,
        new_name: String,
    },
}

#[derive(Clone, Debug, Error)]
pub enum ExplorerError {
    #[error("No note selected. Should be unreachable.")]
    NoNoteSelected,
}

#[derive(Debug)]
pub struct NoteData {
    note: Note,
    tags: Vec<String>,
    links: Vec<i64>,
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

                                    let new_note =
                                        Note::new(name.clone(), String::new(), notebook.db())?;

                                    State::NoteViewing {
                                        note_data: NoteData {
                                            note: new_note,
                                            tags: Vec::new(),
                                            links: Vec::new(),
                                        },
                                    }
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
                            State::NoteViewing { mut note_data } => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note viewing.");
                                    State::Nothing
                                }
                                KeyCode::Char('q') => {
                                    info!("Quit notebook.");
                                    break;
                                }
                                KeyCode::Char('e') => {
                                    info!("Edit note {}", note_data.note.name);
                                    edit_note(&mut note_data.note, notebook)?;
                                    forced_redraw = true;
                                    State::NoteViewing { note_data }
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
                                        note_data,
                                        delete: false,
                                    }
                                }
                                KeyCode::Char('r') => {
                                    info!("Prompt note new name");
                                    State::NoteRenaming {
                                        note_data,
                                        new_name: String::new(),
                                    }
                                }
                                _ => State::NoteViewing { note_data },
                            },
                            State::NoteDeleting { note_data, delete } => match key.code {
                                KeyCode::Esc => {
                                    info!("Cancel deleting");
                                    State::NoteViewing { note_data }
                                }
                                KeyCode::Tab => State::NoteDeleting {
                                    note_data,
                                    delete: !delete,
                                },
                                KeyCode::Enter => {
                                    if delete {
                                        note_data.note.delete(notebook.db())?;
                                        State::Nothing
                                    } else {
                                        State::NoteViewing { note_data }
                                    }
                                }
                                _ => State::NoteDeleting { note_data, delete },
                            },
                            State::NoteRenaming {
                                mut note_data,
                                mut new_name,
                            } => match key.code {
                                KeyCode::Esc => {
                                    info!("Cancel renaming");
                                    State::NoteViewing { note_data }
                                }
                                KeyCode::Enter => {
                                    note_data.note.name = new_name;
                                    note_data.note.update(notebook.db())?;
                                    State::NoteViewing { note_data }
                                }

                                KeyCode::Backspace => {
                                    new_name.pop();
                                    State::NoteRenaming {
                                        note_data,
                                        new_name,
                                    }
                                }
                                KeyCode::Char(c) => {
                                    new_name.push(c);
                                    State::NoteRenaming {
                                        note_data,
                                        new_name,
                                    }
                                }
                                _ => State::NoteRenaming {
                                    note_data,
                                    new_name,
                                },
                            },
                            State::NoteListing {
                                mut pattern,
                                mut selected,
                                mut results,
                            } => match key.code {
                                KeyCode::Esc => {
                                    info!("Stop note searching.");
                                    State::Nothing
                                }
                                KeyCode::Up if selected > 0 => State::NoteListing {
                                    pattern,
                                    selected: selected - 1,
                                    results,
                                },
                                KeyCode::Down if selected < results.len() - 1 => {
                                    State::NoteListing {
                                        pattern,
                                        selected: selected + 1,
                                        results,
                                    }
                                }
                                KeyCode::Enter if !results.is_empty() => {
                                    let note_summary = results
                                        .get(selected)
                                        .ok_or(ExplorerError::NoNoteSelected)?;
                                    let note = Note::load(note_summary.id, notebook.db())?;
                                    let tags = note
                                        .get_tags(notebook.db())?
                                        .into_iter()
                                        .map(|tag| tag.name.clone())
                                        .collect();
                                    let links = note.get_links(notebook.db())?;

                                    info!("Open note {}", note_summary.name);

                                    State::NoteViewing {
                                        note_data: NoteData { note, tags, links },
                                    }
                                }
                                KeyCode::Backspace => {
                                    pattern.pop();
                                    selected = 0;
                                    results = notebook.search_name(pattern.as_str())?;
                                    State::NoteListing {
                                        pattern,
                                        selected,
                                        results,
                                    }
                                }
                                KeyCode::Char(c) => {
                                    pattern.push(c);
                                    selected = 0;
                                    results = notebook.search_name(pattern.as_str())?;
                                    State::NoteListing {
                                        pattern,
                                        selected,
                                        results,
                                    }
                                }
                                _ => State::NoteListing {
                                    pattern,
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
                State::NoteViewing { ref note_data } => {
                    let content = render(note_data.note.content.as_str());

                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note_data.note.name.as_str(),
                            note_data.tags.as_slice(),
                            content,
                        );
                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteDeleting {
                    ref note_data,
                    delete,
                } => {
                    let content = render(note_data.note.content.as_str());

                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note_data.note.name.as_str(),
                            note_data.tags.as_slice(),
                            content,
                        );
                        draw_deleting_popup(frame, main_rect, delete);

                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteRenaming {
                    ref note_data,
                    ref new_name,
                } => {
                    let content = render(note_data.note.content.as_str());

                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_viewed_note(
                            frame,
                            main_rect,
                            note_data.note.name.as_str(),
                            note_data.tags.as_slice(),
                            content,
                        );
                        draw_renaming_popup(frame, main_rect, new_name.as_str());

                        frame.render_widget(main_frame, frame.size());
                    })?;
                }
                State::NoteListing {
                    ref pattern,
                    selected,
                    ref results,
                } => {
                    terminal.draw(|frame| {
                        let main_rect = main_frame.inner(frame.size());
                        draw_note_listing(
                            frame,
                            main_rect,
                            pattern.as_str(),
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

fn draw_nothing(frame: &mut Frame, main_rect: Rect, name: &str) {
    let title = Paragraph::new(Line::from(vec![Span::raw(name.capitalize()).style(
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]))
    .alignment(Alignment::Center);

    frame.render_widget(title, create_popup_proportion((40, 10), main_rect));
}

fn draw_new_note(frame: &mut Frame, main_rect: Rect, entry: &str) {
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

    frame.render_widget(new_note_entry, create_popup_size((30, 5), main_rect));
}

fn draw_viewed_note(
    frame: &mut Frame,
    main_rect: Rect,
    title: &str,
    tags: &[String],
    content: Paragraph,
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

    let note_title = Paragraph::new(title)
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
        .rows([Row::new(tags.iter().map(Text::raw))])
        .block(
            Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red)),
        );
    let note_content = content.block(
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
    pattern: &str,
    results: &[NoteSummary],
    selected: usize,
) {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(5), Constraint::Min(0)],
    )
    .split(main_rect);

    let search_bar = Paragraph::new(Line::from(vec![
        Span::raw(pattern).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::new()
            .title("Searching")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if results.is_empty() {
                Color::Red
            } else {
                Color::Green
            }))
            .padding(Padding::uniform(1)),
    );

    let list_results = List::new(results.iter().map(|note| Span::raw(note.name.as_str())))
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
        &mut ListState::with_selected(ListState::default(), Some(selected)),
    );
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

fn draw_renaming_popup(frame: &mut Frame, main_rect: Rect, new_name: &str) {
    let popup_area = create_popup_size((30, 5), main_rect);

    let new_note_entry = Paragraph::new(Line::from(vec![
        Span::raw(new_name).style(Style::default().add_modifier(Modifier::UNDERLINED))
    ]))
    .block(
        Block::default()
            .title("Rename note")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .padding(Padding::uniform(1)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(new_note_entry, create_popup_size((30, 5), main_rect));
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
