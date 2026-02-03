//! Markdown renderer component(s) for Fret.

use std::sync::Arc;

use fret_core::{
    AttributedText, Axis, Edges, FontId, FontWeight, Px, SemanticsRole, StrikethroughStyle,
    TextOverflow, TextPaintStyle, TextShapingStyle, TextSlant, TextSpan, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PositionStyle, PressableProps, ScrollAxis, ScrollProps, SelectableTextProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

pub use mdstream::BlockId;

#[cfg(feature = "mathjax-svg")]
mod mathjax_svg_support;
mod mermaid;
mod open_url;
mod pulldown_render;
#[cfg(test)]
mod tests;
mod theme;

use mermaid::{detect_mermaid_diagram_type, is_mermaid_language, render_mermaid_header_label};
pub use open_url::{OnLinkActivate, is_safe_open_url, on_link_activate_open_url};
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

#[derive(Debug)]
struct MarkdownSnapshot {
    doc: mdstream::DocumentState,
    adapter: mdstream::adapters::pulldown::PulldownAdapter,
}

#[derive(Debug)]
struct MarkdownCachedState {
    source: Arc<str>,
    snapshot: Option<Arc<MarkdownSnapshot>>,
}

impl MarkdownCachedState {
    fn new() -> Self {
        Self {
            source: Arc::from(""),
            snapshot: None,
        }
    }

    fn snapshot_for_source(&mut self, source: &str) -> Arc<MarkdownSnapshot> {
        if self.source.as_ref() == source {
            if let Some(snapshot) = self.snapshot.as_ref() {
                return snapshot.clone();
            }
        }

        self.source = Arc::from(source);

        let mut stream = mdstream::MdStream::new(mdstream_options_for_markdown());
        let update = stream.append(self.source.as_ref());

        let mut state = MarkdownPulldownState::new();
        state.apply_update(update);
        state.apply_update(stream.finalize());

        let snapshot = Arc::new(MarkdownSnapshot {
            doc: state.doc,
            adapter: state.adapter,
        });
        self.snapshot = Some(snapshot.clone());
        snapshot
    }
}

#[track_caller]
pub fn markdown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    source: &str,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let markdown_theme = MarkdownTheme::resolve(&theme);

    let snapshot = cx.named("markdown", |cx| {
        cx.with_state(MarkdownCachedState::new, |state| {
            state.snapshot_for_source(source)
        })
    });

    markdown_mdstream_pulldown_with(
        cx,
        &theme,
        markdown_theme,
        &snapshot.doc,
        &snapshot.adapter,
        components,
    )
}

pub fn mdstream_options_for_markdown() -> mdstream::Options {
    // mdstream defaults to `FootnotesMode::SingleBlock`, which intentionally collapses documents
    // with footnotes into one block (stability-first). For UI rendering we prefer keeping blocks
    // so headings, lists, math blocks, etc can be laid out independently.
    mdstream::Options {
        footnotes: mdstream::FootnotesMode::Invalidate,
        ..Default::default()
    }
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
    pub id: BlockId,
    pub language: Option<Arc<str>>,
    pub code: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct InlineMathInfo {
    pub latex: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct MathBlockInfo {
    pub latex: Arc<str>,
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

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub src: Arc<str>,
    pub alt: Arc<str>,
    pub title: Option<Arc<str>>,
    pub is_svg: bool,
}

pub type HeadingRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, HeadingInfo) -> AnyElement;
pub type ParagraphRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, ParagraphInfo) -> AnyElement;
pub type CodeBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type CodeBlockActionsRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, CodeBlockInfo) -> AnyElement;
pub type CodeBlockUiResolver<H> = dyn for<'a> Fn(
    &mut ElementContext<'a, H>,
    &CodeBlockInfo,
    &mut fret_code_view::CodeBlockUiOptions,
) -> ();
pub type RawBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, RawBlockInfo) -> AnyElement;
pub type ListRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, ListInfo) -> AnyElement;
pub type BlockQuoteRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, BlockQuoteInfo) -> AnyElement;
pub type TableRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, TableInfo) -> AnyElement;
pub type ThematicBreakRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, ThematicBreakInfo) -> AnyElement;
pub type LinkRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, LinkInfo) -> AnyElement;
pub type ImageRenderer<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, ImageInfo) -> AnyElement;
pub type InlineMathRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, InlineMathInfo) -> AnyElement;
pub type MathBlockRenderer<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, MathBlockInfo) -> AnyElement;

