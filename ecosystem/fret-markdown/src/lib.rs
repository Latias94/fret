//! Markdown renderer component(s) for Fret.

use std::sync::Arc;

use fret_core::{
    Axis, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Effect;
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, ScrollAxis, ScrollProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

pub use mdstream::BlockId;

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

    pub fn into_element_with<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        components: &MarkdownComponents<H>,
    ) -> AnyElement {
        markdown_with(cx, &self.source, components)
    }
}

pub fn markdown<H: UiHost>(cx: &mut ElementContext<'_, H>, source: &str) -> AnyElement {
    markdown_with(cx, source, &MarkdownComponents::default())
}

pub fn markdown_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source: &str,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let blocks = parse_blocks(source);

    markdown_blocks_with(
        cx,
        MarkdownBlocks::from_committed(&blocks),
        &theme,
        components,
    )
}

pub fn markdown_blocks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    blocks: MarkdownBlocks<'_>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    markdown_blocks_with(cx, blocks, &theme, &MarkdownComponents::default())
}

pub fn markdown_streaming<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownStreamingState,
) -> AnyElement {
    markdown_blocks(cx, state.view())
}

pub fn markdown_streaming_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownStreamingState,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    markdown_blocks_with(cx, state.view(), &theme, components)
}

pub fn markdown_blocks_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    blocks: MarkdownBlocks<'_>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().flex_shrink_0()),
        |cx| {
            let mut all =
                Vec::with_capacity(blocks.committed.len() + usize::from(blocks.pending.is_some()));

            cx.for_each_keyed(
                blocks.committed,
                |b| b.id,
                |cx, _i, block| {
                    all.push(block.clone().render(cx, theme, components));
                },
            );

            if let Some(pending) = blocks.pending {
                cx.keyed(pending.id, |cx| {
                    all.push(pending.clone().render(cx, theme, components));
                });
            }

            all
        },
    )
}

#[derive(Debug, Clone)]
pub struct MarkdownBlock {
    pub id: BlockId,
    pub kind: MarkdownBlockKind,
}

