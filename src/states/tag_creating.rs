use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::notebook::Notebook;
use crate::states::tag_managing::TagsManagingStateData;
use crate::states::State;
use crate::tags::Tag;

use super::tag_managing::draw_tags_managing;

#[derive(Debug)]
pub struct TagsCreatingStateData {
    pub tags_search: TagsManagingStateData,
    pub name: String,
}

pub fn run_tag_creating_state(
    TagsCreatingStateData {
        tags_search,
        mut name,
    }: TagsCreatingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::TagsManaging(tags_search),
        KeyCode::Enter if !name.is_empty() => {
            if Tag::tag_exists(name.as_str(), notebook.db())? {
                State::TagCreating(TagsCreatingStateData { tags_search, name })
            } else {
                Tag::new(name.as_str(), notebook.db())?;
                State::TagsManaging(TagsManagingStateData {
                    pattern_editing: false,
                    selected: 0,
                    tags: Tag::search_by_name(tags_search.pattern.as_str(), notebook.db())?,
                    pattern: tags_search.pattern,
                })
            }
        }
        KeyCode::Backspace => {
            name.pop();
            State::TagCreating(TagsCreatingStateData { tags_search, name })
        }
        KeyCode::Char(c) if !c.is_whitespace() => {
            name.push(c);
            State::TagCreating(TagsCreatingStateData { tags_search, name })
        }
        _ => State::TagCreating(TagsCreatingStateData { tags_search, name }),
    })
}

pub fn draw_tags_creating_state(
    TagsCreatingStateData { tags_search, name }: &TagsCreatingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    notebook: &Notebook,
    main_frame: Block,
) -> Result<()> {
    let taken = Tag::tag_exists(name, notebook.db())?;

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_search, main_rect);

            let popup_area = create_popup_size((30, 5), main_rect);

            let new_tag_entry = Paragraph::new(Line::from(vec![
                Span::raw(name).style(Style::default().add_modifier(Modifier::UNDERLINED))
            ]))
            .block(
                Block::default()
                    .title("Tag name")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(if taken {
                        Color::Red
                    } else {
                        Color::Green
                    }))
                    .padding(Padding::uniform(1)),
            );

            frame.render_widget(Clear, popup_area);
            frame.render_widget(new_tag_entry, popup_area);

            frame.render_widget(main_frame, frame.size());
        })
        .map(|_| ())
        .map_err(anyhow::Error::from)
}
