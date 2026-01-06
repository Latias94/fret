//! Markdown renderer component(s) for Fret.

use std::ops::Range;
use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, ScrollAxis, ScrollProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

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
            Block::CodeBlock { lang, code } => render_code_block(cx, theme, lang.as_deref(), &code),
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

fn render_code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    lang: Option<&str>,
    code: &str,
) -> AnyElement {
    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .p(Space::N2)
            .rounded(Radius::Md)
            .border_1()
            .bg(ColorRef::Color(theme.colors.panel_background))
            .border_color(ColorRef::Color(theme.colors.panel_border)),
        LayoutRefinement::default().w_full(),
    );

    let language = lang.unwrap_or("");
    let spans = fret_syntax::highlight(code, language).unwrap_or_default();

    cx.container(props, |cx| {
        let mut scroll_props = ScrollProps::default();
        scroll_props.axis = ScrollAxis::X;

        vec![cx.scroll(scroll_props, |cx| {
            let lines = split_lines(code);
            let mut out = Vec::with_capacity(lines.len());
            for line in lines {
                out.push(render_code_line(cx, theme, line, &spans));
            }
            out
        })]
    })
}

fn render_code_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line: LineSlice<'_>,
    spans: &[fret_syntax::HighlightSpan],
) -> AnyElement {
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metrics.mono_font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(theme.metrics.mono_font_line_height),
        letter_spacing_em: None,
    };

    let segments = segments_for_range(line.range.clone(), spans, line.text);

    stack::hstack(cx, stack::HStackProps::default().gap(Space::N0), |cx| {
        segments
            .into_iter()
            .map(|(text, highlight)| {
                let color = highlight
                    .and_then(|h| syntax_color(theme, h))
                    .unwrap_or(theme.colors.text_primary);
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::<str>::from(text),
                    style: Some(text_style.clone()),
                    color: Some(color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
            })
            .collect()
    })
}

fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let key = format!("color.syntax.{highlight}");
    if let Some(c) = theme.color_by_key(&key) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    match fallback {
        "comment" => Some(theme.colors.text_muted),
        "string" => Some(theme.colors.viewport_gizmo_y),
        "number" | "boolean" | "constant" => Some(theme.colors.viewport_rotate_gizmo),
        "keyword" | "operator" => Some(theme.colors.accent),
        "type" | "constructor" => Some(theme.colors.viewport_marker),
        "function" => Some(theme.colors.viewport_drag_line_orbit),
        "property" | "variable" => Some(theme.colors.text_primary),
        "punctuation" => Some(theme.colors.text_muted),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct LineSlice<'a> {
    range: Range<usize>,
    text: &'a str,
}

fn split_lines(text: &str) -> Vec<LineSlice<'_>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, b) in text.as_bytes().iter().enumerate() {
        if *b == b'\n' {
            out.push(LineSlice {
                range: start..i,
                text: &text[start..i],
            });
            start = i + 1;
        }
    }
    out.push(LineSlice {
        range: start..text.len(),
        text: &text[start..],
    });
    out
}

fn segments_for_range(
    global_range: Range<usize>,
    spans: &[fret_syntax::HighlightSpan],
    line_text: &str,
) -> Vec<(String, Option<&'static str>)> {
    let mut segments = Vec::new();
    let mut cursor = global_range.start;

    for span in spans {
        if span.range.end <= global_range.start || span.range.start >= global_range.end {
            continue;
        }
        let start = span.range.start.max(global_range.start);
        let end = span.range.end.min(global_range.end);
        if cursor < start {
            let rel = cursor - global_range.start;
            let rel_end = start - global_range.start;
            segments.push((line_text[rel..rel_end].to_string(), None));
        }
        let rel = start - global_range.start;
        let rel_end = end - global_range.start;
        segments.push((line_text[rel..rel_end].to_string(), span.highlight));
        cursor = end;
    }

    if cursor < global_range.end {
        let rel = cursor - global_range.start;
        let rel_end = global_range.end - global_range.start;
        segments.push((line_text[rel..rel_end].to_string(), None));
    }

    if segments.is_empty() {
        segments.push((line_text.to_string(), None));
    }

    segments
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
                    CodeBlockKind::Fenced(info) => {
                        let info = info.trim();
                        if info.is_empty() {
                            None
                        } else {
                            Some(Arc::<str>::from(info.to_string()))
                        }
                    }
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