impl MarkdownBlock {
    fn render<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        theme: &Theme,
        components: &MarkdownComponents<H>,
    ) -> AnyElement {
        match self.kind {
            MarkdownBlockKind::Heading { level, text } => {
                let info = HeadingInfo { level, text };
                if let Some(render) = &components.heading {
                    render(cx, info)
                } else {
                    render_heading(cx, theme, info.level, info.text)
                }
            }
            MarkdownBlockKind::Paragraph { text } => {
                let info = ParagraphInfo { text };
                if let Some(render) = &components.paragraph {
                    render(cx, info)
                } else {
                    render_paragraph(cx, theme, info.text)
                }
            }
            MarkdownBlockKind::CodeBlock { language, code } => {
                let info = CodeBlockInfo { language, code };
                if let Some(render) = &components.code_block {
                    render(cx, info)
                } else {
                    render_code_block(cx, info, components)
                }
            }
            MarkdownBlockKind::Raw { kind, text } => match kind {
                RawBlockKind::ThematicBreak => {
                    if let Some(render) = &components.thematic_break {
                        render(cx, ThematicBreakInfo)
                    } else {
                        render_thematic_break(cx, theme)
                    }
                }
                RawBlockKind::BlockQuote => {
                    let info = BlockQuoteInfo {
                        text: strip_blockquote_prefix(&text),
                    };
                    if let Some(render) = &components.blockquote {
                        render(cx, info)
                    } else {
                        render_blockquote(cx, theme, components, info)
                    }
                }
                RawBlockKind::List => {
                    let info = parse_list_info(&text);
                    if let Some(render) = &components.list {
                        render(cx, info)
                    } else {
                        render_list(cx, theme, info)
                    }
                }
                RawBlockKind::Table => {
                    let info = TableInfo { text };
                    if let Some(render) = &components.table {
                        render(cx, info)
                    } else {
                        render_table(cx, theme, info)
                    }
                }
                _ => {
                    let info = RawBlockInfo { kind, text };
                    if let Some(render) = &components.raw_block {
                        render(cx, info)
                    } else {
                        render_paragraph(cx, theme, info.text)
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum MarkdownBlockKind {
    Heading {
        level: u8,
        text: Arc<str>,
    },
    Paragraph {
        text: Arc<str>,
    },
    CodeBlock {
        language: Option<Arc<str>>,
        code: Arc<str>,
    },
    Raw {
        kind: RawBlockKind,
        text: Arc<str>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawBlockKind {
    ThematicBreak,
    List,
    BlockQuote,
    Table,
    HtmlBlock,
    MathBlock,
    FootnoteDefinition,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MarkdownBlocks<'a> {
    pub committed: &'a [MarkdownBlock],
    pub pending: Option<&'a MarkdownBlock>,
}

impl<'a> MarkdownBlocks<'a> {
    pub fn from_committed(committed: &'a [MarkdownBlock]) -> Self {
        Self {
            committed,
            pending: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeadingInfo {
    pub level: u8,
    pub text: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct ParagraphInfo {
    pub text: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct CodeBlockInfo {
    pub language: Option<Arc<str>>,
    pub code: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct RawBlockInfo {
    pub kind: RawBlockKind,
    pub text: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct ListInfo {
    pub ordered: bool,
    pub start: u32,
    pub items: Vec<Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct BlockQuoteInfo {
    pub text: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub text: Arc<str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThematicBreakInfo;

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub href: Arc<str>,
    pub text: Arc<str>,
}

pub type HeadingRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, HeadingInfo) -> AnyElement;
pub type ParagraphRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, ParagraphInfo) -> AnyElement;
pub type CodeBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type CodeBlockActionsRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type RawBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, RawBlockInfo) -> AnyElement;
pub type ListRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, ListInfo) -> AnyElement;
pub type BlockQuoteRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, BlockQuoteInfo) -> AnyElement;
pub type TableRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, TableInfo) -> AnyElement;
pub type ThematicBreakRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, ThematicBreakInfo) -> AnyElement;
pub type LinkRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, LinkInfo) -> AnyElement;
pub type OnLinkActivate =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason, LinkInfo) + 'static>;

/// A conservative allowlist for `Effect::OpenUrl` to avoid surprising/suspicious schemes in UI.
///
/// This is intentionally strict:
/// - allow: `http://`, `https://`, `mailto:`
/// - deny: `javascript:`, `data:`, `file:`, empty, whitespace-only
pub fn is_safe_open_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return false;
    }

    let lower = url.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }

    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

/// Convenience: open links via the runner's `Effect::OpenUrl` plumbing (desktop/web).
///
/// Usage:
/// - `components.on_link_activate = Some(fret_markdown::on_link_activate_open_url());`
pub fn on_link_activate_open_url() -> OnLinkActivate {
    Arc::new(|host, _cx, _reason, link| {
        if !is_safe_open_url(&link.href) {
            return;
        }
        host.push_effect(Effect::OpenUrl {
            url: link.href.to_string(),
        });
    })
}
#[derive(Clone)]
pub struct MarkdownComponents<H: UiHost> {
    pub heading: Option<Arc<HeadingRenderer<H>>>,
    pub paragraph: Option<Arc<ParagraphRenderer<H>>>,
    pub code_block: Option<Arc<CodeBlockRenderer<H>>>,
    /// Render an optional “actions” area for fenced code blocks.
    ///
    /// Note: This is only used by the default code block renderer. If you provide `code_block`,
    /// you own the full code fence rendering (including actions).
    pub code_block_actions: Option<Arc<CodeBlockActionsRenderer<H>>>,
    pub raw_block: Option<Arc<RawBlockRenderer<H>>>,
    pub list: Option<Arc<ListRenderer<H>>>,
    pub blockquote: Option<Arc<BlockQuoteRenderer<H>>>,
    pub table: Option<Arc<TableRenderer<H>>>,
    pub thematic_break: Option<Arc<ThematicBreakRenderer<H>>>,
    pub link: Option<Arc<LinkRenderer<H>>>,
    pub on_link_activate: Option<OnLinkActivate>,
}

impl<H: UiHost> Default for MarkdownComponents<H> {
    fn default() -> Self {
        Self {
            heading: None,
            paragraph: None,
            code_block: None,
            code_block_actions: None,
            raw_block: None,
            list: None,
            blockquote: None,
            table: None,
            thematic_break: None,
            link: None,
            on_link_activate: None,
        }
    }
}

impl<H: UiHost> MarkdownComponents<H> {
    pub fn with_open_url(mut self) -> Self {
        self.on_link_activate = Some(on_link_activate_open_url());
        self
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

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.shrink = 0.0;

    cx.text_props(TextProps {
        layout,
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
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.shrink = 0.0;

    cx.text_props(TextProps {
        layout,
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

fn parse_blocks(source: &str) -> Vec<MarkdownBlock> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

    enum State {
        None,
        Paragraph { buf: String },
        Heading { level: u8, buf: String },
        CodeBlock { lang: Option<Arc<str>>, buf: String },
    }

    let mut blocks: Vec<MarkdownBlockKind> = Vec::new();
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
                    blocks.push(MarkdownBlockKind::Paragraph {
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
                    blocks.push(MarkdownBlockKind::Heading {
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
                blocks.push(MarkdownBlockKind::CodeBlock {
                    language: lang.clone(),
                    code: Arc::<str>::from(buf.clone()),
                });
                state = State::None;
            }

            (_, _) => {}
        }
    }

    blocks
        .into_iter()
        .enumerate()
        .map(|(i, kind)| MarkdownBlock {
            id: BlockId((i as u64) + 1),
            kind,
        })
        .collect()
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

fn render_code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    info: CodeBlockInfo,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let code_view = fret_code_view::code_block(cx, &info.code, info.language.as_deref(), false);

    let Some(render_actions) = &components.code_block_actions else {
        return code_view;
    };

    let actions = render_actions(cx, info);
    stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
        vec![actions, code_view]
    })
}

fn render_thematic_break<H: UiHost>(cx: &mut ElementContext<'_, H>, theme: &Theme) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(Px(1.0));

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(0.0)),
            background: Some(theme.colors.panel_border),
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Default::default(),
        },
        |_cx| Vec::new(),
    )
}

fn render_blockquote<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    info: BlockQuoteInfo,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::all(theme.metrics.padding_sm);
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: Px(3.0),
    };
    props.border_color = Some(theme.colors.panel_border);

    cx.container(props, |cx| vec![markdown_with(cx, &info.text, components)])
}

fn render_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    info: ListInfo,
) -> AnyElement {
    if info.items.is_empty() {
        return cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::<str>::from(""),
            style: None,
            color: Some(theme.colors.text_primary),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });
    }

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |cx| {
        info.items
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                let marker = if info.ordered {
                    Arc::<str>::from(format!("{}.", info.start.saturating_add(i as u32)))
                } else {
                    Arc::<str>::from("•".to_string())
                };

                stack::hstack(cx, stack::HStackProps::default().gap(Space::N2), |cx| {
                    let marker_el = cx.text_props(TextProps {
                        layout: Default::default(),
                        text: marker,
                        style: None,
                        color: Some(theme.colors.text_muted),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    });
                    let text_el = render_paragraph(cx, theme, text);
                    vec![marker_el, text_el]
                })
            })
            .collect()
    })
}

