mod elements;

use markdown::{to_mdast, ParseOptions};

use ratatui::prelude::Alignment;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::{Paragraph, Wrap};

use crate::markdown::elements::{BlockElement, BlockElements, InlineElement};

use self::elements::SelectableInlineElements;

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

pub fn parse(content: &str) -> Vec<BlockElements<SelectableInlineElements>> {
    BlockElements::parse_node(&to_mdast(content, &ParseOptions::default()).unwrap())
}

pub fn lines<T, U>(blocks: &[T], max_len: u16) -> usize
where
    U: InlineElement,
    T: BlockElement<U>,
{
    blocks
        .iter()
        .map(|block| block.line_number(max_len as usize))
        .sum()
}

pub fn render<T, U>(blocks: &[T]) -> Paragraph
where
    U: InlineElement,
    T: BlockElement<U>,
{
    Paragraph::new(
        blocks
            .iter()
            .flat_map(BlockElement::build_lines)
            .collect::<Vec<_>>(),
    )
    .wrap(Wrap { trim: true })
}
