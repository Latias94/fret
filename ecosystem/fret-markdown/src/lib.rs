//! Markdown renderer component(s) for Fret.
//!
//! This crate renders Markdown into Fret declarative elements with a focus on editor-grade UI:
//! - streaming/blocked rendering via `mdstream`,
//! - code fences via `fret-code-view`,
//! - optional Mermaid and MathJax integrations behind feature flags.

use std::sync::Arc;

use fret_core::{
    AttributedText, Axis, Edges, FontId, FontWeight, Px, SemanticsRole, StrikethroughStyle,
    TextOverflow, TextPaintStyle, TextShapingStyle, TextSlant, TextSpan, TextStyle, TextWrap,
    UnderlineStyle,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PositionStyle, PressableKeyActivation, PressableProps, ScrollAxis, ScrollProps,
    SelectableTextProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::typography;
use fret_ui_kit::{LayoutRefinement, Space};

pub use mdstream::BlockId;

mod anchors;
mod components;
#[cfg(feature = "imui")]
pub mod imui;
#[cfg(feature = "mathjax-svg")]
mod mathjax_svg_support;
mod mdstream_render;
mod mermaid;
#[cfg(feature = "mermaid")]
mod mermaid_svg_support;
mod open_url;
mod parse;
mod pulldown_render;
#[cfg(test)]
mod semantics_tests;
#[cfg(test)]
mod tests;
mod theme;

pub use anchors::anchor_test_id_from_fragment;
pub use components::*;
pub use mdstream_render::{
    MarkdownPulldownState, MarkdownStreamState, markdown_streaming_pulldown,
    markdown_streaming_pulldown_with, markdown_with,
};
use mermaid::{detect_mermaid_diagram_type, is_mermaid_language, render_mermaid_header_label};
pub use open_url::{OnLinkActivate, is_safe_open_url, on_link_activate_open_url};
use parse::{
    display_math_only_events, heading_level_to_u8, is_display_math_block_text,
    latex_from_pulldown_math_events, parse_code_fence_body, parse_fenced_code_language,
    parse_heading_text, parse_list_info, parse_math_block_body, raw_block_kind_from_mdstream,
    split_trailing_heading_id, strip_blockquote_prefix,
};
use theme::MarkdownTheme;

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

    #[track_caller]
    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        markdown(cx, &self.source)
    }

    #[track_caller]
    pub fn into_element_with<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        components: &MarkdownComponents<H>,
    ) -> AnyElement {
        markdown_with(cx, &self.source, components)
    }
}

#[track_caller]
pub fn markdown<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, source: &str) -> AnyElement {
    markdown_with(cx, source, &MarkdownComponents::default())
}

// `markdown_with` and the mdstream-backed streaming state live in `mdstream_render.rs`.

pub fn mdstream_options_for_markdown() -> mdstream::Options {
    // mdstream defaults to `FootnotesMode::SingleBlock`, which intentionally collapses documents
    // with footnotes into one block (stability-first). For UI rendering we prefer keeping blocks
    // so headings, lists, math blocks, etc can be laid out independently.
    mdstream::Options {
        footnotes: mdstream::FootnotesMode::Invalidate,
        ..Default::default()
    }
}

fn render_code_block<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    info: CodeBlockInfo,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let mut options = components.code_block_ui;
    if components.code_block_max_height_from_theme {
        resolve_code_block_ui(&theme, &mut options);
    }
    if let Some(resolve) = &components.code_block_ui_resolver {
        resolve(cx, &info, &mut options);
    }

    let mut header = fret_code_view::CodeBlockHeaderSlots::default();
    if is_mermaid_language(info.language.as_deref()) {
        let diagram_type = detect_mermaid_diagram_type(&info.code);
        header = header
            .show_language(false)
            .push_left(render_mermaid_header_label(cx, &theme, diagram_type));
    }
    if let Some(render_actions) = &components.code_block_actions {
        header = header.push_right(render_actions(cx, info.clone()));
    }

    #[cfg(feature = "mermaid")]
    if is_mermaid_language(info.language.as_deref()) {
        return mermaid_svg_support::render_mermaid_code_fence(cx, &theme, info, options, header);
    }

    fret_code_view::code_block_with_header_slots(
        cx,
        &info.code,
        info.language.as_deref(),
        false,
        options,
        header,
    )
}