fn render_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    info: TableInfo,
) -> AnyElement {
    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;

    let style = TextStyle {
        font: FontId::monospace(),
        size: theme.metrics.mono_font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(theme.metrics.mono_font_line_height),
        letter_spacing_em: None,
    };

    cx.scroll(scroll_props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: info.text,
            style: Some(style),
            color: Some(theme.colors.text_primary),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}

fn strip_blockquote_prefix(text: &str) -> Arc<str> {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let mut s = line;
        let mut spaces = 0usize;
        while spaces < 3 && s.starts_with(' ') {
            s = &s[1..];
            spaces += 1;
        }
        if let Some(rest) = s.strip_prefix('>') {
            out.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        } else {
            out.push_str(s);
        }
    }
    Arc::<str>::from(out.trim_end().to_string())
}

fn parse_list_info(text: &str) -> ListInfo {
    let mut ordered = None::<bool>;
    let mut start = 1u32;
    let mut items: Vec<String> = Vec::new();
    let mut cur: Option<String> = None;

    for line in text.lines() {
        if let Some((o, num, content)) = parse_list_item_start(line) {
            if let Some(prev) = cur.take() {
                if !prev.trim().is_empty() {
                    items.push(prev.trim_end().to_string());
                }
            }
            if ordered.is_none() {
                ordered = Some(o);
                if o {
                    start = num.max(1);
                }
            }
            cur = Some(content.to_string());
            continue;
        }

        if let Some(buf) = cur.as_mut() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(trimmed.trim_start());
        }
    }

    if let Some(prev) = cur.take() {
        if !prev.trim().is_empty() {
            items.push(prev.trim_end().to_string());
        }
    }

    ListInfo {
        ordered: ordered.unwrap_or(false),
        start,
        items: items.into_iter().map(|s| Arc::<str>::from(s)).collect(),
    }
}

fn parse_list_item_start(line: &str) -> Option<(bool, u32, &str)> {
    let mut s = line;
    let mut spaces = 0usize;
    while spaces < 3 && s.starts_with(' ') {
        s = &s[1..];
        spaces += 1;
    }

    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        match bytes[0] {
            b'-' | b'+' | b'*' if bytes[1] == b' ' || bytes[1] == b'\t' => {
                return Some((false, 1, s[2..].trim_start()));
            }
            _ => {}
        }
    }

    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i + 1 >= bytes.len() {
        return None;
    }
    let delim = bytes[i];
    if delim != b'.' && delim != b')' {
        return None;
    }
    let ws = bytes[i + 1];
    if ws != b' ' && ws != b'\t' {
        return None;
    }
    let num: u32 = s[..i].parse().ok()?;
    Some((true, num, s[i + 2..].trim_start()))
}

#[derive(Debug, Default, Clone)]
pub struct MarkdownStreamingState {
    committed: Vec<MarkdownBlock>,
    pending: Option<MarkdownBlock>,
}

impl MarkdownStreamingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn committed(&self) -> &[MarkdownBlock] {
        &self.committed
    }

    pub fn pending(&self) -> Option<&MarkdownBlock> {
        self.pending.as_ref()
    }

    pub fn view(&self) -> MarkdownBlocks<'_> {
        MarkdownBlocks {
            committed: &self.committed,
            pending: self.pending.as_ref(),
        }
    }

    pub fn clear(&mut self) {
        self.committed.clear();
        self.pending = None;
    }

    pub fn apply_update(&mut self, update: mdstream::Update) -> mdstream::AppliedUpdate {
        if update.reset {
            self.clear();
        }

        for block in &update.committed {
            self.committed.push(convert_mdstream_block(block));
        }

        self.pending = update.pending.as_ref().map(convert_mdstream_block);

        mdstream::AppliedUpdate {
            reset: update.reset,
            invalidated: update.invalidated,
        }
    }

    pub fn apply_update_ref(
        &mut self,
        update: &mdstream::UpdateRef<'_>,
    ) -> mdstream::AppliedUpdate {
        if update.reset {
            self.clear();
        }

        for block in update.committed {
            self.committed.push(convert_mdstream_block(block));
        }

        self.pending = update.pending.as_ref().map(convert_mdstream_pending_ref);

        mdstream::AppliedUpdate {
            reset: update.reset,
            invalidated: update.invalidated.clone(),
        }
    }
}

