use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use log::info;
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::notebook::Notebook;
use crate::states::note_viewing::{draw_viewed_note, NoteViewingStateData};
use crate::states::State;

#[derive(Debug)]
pub struct NoteDeletingStateData {
    pub viewing_data: NoteViewingStateData,
    pub delete: bool,
}

pub fn run_note_deleting_state(
    NoteDeletingStateData {
        viewing_data: NoteViewingStateData { note_data, scroll },
        delete,
    }: NoteDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel deleting");
            State::NoteViewing(NoteViewingStateData { note_data, scroll })
        }
        KeyCode::Tab => State::NoteDeleting(NoteDeletingStateData {
            viewing_data: NoteViewingStateData { note_data, scroll },
            delete: !delete,
        }),
        KeyCode::Enter => {
            if delete {
                note_data.note.delete(notebook.db())?;
                State::Nothing
            } else {
                State::NoteViewing(NoteViewingStateData { note_data, scroll })
            }
        }
        _ => State::NoteDeleting(NoteDeletingStateData {
            viewing_data: NoteViewingStateData { note_data, scroll },
            delete,
        }),
    })
}

pub fn draw_note_deleting_state(
    NoteDeletingStateData {
        viewing_data,
        delete,
    }: &NoteDeletingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_viewed_note(frame, viewing_data, main_rect);

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

            let yes = Paragraph::new(Line::from(vec![if *delete {
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
            let no = Paragraph::new(Line::from(vec![if *delete {
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

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