fn resolve_code_block_ui(theme: &Theme, options: &mut fret_code_view::CodeBlockUiOptions) {
    if options.max_height.is_none() {
        let component = "component.markdown.code_block.max_height";
        let canonical = "fret.markdown.code_block.max_height";
        let compat = "markdown.code_block.max_height";
        options.max_height = if theme.metric_key_configured(component) {
            theme.metric_by_key(component)
        } else if theme.metric_key_configured(canonical) {
            theme.metric_by_key(canonical)
        } else if theme.metric_key_configured(compat) {
            theme.metric_by_key(compat)
        } else {
            theme
                .metric_by_key(component)
                .or_else(|| theme.metric_by_key(canonical))
                .or_else(|| theme.metric_by_key(compat))
        };
    }
}

fn render_thematic_break<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    markdown_theme: MarkdownTheme,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(Px(1.0));

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(0.0)).into(),
            background: Some(markdown_theme.hr),
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Default::default(),
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}

fn render_heading_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    info: HeadingInfo,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let font_size = theme.metric_token("metric.font.size");
    let line_height = theme.metric_token("metric.font.line_height");
    let size = match info.level {
        1 => Px(font_size.0 * 1.6),
        2 => Px(font_size.0 * 1.4),
        3 => Px(font_size.0 * 1.2),
        _ => font_size,
    };

    let base = InlineBaseStyle {
        font: FontId::default(),
        size,
        weight: FontWeight::SEMIBOLD,
        line_height: Some(Px(line_height.0 * 1.2)),
    };

    let mut pieces = inline_pieces_maybe_unwrapped(events);
    strip_trailing_heading_id_from_inline_pieces(&mut pieces);
    render_inline_flow_or_rich(cx, theme, markdown_theme, components, base, &pieces)
}

fn strip_trailing_heading_id_from_inline_pieces(pieces: &mut Vec<InlinePiece>) {
    while matches!(pieces.last(), Some(InlinePiece { kind: InlinePieceKind::Text(t), .. }) if t.trim().is_empty())
    {
        pieces.pop();
    }

    let Some((last_text, last_style)) = pieces.last().and_then(|last| match &last.kind {
        InlinePieceKind::Text(t) => Some((t.clone(), last.style.clone())),
        _ => None,
    }) else {
        return;
    };

    let (title, id) = split_trailing_heading_id(&last_text);
    if id.is_none() {
        return;
    }

    pieces.pop();
    if !title.trim().is_empty() {
        pieces.push(InlinePiece {
            kind: InlinePieceKind::Text(title.to_string()),
            style: last_style,
        });
        return;
    }

    while matches!(pieces.last(), Some(InlinePiece { kind: InlinePieceKind::Text(t), .. }) if t.trim().is_empty())
    {
        pieces.pop();
    }
}

fn render_paragraph_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let font_size = theme.metric_token("metric.font.size");
    let line_height = theme.metric_token("metric.font.line_height");
    let base = InlineBaseStyle {
        font: FontId::default(),
        size: font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(line_height),
    };

    let pieces = inline_pieces_maybe_unwrapped(events);
    render_inline_flow_or_rich(cx, theme, markdown_theme, components, base, &pieces)
}

fn render_inline_flow_or_rich<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: InlineBaseStyle,
    pieces: &[InlinePiece],
) -> AnyElement {
    if let Some(el) = render_rich_text_inline(cx, markdown_theme, components, &base, pieces) {
        return el;
    }
    render_inline_flow(cx, theme, markdown_theme, components, base, pieces)
}

