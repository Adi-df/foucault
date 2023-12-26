use std::io::Stdout;

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Terminal;

use crate::helpers::create_popup_size;
use crate::states::State;
use crate::{notebook::Notebook, states::note_tags_managing::NoteTagsManagingStateData};

use crate::states::note_tags_managing::draw_note_tags_managing;

#[derive(Debug)]
pub struct NoteTagDeletingStateData {
    pub tags_managing: NoteTagsManagingStateData,
    pub delete: bool,
}

pub fn run_note_tag_deleting_state(
    NoteTagDeletingStateData {
        tags_managing:
            NoteTagsManagingStateData {
                new_tag,
                selected,
                mut tags,
                note,
            },
        delete,
    }: NoteTagDeletingStateData,
    key_code: KeyCode,
    notebook: &Notebook,
) -> Result<State> {
    Ok(match key_code {
        KeyCode::Esc => State::NoteTagsManaging(NoteTagsManagingStateData {
            new_tag,
            selected,
            tags,
            note,
        }),
        KeyCode::Enter => {
            if delete {
                let tag = tags.swap_remove(selected);
                note.remove_tag(tag.id, notebook.db())?;

                State::NoteTagsManaging(NoteTagsManagingStateData {
                    new_tag,
                    selected: 0,
                    tags,
                    note,
                })
            } else {
                State::NoteTagsManaging(NoteTagsManagingStateData {
                    new_tag,
                    selected,
                    tags,
                    note,
                })
            }
        }
        KeyCode::Tab => State::NoteTagDeleting(NoteTagDeletingStateData {
            tags_managing: NoteTagsManagingStateData {
                new_tag,
                selected,
                tags,
                note,
            },
            delete: !delete,
        }),
        _ => State::NoteTagDeleting(NoteTagDeletingStateData {
            tags_managing: NoteTagsManagingStateData {
                new_tag,
                selected,
                tags,
                note,
            },
            delete,
        }),
    })
}

pub fn draw_note_tag_deleting_state_data(
    NoteTagDeletingStateData {
        tags_managing,
        delete,
    }: &NoteTagDeletingStateData,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    main_frame: Block,
) -> Result<()> {
    terminal
        .draw(|frame| {
            let main_rect = main_frame.inner(frame.size());

            draw_note_tags_managing(frame, tags_managing, false, main_rect);

            let popup_area = create_popup_size((50, 5), main_rect);
            let block = Block::new()
                .title("Remove tag ?")
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
        .map_err(anyhow::Error::from)
        .map(|_| ())
}