fn convert_mdstream_pending_ref(p: &mdstream::PendingBlockRef<'_>) -> MarkdownBlock {
    let raw = p.display.unwrap_or(p.raw);
    let kind = convert_mdstream_kind(p.kind, raw);
    MarkdownBlock { id: p.id, kind }
}

fn convert_mdstream_block(block: &mdstream::Block) -> MarkdownBlock {
    let raw = block.display_or_raw();
    let kind = convert_mdstream_kind(block.kind, raw);
    MarkdownBlock { id: block.id, kind }
}

fn convert_mdstream_kind(kind: mdstream::BlockKind, raw: &str) -> MarkdownBlockKind {
    match kind {
        mdstream::BlockKind::Heading => {
            if let Some((level, text)) = parse_heading_text(raw) {
                MarkdownBlockKind::Heading { level, text }
            } else {
                MarkdownBlockKind::Raw {
                    kind: RawBlockKind::Unknown,
                    text: Arc::<str>::from(raw.trim().to_string()),
                }
            }
        }
        mdstream::BlockKind::Paragraph => MarkdownBlockKind::Paragraph {
            text: Arc::<str>::from(normalize_paragraph_text(raw)),
        },
        mdstream::BlockKind::CodeFence => {
            let (language, code) = parse_code_fence_body(raw);
            MarkdownBlockKind::CodeBlock { language, code }
        }
        mdstream::BlockKind::ThematicBreak => MarkdownBlockKind::Raw {
            kind: RawBlockKind::ThematicBreak,
            text: Arc::<str>::from(raw.trim().to_string()),
        },
        mdstream::BlockKind::List => MarkdownBlockKind::Raw {
            kind: RawBlockKind::List,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::BlockQuote => MarkdownBlockKind::Raw {
            kind: RawBlockKind::BlockQuote,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::Table => MarkdownBlockKind::Raw {
            kind: RawBlockKind::Table,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::HtmlBlock => MarkdownBlockKind::Raw {
            kind: RawBlockKind::HtmlBlock,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::MathBlock => MarkdownBlockKind::Raw {
            kind: RawBlockKind::MathBlock,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::FootnoteDefinition => MarkdownBlockKind::Raw {
            kind: RawBlockKind::FootnoteDefinition,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
        mdstream::BlockKind::Unknown => MarkdownBlockKind::Raw {
            kind: RawBlockKind::Unknown,
            text: Arc::<str>::from(raw.trim_end().to_string()),
        },
    }
}

fn normalize_paragraph_text(raw: &str) -> String {
    raw.split('\n')
        .map(str::trim_end)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_heading_text(raw: &str) -> Option<(u8, Arc<str>)> {
    let mut lines = raw.lines();
    let first = lines.next()?.trim_end();
    let second = lines.next().map(str::trim_end);

    // ATX: ### Title
    let atx = first.trim_start_matches(' ');
    if let Some(rest) = atx.strip_prefix('#') {
        let mut level = 1u8;
        let mut tail = rest;
        while level < 6 && tail.starts_with('#') {
            level += 1;
            tail = &tail[1..];
        }
        if !tail.starts_with([' ', '\t']) {
            return None;
        }
        let text = tail.trim();
        if text.is_empty() {
            return None;
        }
        return Some((level, Arc::<str>::from(text.to_string())));
    }

    // Setext:
    // Title
    // -----
    if let Some(underline) = second {
        let underline_trimmed = underline.trim_start_matches(' ').trim_end();
        if underline_trimmed.chars().all(|c| c == '=') && underline_trimmed.len() >= 2 {
            let text = first.trim();
            if text.is_empty() {
                return None;
            }
            return Some((1, Arc::<str>::from(text.to_string())));
        }
        if underline_trimmed.chars().all(|c| c == '-') && underline_trimmed.len() >= 2 {
            let text = first.trim();
            if text.is_empty() {
                return None;
            }
            return Some((2, Arc::<str>::from(text.to_string())));
        }
    }

    None
}

fn parse_code_fence_body(raw: &str) -> (Option<Arc<str>>, Arc<str>) {
    let header = mdstream::syntax::parse_code_fence_header_from_block(raw);
    let language = header
        .and_then(|h| h.language)
        .and_then(|lang| parse_fenced_code_language(lang))
        .or_else(|| {
            header
                .and_then(|h| h.language)
                .map(|s| Arc::<str>::from(s.to_string()))
        });

    let mut lines = raw.lines();
    let first = lines.next().unwrap_or("");
    let mut body_lines: Vec<&str> = lines.collect();

    if let Some(h) = header {
        if let Some(last) = body_lines.last().copied()
            && mdstream::syntax::is_code_fence_closing_line(last, h.fence_char, h.fence_len)
        {
            body_lines.pop();
        }
    }

    let _ = first;
    let body = body_lines.join("\n");
    (language, Arc::<str>::from(body))
}

#[derive(Debug, Default)]
pub struct MarkdownPulldownState {
    doc: mdstream::DocumentState,
    adapter: mdstream::adapters::pulldown::PulldownAdapter,
}

impl MarkdownPulldownState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn doc(&self) -> &mdstream::DocumentState {
        &self.doc
    }

    pub fn clear(&mut self) {
        self.doc.clear();
        self.adapter.clear();
    }

    pub fn apply_update(&mut self, update: mdstream::Update) -> mdstream::AppliedUpdate {
        self.adapter.apply_update(&update);
        self.doc.apply(update)
    }

    pub fn apply_update_ref(
        &mut self,
        update: &mdstream::UpdateRef<'_>,
    ) -> mdstream::AppliedUpdate {
        // Note: `UpdateRef` borrows from `MdStream`. Convert to an owned update to keep this state
        // render- and pipeline-agnostic (safe to store).
        self.apply_update(update.to_owned())
    }
}

pub fn markdown_streaming_pulldown<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
) -> AnyElement {
    markdown_streaming_pulldown_with(cx, state, &MarkdownComponents::default())
}

pub fn markdown_streaming_pulldown_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    markdown_mdstream_pulldown_with(cx, &theme, state.doc(), &state.adapter, components)
}

fn markdown_mdstream_pulldown_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    doc: &mdstream::DocumentState,
    adapter: &mdstream::adapters::pulldown::PulldownAdapter,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let committed = doc.committed();
    let pending = doc.pending();

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            let mut out = Vec::with_capacity(committed.len() + usize::from(pending.is_some()));

            cx.for_each_keyed(
                committed,
                |b| b.id,
                |cx, _i, block| match adapter.committed_events(block.id) {
                    Some(events) => out.push(render_mdstream_block_with_events(
                        cx, theme, components, block, events,
                    )),
                    None => {
                        let tmp = parse_events(block.display_or_raw());
                        out.push(render_mdstream_block_with_events(
                            cx, theme, components, block, &tmp,
                        ));
                    }
                },
            );

            if let Some(pending) = pending {
                cx.keyed(pending.id, |cx| {
                    let events = adapter.parse_pending(pending);
                    out.push(render_mdstream_block_with_events(
                        cx, theme, components, pending, &events,
                    ));
                });
            }

            out
        },
    )
}

fn render_mdstream_block_with_events<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    block: &mdstream::Block,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    match block.kind {
        mdstream::BlockKind::Heading => {
            let (level, text) = parse_heading_text(block.display_or_raw()).unwrap_or((
                1,
                Arc::<str>::from(block.display_or_raw().trim().to_string()),
            ));
            let info = HeadingInfo { level, text };
            if let Some(render) = &components.heading {
                render(cx, info)
            } else {
                render_heading_inline(cx, theme, components, info, events)
            }
        }
        mdstream::BlockKind::Paragraph => {
            let info = ParagraphInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.paragraph {
                render(cx, info)
            } else {
                render_paragraph_inline(cx, theme, components, events)
            }
        }
        mdstream::BlockKind::ThematicBreak => {
            if let Some(render) = &components.thematic_break {
                render(cx, ThematicBreakInfo)
            } else {
                render_thematic_break(cx, theme)
            }
        }
        mdstream::BlockKind::CodeFence => {
            let (language, code) = parse_code_fence_body(block.display_or_raw());
            let info = CodeBlockInfo { language, code };
            if let Some(render) = &components.code_block {
                render(cx, info)
            } else {
                render_code_block(cx, info, components)
            }
        }
        mdstream::BlockKind::List => {
            let list = parse_list_info(block.display_or_raw());
            if let Some(render) = &components.list {
                render(cx, list)
            } else {
                render_pulldown_events_root(cx, theme, components, events)
            }
        }
        mdstream::BlockKind::BlockQuote => {
            let info = BlockQuoteInfo {
                text: strip_blockquote_prefix(block.display_or_raw()),
            };
            if let Some(render) = &components.blockquote {
                render(cx, info)
            } else {
                render_pulldown_events_root(cx, theme, components, events)
            }
        }
        mdstream::BlockKind::Table => {
            let info = TableInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.table {
                render(cx, info)
            } else {
                render_table(cx, theme, info)
            }
        }
        _ => {
            let info = RawBlockInfo {
                kind: RawBlockKind::Unknown,
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.raw_block {
                render(cx, info)
            } else {
                render_paragraph(cx, theme, info.text)
            }
        }
    }
}

fn render_heading_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    info: HeadingInfo,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let size = match info.level {
        1 => Px(theme.metrics.font_size.0 * 1.6),
        2 => Px(theme.metrics.font_size.0 * 1.4),
        3 => Px(theme.metrics.font_size.0 * 1.2),
        _ => theme.metrics.font_size,
    };

    let base = InlineBaseStyle {
        font: FontId::default(),
        size,
        weight: FontWeight::SEMIBOLD,
        line_height: Some(Px(theme.metrics.font_line_height.0 * 1.2)),
        color: theme.colors.text_primary,
    };

    let pieces = inline_pieces_from_events(events);
    render_inline_flow(cx, theme, components, base, &pieces)
}

