use std::io::Stdout;

use anyhow::Result;
use log::info;

use crossterm::event::KeyCode;
use ratatui::prelude::{Alignment, CrosstermBackend};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_proportion;
use crate::helpers::Capitalize;
use crate::note::Note;
use crate::notebook::Notebook;
use crate::states::note_creating::NoteCreatingStateData;
use crate::states::note_managing::NotesManagingStateData;
use crate::states::tag_managing::TagsManagingStateData;
use crate::states::State;
use crate::tags::Tag;

pub fn run_nothing_state(key_code: KeyCode, notebook: &Notebook) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc | KeyCode::Char('q') => {
            info!("Quit notebook.");
            State::Exit
        }
        KeyCode::Char('c') => {
            info!("Create new note.");
            State::NoteCreating(NoteCreatingStateData {
                name: String::new(),
            })
        }
        KeyCode::Char('s') => {
            info!("List notes.");
            State::NotesManaging(NotesManagingStateData {
                pattern: String::new(),
                selected: 0,
                notes: Note::search_by_name("", notebook.db())?,
            })
        }
        KeyCode::Char('t') => {
            info!("Manage tags.");
            State::TagsManaging(TagsManagingStateData {
                pattern: String::new(),
                pattern_editing: false,
                selected: 0,
                tags: Tag::search_by_name("", notebook.db())?,
            })
        }
        _ => State::Nothing,
    })
}

pub fn draw_nothing_state(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    notebook: &Notebook,
    main_frame: Block,
) -> Result<()> {
    terminal.draw(|frame| {
        let main_rect = main_frame.inner(frame.size());

        let title = Paragraph::new(Line::from(vec![Span::raw(notebook.name.capitalize())
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]))
        .alignment(Alignment::Center);

        frame.render_widget(title, create_popup_proportion((40, 10), main_rect));

        frame.render_widget(main_frame, frame.size());
    })?;
    Ok(())
}
