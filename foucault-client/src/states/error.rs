use anyhow::Result;
use log::info;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{helpers::create_popup, states::State, NotebookAPI};

pub struct ErrorStateData {
    pub inner_state: Box<State>,
    pub error_message: String,
}

pub async fn run_error_state(state_data: ErrorStateData, key_event: KeyEvent) -> Result<State> {
    Ok(match key_event.code {
        KeyCode::Char('q') => {
            info!("Quit foucault.");
            State::Exit
        }
        KeyCode::Enter => {
            info!("Close error popup.");
            *state_data.inner_state
        }
        _ => State::Error(state_data),
    })
}

pub fn draw_error_state(
    notebook: &NotebookAPI,
    state_data: &ErrorStateData,
    frame: &mut Frame,
    main_rect: Rect,
) {
    state_data.inner_state.draw(notebook, frame, main_rect);

    let line_width = main_rect.width * 80 / 100;
    let wrapped_text = textwrap::wrap(state_data.error_message.as_str(), line_width as usize);
    let line_count = wrapped_text.len();

    let popup_area = create_popup(
        (
            Constraint::Length(80),
            Constraint::Length(u16::try_from(line_count + 2).unwrap()),
        ),
        main_rect,
    );
    let err_popup = Paragraph::new(wrapped_text.join("\n")).block(
        Block::new()
            .title("Error")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(Color::Red)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(err_popup, popup_area);
}
