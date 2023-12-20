use std::ffi::OsString;
use std::fs;
use std::io::stdout;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use log::info;
use thiserror::Error;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::{Alignment, CrosstermBackend};
use ratatui::style::Style;
use ratatui::style::{Color, Modifier};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, List, ListDirection, ListState, Padding};
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
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut selected_notebook = ListState::default().with_selected(Some(0));

    let mut selector_loop = || {
        loop {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                            info!("Quit notebook selector.");
                            break Ok(None);
                        } else if let Some(selected) = selected_notebook.selected_mut() {
                            match key.code {
                                KeyCode::Up if *selected > 0 => *selected -= 1,
                                KeyCode::Down if *selected < notebooks.len() - 1 => *selected += 1,
                                KeyCode::Enter => {
                                    break Ok(Some(notebooks[*selected].clone()));
                                }
                                _ => {}
                            }
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

                frame.render_stateful_widget(
                    list,
                    main_block.inner(frame.size()),
                    &mut selected_notebook,
                );
                frame.render_widget(main_block, frame.size());
            })?;
        }
    };

    let result = selector_loop();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}
