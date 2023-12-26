use std::io::Stdout;

use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::note::{Note, NoteData};
use crate::notebook::Notebook;
use crate::states::note_viewing::NoteViewingStateData;
use crate::states::State;

#[derive(Debug)]
pub struct NoteCreatingStateData {
    pub name: String,
}

pub fn run_note_creating_state(
    NoteCreatingStateData { mut name }: NoteCreatingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Enter => {
            info!("Complete note creation : {}.", name.as_str());

            let new_note = Note::new(name.clone(), String::new(), notebook.db())?;

            State::NoteViewing(NoteViewingStateData {
                note_data: NoteData {
                    note: new_note,
                    tags: Vec::new(),
                    links: Vec::new(),
                },
                scroll: 0,
            })
        }
        KeyCode::Esc => {
            info!("Cancel new note.");
            State::Nothing
        }
        KeyCode::Backspace => {
            name.pop();
            State::NoteCreating(NoteCreatingStateData { name })
        }
        KeyCode::Char(c) => {
            name.push(c);
            State::NoteCreating(NoteCreatingStateData { name })
        }
        _ => State::NoteCreating(NoteCreatingStateData { name }),
    })
}

pub fn draw_note_creating_state(
    NoteCreatingStateData { name }: &NoteCreatingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal.draw(|frame| {
        let main_rect = main_frame.inner(frame.size());

        let new_note_entry = Paragraph::new(Line::from(vec![
            Span::raw(name).style(Style::default().add_modifier(Modifier::UNDERLINED))
        ]))
        .block(
            Block::default()
                .title("Note name")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .padding(Padding::uniform(1)),
        );

        frame.render_widget(new_note_entry, create_popup_size((30, 5), main_rect));

        frame.render_widget(main_frame, frame.size());
    })?;
    Ok(())
}
