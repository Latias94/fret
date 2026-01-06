//! Markdown renderer component(s) for Fret.

use std::sync::Arc;

use fret_core::{FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

#[derive(Debug, Clone)]
pub struct Markdown {
    source: Arc<str>,
}

impl Markdown {
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        markdown(cx, &self.source)
    }
}

pub fn markdown<H: UiHost>(cx: &mut ElementContext<'_, H>, source: &str) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let blocks = parse_blocks(source);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full()),
        |cx| blocks.into_iter().map(|b| b.render(cx, &theme)).collect(),
    )
}

#[derive(Debug, Clone)]
enum Block {
    Heading {
        level: u8,
        text: Arc<str>,
    },
    Paragraph {
        text: Arc<str>,
    },
    CodeBlock {
        lang: Option<Arc<str>>,
        code: Arc<str>,
    },
}

impl Block {
    fn render<H: UiHost>(self, cx: &mut ElementContext<'_, H>, theme: &Theme) -> AnyElement {
        match self {
            Block::Heading { level, text } => render_heading(cx, theme, level, text),
            Block::Paragraph { text } => render_paragraph(cx, theme, text),
            Block::CodeBlock { lang, code } => {
                fret_code_view::code_block(cx, &code, lang.as_deref(), false)
            }
        }
    }
}

fn render_heading<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    level: u8,
    text: Arc<str>,
) -> AnyElement {
    let size = match level {
        1 => Px(theme.metrics.font_size.0 * 1.6),
        2 => Px(theme.metrics.font_size.0 * 1.4),
        3 => Px(theme.metrics.font_size.0 * 1.2),
        _ => theme.metrics.font_size,
    };

    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: Some(TextStyle {
            font: FontId::default(),
            size,
            weight: FontWeight::SEMIBOLD,
            line_height: Some(Px(theme.metrics.font_line_height.0 * 1.2)),
            letter_spacing_em: None,
        }),
        color: Some(theme.colors.text_primary),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn render_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: Arc<str>,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: Some(TextStyle {
            font: FontId::default(),
            size: theme.metrics.font_size,
            weight: FontWeight::NORMAL,
            line_height: Some(theme.metrics.font_line_height),
            letter_spacing_em: None,
        }),
        color: Some(theme.colors.text_primary),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn parse_blocks(source: &str) -> Vec<Block> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

    enum State {
        None,
        Paragraph { buf: String },
        Heading { level: u8, buf: String },
        CodeBlock { lang: Option<Arc<str>>, buf: String },
    }

    let mut blocks = Vec::new();
    let mut state = State::None;

    let parser = Parser::new(source);
    for event in parser {
        match (&mut state, event) {
            (State::None, Event::Start(Tag::Paragraph)) => {
                state = State::Paragraph { buf: String::new() };
            }
            (State::None, Event::Start(Tag::Heading { level, .. })) => {
                state = State::Heading {
                    level: heading_level_to_u8(level),
                    buf: String::new(),
                };
            }
            (State::None, Event::Start(Tag::CodeBlock(kind))) => {
                let lang = match kind {
                    CodeBlockKind::Indented => None,
                    CodeBlockKind::Fenced(info) => parse_fenced_code_language(&info),
                };
                state = State::CodeBlock {
                    lang,
                    buf: String::new(),
                };
            }

            (State::Paragraph { buf }, Event::Text(t)) => buf.push_str(t.as_ref()),
            (State::Paragraph { buf }, Event::Code(t)) => {
                buf.push('`');
                buf.push_str(t.as_ref());
                buf.push('`');
            }
            (State::Paragraph { buf }, Event::SoftBreak) => buf.push(' '),
            (State::Paragraph { buf }, Event::HardBreak) => buf.push('\n'),
            (State::Paragraph { buf }, Event::End(TagEnd::Paragraph)) => {
                if !buf.trim().is_empty() {
                    blocks.push(Block::Paragraph {
                        text: Arc::<str>::from(buf.trim().to_string()),
                    });
                }
                state = State::None;
            }

            (State::Heading { buf, .. }, Event::Text(t)) => buf.push_str(t.as_ref()),
            (State::Heading { buf, .. }, Event::Code(t)) => buf.push_str(t.as_ref()),
            (State::Heading { buf, .. }, Event::SoftBreak) => buf.push(' '),
            (State::Heading { buf, .. }, Event::HardBreak) => buf.push(' '),
            (State::Heading { level, buf }, Event::End(TagEnd::Heading(_))) => {
                if !buf.trim().is_empty() {
                    blocks.push(Block::Heading {
                        level: *level,
                        text: Arc::<str>::from(buf.trim().to_string()),
                    });
                }
                state = State::None;
            }

            (State::CodeBlock { buf, .. }, Event::Text(t)) => buf.push_str(t.as_ref()),
            (State::CodeBlock { buf, .. }, Event::SoftBreak) => buf.push('\n'),
            (State::CodeBlock { buf, .. }, Event::HardBreak) => buf.push('\n'),
            (State::CodeBlock { lang, buf }, Event::End(TagEnd::CodeBlock)) => {
                blocks.push(Block::CodeBlock {
                    lang: lang.clone(),
                    code: Arc::<str>::from(buf.clone()),
                });
                state = State::None;
            }

            (_, _) => {}
        }
    }

    blocks
}

fn parse_fenced_code_language(info: &str) -> Option<Arc<str>> {
    let info = info.trim();
    if info.is_empty() {
        return None;
    }

    let token = info.split_whitespace().next().unwrap_or("");
    if token.is_empty() {
        return None;
    }

    // Common patterns seen in the wild:
    // - ```rust
    // - ```rust,ignore
    // - ```language-rust
    // - ```{.rust}
    // - ```{.rust .numberLines}
    // - ```{#id .rust}
    let token = token.trim_matches(|c| c == '{' || c == '}');
    let token = token.strip_prefix("language-").unwrap_or(token);
    let token = token.strip_prefix("lang-").unwrap_or(token);

    let token = if token.contains('.') {
        token.split('.').find(|s| !s.is_empty()).unwrap_or(token)
    } else {
        token
    };

    let token = token.split(',').next().unwrap_or(token).trim();
    if token.is_empty() {
        return None;
    }

    Some(Arc::<str>::from(token.to_string()))
}

fn heading_level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    use pulldown_cmark::HeadingLevel;
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
