use markdown::mdast;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::markdown::{
    BLOCKQUOTE, BLOCKQUOTE_ALIGNEMENT, CROSS_REF, HEADER_ALIGNEMENT, HEADER_COLOR, HEADER_MODIFIER,
    HYPERLINK, ITALIC, RICH_TEXT_COLOR, STRONG, TEXT,
};

const TEXT_STYLE: Style = Style::new().fg(RICH_TEXT_COLOR[TEXT]);

const ITALIC_STYLE: Style = Style::new()
    .add_modifier(Modifier::UNDERLINED)
    .fg(RICH_TEXT_COLOR[ITALIC]);

const STRONG_STYLE: Style = Style::new()
    .add_modifier(Modifier::BOLD)
    .fg(RICH_TEXT_COLOR[STRONG]);

const HYPER_LINK_STYLE: Style = Style::new()
    .add_modifier(Modifier::UNDERLINED)
    .fg(RICH_TEXT_COLOR[HYPERLINK]);

const CROSS_REF_STYLE: Style = Style::new().fg(RICH_TEXT_COLOR[CROSS_REF]);

const BLOCKQUOTE_STYLE: Style = Style::new()
    .fg(RICH_TEXT_COLOR[BLOCKQUOTE])
    .add_modifier(Modifier::ITALIC);

const HEADING_STYLE: [Style; 6] = [
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[0], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[0]),
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[1], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[1]),
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[2], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[2]),
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[3], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[3]),
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[4], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[4]),
    Style::new()
        .add_modifier(Modifier::union(HEADER_MODIFIER[5], Modifier::UNDERLINED))
        .fg(HEADER_COLOR[5]),
];

#[derive(Debug, Clone)]
pub enum InlineElements {
    RichText { span: Span<'static> },
    HyperLink { span: Span<'static>, dest: String },
    CrossRef { span: Span<'static>, dest: String },
}

impl InlineElements {
    pub fn parse_node(node: &mdast::Node) -> Vec<InlineElements> {
        match node {
            mdast::Node::Emphasis(italic) => italic
                .children
                .iter()
                .flat_map(InlineElements::parse_node)
                .map(|el| el.richify(ITALIC_STYLE))
                .collect(),
            mdast::Node::Strong(strong) => strong
                .children
                .iter()
                .flat_map(InlineElements::parse_node)
                .map(|el| el.richify(STRONG_STYLE))
                .collect(),
            mdast::Node::Link(link) => vec![InlineElements::HyperLink {
                span: Span::raw(
                    link.children
                        .iter()
                        .flat_map(InlineElements::parse_node)
                        .map(InlineElements::discard_style)
                        .collect::<String>(),
                )
                .style(HYPER_LINK_STYLE),
                dest: link.url.to_string(),
            }],
            mdast::Node::Text(text) => parse_cross_links(text.value.as_str()),
            _ => Vec::new(),
        }
    }

    fn richify(self, style: Style) -> Self {
        match self {
            Self::RichText { mut span } => {
                span.patch_style(style);
                Self::RichText { span }
            }
            s => s,
        }
    }

    fn discard_style(self) -> String {
        match self {
            Self::RichText { span }
            | Self::HyperLink { span, .. }
            | Self::CrossRef { span, .. } => span.content.to_string(),
        }
    }

    fn build_span(&self) -> Span<'static> {
        match self {
            Self::RichText { span }
            | Self::HyperLink { span, .. }
            | Self::CrossRef { span, .. } => span.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockElements {
    Paragraph {
        content: Vec<InlineElements>,
    },
    Heading {
        content: Vec<InlineElements>,
        level: u8,
    },
    BlockQuote {
        content: Vec<InlineElements>,
    },
}

impl BlockElements {
    pub fn parse_node(node: &mdast::Node) -> Vec<BlockElements> {
        match node {
            mdast::Node::Root(root) => root
                .children
                .iter()
                .flat_map(BlockElements::parse_node)
                .collect(),
            mdast::Node::BlockQuote(blockquote) => vec![Self::BlockQuote {
                content: blockquote
                    .children
                    .iter()
                    .flat_map(BlockElements::parse_node)
                    .flat_map(BlockElements::content)
                    .collect(),
            }],
            mdast::Node::Heading(heading) => vec![Self::Heading {
                level: heading.depth - 1,
                content: heading
                    .children
                    .iter()
                    .flat_map(InlineElements::parse_node)
                    .collect(),
            }],
            mdast::Node::Paragraph(paragraph) => vec![Self::Paragraph {
                content: paragraph
                    .children
                    .iter()
                    .flat_map(InlineElements::parse_node)
                    .collect(),
            }],
            _ => Vec::new(),
        }
    }

    fn content(self) -> Vec<InlineElements> {
        match self {
            BlockElements::Paragraph { content }
            | BlockElements::Heading { content, .. }
            | BlockElements::BlockQuote { content } => content,
        }
    }

    fn get_content(&self) -> &[InlineElements] {
        match self {
            BlockElements::Paragraph { content }
            | BlockElements::Heading { content, .. }
            | BlockElements::BlockQuote { content } => content,
        }
    }

    pub fn text(&self) -> String {
        self.get_content()
            .iter()
            .map(|el| el.clone().discard_style())
            .collect()
    }

    pub fn lines(&self, max_len: usize) -> usize {
        textwrap::wrap(self.text().as_str(), max_len).len()
    }

    pub fn build_lines(&self) -> Vec<Line<'static>> {
        match self {
            Self::Paragraph { content } => {
                vec![Line::from(
                    content
                        .iter()
                        .map(InlineElements::build_span)
                        .collect::<Vec<Span<'static>>>(),
                )]
            }
            BlockElements::Heading { content, level } => vec![Line::from(
                content
                    .iter()
                    .map(|el| el.clone().richify(HEADING_STYLE[*level as usize]))
                    .map(|el| el.build_span())
                    .collect::<Vec<Span<'static>>>(),
            )
            .alignment(HEADER_ALIGNEMENT[*level as usize])],
            BlockElements::BlockQuote { content } => vec![Line::from(
                content
                    .iter()
                    .map(|el| el.clone().richify(BLOCKQUOTE_STYLE))
                    .map(|el| el.build_span())
                    .collect::<Vec<Span<'static>>>(),
            )
            .alignment(BLOCKQUOTE_ALIGNEMENT)],
        }
    }
}

fn parse_cross_links(text: &str) -> Vec<InlineElements> {
    let mut content_iter = text.chars().peekable();
    let mut escape = false;
    let mut cross_ref = false;
    let mut current_span = String::new();
    let mut spans = Vec::new();

    while let Some(c) = content_iter.next() {
        if cross_ref {
            if c == ']' && matches!(content_iter.peek(), Some(']')) {
                spans.push(InlineElements::CrossRef {
                    span: Span::raw(format!("[{current_span}]")).style(CROSS_REF_STYLE),
                    dest: current_span,
                });
                current_span = String::new();
                cross_ref = false;
                content_iter.next();
            } else {
                current_span.push(c);
            }
        } else {
            if escape {
                current_span.push(c);
                escape = false;
                continue;
            }

            if c == '[' && matches!(content_iter.peek(), Some('[')) {
                spans.push(InlineElements::RichText {
                    span: Span::raw(current_span).style(TEXT_STYLE),
                });
                current_span = String::new();
                cross_ref = true;

                content_iter.next();
            } else {
                current_span.push(c);
            }
        }
    }

    if !current_span.is_empty() {
        spans.push(InlineElements::RichText {
            span: Span::raw(current_span),
        });
    }

    spans
}
