use std::borrow::Cow;

use markdown::mdast;

use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use unicode_segmentation::UnicodeSegmentation;

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

pub(super) const HEADING_STYLE: [Style; 6] = [
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

pub trait InlineElement: Sized {
    fn raw<T: Into<Cow<'static, str>>>(content: T) -> Self;
    fn parse_node(node: &mdast::Node) -> Vec<Self>;
    fn get_inner_span(&self) -> &Span<'static>;
    fn get_inner_span_mut(&mut self) -> &mut Span<'static>;

    fn is_empty(&self) -> bool {
        self.get_inner_span().content.is_empty()
    }

    fn inner_text(&self) -> &str {
        self.get_inner_span().content.as_ref()
    }
    fn into_span(self) -> Span<'static> {
        self.get_inner_span().clone()
    }

    fn patch_style(&mut self, style: Style) {
        *self.get_inner_span_mut() = self.get_inner_span().clone().patch_style(style);
    }
}

pub trait ChainInlineElement: InlineElement + Sized {
    fn patch_style(mut self, style: Style) -> Self {
        InlineElement::patch_style(&mut self, style);
        self
    }
}

impl<T> ChainInlineElement for T where T: InlineElement + Sized {}

pub trait BlockElement<T>: Sized
where
    T: InlineElement,
{
    fn parse_node(node: &mdast::Node) -> Vec<Self>;
    fn content(self) -> Vec<T>;
    fn get_content(&self) -> &[T];
    fn get_content_mut(&mut self) -> &mut [T];
    fn render_lines(&self) -> RenderedBlock;

    fn len(&self) -> usize {
        self.get_content().len()
    }
}

#[derive(Debug, Clone)]
pub struct RenderedBlock {
    content: Vec<Line<'static>>,
}

impl RenderedBlock {
    pub fn lines(&self) -> &[Line<'static>] {
        &self.content
    }

    pub fn build_paragraph(self) -> Paragraph<'static> {
        Paragraph::new(self.content)
    }

    pub fn wrap_lines(self, max_len: usize) -> Self {
        let new_content: Vec<Line<'static>> = self
            .content
            .into_iter()
            .flat_map(|line| {
                let mut new_lines: Vec<Line<'static>> = vec![Line::from(Vec::new())];
                let mut current_size: usize = 0;

                for span in &line.spans {
                    let mut new_span: String = String::new();
                    for grapheme in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
                        if current_size == max_len || grapheme == "\n" || grapheme == "\r\n" {
                            new_lines
                                .last_mut()
                                .unwrap()
                                .spans
                                .push(Span::raw(new_span.clone()).style(span.style));

                            new_span = String::new();
                            new_lines.push(Line::from(Vec::new()));
                            current_size = 0;
                        }

                        new_span.push_str(grapheme);
                        current_size += 1;
                    }
                    if !new_span.is_empty() {
                        new_lines
                            .last_mut()
                            .unwrap()
                            .spans
                            .push(Span::raw(new_span).style(span.style));
                    }
                }

                new_lines.into_iter().map(move |l| Line {
                    alignment: line.alignment,
                    ..l
                })
            })
            .collect();
        Self {
            content: new_content,
        }
    }

    pub fn line_count(&self) -> usize {
        self.content.len()
    }
}

impl From<Vec<Line<'static>>> for RenderedBlock {
    fn from(content: Vec<Line<'static>>) -> Self {
        Self { content }
    }
}