fn render_paragraph_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let base = InlineBaseStyle {
        font: FontId::default(),
        size: theme.metrics.font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(theme.metrics.font_line_height),
        color: theme.colors.text_primary,
    };

    let pieces = inline_pieces_from_events(events);
    render_inline_flow(cx, theme, components, base, &pieces)
}

fn render_pulldown_events_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let mut cursor = 0usize;
    let children = render_pulldown_blocks(cx, theme, components, events, &mut cursor, None);
    if children.len() == 1 {
        return children.into_iter().next().unwrap();
    }

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
        children
    })
}

#[derive(Debug, Clone, Copy)]
enum PulldownStop {
    Item,
    BlockQuote,
}

fn stop_matches(end: &pulldown_cmark::TagEnd, stop: PulldownStop) -> bool {
    use pulldown_cmark::TagEnd;
    match (stop, end) {
        (PulldownStop::Item, TagEnd::Item) => true,
        (PulldownStop::BlockQuote, TagEnd::BlockQuote(_)) => true,
        _ => false,
    }
}

fn render_pulldown_blocks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    stop: Option<PulldownStop>,
) -> Vec<AnyElement> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut out = Vec::new();
    while *cursor < events.len() {
        match (&events[*cursor], stop) {
            (Event::End(end), Some(stop)) if stop_matches(end, stop) => {
                *cursor += 1;
                break;
            }
            _ => {}
        }

        match &events[*cursor] {
            Event::Start(Tag::Paragraph) => out.push(render_pulldown_paragraph(
                cx, theme, components, events, cursor,
            )),
            Event::Start(Tag::Heading { level, .. }) => out.push(render_pulldown_heading(
                cx,
                theme,
                components,
                events,
                cursor,
                heading_level_to_u8(*level),
            )),
            Event::Start(Tag::CodeBlock(kind)) => out.push(render_pulldown_code_block(
                cx,
                components,
                events,
                cursor,
                kind.clone(),
            )),
            Event::Start(Tag::List(start)) => out.push(render_pulldown_list(
                cx, theme, components, events, cursor, *start,
            )),
            Event::Start(Tag::BlockQuote(_)) => out.push(render_pulldown_blockquote(
                cx, theme, components, events, cursor,
            )),
            Event::Rule => {
                out.push(render_thematic_break(cx, theme));
                *cursor += 1;
            }
            Event::End(TagEnd::List(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::BlockQuote(_)) => {
                *cursor += 1;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    out
}

fn render_pulldown_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::Paragraph)) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }
    render_paragraph_inline(cx, theme, components, &events[start..*cursor])
}

