use std::io::Stdout;

use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::{NoteData, State};

use super::note_viewing::draw_viewed_note;

#[derive(Debug)]
pub struct NoteRenamingStateData {
    pub note_data: NoteData,
    pub new_name: String,
}

pub fn run_note_renaming_state(
    NoteRenamingStateData {
        mut note_data,
        mut new_name,
    }: NoteRenamingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => {
            info!("Cancel renaming");
            State::NoteViewing(NoteViewingStateData { note_data })
        }
        KeyCode::Enter => {
            note_data.note.name = new_name;
            note_data.note.update(notebook.db())?;
            State::NoteViewing(NoteViewingStateData { note_data })
        }

        KeyCode::Backspace => {
            new_name.pop();
            State::NoteRenaming(NoteRenamingStateData {
                note_data,
                new_name,
            })
        }
        KeyCode::Char(c) => {
            new_name.push(c);
            State::NoteRenaming(NoteRenamingStateData {
                note_data,
                new_name,
            })
        }
        _ => State::NoteRenaming(NoteRenamingStateData {
            note_data,
            new_name,
        }),
    })
}

pub fn draw_note_renaming_state(
    NoteRenamingStateData {
        note_data,
        new_name,
    }: &NoteRenamingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());
            draw_viewed_note(frame, note_data, main_rect);
            let popup_area = create_popup_size((30, 5), main_rect);

            let new_note_entry = Paragraph::new(Line::from(vec![
                Span::raw(new_name).style(Style::default().add_modifier(Modifier::UNDERLINED))
            ]))
            .block(
                Block::default()
                    .title("Rename note")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green))
                    .padding(Padding::uniform(1)),
            );

            frame.render_widget(Clear, popup_area);
            frame.render_widget(new_note_entry, create_popup_size((30, 5), main_rect));

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
