use std::io::Stdout;

use anyhow::Result;

use crossterm::event::KeyCode;
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::notebook::Notebook;
use crate::states::tag_managing::TagsManagingStateData;
use crate::states::State;
use crate::tags::Tag;

use super::tag_managing::draw_tags_managing;

#[derive(Debug)]
pub struct TagsDeletingStateData {
    pub tags_managing: TagsManagingStateData,
    pub selected: usize,
    pub delete: bool,
}

pub fn run_tag_deleting_state(
    TagsDeletingStateData {
        mut tags_managing,
        selected,
        delete,
    }: TagsDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::TagsManaging(tags_managing),
        KeyCode::Enter => {
            if delete {
                tags_managing
                    .tags
                    .swap_remove(selected)
                    .delete(notebook.db())?;
            }
            State::TagsManaging(tags_managing)
        }
        KeyCode::Tab => State::TagDeleting(TagsDeletingStateData {
            tags_managing,
            selected,
            delete: !delete,
        }),
        _ => State::TagDeleting(TagsDeletingStateData {
            tags_managing,
            selected,
            delete,
        }),
    })
}

pub fn draw_tag_deleting_state(
    TagsDeletingStateData {
        tags_managing,
        selected,
        delete,
    }: &TagsDeletingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    let Tag { name, .. } = &tags_managing.tags[*selected];

    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_tags_managing(frame, tags_managing, main_rect);

            let popup_area = create_popup_size((50, 5), main_rect);
            let block = Block::new()
                .title(format!("Delete tag {name} ?"))
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