#[derive(Clone)]
pub struct MarkdownComponents<H: UiHost> {
    pub heading: Option<Arc<HeadingRenderer<H>>>,
    pub paragraph: Option<Arc<ParagraphRenderer<H>>>,
    pub code_block: Option<Arc<CodeBlockRenderer<H>>>,
    /// UI policy for the default fenced code block renderer (`fret-code-view`).
    ///
    /// If you set `code_block`, you own the full rendering and this value is ignored.
    pub code_block_ui: fret_code_view::CodeBlockUiOptions,
    /// Whether the default fenced code block renderer should resolve `max_height` from theme
    /// tokens when `code_block_ui.max_height` is unset.
    ///
    /// This is a policy knob on `fret-markdown` rather than `fret-code-view` because only
    /// Markdown knows which theme token names are relevant.
    pub code_block_max_height_from_theme: bool,
    /// Per-code-block UI tweaks for the default fenced code block renderer (`fret-code-view`).
    ///
    /// This is applied after theme token resolution, so the resolver can override the final
    /// `CodeBlockUiOptions` for specific blocks (expand/collapse, wrap overrides, etc.).
    pub code_block_ui_resolver: Option<Arc<CodeBlockUiResolver<H>>>,
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
    /// Render an inline image (`![alt](src "title")`).
    ///
    /// Notes:
    /// - `fret-markdown` does not fetch images. The host is responsible for loading and caching.
    /// - See `ecosystem/fret-ui-assets` for integrating `ImageAssetCache` / `SvgAssetCache`.
    pub image: Option<Arc<ImageRenderer<H>>>,
    /// Render an inline math span (`$...$`).
    pub inline_math: Option<Arc<InlineMathRenderer<H>>>,
    /// Render a display math block (`$$...$$`).
    pub math_block: Option<Arc<MathBlockRenderer<H>>>,
    pub on_link_activate: Option<OnLinkActivate>,
}

impl<H: UiHost> Default for MarkdownComponents<H> {
    fn default() -> Self {
        let mut code_block_ui = fret_code_view::CodeBlockUiOptions::default();
        code_block_ui.show_header = true;
        code_block_ui.header_divider = true;
        code_block_ui.header_background = fret_code_view::CodeBlockHeaderBackground::Secondary;
        code_block_ui.show_copy_button = true;
        code_block_ui.copy_button_on_hover = true;
        code_block_ui.copy_button_placement = fret_code_view::CodeBlockCopyButtonPlacement::Header;
        code_block_ui.show_scrollbar_x = true;
        code_block_ui.scrollbar_x_on_hover = true;

        Self {
            heading: None,
            paragraph: None,
            code_block: None,
            code_block_ui,
            code_block_max_height_from_theme: true,
            code_block_ui_resolver: None,
            code_block_actions: None,
            raw_block: None,
            list: None,
            blockquote: None,
            table: None,
            thematic_break: None,
            link: None,
            image: None,
            inline_math: None,
            math_block: None,
            on_link_activate: None,
        }
    }
}

impl<H: UiHost> MarkdownComponents<H> {
    pub fn with_open_url(mut self) -> Self {
        self.on_link_activate = Some(on_link_activate_open_url());
        self
    }

    pub fn with_code_block_wrap(mut self, wrap: fret_code_view::CodeBlockWrap) -> Self {
        self.code_block_ui.wrap = wrap;
        self
    }

    pub fn with_code_block_max_height(mut self, max_height: Option<Px>) -> Self {
        self.code_block_ui.max_height = max_height;
        self
    }

    pub fn with_code_block_max_height_from_theme(mut self, enabled: bool) -> Self {
        self.code_block_max_height_from_theme = enabled;
        self
    }

    pub fn with_code_block_ui_resolver(
        mut self,
        resolver: Option<Arc<CodeBlockUiResolver<H>>>,
    ) -> Self {
        self.code_block_ui_resolver = resolver;
        self
    }

    pub fn with_code_block_scrollbar_x(mut self, show: bool) -> Self {
        self.code_block_ui.show_scrollbar_x = show;
        self
    }

