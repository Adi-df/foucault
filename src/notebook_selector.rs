use std::{env, ffi::OsString, fs, io::stdout, path::Path, time::Duration};

use anyhow::Result;
use log::info;
use scopeguard::defer;
use thiserror::Error;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Alignment, CrosstermBackend, Margin},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{
        Block, BorderType, Borders, List, ListDirection, ListState, Padding, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Terminal,
};

#[derive(Clone, Debug, Error)]
pub enum NotebookSelectorError {
    #[error("The notebook name couldn't be decoded : {name:?}")]
    InvalidNotebookName { name: OsString },
}

pub fn open_selector(dir: &Path) -> Result<Option<String>> {
    info!("Open notebook selector.");

    // Retreive notebooks

    let notebooks = fs::read_dir(dir)?
        .chain(fs::read_dir(env::current_dir()?)?)
        .filter_map(|file| {
            file.map_err(anyhow::Error::from)
                .map(|file| {
                    let file_path = file.path();
                    match file_path.extension() {
                        Some(extension) if extension == "book" => Some(file_path),
                        _ => None,
                    }
                })
                .transpose()
        })
        .map(|file_path| {
            file_path.and_then(|file_path| {
                file_path
                    .file_stem()
                    .ok_or(
                        NotebookSelectorError::InvalidNotebookName {
                            name: file_path.file_name().unwrap().to_os_string(),
                        }
                        .into(),
                    )
                    .and_then(|stem| {
                        stem.to_os_string().into_string().map_err(|e| {
                            NotebookSelectorError::InvalidNotebookName { name: e.clone() }.into()
                        })
                    })
            })
        })
        .collect::<Result<Vec<String>>>()?;

    // Display
    enable_raw_mode().expect("Prepare terminal");
    stdout()
        .execute(EnterAlternateScreen)
        .expect("Prepare terminal");

    defer! {
        stdout().execute(LeaveAlternateScreen).expect("Reset terminal");
        disable_raw_mode().expect("Reset terminal");
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut selected = 0;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            info!("Quit notebook selector.");
                            break Ok(None);
                        }
                        KeyCode::Up | KeyCode::Char('k') if selected > 0 => selected -= 1,
                        KeyCode::Down | KeyCode::Char('j')
                            if selected < notebooks.len().saturating_sub(1) =>
                        {
                            selected += 1;
                        }
                        KeyCode::Enter => {
                            break Ok(Some(notebooks[selected].clone()));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Draw
        terminal.draw(|frame| {
            let main_block = Block::new()
                .title("Foucault")
                .title_alignment(Alignment::Center)
                .title_style(Style::new().add_modifier(Modifier::BOLD))
                .padding(Padding::new(2, 2, 1, 1))
                .borders(Borders::all())
                .border_style(Style::new().fg(Color::White))
                .border_type(BorderType::Rounded);

            let list = List::default()
                .items(
                    notebooks
                        .iter()
                        .map(|notebook| Text::styled(notebook, Style::new())),
                )
                .highlight_symbol(">>")
                .highlight_style(Style::new().fg(Color::Black).bg(Color::White))
                .direction(ListDirection::TopToBottom);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            frame.render_stateful_widget(
                list,
                main_block.inner(frame.size()),
                &mut ListState::default().with_selected(Some(selected)),
            );
            frame.render_widget(main_block, frame.size());
            frame.render_stateful_widget(
                scrollbar,
                frame.size().inner(&Margin::new(0, 1)),
                &mut ScrollbarState::new(notebooks.len()).position(selected),
            );
        })?;
    }
}