fn render_rich_text_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: &InlineBaseStyle,
    pieces: &[InlinePiece],
) -> Option<AnyElement> {
    // If callers provide a custom link renderer, fall back to the tokenized inline-flow path so
    // links can be represented as full declarative subtrees.
    if components.link.is_some() && pieces.iter().any(|p| p.style.link.is_some()) {
        return None;
    }

    let (rich, link_spans) = build_rich_attributed_text(markdown_theme, pieces)?;
    let interactive_spans: Vec<fret_ui::element::SelectableTextInteractiveSpan> =
        if components.on_link_activate.is_some() {
            link_spans
                .into_iter()
                .map(|s| fret_ui::element::SelectableTextInteractiveSpan {
                    range: s.range,
                    tag: s.href,
                })
                .collect()
        } else {
            Vec::new()
        };

    let mut props = SelectableTextProps::new(rich);
    props.layout.size.width = Length::Fill;
    props.style = Some(typography::as_content_text(TextStyle {
        font: base.font.clone(),
        size: base.size,
        weight: base.weight,
        slant: TextSlant::Normal,
        line_height: base.line_height,
        letter_spacing_em: None,
        ..Default::default()
    }));
    // Prefer inheriting the foreground color (DOM `currentColor`-style) so markdown can be used
    // inside composition-level `ForegroundScope` wrappers (e.g. muted surfaces, cards).
    //
    // Explicit per-span colors (links / inline code) are still encoded into `AttributedText`.
    props.color = None;
    // Markdown prose frequently contains long tokens (URLs, paths, identifiers). Default to a
    // break-words policy to prevent horizontal overflow in narrow surfaces.
    props.wrap = TextWrap::WordBreak;
    props.overflow = TextOverflow::Clip;

    if interactive_spans.is_empty() {
        return Some(cx.selectable_text_props(props));
    }

    props.interactive_spans = Arc::from(interactive_spans);

    let on_link_activate = components.on_link_activate.clone();
    let full_text = props.rich.text.clone();
    Some(cx.selectable_text_with_id_props(|cx, id| {
        if let Some(on_link_activate) = on_link_activate {
            cx.selectable_text_on_activate_span_for(
                id,
                Arc::new(move |host, action_cx, reason, activation| {
                    let display = full_text
                        .get(activation.range.clone())
                        .unwrap_or_default()
                        .trim_end()
                        .to_string();
                    on_link_activate(
                        host,
                        action_cx,
                        reason,
                        LinkInfo {
                            href: activation.tag,
                            text: Arc::<str>::from(display),
                        },
                    );
                }),
            );
        }
        props
    }))
}

#[derive(Debug, Clone)]
struct RichLinkSpan {
    range: std::ops::Range<usize>,
    href: Arc<str>,
}

fn build_rich_attributed_text(
    markdown_theme: MarkdownTheme,
    pieces: &[InlinePiece],
) -> Option<(AttributedText, Vec<RichLinkSpan>)> {
    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();
    let mut link_spans: Vec<RichLinkSpan> = Vec::new();

    for p in pieces {
        let InlinePieceKind::Text(t) = &p.kind else {
            return None;
        };
        if t.is_empty() {
            continue;
        }

        let start = text.len();
        text.push_str(t);
        let end = text.len();

        let run_weight = p.style.strong.then_some(FontWeight::SEMIBOLD);
        let run_slant = p.style.emphasis.then_some(TextSlant::Italic);
        let run_font = p.style.code.then_some(FontId::monospace());

        let run_color = if p.style.code {
            Some(markdown_theme.inline_code_fg)
        } else if p.style.link.is_some() {
            Some(markdown_theme.link)
        } else {
            None
        };

        let run_bg = p.style.code.then_some(markdown_theme.inline_code_bg);

        let run_underline = p.style.link.is_some().then_some(UnderlineStyle {
            color: None,
            style: fret_core::DecorationLineStyle::Solid,
        });

        let run_strikethrough = p.style.strikethrough.then_some(StrikethroughStyle {
            color: None,
            style: fret_core::DecorationLineStyle::Solid,
        });

        spans.push(TextSpan {
            len: t.len(),
            shaping: TextShapingStyle {
                font: run_font,
                weight: run_weight,
                slant: run_slant,
                ..Default::default()
            },
            paint: TextPaintStyle {
                fg: run_color,
                bg: run_bg,
                underline: run_underline,
                strikethrough: run_strikethrough,
            },
        });

        if let Some(href) = p.style.link.clone()
            && start < end
        {
            link_spans.push(RichLinkSpan {
                range: start..end,
                href,
            });
        }
    }

    if text.is_empty() {
        return None;
    }

    Some((
        AttributedText::new(Arc::<str>::from(text), spans),
        link_spans,
    ))
}

fn inline_pieces_maybe_unwrapped(events: &[pulldown_cmark::Event<'static>]) -> Vec<InlinePiece> {
    use pulldown_cmark::{Event, Tag};

    let has_wrapper = events.iter().any(|e| match e {
        Event::Start(Tag::Paragraph) | Event::Start(Tag::Heading { .. }) => true,
        _ => false,
    });

    if has_wrapper {
        inline_pieces_from_events(events)
    } else {
        inline_pieces_from_events_unwrapped(events)
    }
}

fn render_math_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    latex: Arc<str>,
) -> AnyElement {
    let latex = Arc::<str>::from(latex.trim().to_string());

    if let Some(render) = &components.math_block {
        return render(cx, MathBlockInfo { latex });
    }

    render_math_block_builtin(cx, theme, markdown_theme, latex)
}