    pub fn with_code_block_scrollbar_x_on_hover(mut self, on_hover: bool) -> Self {
        self.code_block_ui.scrollbar_x_on_hover = on_hover;
        self
    }

    pub fn with_code_block_scrollbar_y(mut self, show: bool) -> Self {
        self.code_block_ui.show_scrollbar_y = show;
        self
    }

    pub fn with_code_block_scrollbar_y_on_hover(mut self, on_hover: bool) -> Self {
        self.code_block_ui.scrollbar_y_on_hover = on_hover;
        self
    }
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
        let canonical = "fret.markdown.code_block.max_height";
        let compat = "markdown.code_block.max_height";
        options.max_height = if theme.metric_key_configured(canonical) {
            theme.metric_by_key(canonical)
        } else if theme.metric_key_configured(compat) {
            theme.metric_by_key(compat)
        } else {
            theme
                .metric_by_key(canonical)
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
            padding: Edges::all(Px(0.0)),
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

#[derive(Debug)]
pub struct MarkdownPulldownState {
    doc: mdstream::DocumentState,
    adapter: mdstream::adapters::pulldown::PulldownAdapter,
}

impl MarkdownPulldownState {
    pub fn new() -> Self {
        Self {
            doc: mdstream::DocumentState::default(),
            adapter: mdstream::adapters::pulldown::PulldownAdapter::new(
                mdstream::adapters::pulldown::PulldownAdapterOptions {
                    pulldown: pulldown_options_default(),
                    prefer_display_for_pending: true,
                },
            ),
        }
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

impl Default for MarkdownPulldownState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct MarkdownStreamState {
    opts: mdstream::Options,
    stream: mdstream::MdStream,
    state: MarkdownPulldownState,
}

impl MarkdownStreamState {
    pub fn new() -> Self {
        Self::new_with_options(mdstream_options_for_markdown())
    }

    pub fn new_with_options(opts: mdstream::Options) -> Self {
        Self {
            stream: mdstream::MdStream::new(opts.clone()),
            opts,
            state: MarkdownPulldownState::new(),
        }
    }

    pub fn state(&self) -> &MarkdownPulldownState {
        &self.state
    }

    pub fn clear(&mut self) {
        self.stream = mdstream::MdStream::new(self.opts.clone());
        self.state.clear();
    }

    pub fn append(&mut self, chunk: &str) -> mdstream::AppliedUpdate {
        self.state.apply_update(self.stream.append(chunk))
    }

    pub fn finalize(&mut self) -> mdstream::AppliedUpdate {
        self.state.apply_update(self.stream.finalize())
    }
}

impl Default for MarkdownStreamState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn markdown_streaming_pulldown<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
) -> AnyElement {
    markdown_streaming_pulldown_with(cx, state, &MarkdownComponents::default())
}

pub fn markdown_streaming_pulldown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let markdown_theme = MarkdownTheme::resolve(&theme);
    markdown_mdstream_pulldown_with(
        cx,
        &theme,
        markdown_theme,
        state.doc(),
        &state.adapter,
        components,
    )
}

fn markdown_mdstream_pulldown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    doc: &mdstream::DocumentState,
    adapter: &mdstream::adapters::pulldown::PulldownAdapter,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let committed = doc.committed();
    let pending = doc.pending();

    let log_once = cx.with_state(
        || false,
        |logged| {
            if *logged {
                false
            } else {
                *logged = true;
                true
            }
        },
    );
    if log_once {
        let mut lines: Vec<String> = Vec::new();
        for block in committed.iter().take(32) {
            let raw = block.display_or_raw();
            let raw_one_line = raw
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let has_dollars = raw.contains("$$");
            let has_adapter_events = adapter.committed_events(block.id).is_some();
            lines.push(format!(
                "{:?} id={:?} adapter_events={} has_$$={} raw0={:?}",
                block.kind, block.id, has_adapter_events, has_dollars, raw_one_line
            ));
        }
        if let Some(p) = pending {
            let raw = p.display_or_raw();
            let raw_one_line = raw
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let has_dollars = raw.contains("$$");
            let has_adapter_events = adapter.parse_pending(p).iter().any(|_| true);
            lines.push(format!(
                "PENDING {:?} id={:?} adapter_events={} has_$$={} raw0={:?}",
                p.kind, p.id, has_adapter_events, has_dollars, raw_one_line
            ));
        }

        tracing::debug!(
            target: "fret_markdown::mdstream",
            committed = committed.len(),
            pending = pending.is_some(),
            "mdstream blocks:\n{}",
            lines.join("\n")
        );
    }

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
                        cx,
                        theme,
                        markdown_theme,
                        components,
                        block,
                        events,
                    )),
                    None => {
                        let tmp = parse_events(block.display_or_raw());
                        out.push(render_mdstream_block_with_events(
                            cx,
                            theme,
                            markdown_theme,
                            components,
                            block,
                            &tmp,
                        ));
                    }
                },
            );

            if let Some(pending) = pending {
                cx.keyed(pending.id, |cx| {
                    let events = adapter.parse_pending(pending);
                    out.push(render_mdstream_block_with_events(
                        cx,
                        theme,
                        markdown_theme,
                        components,
                        pending,
                        &events,
                    ));
                });
            }

            out
        },
    )
}

