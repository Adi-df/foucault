use std::io::{stdout, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use log::info;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Padding};
use ratatui::Terminal;

use crate::note::Note;
use crate::notebook::Notebook;

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
    let mut openened_note: Option<Note> = None;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                        info!("Quit notebook");
                        break;
                    }
                }
            }
        }

        terminal.draw(|frame| {
            let main_frame = Block::default()
                .title(notebook.name.as_str())
                .padding(Padding::uniform(1))
                .borders(Borders::all())
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White));

            frame.render_widget(main_frame, frame.size());
        })?;
    }

    Ok(())
}
