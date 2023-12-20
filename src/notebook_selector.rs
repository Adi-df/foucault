use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::{event, ExecutableCommand};
use log::trace;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::{Alignment, CrosstermBackend};
use ratatui::style::Style;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Terminal;

pub fn open_selector(dir: &Path) -> Result<Option<PathBuf>> {
    trace!("Open notebook selector.");
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut selector_loop = || -> Result<()> {
        while !handle_events()? {
            terminal.draw(|frame| {
                let main_block = Block::default()
                    .title("Foucault")
                    .title_alignment(Alignment::Center)
                    .title_style(Style::default().add_modifier(Modifier::BOLD))
                    .borders(Borders::all())
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded);

                frame.render_widget(main_block, frame.size());
            })?;
        }
        Ok(())
    };

    let result = selector_loop();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result.map(|_| None)
}

fn handle_events() -> Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press
                && (key.code == KeyCode::Esc || key.code == KeyCode::Char('q'))
            {
                trace!("Quit notebook selector");
                return Ok(true);
            }
        }
    }

    Ok(false)
}
