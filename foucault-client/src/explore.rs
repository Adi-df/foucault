use std::{io::stdout, time::Duration};

use anyhow::Result;
use log::info;
use scopeguard::defer;

use crossterm::{event, ExecutableCommand};
use crossterm::{
    event::{Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{prelude::CrosstermBackend, widgets::Clear, Terminal};

use crate::states::State;
use crate::NotebookAPI;

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
            if forced_redraw {
                terminal.draw(|frame| frame.render_widget(Clear, frame.size()))?;
            }
            forced_redraw = false;

            state.draw(notebook, &mut terminal)?;
        }
    }

    Ok(())
}
