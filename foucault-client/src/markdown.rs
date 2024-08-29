pub mod elements;

use markdown::{to_mdast, ParseOptions};

use ratatui::{
    prelude::Alignment,
    style::{Color, Modifier},
    text::Span,
};

use crate::markdown::elements::{
    BlockElement, BlockElements, InlineElement, RenderedBlock, SelectableInlineElements,
    HEADING_STYLE,
};

const HEADER_COLOR: [Color; 6] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
];
const HEADER_MODIFIER: [Modifier; 6] = [
    Modifier::BOLD,
    Modifier::empty(),
    Modifier::ITALIC,
    Modifier::empty(),
    Modifier::DIM,
    Modifier::DIM,
];
const HEADER_ALIGNEMENT: [Alignment; 6] = [
    Alignment::Center,
    Alignment::Center,
    Alignment::Left,
    Alignment::Left,
    Alignment::Left,
    Alignment::Left,
];

const BLOCKQUOTE_ALIGNEMENT: Alignment = Alignment::Center;

const TEXT: usize = 0;
const ITALIC: usize = 1;
const STRONG: usize = 2;
const HYPERLINK: usize = 3;
const CROSS_REF: usize = 4;
const BLOCKQUOTE: usize = 5;

const RICH_TEXT_COLOR: [Color; 6] = [
    Color::Reset,     // Text
    Color::Green,     // Italic
    Color::Yellow,    // Strong
    Color::LightBlue, // Link
    Color::Cyan,      // Cross ref
    Color::Yellow,    // Blockquote
];

#[derive(Debug)]
pub struct Header {
    pub text: String,
    level: u8,
}

impl Header {
    pub fn build_span(&self) -> Span<'_> {
        Span::raw(&self.text).style(HEADING_STYLE[usize::from(self.level)])
    }
}

pub struct ParsedMarkdown {
    parsed_content: Vec<BlockElements<SelectableInlineElements>>,
}

impl ParsedMarkdown {
    pub fn get_element(&self, el: (usize, usize)) -> Option<&SelectableInlineElements> {
        if let Some(block) = &self.parsed_content.get(el.1) {
            block.get_content().get(el.0)
        } else {
            None
        }
    }

    pub fn select(&mut self, el: (usize, usize), selected: bool) {
        if let Some(block) = self.parsed_content.get_mut(el.1) {
            if let Some(element) = block.get_content_mut().get_mut(el.0) {
                element.select(selected);
            }
        }
    }

    pub fn list_links(&self) -> Vec<&str> {
        self.parsed_content
            .iter()
            .flat_map(|block| block.get_content().iter())
            .map(|el| &el.element)
            .filter_map(|el| el.link_dest())
            .collect()
    }

    pub fn related_header(&self, block: usize) -> Option<usize> {
        self.parsed_content[0..=block]
            .iter()
            .fold(None, |acc, el| match el {
                BlockElements::Heading { .. } => Some(match acc {
                    Some(v) => v + 1,
                    None => 0,
                }),
                _ => acc,
            })
    }

    pub fn header_index(&self, header: usize) -> Option<usize> {
        let mut header_counter = 0;
        self.parsed_content
            .iter()
            .enumerate()
            .find_map(move |(block, el)| match el {
                BlockElements::Heading { .. } => {
                    if header_counter == header {
                        Some(block)
                    } else {
                        header_counter += 1;
                        None
                    }
                }
                _ => None,
            })
    }

    pub fn list_headers(&self) -> Vec<Header> {
        self.parsed_content
            .iter()
            .filter_map(|el| match el {
                BlockElements::Heading { content, level } => {
                    let text = content.iter().map(InlineElement::inner_text).collect();
                    Some(Header {
                        text,
                        level: *level,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn render_blocks(&self, max_len: usize) -> Vec<RenderedBlock> {
        self.parsed_content
            .iter()
            .map(BlockElement::render_lines)
            .map(|block| block.wrap_lines(max_len))
            .collect()
    }

    pub fn block_count(&self) -> usize {
        self.parsed_content.len()
    }

    pub fn block_length(&self, block: usize) -> usize {
        self.parsed_content.get(block).map_or(0, BlockElement::len)
    }
}

pub fn parse(content: &str) -> ParsedMarkdown {
    ParsedMarkdown {
        parsed_content: BlockElements::parse_node(
            &to_mdast(content, &ParseOptions::default()).unwrap(),
        ),
    }
}

pub fn lines(blocks: &[RenderedBlock]) -> usize {
    blocks.iter().map(RenderedBlock::line_count).sum()
}

pub fn combine(blocks: &[RenderedBlock]) -> RenderedBlock {
    blocks
        .iter()
        .flat_map(|el| el.lines().iter())
        .cloned()
        .collect::<Vec<_>>()
        .into()
}
