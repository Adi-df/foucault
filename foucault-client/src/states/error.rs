use anyhow::Result;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{helpers::create_popup_size, states::State, NotebookAPI};

pub struct ErrorStateData {
    pub inner_state: Box<State>,
    pub error_message: String,
}

pub async fn run_error_state(
    mut state_data: ErrorStateData,
    key_event: KeyEvent,
    notebook: &NotebookAPI,
) -> Result<State> {
    // todo!();
    Ok(State::Error(state_data))
}

pub fn draw_error_state(
    notebook: &NotebookAPI,
    state_data: &ErrorStateData,
    frame: &mut Frame,
    main_rect: Rect,
) {
    state_data.inner_state.draw(notebook, frame, main_rect);

    let popup_area = create_popup_size((60, 5), main_rect);
    let err_popup = Paragraph::new(state_data.error_message.as_str())
        .wrap(Wrap { trim: false })
        .block(
            Block::new()
                .title("Error")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Red)),
        );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(err_popup, popup_area);
}