#[derive(Debug, Clone)]
pub enum InlineElements {
    RawText { span: Span<'static> },
    RichText { span: Span<'static> },
    HyperLink { span: Span<'static>, dest: String },
    CrossRef { span: Span<'static>, dest: String },
}

impl InlineElement for InlineElements {
    fn raw<T: Into<Cow<'static, str>>>(content: T) -> Self {
        Self::RawText {
            span: Span::raw(content),
        }
    }

    fn parse_node(node: &mdast::Node) -> Vec<InlineElements> {
        match node {
            mdast::Node::Emphasis(italic) => italic
                .children
                .iter()
                .flat_map(InlineElements::parse_node)
                .filter(|el| !el.is_empty())
                .map(|el| ChainInlineElement::patch_style(el, ITALIC_STYLE))
                .collect(),
            mdast::Node::Strong(strong) => strong
                .children
                .iter()
                .flat_map(InlineElements::parse_node)
                .filter(|el| !el.is_empty())
                .map(|el| ChainInlineElement::patch_style(el, STRONG_STYLE))
                .collect(),
            mdast::Node::Link(link) => vec![InlineElements::HyperLink {
                span: Span::raw(
                    link.children
                        .iter()
                        .flat_map(InlineElements::parse_node)
                        .filter(|el| !el.is_empty())
                        .map(|el| el.inner_text().to_string())
                        .collect::<String>(),
                )
                .style(HYPER_LINK_STYLE),
                dest: link.url.to_string(),
            }],
            mdast::Node::Text(text) => parse_cross_links(text.value.as_str()),
            _ => Vec::new(),
        }
    }

    fn get_inner_span(&self) -> &Span<'static> {
        match self {
            Self::RawText { span }
            | Self::RichText { span }
            | Self::HyperLink { span, .. }
            | Self::CrossRef { span, .. } => span,
        }
    }

    fn get_inner_span_mut(&mut self) -> &mut Span<'static> {
        match self {
            Self::RawText { span }
            | Self::RichText { span }
            | Self::HyperLink { span, .. }
            | Self::CrossRef { span, .. } => span,
        }
    }
}

impl InlineElements {
    pub fn link_dest(&self) -> Option<&str> {
        match self {
            Self::CrossRef { dest, .. } => Some(dest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectableInlineElements {
    pub element: InlineElements,
    pub selected: bool,
}

impl SelectableInlineElements {
    pub fn select(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl From<InlineElements> for SelectableInlineElements {
    fn from(element: InlineElements) -> Self {
        Self {
            element,
            selected: false,
        }
    }
}

impl<'a> From<&'a SelectableInlineElements> for &'a InlineElements {
    fn from(selectable_element: &'a SelectableInlineElements) -> Self {
        &selectable_element.element
    }
}

impl InlineElement for SelectableInlineElements {
    fn raw<T: Into<Cow<'static, str>>>(content: T) -> Self {
        Self {
            element: InlineElements::raw(content),
            selected: false,
        }
    }

    fn parse_node(node: &mdast::Node) -> Vec<Self> {
        InlineElements::parse_node(node)
            .into_iter()
            .map(SelectableInlineElements::from)
            .collect()
    }

    fn get_inner_span(&self) -> &Span<'static> {
        self.element.get_inner_span()
    }

    fn get_inner_span_mut(&mut self) -> &mut Span<'static> {
        self.element.get_inner_span_mut()
    }

    fn into_span(self) -> Span<'static> {
        let span = self.element.into_span();

        if self.selected {
            span.on_black()
        } else {
            span
        }
    }
}

#[derive(Debug)]
pub enum BlockElements<T>
where
    T: InlineElement,
{
    Paragraph { content: Vec<T> },
    Heading { content: Vec<T>, level: u8 },
    BlockQuote { content: Vec<T> },
    ListItem { content: Vec<T> },
    UnformatedText { content: Vec<T> },
}

impl<T> BlockElement<T> for BlockElements<T>
where
    T: InlineElement + Clone,
{
    fn parse_node(node: &mdast::Node) -> Vec<BlockElements<T>> {
        match node {
            mdast::Node::Root(root) => root
                .children
                .iter()
                .flat_map(BlockElements::parse_node)
                .collect(),
            mdast::Node::Blockquote(blockquote) => vec![Self::BlockQuote {
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
                    .flat_map(InlineElement::parse_node)
                    .collect(),
            }],
            mdast::Node::Paragraph(paragraph) => vec![Self::Paragraph {
                content: paragraph
                    .children
                    .iter()
                    .flat_map(InlineElement::parse_node)
                    .collect(),
            }],
            mdast::Node::List(list) => list
                .children
                .iter()
                .filter_map(|el| {
                    if let mdast::Node::ListItem(item) = el {
                        Some(item)
                    } else {
                        None
                    }
                })
                .map(|item| Self::ListItem {
                    content: item
                        .children
                        .iter()
                        .flat_map(BlockElements::parse_node)
                        .flat_map(BlockElements::content)
                        .collect(),
                })
                .collect(),
            mdast::Node::Code(code) if code.lang.is_none() => vec![Self::UnformatedText {
                content: code
                    .value
                    .lines()
                    .map(String::from)
                    .map(InlineElement::raw)
                    .collect(),
            }],
            _ => Vec::new(),
        }
    }

    fn content(self) -> Vec<T> {
        match self {
            Self::Paragraph { content }
            | Self::Heading { content, .. }
            | Self::BlockQuote { content }
            | Self::ListItem { content }
            | Self::UnformatedText { content } => content,
        }
    }

    fn get_content(&self) -> &[T] {
        match self {
            Self::Paragraph { content }
            | Self::Heading { content, .. }
            | Self::BlockQuote { content }
            | Self::ListItem { content }
            | Self::UnformatedText { content } => content,
        }
    }

    fn get_content_mut(&mut self) -> &mut [T] {
        match self {
            Self::Paragraph { content }
            | Self::Heading { content, .. }
            | Self::BlockQuote { content }
            | Self::ListItem { content }
            | Self::UnformatedText { content } => content,
        }
    }

    fn render_lines(&self) -> RenderedBlock {
        match self {
            Self::Paragraph { content } => {
                vec![
                    Line::from(
                        content
                            .iter()
                            .cloned()
                            .map(InlineElement::into_span)
                            .collect::<Vec<Span<'static>>>(),
                    ),
                    Line::default(),
                ]
            }
            BlockElements::Heading { content, level } => vec![Line::from(
                content
                    .iter()
                    .cloned()
                    .map(|el| ChainInlineElement::patch_style(el, HEADING_STYLE[*level as usize]))
                    .map(InlineElement::into_span)
                    .collect::<Vec<_>>(),
            )
            .alignment(HEADER_ALIGNEMENT[*level as usize])],
            BlockElements::BlockQuote { content } => vec![
                Line::from(
                    content
                        .iter()
                        .cloned()
                        .map(|el| ChainInlineElement::patch_style(el, BLOCKQUOTE_STYLE))
                        .map(InlineElement::into_span)
                        .collect::<Vec<_>>(),
                )
                .alignment(BLOCKQUOTE_ALIGNEMENT),
                Line::default(),
            ],
            BlockElements::ListItem { content } => vec![Line::from(
                [Span::raw("  - ").style(Style::new().fg(Color::Blue))]
                    .into_iter()
                    .chain(content.iter().cloned().map(InlineElement::into_span))
                    .collect::<Vec<_>>(),
            )],
            BlockElements::UnformatedText { content } => content
                .iter()
                .cloned()
                .map(InlineElement::into_span)
                .map(|span| Line::from(vec![span]))
                .collect(),
        }
        .into()
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

    spans.retain(|el| !el.is_empty());

    spans
}