fn render_pulldown_heading<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    level: u8,
) -> AnyElement {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::Heading(_))) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }

    let slice = &events[start..*cursor];
    let info = HeadingInfo {
        level,
        text: plain_text_from_events(slice),
    };
    render_heading_inline(cx, theme, components, info, slice)
}

fn render_pulldown_code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    kind: pulldown_cmark::CodeBlockKind<'static>,
) -> AnyElement {
    use pulldown_cmark::{CodeBlockKind, Event, TagEnd};

    let language = match &kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => parse_fenced_code_language(info),
    };

    *cursor += 1;
    let mut buf = String::new();
    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Text(t) => buf.push_str(t.as_ref()),
            Event::SoftBreak | Event::HardBreak => buf.push('\n'),
            Event::End(TagEnd::CodeBlock) => {
                *cursor += 1;
                break;
            }
            _ => {}
        }
        *cursor += 1;
    }

    let info = CodeBlockInfo {
        language,
        code: Arc::<str>::from(buf),
    };
    if let Some(render) = &components.code_block {
        render(cx, info)
    } else {
        render_code_block(cx, info, components)
    }
}

fn render_pulldown_blockquote<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    *cursor += 1;
    let children = render_pulldown_blocks(
        cx,
        theme,
        components,
        events,
        cursor,
        Some(PulldownStop::BlockQuote),
    );
    render_blockquote_container(cx, theme, children)
}

fn render_blockquote_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::all(theme.metrics.padding_sm);
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: Px(3.0),
    };
    props.border_color = Some(theme.colors.panel_border);

    cx.container(props, |cx| {
        if children.len() == 1 {
            children
        } else {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                |_cx| children,
            )]
        }
    })
}

fn render_pulldown_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    start: Option<u64>,
) -> AnyElement {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let ordered = start.is_some();
    let start_no = start.unwrap_or(1) as u32;

    *cursor += 1;
    let mut items: Vec<Vec<AnyElement>> = Vec::new();

    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::Item) => {
                *cursor += 1;
                let children = render_pulldown_blocks(
                    cx,
                    theme,
                    components,
                    events,
                    cursor,
                    Some(PulldownStop::Item),
                );
                items.push(children);
            }
            Event::End(TagEnd::List(_)) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |cx| {
        items
            .into_iter()
            .enumerate()
            .map(|(i, children)| {
                let marker = if ordered {
                    Arc::<str>::from(format!("{}.", start_no.saturating_add(i as u32)))
                } else {
                    Arc::<str>::from("•".to_string())
                };

                let marker_el = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: marker,
                    style: None,
                    color: Some(theme.colors.text_muted),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                });

                let body = if children.len() == 1 {
                    children.into_iter().next().unwrap()
                } else {
                    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
                        children
                    })
                };

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_start(),
                    |_cx| vec![marker_el, body],
                )
            })
            .collect()
    })
}