#[cfg(feature = "mathjax-svg")]
fn render_math_block_builtin<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    mathjax_svg_support::render_math_block_mathjax_svg(cx, theme, markdown_theme, latex)
}

#[cfg(not(feature = "mathjax-svg"))]
fn render_math_block_builtin<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;
    scroll_props.layout.size.width = Length::Fill;

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Fill;
    container.padding = Edges::all(markdown_theme.math_block_padding).into();
    container.background = Some(markdown_theme.math_block_bg);
    container.border = Edges::all(Px(0.0));
    container.corner_radii = fret_core::Corners::all(theme.metric_token("metric.radius.md"));

    cx.container(container, |cx| {
        vec![cx.scroll(scroll_props, |cx| {
            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: latex,
                style: Some(typography::as_content_text(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_token("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_token("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                    ..Default::default()
                })),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })]
        })]
    })
}

#[derive(Debug, Clone)]
struct InlineBaseStyle {
    font: FontId,
    size: Px,
    weight: FontWeight,
    line_height: Option<Px>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InlineStyle {
    strong: bool,
    emphasis: bool,
    strikethrough: bool,
    code: bool,
    link: Option<Arc<str>>,
}

fn is_likely_svg_src(src: &str) -> bool {
    let s = src.trim();
    if s.is_empty() {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    lower.ends_with(".svg") || lower.starts_with("data:image/svg+xml")
}

#[derive(Debug, Clone)]
enum InlinePieceKind {
    Text(String),
    Image(ImageInfo),
    InlineMath(InlineMathInfo),
}

#[derive(Debug, Clone)]
struct InlinePiece {
    kind: InlinePieceKind,
    style: InlineStyle,
}

fn parse_events(source: &str) -> Vec<pulldown_cmark::Event<'static>> {
    pulldown_cmark::Parser::new_ext(source, pulldown_options_default())
        .map(|e| e.into_static())
        .collect()
}

fn pulldown_options_default() -> pulldown_cmark::Options {
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    opts.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
    opts.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    opts.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    opts.insert(pulldown_cmark::Options::ENABLE_MATH);
    opts
}

fn inline_pieces_from_events(events: &[pulldown_cmark::Event<'static>]) -> Vec<InlinePiece> {
    inline_pieces_from_events_impl(events, true)
}

fn inline_pieces_from_events_unwrapped(
    events: &[pulldown_cmark::Event<'static>],
) -> Vec<InlinePiece> {
    inline_pieces_from_events_impl(events, false)
}

fn inline_pieces_from_events_impl(
    events: &[pulldown_cmark::Event<'static>],
    require_wrapper: bool,
) -> Vec<InlinePiece> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut strong_depth = 0usize;
    let mut emphasis_depth = 0usize;
    let mut strikethrough_depth = 0usize;
    let mut link_stack: Vec<Arc<str>> = Vec::new();
    let mut pieces: Vec<InlinePiece> = Vec::new();

    let mut wrapper_depth = 0usize;
    let mut image_stack: Vec<(Arc<str>, Option<Arc<str>>, String)> = Vec::new();

    for event in events {
        match event {
            Event::Start(Tag::Paragraph) | Event::Start(Tag::Heading { .. }) => {
                wrapper_depth += 1;
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                wrapper_depth = wrapper_depth.saturating_sub(1);
            }
            _ => {}
        }

        if require_wrapper && wrapper_depth == 0 {
            continue;
        }

        if let Some((_src, _title, alt_buf)) = image_stack.last_mut() {
            match event {
                Event::Text(t) | Event::Code(t) | Event::InlineMath(t) => {
                    alt_buf.push_str(t.as_ref());
                    continue;
                }
                Event::SoftBreak => {
                    alt_buf.push(' ');
                    continue;
                }
                Event::HardBreak => {
                    alt_buf.push('\n');
                    continue;
                }
                _ => {}
            }
        }

        match event {
            Event::Start(Tag::Strong) => strong_depth += 1,
            Event::End(TagEnd::Strong) => strong_depth = strong_depth.saturating_sub(1),
            Event::Start(Tag::Emphasis) => emphasis_depth += 1,
            Event::End(TagEnd::Emphasis) => emphasis_depth = emphasis_depth.saturating_sub(1),
            Event::Start(Tag::Strikethrough) => strikethrough_depth += 1,
            Event::End(TagEnd::Strikethrough) => {
                strikethrough_depth = strikethrough_depth.saturating_sub(1);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_stack.push(Arc::<str>::from(dest_url.to_string()));
            }
            Event::End(TagEnd::Link) => {
                link_stack.pop();
            }
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                let src = Arc::<str>::from(dest_url.to_string());
                let title = if title.is_empty() {
                    None
                } else {
                    Some(Arc::<str>::from(title.to_string()))
                };
                image_stack.push((src, title, String::new()));
            }
            Event::End(TagEnd::Image) => {
                if let Some((src, title, alt)) = image_stack.pop() {
                    pieces.push(InlinePiece {
                        kind: InlinePieceKind::Image(ImageInfo {
                            is_svg: is_likely_svg_src(&src),
                            src,
                            alt: Arc::<str>::from(alt),
                            title,
                        }),
                        style: InlineStyle {
                            strong: false,
                            emphasis: false,
                            strikethrough: false,
                            code: false,
                            link: None,
                        },
                    });
                }
            }
            Event::Text(t) => push_inline_text(
                &mut pieces,
                t.as_ref(),
                InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::Code(t) => push_inline_text(
                &mut pieces,
                t.as_ref(),
                InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: true,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::Html(t) | Event::InlineHtml(t) => push_inline_text(
                &mut pieces,
                {
                    let s = t.as_ref();
                    let trimmed = s.trim();
                    if trimmed.eq_ignore_ascii_case("<br>")
                        || trimmed.eq_ignore_ascii_case("<br/>")
                        || trimmed.eq_ignore_ascii_case("<br />")
                    {
                        "\n"
                    } else {
                        s
                    }
                },
                InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::InlineMath(t) => pieces.push(InlinePiece {
                kind: InlinePieceKind::InlineMath(InlineMathInfo {
                    latex: Arc::<str>::from(t.to_string()),
                }),
                style: InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            }),
            Event::FootnoteReference(label) => {
                let href = Arc::<str>::from(format!("#fn-{}", label));
                push_inline_text(
                    &mut pieces,
                    &format!("[^{}]", label),
                    InlineStyle {
                        strong: false,
                        emphasis: false,
                        strikethrough: false,
                        code: false,
                        link: Some(href),
                    },
                );
            }
            Event::SoftBreak => push_inline_text(
                &mut pieces,
                " ",
                InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::HardBreak => push_inline_text(
                &mut pieces,
                "\n",
                InlineStyle {
                    strong: strong_depth > 0,
                    emphasis: emphasis_depth > 0,
                    strikethrough: strikethrough_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            _ => {}
        }
    }

    pieces
}

fn push_inline_text(pieces: &mut Vec<InlinePiece>, text: &str, style: InlineStyle) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = pieces.last_mut()
        && last.style == style
    {
        if let InlinePieceKind::Text(t) = &mut last.kind {
            t.push_str(text);
        }
        return;
    }
    pieces.push(InlinePiece {
        kind: InlinePieceKind::Text(text.to_string()),
        style,
    });
}

fn render_inline_flow<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: InlineBaseStyle,
    pieces: &[InlinePiece],
) -> AnyElement {
    render_inline_flow_with_layout(
        cx,
        theme,
        markdown_theme,
        components,
        base,
        pieces,
        MainAlign::Start,
    )
}

fn render_inline_flow_with_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: InlineBaseStyle,
    pieces: &[InlinePiece],
    justify: MainAlign,
) -> AnyElement {
    let mut lines: Vec<Vec<InlinePiece>> = Vec::new();
    let mut cur: Vec<InlinePiece> = Vec::new();

    for piece in pieces {
        match &piece.kind {
            InlinePieceKind::Text(text) => {
                let splits: Vec<&str> = text.split('\n').collect();
                for (i, split) in splits.iter().enumerate() {
                    if !split.is_empty() {
                        cur.extend(split_piece_into_tokens(split, &piece.style));
                    }
                    if i + 1 < splits.len() {
                        lines.push(std::mem::take(&mut cur));
                    }
                }
            }
            InlinePieceKind::Image(_) | InlinePieceKind::InlineMath(_) => cur.push(piece.clone()),
        }
    }
    lines.push(cur);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N0)
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            lines
                .into_iter()
                .map(|line| {
                    render_inline_line_with_layout(
                        cx,
                        theme,
                        markdown_theme,
                        components,
                        &base,
                        line,
                        justify,
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn render_inline_line_with_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: &InlineBaseStyle,
    pieces: Vec<InlinePiece>,
    justify: MainAlign,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.layout.size.width = Length::Fill;
    props.direction = Axis::Horizontal;
    props.gap = Px(0.0).into();
    props.padding = Edges::all(Px(0.0)).into();
    props.justify = justify;
    props.align = CrossAlign::Start;
    props.wrap = true;

    cx.flex(props, |cx| {
        coalesce_link_runs(pieces)
            .into_iter()
            .map(|piece| render_inline_token(cx, theme, markdown_theme, components, base, piece))
            .collect::<Vec<_>>()
    })
}

fn render_inline_token<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    base: &InlineBaseStyle,
    piece: InlinePiece,
) -> AnyElement {
    let (kind, style) = (piece.kind, piece.style);

    let raw_text = match kind {
        InlinePieceKind::Image(info) => {
            if let Some(render) = &components.image {
                return render(cx, info);
            }
            return render_image_placeholder(cx, theme, markdown_theme, components, info);
        }
        InlinePieceKind::InlineMath(info) => {
            if let Some(render) = &components.inline_math {
                return render(cx, info);
            }
            return render_inline_math_default(cx, theme, markdown_theme, info);
        }
        InlinePieceKind::Text(text) => text,
    };

    let (font, size, line_height) = if style.code {
        (
            FontId::monospace(),
            theme.metric_token("metric.font.mono_size"),
            Some(theme.metric_token("metric.font.mono_line_height")),
        )
    } else {
        (base.font.clone(), base.size, base.line_height)
    };

    let weight = if style.strong {
        FontWeight::SEMIBOLD
    } else {
        base.weight
    };

    let slant = if style.emphasis {
        TextSlant::Italic
    } else {
        TextSlant::Normal
    };

    // Prefer inheriting the foreground color for normal text runs (DOM `currentColor`-style).
    // Keep links explicitly colored.
    let color = style.link.is_some().then_some(markdown_theme.link);

    if style.code {
        let mut props = ContainerProps::default();
        props.padding = Edges {
            top: markdown_theme.inline_code_padding_y,
            right: markdown_theme.inline_code_padding_x,
            bottom: markdown_theme.inline_code_padding_y,
            left: markdown_theme.inline_code_padding_x,
        }
        .into();
        props.background = Some(markdown_theme.inline_code_bg);
        props.border = Edges::all(Px(0.0));
        props.corner_radii = fret_core::Corners::all(theme.metric_token("metric.radius.sm"));

        return cx.container(props, |cx| {
            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(raw_text),
                style: Some(TextStyle {
                    font,
                    size,
                    weight,
                    slant: TextSlant::Normal,
                    line_height,
                    letter_spacing_em: None,
                    ..Default::default()
                }),
                color: Some(markdown_theme.inline_code_fg),
                // Inline code participates in Markdown prose layout; allow break-words to avoid
                // pathological overflow from long tokens (e.g. long identifiers / URLs).
                wrap: TextWrap::WordBreak,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })]
        });
    }

    if let Some(href) = style.link.clone() {
        let href = href.clone();
        if let Some(render) = &components.link {
            return render(
                cx,
                LinkInfo {
                    href,
                    text: Arc::<str>::from(raw_text.clone()),
                },
            );
        }

        if let Some(on_link_activate) = components.on_link_activate.clone() {
            let link_text = Arc::<str>::from(raw_text.trim_end().to_string());
            let display_text = Arc::<str>::from(raw_text.clone());

            let mut props = PressableProps::default();
            props.a11y.role = Some(SemanticsRole::Link);
            props.a11y.label = Some(link_text.clone());
            props.key_activation = PressableKeyActivation::EnterOnly;

            let el = cx.pressable(props, |cx, _state| {
                let href = href.clone();
                let link_text = link_text.clone();
                let on_link_activate = on_link_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, cx, reason| {
                    on_link_activate(
                        host,
                        cx,
                        reason,
                        LinkInfo {
                            href: href.clone(),
                            text: link_text.clone(),
                        },
                    );
                }));

                vec![render_inline_text_token(
                    cx,
                    theme,
                    font,
                    size,
                    weight,
                    slant,
                    line_height,
                    color,
                    style.strikethrough,
                    TextWrap::WordBreak,
                    display_text.clone(),
                )]
            });
            return el
                .attach_semantics(SemanticsDecoration::default().url(href.clone()).value(href));
        }
    }

    render_inline_text_token(
        cx,
        theme,
        font,
        size,
        weight,
        slant,
        line_height,
        color,
        style.strikethrough,
        TextWrap::WordBreak,
        Arc::<str>::from(raw_text),
    )
}

fn render_inline_text_token<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    font: FontId,
    size: Px,
    weight: FontWeight,
    slant: TextSlant,
    line_height: Option<Px>,
    color: Option<fret_core::Color>,
    strikethrough: bool,
    wrap: TextWrap,
    text: Arc<str>,
) -> AnyElement {
    if !strikethrough {
        return cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(typography::as_content_text(TextStyle {
                font,
                size,
                weight,
                slant,
                line_height,
                letter_spacing_em: None,
                ..Default::default()
            })),
            color,
            wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });
    }

    // Escape hatch: strikethrough currently uses an explicit line element, which needs a concrete
    // color. When no explicit color was requested, fall back to the theme foreground.
    let effective_color = color.unwrap_or_else(|| theme.color_token("foreground"));
    let effective_line_height = line_height.unwrap_or(Px(size.0.max(1.0)));
    let line_y = Px(effective_line_height.0 * 0.55);

    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Relative;
    props.padding = Edges::all(Px(0.0)).into();
    props.border = Edges::all(Px(0.0));

    cx.container(props, |cx| {
        let text_el = cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(typography::as_content_text(TextStyle {
                font,
                size,
                weight,
                slant,
                line_height,
                letter_spacing_em: None,
                ..Default::default()
            })),
            color: Some(effective_color),
            wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let mut line_layout = LayoutStyle::default();
        line_layout.position = PositionStyle::Absolute;
        line_layout.inset.left = Some(Px(0.0)).into();
        line_layout.inset.right = Some(Px(0.0)).into();
        line_layout.inset.top = Some(line_y).into();
        line_layout.size.height = Length::Px(Px(1.0));

        let line_el = cx.container(
            ContainerProps {
                layout: line_layout,
                padding: Edges::all(Px(0.0)).into(),
                background: Some(effective_color),
                border: Edges::all(Px(0.0)),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        vec![text_el, line_el]
    })
}

fn render_image_placeholder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    info: ImageInfo,
) -> AnyElement {
    let label = if info.alt.trim().is_empty() {
        Arc::<str>::from("[image]".to_string())
    } else {
        Arc::<str>::from(format!("[image: {}]", info.alt.trim()))
    };

    if let Some(render) = &components.link {
        return render(
            cx,
            LinkInfo {
                href: info.src,
                text: label,
            },
        );
    }

    if let Some(on_link_activate) = components.on_link_activate.clone() {
        let href = info.src.clone();
        let link_text = label.clone();
        let display_text = label.clone();

        let mut props = PressableProps::default();
        props.a11y.role = Some(SemanticsRole::Link);
        props.a11y.label = Some(link_text.clone());
        props.key_activation = PressableKeyActivation::EnterOnly;

        let el = cx.pressable(props, |cx, _state| {
            let href = href.clone();
            let activate_text = link_text.clone();
            let display_text = display_text.clone();
            let on_link_activate = on_link_activate.clone();
            cx.pressable_on_activate(Arc::new(move |host, cx, reason| {
                on_link_activate(
                    host,
                    cx,
                    reason,
                    LinkInfo {
                        href: href.clone(),
                        text: activate_text.clone(),
                    },
                );
            }));

            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: display_text.clone(),
                style: Some(typography::as_content_text(TextStyle {
                    font: FontId::default(),
                    size: theme.metric_token("metric.font.size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_token("metric.font.line_height")),
                    letter_spacing_em: None,
                    ..Default::default()
                })),
                color: Some(markdown_theme.link),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })]
        });
        return el.attach_semantics(SemanticsDecoration::default().value(href));
    }

    cx.text_props(TextProps {
        layout: Default::default(),
        text: label,
        style: Some(typography::as_content_text(TextStyle {
            font: FontId::default(),
            size: theme.metric_token("metric.font.size"),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: Some(theme.metric_token("metric.font.line_height")),
            letter_spacing_em: None,
            ..Default::default()
        })),
        color: Some(markdown_theme.muted),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: Default::default(),
    })
}

