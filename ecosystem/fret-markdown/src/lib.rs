//! Markdown renderer component(s) for Fret.

use std::sync::Arc;

use fret_core::{FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
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
            .layout(LayoutRefinement::default().w_full()),
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
            MarkdownBlockKind::Raw { kind, text } => {
                let info = RawBlockInfo { kind, text };
                if let Some(render) = &components.raw_block {
                    render(cx, info)
                } else {
                    render_paragraph(cx, theme, info.text)
                }
            }
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

pub type HeadingRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, HeadingInfo) -> AnyElement;
pub type ParagraphRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, ParagraphInfo) -> AnyElement;
pub type CodeBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type CodeBlockActionsRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type RawBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, RawBlockInfo) -> AnyElement;

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
}

impl<H: UiHost> Default for MarkdownComponents<H> {
    fn default() -> Self {
        Self {
            heading: None,
            paragraph: None,
            code_block: None,
            code_block_actions: None,
            raw_block: None,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
