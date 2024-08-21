use std::{io::stdout, time::Duration};

use anyhow::Result;
use log::info;
use scopeguard::defer;

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding},
    Terminal,
};

use crate::{states::State, NotebookAPI};

pub async fn explore(notebook: &NotebookAPI) -> Result<()> {
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
                        state = state.run(key, notebook, &mut forced_redraw).await?;
                    }
                }
            }

            if matches!(state, State::Exit) {
                break;
            }
        }

        {
            terminal.draw(|frame| {
                if forced_redraw {
                    frame.render_widget(Clear, frame.size());
                }
                forced_redraw = false;

                let main_frame = Block::new()
                    .title(notebook.name.as_str())
                    .padding(Padding::uniform(1))
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(Color::White));

                let main_rect = main_frame.inner(frame.size());

                state.draw(notebook, frame, main_rect);
                frame.render_widget(main_frame, frame.size());
            })?;
        }
    }

    Ok(())
}
