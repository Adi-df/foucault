use markdown::mdast::Node;
use markdown::{to_mdast, ParseOptions};

use ratatui::{
    prelude::Alignment,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

static HEADER_COLOR: [Color; 6] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
];
static HEADER_MODIFIER: [Modifier; 6] = [
    Modifier::BOLD,
    Modifier::empty(),
    Modifier::ITALIC,
    Modifier::empty(),
    Modifier::DIM,
    Modifier::DIM,
];
static HEADER_ALIGNEMENT: [Alignment; 6] = [
    Alignment::Center,
    Alignment::Center,
    Alignment::Left,
    Alignment::Left,
    Alignment::Left,
    Alignment::Left,
];

static TEXT: usize = 0;
static ITALIC: usize = 1;
static STRONG: usize = 2;
static LINK: usize = 3;
static CROSS_REF: usize = 4;
static BLOCKQUOTE: usize = 5;

static RICH_TEXT_COLOR: [Color; 6] = [
    Color::Reset,     // Text
    Color::Green,     // Italic
    Color::Yellow,    // Strong
    Color::LightBlue, // Link
    Color::Cyan,      // Cross ref
    Color::Yellow,    // Blockquote
];

pub fn links(content: &str) -> Vec<String> {
    todo!();
}

pub fn render<'a>(content: &str) -> Paragraph<'a> {
    let ast = to_mdast(content, &ParseOptions::default()).unwrap();

    Paragraph::new(parse(&ast)).wrap(Wrap { trim: true })
}

fn parse<'a>(ast: &Node) -> Vec<Line<'a>> {
    match ast {
        Node::Root(root) => root.children.iter().flat_map(parse).collect(),
        Node::Paragraph(paragraph) => vec![Line::from(
            paragraph
                .children
                .iter()
                .flat_map(parse_text)
                .collect::<Vec<Span>>(),
        )],
        Node::Heading(header) => {
            let depth = header.depth as usize - 1;
            vec![Line::from(
                header
                    .children
                    .iter()
                    .flat_map(parse_text)
                    .map(|text| {
                        text.style(
                            Style::default()
                                .fg(HEADER_COLOR[depth])
                                .add_modifier(HEADER_MODIFIER[depth] | Modifier::UNDERLINED),
                        )
                    })
                    .collect::<Vec<Span>>(),
            )
            .alignment(HEADER_ALIGNEMENT[depth])]
        }
        Node::BlockQuote(block) => block
            .children
            .iter()
            .flat_map(parse)
            .map(|line| {
                let mut line = line.alignment(Alignment::Center);
                line.patch_style(
                    Style::default()
                        .fg(RICH_TEXT_COLOR[BLOCKQUOTE])
                        .add_modifier(Modifier::ITALIC),
                );
                line
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_text<'a>(child: &Node) -> Vec<Span<'a>> {
    match child {
        Node::Text(text) => parse_cross_links(text.value.as_str()),
        Node::Emphasis(empathis) => empathis
            .children
            .iter()
            .flat_map(parse_text)
            .map(|span| {
                span.add_modifier(Modifier::ITALIC)
                    .fg(RICH_TEXT_COLOR[ITALIC])
            })
            .collect(),
        Node::Strong(strong) => strong
            .children
            .iter()
            .flat_map(parse_text)
            .map(|span| {
                span.add_modifier(Modifier::BOLD)
                    .fg(RICH_TEXT_COLOR[STRONG])
            })
            .collect(),
        Node::Link(link) => link
            .children
            .iter()
            .flat_map(parse_text)
            .map(|span| {
                span.style(
                    Style::default()
                        .fg(RICH_TEXT_COLOR[LINK])
                        .add_modifier(Modifier::UNDERLINED),
                )
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_cross_links<'a>(content: &str) -> Vec<Span<'a>> {
    let mut content_iter = content.chars().peekable();
    let mut escape = false;
    let mut spans = Vec::new();
    let mut current_span = String::new();

    while let Some(c) = content_iter.next() {
        if escape {
            current_span.push(c);
            escape = false;
            continue;
        }

        if c == '[' && matches!(content_iter.peek(), Some('[')) {
            spans.push(Span::raw(current_span).style(Style::default().fg(RICH_TEXT_COLOR[TEXT])));
            current_span = String::from("[");
            content_iter.next();
        } else if c == ']' && matches!(content_iter.peek(), Some(']')) {
            current_span.push(']');
            spans.push(Span::raw(current_span).style(Style::new().fg(RICH_TEXT_COLOR[CROSS_REF])));
            current_span = String::new();
            content_iter.next();
        } else {
            current_span.push(c);
        }
    }

    if !current_span.is_empty() {
        spans.push(Span::raw(current_span).style(Style::default().fg(RICH_TEXT_COLOR[TEXT])));
    }

    spans
}
