use std::ffi::OsString;
use std::fs;
use std::io::stdout;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use log::info;
use scopeguard::defer;
use thiserror::Error;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::{Alignment, CrosstermBackend, Margin};
use ratatui::style::Style;
use ratatui::style::{Color, Modifier};
use ratatui::text::Text;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, ListState, Padding, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};
use ratatui::Terminal;

#[derive(Clone, Debug, Error)]
pub enum NotebookSelectorError {
    #[error("The notebook name couldn't be decoded : {name:?}")]
    InvalidNotebookName { name: OsString },
}

pub fn open_selector(dir: &Path) -> Result<Option<String>> {
    info!("Open notebook selector.");

    // Retreive notebooks

    let notebooks = fs::read_dir(dir)?
        .map(|file| {
            file.map_err(anyhow::Error::from).and_then(|file| {
                file.file_name()
                    .into_string()
                    .map_err(|e| NotebookSelectorError::InvalidNotebookName { name: e }.into())
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
                        KeyCode::Up if selected > 0 => selected -= 1,
                        KeyCode::Down if selected < notebooks.len() - 1 => selected += 1,
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
            let main_block = Block::default()
                .title("Foucault")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .padding(Padding::new(2, 2, 1, 1))
                .borders(Borders::all())
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded);

            let list = List::default()
                .items(
                    notebooks
                        .iter()
                        .map(|notebook| Text::styled(notebook, Style::default())),
                )
                .highlight_symbol(">>")
                .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
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