fn plain_text_from_events(events: &[pulldown_cmark::Event<'static>]) -> Arc<str> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut out = String::new();
    let mut wrapper_depth = 0usize;

    for e in events {
        match e {
            Event::Start(Tag::Paragraph) | Event::Start(Tag::Heading { .. }) => {
                wrapper_depth += 1;
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                wrapper_depth = wrapper_depth.saturating_sub(1);
            }
            _ => {}
        }

        if wrapper_depth == 0 {
            continue;
        }

        match e {
            Event::Text(t) | Event::Code(t) => out.push_str(t.as_ref()),
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push('\n'),
            _ => {}
        }
    }

    Arc::<str>::from(out.trim().to_string())
}

#[derive(Debug, Clone)]
struct InlineBaseStyle {
    font: FontId,
    size: Px,
    weight: FontWeight,
    line_height: Option<Px>,
    color: fret_core::Color,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InlineStyle {
    strong: bool,
    code: bool,
    link: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
struct InlinePiece {
    text: String,
    style: InlineStyle,
}

fn parse_events(source: &str) -> Vec<pulldown_cmark::Event<'static>> {
    pulldown_cmark::Parser::new(source)
        .map(|e| e.into_static())
        .collect()
}

fn inline_pieces_from_events(events: &[pulldown_cmark::Event<'static>]) -> Vec<InlinePiece> {
    inline_pieces_from_events_impl(events, true)
}

#[cfg(test)]
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
    let mut link_stack: Vec<Arc<str>> = Vec::new();
    let mut pieces: Vec<InlinePiece> = Vec::new();

    let mut wrapper_depth = 0usize;

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

        match event {
            Event::Start(Tag::Strong) => strong_depth += 1,
            Event::End(TagEnd::Strong) => strong_depth = strong_depth.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_stack.push(Arc::<str>::from(dest_url.to_string()));
            }
            Event::End(TagEnd::Link) => {
                link_stack.pop();
            }
            Event::Text(t) => push_inline_text(
                &mut pieces,
                t.as_ref(),
                InlineStyle {
                    strong: strong_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::Code(t) => push_inline_text(
                &mut pieces,
                t.as_ref(),
                InlineStyle {
                    strong: strong_depth > 0,
                    code: true,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::SoftBreak => push_inline_text(
                &mut pieces,
                " ",
                InlineStyle {
                    strong: strong_depth > 0,
                    code: false,
                    link: link_stack.last().cloned(),
                },
            ),
            Event::HardBreak => push_inline_text(
                &mut pieces,
                "\n",
                InlineStyle {
                    strong: strong_depth > 0,
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
        last.text.push_str(text);
        return;
    }
    pieces.push(InlinePiece {
        text: text.to_string(),
        style,
    });
}

fn render_inline_flow<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    base: InlineBaseStyle,
    pieces: &[InlinePiece],
) -> AnyElement {
    let mut lines: Vec<Vec<InlinePiece>> = Vec::new();
    let mut cur: Vec<InlinePiece> = Vec::new();

    for piece in pieces {
        let splits: Vec<&str> = piece.text.split('\n').collect();
        for (i, split) in splits.iter().enumerate() {
            if !split.is_empty() {
                cur.extend(split_piece_into_tokens(split, &piece.style));
            }
            if i + 1 < splits.len() {
                lines.push(std::mem::take(&mut cur));
            }
        }
    }
    lines.push(cur);

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N0), |cx| {
        lines
            .into_iter()
            .map(|line| render_inline_line(cx, theme, components, &base, line))
            .collect()
    })
}

fn render_inline_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    base: &InlineBaseStyle,
    pieces: Vec<InlinePiece>,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.layout.size.width = Length::Fill;
    props.direction = Axis::Horizontal;
    props.gap = Px(0.0);
    props.padding = Edges::all(Px(0.0));
    props.justify = MainAlign::Start;
    props.align = CrossAlign::Start;
    props.wrap = true;

    cx.flex(props, |cx| {
        coalesce_link_runs(pieces)
            .into_iter()
            .map(|piece| render_inline_token(cx, theme, components, base, piece))
            .collect()
    })
}

fn render_inline_token<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    components: &MarkdownComponents<H>,
    base: &InlineBaseStyle,
    piece: InlinePiece,
) -> AnyElement {
    let (font, size, line_height) = if piece.style.code {
        (
            FontId::monospace(),
            theme.metrics.mono_font_size,
            Some(theme.metrics.mono_font_line_height),
        )
    } else {
        (base.font.clone(), base.size, base.line_height)
    };

    let weight = if piece.style.strong {
        FontWeight::SEMIBOLD
    } else {
        base.weight
    };

    let color = if piece.style.link.is_some() {
        theme.colors.accent
    } else {
        base.color
    };

    if let Some(href) = piece.style.link.clone() {
        let href = href.clone();
        if let Some(render) = &components.link {
            return render(
                cx,
                LinkInfo {
                    href,
                    text: Arc::<str>::from(piece.text),
                },
            );
        }

        if let Some(on_link_activate) = components.on_link_activate.clone() {
            let link_text = Arc::<str>::from(piece.text.trim_end().to_string());
            let display_text = Arc::<str>::from(piece.text);

            let mut props = PressableProps::default();
            props.layout.flex.shrink = 0.0;
            props.a11y.role = Some(SemanticsRole::Button);
            props.a11y.label = Some(link_text.clone());

            return cx.pressable(props, |cx, _state| {
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

                let mut layout = LayoutStyle::default();
                layout.flex.shrink = 0.0;

                vec![cx.text_props(TextProps {
                    layout,
                    text: display_text.clone(),
                    style: Some(TextStyle {
                        font,
                        size,
                        weight,
                        line_height,
                        letter_spacing_em: None,
                    }),
                    color: Some(color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            });
        }
    }

    let mut layout = LayoutStyle::default();
    layout.flex.shrink = 0.0;

    cx.text_props(TextProps {
        layout,
        text: Arc::<str>::from(piece.text),
        style: Some(TextStyle {
            font,
            size,
            weight,
            line_height,
            letter_spacing_em: None,
        }),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn split_piece_into_tokens(text: &str, style: &InlineStyle) -> Vec<InlinePiece> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    if style.code {
        return vec![InlinePiece {
            text: text.to_string(),
            style: style.clone(),
        }];
    }

    let mut out: Vec<InlinePiece> = Vec::new();
    let words: Vec<&str> = text.split_whitespace().filter(|s| !s.is_empty()).collect();
    for (i, w) in words.iter().enumerate() {
        let mut token = w.to_string();
        if i + 1 < words.len() {
            token.push(' ');
        }
        out.push(InlinePiece {
            text: token,
            style: style.clone(),
        });
    }
    out
}

fn coalesce_link_runs(pieces: Vec<InlinePiece>) -> Vec<InlinePiece> {
    let mut out: Vec<InlinePiece> = Vec::new();
    for piece in pieces {
        if let Some(last) = out.last_mut()
            && last.style == piece.style
            && last.style.link.is_some()
        {
            last.text.push_str(&piece.text);
            continue;
        }
        out.push(piece);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_top_level_list_items(events: &[pulldown_cmark::Event<'static>]) -> usize {
        use pulldown_cmark::{Event, Tag, TagEnd};

        let mut list_depth = 0usize;
        let mut count = 0usize;

        for e in events {
            match e {
                Event::Start(Tag::List(_)) => list_depth += 1,
                Event::End(TagEnd::List(_)) => list_depth = list_depth.saturating_sub(1),
                Event::Start(Tag::Item) if list_depth == 1 => count += 1,
                _ => {}
            }
        }

        count
    }

    #[test]
    fn parses_fenced_language_variants() {
        assert_eq!(parse_fenced_code_language("rust").as_deref(), Some("rust"));
        assert_eq!(
            parse_fenced_code_language("rust,ignore").as_deref(),
            Some("rust")
        );
        assert_eq!(
            parse_fenced_code_language("language-rust").as_deref(),
            Some("rust")
        );
        assert_eq!(
            parse_fenced_code_language("{.rust .numberLines}").as_deref(),
            Some("rust")
        );
    }

    #[test]
    fn assigns_stable_index_ids_for_static_parse() {
        let blocks = parse_blocks("# A\n\nB\n\n```rust\nfn main() {}\n```\n");
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].id, BlockId(1));
        assert_eq!(blocks[1].id, BlockId(2));
        assert_eq!(blocks[2].id, BlockId(3));
    }

    #[test]
    fn mdstream_blocks_apply_incrementally() {
        let mut stream = mdstream::MdStream::default();
        let mut state = MarkdownStreamingState::new();

        let u1 = stream.append("Hello\n\n```rust\nfn main() {");
        let a1 = state.apply_update(u1);
        assert!(!a1.reset);
        assert_eq!(state.committed().len(), 1);
        assert!(state.pending().is_some());

        let u2 = stream.append("}\n```\n");
        let _a2 = state.apply_update(u2);
        assert_eq!(state.committed().len(), 2);
        assert!(state.pending().is_none());
    }

    #[test]
    fn parses_list_items() {
        let info = parse_list_info("- a\n- b\n  c\n");
        assert!(!info.ordered);
        assert_eq!(info.items.len(), 2);
        assert_eq!(info.items[0].as_ref(), "a");
        assert_eq!(info.items[1].as_ref(), "b\nc");

        let info = parse_list_info("2. a\n3. b\n");
        assert!(info.ordered);
        assert_eq!(info.start, 2);
        assert_eq!(info.items.len(), 2);
        assert_eq!(info.items[0].as_ref(), "a");
        assert_eq!(info.items[1].as_ref(), "b");
    }

    #[test]
    fn strips_blockquote_prefixes() {
        let text = Arc::<str>::from("> a\n> b\n  > c\n");
        let out = strip_blockquote_prefix(&text);
        assert_eq!(out.as_ref(), "a\nb\nc");
    }

    #[test]
    fn pulldown_extracts_link_and_strong() {
        let events = parse_events("Hello **world** [link](https://example.com)\n");
        let pieces = inline_pieces_from_events_unwrapped(&events);
        assert!(pieces.iter().any(|p| p.style.strong));
        assert!(pieces.iter().any(|p| p.style.link.is_some()));
    }

    #[test]
    fn pulldown_counts_list_items() {
        let events = parse_events("- a\n- b\n");
        assert_eq!(count_top_level_list_items(&events), 2);
    }

    #[test]
    fn open_url_filter_is_conservative() {
        assert!(is_safe_open_url("https://example.com"));
        assert!(is_safe_open_url("http://example.com"));
        assert!(is_safe_open_url("mailto:test@example.com"));

        assert!(!is_safe_open_url(""));
        assert!(!is_safe_open_url("   "));
        assert!(!is_safe_open_url("javascript:alert(1)"));
        assert!(!is_safe_open_url("data:text/html;base64,PHNjcmlwdD4="));
        assert!(!is_safe_open_url("file:///etc/passwd"));
    }
}