fn render_mdstream_block_with_events<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
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
                render_heading_inline(cx, theme, markdown_theme, components, info, events)
            }
        }
        mdstream::BlockKind::Paragraph => {
            if is_display_math_block_text(block.display_or_raw()) {
                let latex = parse_math_block_body(block.display_or_raw());
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    latex_len = latex.len(),
                    "render paragraph as display math (by raw)"
                );
                return render_math_block(cx, theme, markdown_theme, components, latex);
            }
            if let Some(latex) = display_math_only_events(events) {
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    latex_len = latex.len(),
                    "render paragraph as display math"
                );
                return render_math_block(cx, theme, markdown_theme, components, latex);
            }
            let info = ParagraphInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.paragraph {
                render(cx, info)
            } else {
                render_paragraph_inline(cx, theme, markdown_theme, components, events)
            }
        }
        mdstream::BlockKind::ThematicBreak => {
            if let Some(render) = &components.thematic_break {
                render(cx, ThematicBreakInfo)
            } else {
                render_thematic_break(cx, theme, markdown_theme)
            }
        }
        mdstream::BlockKind::CodeFence => {
            let (language, code) = parse_code_fence_body(block.display_or_raw());
            let info = CodeBlockInfo {
                id: block.id,
                language,
                code,
            };
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
                pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::BlockQuote => {
            let info = BlockQuoteInfo {
                text: strip_blockquote_prefix(block.display_or_raw()),
            };
            if let Some(render) = &components.blockquote {
                render(cx, info)
            } else {
                pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::Table => {
            let info = TableInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.table {
                render(cx, info)
            } else {
                // Intentionally not using fret-ui-kit's TanStack-inspired table:
                // it is a data-grid with fixed-row virtualized layout (sorting/resizing/pinning),
                // while Markdown tables need content-driven, multi-line cell layout.
                pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::MathBlock => {
            // mdstream already classifies the block as MathBlock; don't rely on pulldown to
            // re-discover `Event::DisplayMath` because the adapter may have stripped delimiters.
            let mut latex = parse_math_block_body(block.display_or_raw());
            let latex_from_events = latex_from_pulldown_math_events(events);
            if latex.trim().is_empty() {
                if let Some(from_events) = latex_from_events.clone() {
                    latex = from_events;
                }
            }
            let log_once = cx.with_state(
                || false,
                |logged| {
                    if *logged {
                        false
                    } else {
                        *logged = true;
                        true
                    }
                },
            );
            if log_once {
                let has_display_math_event = events.iter().any(|e| {
                    matches!(
                        e,
                        pulldown_cmark::Event::DisplayMath(_)
                            | pulldown_cmark::Event::InlineMath(_)
                    )
                });
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    raw = %block.display_or_raw().replace('\n', "\\n"),
                    latex_len = latex.len(),
                    latex_from_events_len = latex_from_events.as_ref().map(|s| s.len()),
                    has_math_event = has_display_math_event,
                    "render mdstream math block"
                );
            }
            render_math_block(cx, theme, markdown_theme, components, latex)
        }
        mdstream::BlockKind::HtmlBlock
        | mdstream::BlockKind::FootnoteDefinition
        | mdstream::BlockKind::Unknown => {
            let info = RawBlockInfo {
                kind: raw_block_kind_from_mdstream(block.kind),
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.raw_block {
                render(cx, info)
            } else {
                pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
    }
}

fn raw_block_kind_from_mdstream(kind: mdstream::BlockKind) -> RawBlockKind {
    match kind {
        mdstream::BlockKind::HtmlBlock => RawBlockKind::HtmlBlock,
        mdstream::BlockKind::MathBlock => RawBlockKind::MathBlock,
        mdstream::BlockKind::FootnoteDefinition => RawBlockKind::FootnoteDefinition,
        mdstream::BlockKind::Unknown => RawBlockKind::Unknown,
        _ => RawBlockKind::Unknown,
    }
}

fn is_display_math_block_text(text: &str) -> bool {
    let s = text.trim();
    if s.is_empty() {
        return false;
    }

    (s.starts_with("$$") && s.ends_with("$$")) || (s.starts_with("\\[") && s.ends_with("\\]"))
}

fn parse_math_block_body(text: &str) -> Arc<str> {
    let s = text.trim();
    if s.is_empty() {
        return Arc::<str>::from("");
    }

    // Support common delimiters. mdstream may already have stripped them, so we fall back to `s`.
    if let Some(rest) = s.strip_prefix("$$") {
        let rest = rest.strip_suffix("$$").unwrap_or(rest);
        return Arc::<str>::from(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("\\[") {
        let rest = rest.strip_suffix("\\]").unwrap_or(rest);
        return Arc::<str>::from(rest.trim().to_string());
    }

    Arc::<str>::from(s.to_string())
}

fn latex_from_pulldown_math_events(events: &[pulldown_cmark::Event<'static>]) -> Option<Arc<str>> {
    use pulldown_cmark::Event;

    for e in events {
        if let Event::DisplayMath(latex) = e {
            return Some(Arc::<str>::from(latex.to_string()));
        }
    }

    let mut buf = String::new();
    for e in events {
        match e {
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                buf.push_str(t.as_ref())
            }
            Event::SoftBreak | Event::HardBreak => buf.push('\n'),
            _ => {}
        }
    }

    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(Arc::<str>::from(trimmed.to_string()))
}

fn display_math_only_events(events: &[pulldown_cmark::Event<'static>]) -> Option<Arc<str>> {
    use pulldown_cmark::Event;

    let mut display_latex: Option<Arc<str>> = None;
    let mut has_other = false;

    for e in events {
        match e {
            Event::DisplayMath(latex) => {
                if display_latex.is_some() {
                    return None;
                }
                display_latex = Some(Arc::<str>::from(latex.to_string()));
            }
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) => {
                if !t.trim().is_empty() {
                    has_other = true;
                }
            }
            _ => {}
        }
    }

    if has_other {
        return None;
    }

    display_latex
}

fn render_heading_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    info: HeadingInfo,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let font_size = theme.metric_required("metric.font.size");
    let line_height = theme.metric_required("metric.font.line_height");
    let fg = theme.color_required("foreground");
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
        color: fg,
    };

    let pieces = inline_pieces_maybe_unwrapped(events);
    render_inline_flow_or_rich(cx, theme, markdown_theme, components, base, &pieces)
}

fn render_paragraph_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let font_size = theme.metric_required("metric.font.size");
    let line_height = theme.metric_required("metric.font.line_height");
    let fg = theme.color_required("foreground");
    let base = InlineBaseStyle {
        font: FontId::default(),
        size: font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(line_height),
        color: fg,
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
    let allow_link_color_only = components.link.is_none() && components.on_link_activate.is_none();

    if !allow_link_color_only && pieces.iter().any(|p| p.style.link.is_some()) {
        return None;
    }

    let rich = build_rich_attributed_text(markdown_theme, pieces)?;

    let mut props = SelectableTextProps::new(rich);
    props.layout.size.width = Length::Fill;
    props.style = Some(TextStyle {
        font: base.font.clone(),
        size: base.size,
        weight: base.weight,
        slant: TextSlant::Normal,
        line_height: base.line_height,
        letter_spacing_em: None,
    });
    props.color = Some(base.color);
    props.wrap = TextWrap::Word;
    props.overflow = TextOverflow::Clip;

    Some(cx.selectable_text_props(props))
}

fn build_rich_attributed_text(
    markdown_theme: MarkdownTheme,
    pieces: &[InlinePiece],
) -> Option<AttributedText> {
    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    for p in pieces {
        let InlinePieceKind::Text(t) = &p.kind else {
            return None;
        };
        if t.is_empty() {
            continue;
        }

        text.push_str(t);

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
                underline: None,
                strikethrough: run_strikethrough,
            },
        });
    }

    if text.is_empty() {
        return None;
    }

    Some(AttributedText::new(Arc::<str>::from(text), spans))
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
    container.padding = Edges::all(markdown_theme.math_block_padding);
    container.background = Some(markdown_theme.math_block_bg);
    container.border = Edges::all(Px(0.0));
    container.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.md"));

    cx.container(container, |cx| {
        vec![cx.scroll(scroll_props, |cx| {
            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: latex,
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
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
    color: fret_core::Color,
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
    props.gap = Px(0.0);
    props.padding = Edges::all(Px(0.0));
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
            theme.metric_required("metric.font.mono_size"),
            Some(theme.metric_required("metric.font.mono_line_height")),
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

    let color = if style.link.is_some() {
        markdown_theme.link
    } else {
        base.color
    };

    if style.code {
        let mut props = ContainerProps::default();
        props.padding = Edges {
            top: markdown_theme.inline_code_padding_y,
            right: markdown_theme.inline_code_padding_x,
            bottom: markdown_theme.inline_code_padding_y,
            left: markdown_theme.inline_code_padding_x,
        };
        props.background = Some(markdown_theme.inline_code_bg);
        props.border = Edges::all(Px(0.0));
        props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

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
                }),
                color: Some(markdown_theme.inline_code_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
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

                vec![render_inline_text_token(
                    cx,
                    font,
                    size,
                    weight,
                    slant,
                    line_height,
                    color,
                    style.strikethrough,
                    display_text.clone(),
                )]
            });
        }
    }

    render_inline_text_token(
        cx,
        font,
        size,
        weight,
        slant,
        line_height,
        color,
        style.strikethrough,
        Arc::<str>::from(raw_text),
    )
}

fn render_inline_text_token<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    font: FontId,
    size: Px,
    weight: FontWeight,
    slant: TextSlant,
    line_height: Option<Px>,
    color: fret_core::Color,
    strikethrough: bool,
    text: Arc<str>,
) -> AnyElement {
    if !strikethrough {
        return cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(TextStyle {
                font,
                size,
                weight,
                slant,
                line_height,
                letter_spacing_em: None,
            }),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });
    }

    let effective_line_height = line_height.unwrap_or(Px(size.0.max(1.0)));
    let line_y = Px(effective_line_height.0 * 0.55);

    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Relative;
    props.padding = Edges::all(Px(0.0));
    props.border = Edges::all(Px(0.0));

    cx.container(props, |cx| {
        let text_el = cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(TextStyle {
                font,
                size,
                weight,
                slant,
                line_height,
                letter_spacing_em: None,
            }),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let mut line_layout = LayoutStyle::default();
        line_layout.position = PositionStyle::Absolute;
        line_layout.inset.left = Some(Px(0.0));
        line_layout.inset.right = Some(Px(0.0));
        line_layout.inset.top = Some(line_y);
        line_layout.size.height = Length::Px(Px(1.0));

        let line_el = cx.container(
            ContainerProps {
                layout: line_layout,
                padding: Edges::all(Px(0.0)),
                background: Some(color),
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
        props.a11y.role = Some(SemanticsRole::Button);
        props.a11y.label = Some(link_text.clone());

        return cx.pressable(props, |cx, _state| {
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
                style: Some(TextStyle {
                    font: FontId::default(),
                    size: theme.metric_required("metric.font.size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.link),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        });
    }

    cx.text_props(TextProps {
        layout: Default::default(),
        text: label,
        style: Some(TextStyle {
            font: FontId::default(),
            size: theme.metric_required("metric.font.size"),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: Some(theme.metric_required("metric.font.line_height")),
            letter_spacing_em: None,
        }),
        color: Some(markdown_theme.muted),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
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
    };
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: info.latex,
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: TextSlant::Normal,
                line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(markdown_theme.inline_math_fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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