fn render_inline_math_default<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let info = InlineMathInfo {
        latex: Arc::<str>::from(info.latex.trim().to_string()),
    };

    render_inline_math_builtin(cx, theme, markdown_theme, info)
}

#[cfg(feature = "mathjax-svg")]
fn render_inline_math_builtin<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    mathjax_svg_support::render_inline_math_mathjax_svg(cx, theme, markdown_theme, info)
}

#[cfg(not(feature = "mathjax-svg"))]
fn render_inline_math_builtin<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.padding = Edges {
        top: markdown_theme.inline_math_padding_y,
        right: markdown_theme.inline_math_padding_x,
        bottom: markdown_theme.inline_math_padding_y,
        left: markdown_theme.inline_math_padding_x,
    }
    .into();
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_token("metric.radius.sm"));

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: info.latex,
            style: Some(typography::as_content_text(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_token("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: TextSlant::Normal,
                line_height: Some(theme.metric_token("metric.font.mono_line_height")),
                letter_spacing_em: None,
                ..Default::default()
            })),
            color: Some(markdown_theme.inline_math_fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })]
    })
}

fn split_piece_into_tokens(text: &str, style: &InlineStyle) -> Vec<InlinePiece> {
    if text.is_empty() {
        return Vec::new();
    }
    if style.code {
        return vec![InlinePiece {
            kind: InlinePieceKind::Text(text.to_string()),
            style: style.clone(),
        }];
    }

    let mut out: Vec<InlinePiece> = Vec::new();
    let mut i = 0usize;
    while i < text.len() {
        let ch = text[i..].chars().next().unwrap_or(' ');
        let is_ws = ch.is_whitespace();
        let mut j = i + ch.len_utf8();
        while j < text.len() {
            let next = text[j..].chars().next().unwrap_or(' ');
            if next.is_whitespace() != is_ws {
                break;
            }
            j += next.len_utf8();
        }

        let token = &text[i..j];
        i = j;

        if is_ws {
            out.push(InlinePiece {
                kind: InlinePieceKind::Text(token.to_string()),
                style: style.clone(),
            });
            continue;
        }

        let w = token;

        if style.link.is_none() {
            let mut start = 0usize;
            while start < w.len() {
                let b = w.as_bytes()[start];
                if matches!(b, b'(' | b'[' | b'{') {
                    start += 1;
                } else {
                    break;
                }
            }

            let mut end = w.len();
            while end > start {
                let b = w.as_bytes()[end - 1];
                if matches!(
                    b,
                    b'.' | b',' | b';' | b':' | b'!' | b'?' | b')' | b']' | b'}'
                ) {
                    end -= 1;
                } else {
                    break;
                }
            }

            let prefix = &w[..start];
            let candidate = &w[start..end];
            let suffix = &w[end..];

            if (candidate.starts_with("http://") || candidate.starts_with("https://"))
                && is_safe_open_url(candidate)
            {
                if !prefix.is_empty() {
                    out.push(InlinePiece {
                        kind: InlinePieceKind::Text(prefix.to_string()),
                        style: style.clone(),
                    });
                }

                let mut link_style = style.clone();
                link_style.link = Some(Arc::<str>::from(candidate.to_string()));
                out.push(InlinePiece {
                    kind: InlinePieceKind::Text(candidate.to_string()),
                    style: link_style,
                });

                if !suffix.is_empty() {
                    out.push(InlinePiece {
                        kind: InlinePieceKind::Text(suffix.to_string()),
                        style: style.clone(),
                    });
                }
                continue;
            }
        }

        out.push(InlinePiece {
            kind: InlinePieceKind::Text(w.to_string()),
            style: style.clone(),
        });
    }
    out
}

fn coalesce_link_runs(pieces: Vec<InlinePiece>) -> Vec<InlinePiece> {
    let mut out: Vec<InlinePiece> = Vec::new();
    for piece in pieces {
        let mut merged = false;
        if let Some(last) = out.last_mut()
            && last.style == piece.style
            && last.style.link.is_some()
        {
            if let (InlinePieceKind::Text(last_text), InlinePieceKind::Text(cur_text)) =
                (&mut last.kind, &piece.kind)
            {
                last_text.push_str(cur_text);
                merged = true;
            }
        }
        if merged {
            continue;
        }
        out.push(piece);
    }
    out
}
