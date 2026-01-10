//! Markdown renderer component(s) for Fret.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_core::{
    Axis, Edges, FontId, FontWeight, Px, RichText, SemanticsRole, TextOverflow, TextRun, TextSlant,
    TextStyle, TextWrap,
};
use fret_runtime::Effect;
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PositionStyle, PressableProps, ScrollAxis, ScrollProps, SelectableTextProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

pub use mdstream::BlockId;

#[cfg(feature = "mathjax-svg")]
use fret_core::SvgFit;
#[cfg(feature = "mathjax-svg")]
use fret_ui::SvgSource;
#[cfg(feature = "mathjax-svg")]
use fret_ui::element::SvgIconProps;

#[derive(Debug, Clone, Copy)]
struct MarkdownTheme {
    link: fret_core::Color,
    muted: fret_core::Color,
    hr: fret_core::Color,
    blockquote_border: fret_core::Color,
    blockquote_border_width: Px,
    blockquote_padding: Px,
    inline_code_fg: fret_core::Color,
    inline_code_bg: fret_core::Color,
    inline_code_padding_x: Px,
    inline_code_padding_y: Px,
    task_checked: fret_core::Color,
    task_unchecked: fret_core::Color,
    table_border: fret_core::Color,
    table_header_bg: fret_core::Color,
    table_cell_padding_x: Px,
    table_cell_padding_y: Px,
    inline_math_fg: fret_core::Color,
    inline_math_bg: fret_core::Color,
    inline_math_padding_x: Px,
    inline_math_padding_y: Px,
    #[cfg(feature = "mathjax-svg")]
    inline_math_height: Px,
    math_block_fg: fret_core::Color,
    math_block_bg: fret_core::Color,
    math_block_padding: Px,
    #[cfg(feature = "mathjax-svg")]
    math_block_height: Px,
}

impl MarkdownTheme {
    fn resolve(theme: &Theme) -> Self {
        fn color(theme: &Theme, suffix: &str) -> Option<fret_core::Color> {
            theme
                .color_by_key(&format!("fret.markdown.{suffix}"))
                .or_else(|| theme.color_by_key(&format!("markdown.{suffix}")))
        }

        fn metric(theme: &Theme, suffix: &str) -> Option<Px> {
            theme
                .metric_by_key(&format!("fret.markdown.{suffix}"))
                .or_else(|| theme.metric_by_key(&format!("markdown.{suffix}")))
        }

        let link = color(theme, "link").unwrap_or_else(|| theme.color_required("primary"));
        let muted =
            color(theme, "muted").unwrap_or_else(|| theme.color_required("muted-foreground"));
        let hr = color(theme, "hr").unwrap_or_else(|| theme.color_required("border"));

        let blockquote_border =
            color(theme, "blockquote.border").unwrap_or_else(|| theme.color_required("border"));
        let blockquote_border_width = metric(theme, "blockquote.border_width").unwrap_or(Px(3.0));
        let blockquote_padding = metric(theme, "blockquote.padding")
            .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));

        let inline_code_fg =
            color(theme, "inline_code.fg").unwrap_or_else(|| theme.color_required("foreground"));
        let inline_code_bg =
            color(theme, "inline_code.bg").unwrap_or_else(|| theme.color_required("accent"));
        let inline_code_padding_x = metric(theme, "inline_code.padding_x").unwrap_or(Px(3.0));
        let inline_code_padding_y = metric(theme, "inline_code.padding_y").unwrap_or(Px(1.0));

        let task_checked =
            color(theme, "task.checked").unwrap_or_else(|| theme.color_required("primary"));
        let task_unchecked = color(theme, "task.unchecked")
            .unwrap_or_else(|| theme.color_required("muted-foreground"));

        let table_border =
            color(theme, "table.border").unwrap_or_else(|| theme.color_required("border"));
        let table_header_bg =
            color(theme, "table.header_bg").unwrap_or_else(|| theme.color_required("muted"));
        let table_cell_padding_x = metric(theme, "table.cell.padding_x")
            .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
        let table_cell_padding_y = metric(theme, "table.cell.padding_y")
            .unwrap_or_else(|| Px(theme.metric_required("metric.padding.sm").0 * 0.5));

        let inline_math_fg = color(theme, "math.inline.fg").unwrap_or(inline_code_fg);
        let inline_math_bg = color(theme, "math.inline.bg").unwrap_or(inline_code_bg);
        let inline_math_padding_x =
            metric(theme, "math.inline.padding_x").unwrap_or(inline_code_padding_x);
        let inline_math_padding_y =
            metric(theme, "math.inline.padding_y").unwrap_or(inline_code_padding_y);
        #[cfg(feature = "mathjax-svg")]
        let inline_math_height = metric(theme, "math.inline.height")
            .unwrap_or_else(|| theme.metric_required("metric.font.line_height"));

        let math_block_fg =
            color(theme, "math.block.fg").unwrap_or_else(|| theme.color_required("foreground"));
        let math_block_bg =
            color(theme, "math.block.bg").unwrap_or_else(|| theme.color_required("card"));
        let math_block_padding = metric(theme, "math.block.padding")
            .unwrap_or_else(|| theme.metric_required("metric.padding.md"));
        #[cfg(feature = "mathjax-svg")]
        let math_block_height = metric(theme, "math.block.height").unwrap_or_else(|| {
            let font_size = theme.metric_required("metric.font.size").0;
            let line_height = theme.metric_required("metric.font.line_height").0;
            Px((line_height * 3.25).max(font_size * 4.0))
        });

        Self {
            link,
            muted,
            hr,
            blockquote_border,
            blockquote_border_width,
            blockquote_padding,
            inline_code_fg,
            inline_code_bg,
            inline_code_padding_x,
            inline_code_padding_y,
            task_checked,
            task_unchecked,
            table_border,
            table_header_bg,
            table_cell_padding_x,
            table_cell_padding_y,
            inline_math_fg,
            inline_math_bg,
            inline_math_padding_x,
            inline_math_padding_y,
            #[cfg(feature = "mathjax-svg")]
            inline_math_height,
            math_block_fg,
            math_block_bg,
            math_block_padding,
            #[cfg(feature = "mathjax-svg")]
            math_block_height,
        }
    }
}

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
    let markdown_theme = MarkdownTheme::resolve(&theme);

    let mut stream = mdstream::MdStream::new(mdstream_options_for_markdown());
    let update = stream.append(source);

    let mut state = MarkdownPulldownState::new();
    state.apply_update(update);
    state.apply_update(stream.finalize());

    markdown_mdstream_pulldown_with(
        cx,
        &theme,
        markdown_theme,
        state.doc(),
        &state.adapter,
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

#[cfg(feature = "mathjax-svg")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MathJaxMode {
    Inline,
    Display,
}

#[cfg(feature = "mathjax-svg")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathJaxKey {
    mode: MathJaxMode,
    latex: String,
}

#[cfg(feature = "mathjax-svg")]
#[derive(Debug, Clone)]
struct MathJaxSvgReady {
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
}

#[cfg(feature = "mathjax-svg")]
#[derive(Debug, Clone)]
enum MathJaxSvgEntry {
    Loading,
    Ready(MathJaxSvgReady),
    Error(Arc<str>),
}

#[cfg(feature = "mathjax-svg")]
struct MathJaxWorker {
    tx: std::sync::mpsc::Sender<MathJaxWorkItem>,
}

#[cfg(feature = "mathjax-svg")]
struct MathJaxWorkItem {
    map: Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
    key: MathJaxKey,
}

#[cfg(feature = "mathjax-svg")]
static MATHJAX_WORKER: std::sync::OnceLock<MathJaxWorker> = std::sync::OnceLock::new();

#[cfg(feature = "mathjax-svg")]
fn mathjax_worker() -> &'static MathJaxWorker {
    MATHJAX_WORKER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<MathJaxWorkItem>();
        std::thread::spawn(move || {
            for item in rx {
                let key = item.key;
                let latex = key.latex.clone();
                tracing::debug!(
                    target: "fret_markdown::math",
                    mode = ?key.mode,
                    latex_len = latex.len(),
                    "mathjax svg: convert queued"
                );

                let result = std::panic::catch_unwind(|| match key.mode {
                    MathJaxMode::Inline => mathjax_svg::convert_to_svg_inline(&latex),
                    MathJaxMode::Display => mathjax_svg::convert_to_svg(&latex),
                });

                let mut map_guard = item.map.lock().expect("mathjax svg cache lock");
                match result {
                    Ok(Ok(svg)) => {
                        let has_current_color =
                            svg.contains("currentColor") || svg.contains("currentcolor");
                        let svg = if has_current_color {
                            svg.replace("currentColor", "#000000")
                                .replace("currentcolor", "#000000")
                        } else {
                            svg
                        };

                        tracing::debug!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            has_current_color,
                            "mathjax svg: converted"
                        );

                        let aspect_ratio = svg_viewbox_aspect_ratio(&svg);
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Ready(MathJaxSvgReady {
                                svg_bytes: Arc::<[u8]>::from(svg.into_bytes()),
                                aspect_ratio,
                            }),
                        );
                    }
                    Ok(Err(err)) => {
                        tracing::warn!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            error = %err,
                            "mathjax svg: convert failed"
                        );
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Error(Arc::<str>::from(err.to_string())),
                        );
                    }
                    Err(_) => {
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg: panic")),
                        );
                    }
                }
            }
        });
        MathJaxWorker { tx }
    })
}

#[cfg(feature = "mathjax-svg")]
#[derive(Default, Clone)]
struct MathJaxSvgCache {
    inner: Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
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
pub type OnLinkActivate =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason, LinkInfo) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MermaidDiagramType {
    Flowchart,
    Sequence,
    Class,
    State,
    EntityRelationship,
    UserJourney,
    Gantt,
    Pie,
    Quadrant,
    Requirement,
    GitGraph,
    C4,
    Mindmap,
    Timeline,
    ZenUML,
    Sankey,
    XYChart,
    Block,
    Unknown,
}

impl MermaidDiagramType {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Flowchart => "Flowchart",
            Self::Sequence => "Sequence Diagram",
            Self::Class => "Class Diagram",
            Self::State => "State Diagram",
            Self::EntityRelationship => "Entity-Relationship Diagram",
            Self::UserJourney => "User Journey",
            Self::Gantt => "Gantt Chart",
            Self::Pie => "Pie Chart",
            Self::Quadrant => "Quadrant Chart",
            Self::Requirement => "Requirement Diagram",
            Self::GitGraph => "Git Graph",
            Self::C4 => "C4 Diagram",
            Self::Mindmap => "Mindmap",
            Self::Timeline => "Timeline",
            Self::ZenUML => "ZenUML Diagram",
            Self::Sankey => "Sankey Diagram",
            Self::XYChart => "XY Chart",
            Self::Block => "Block Diagram",
            Self::Unknown => "Diagram",
        }
    }
}

fn is_mermaid_language(language: Option<&str>) -> bool {
    language
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some_and(|s| s.eq_ignore_ascii_case("mermaid"))
}

fn detect_mermaid_diagram_type(source: &str) -> MermaidDiagramType {
    // Find the first non-empty, non-comment line. Mermaid uses `%%` for comments.
    let first_line = source
        .lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && !line.starts_with("%%"))
        .unwrap_or("");

    let first_line_lower = first_line.to_ascii_lowercase();
    if first_line_lower.starts_with("flowchart")
        || first_line_lower.starts_with("graph")
        || first_line_lower.starts_with("flowchart-v2")
    {
        MermaidDiagramType::Flowchart
    } else if first_line_lower.starts_with("sequencediagram")
        || first_line_lower.starts_with("sequence")
    {
        MermaidDiagramType::Sequence
    } else if first_line_lower.starts_with("classdiagram") || first_line_lower.starts_with("class")
    {
        MermaidDiagramType::Class
    } else if first_line_lower.starts_with("statediagram") || first_line_lower.starts_with("state")
    {
        MermaidDiagramType::State
    } else if first_line_lower.starts_with("erdiagram") || first_line_lower == "er" {
        MermaidDiagramType::EntityRelationship
    } else if first_line_lower.starts_with("journey") {
        MermaidDiagramType::UserJourney
    } else if first_line_lower.starts_with("gantt") {
        MermaidDiagramType::Gantt
    } else if first_line_lower.starts_with("pie") {
        MermaidDiagramType::Pie
    } else if first_line_lower.starts_with("quadrantchart") {
        MermaidDiagramType::Quadrant
    } else if first_line_lower.starts_with("requirementdiagram")
        || first_line_lower.starts_with("requirement")
    {
        MermaidDiagramType::Requirement
    } else if first_line_lower.starts_with("gitgraph") {
        MermaidDiagramType::GitGraph
    } else if first_line_lower.starts_with("c4") {
        MermaidDiagramType::C4
    } else if first_line_lower.starts_with("mindmap") {
        MermaidDiagramType::Mindmap
    } else if first_line_lower.starts_with("timeline") {
        MermaidDiagramType::Timeline
    } else if first_line_lower.starts_with("zenuml") {
        MermaidDiagramType::ZenUML
    } else if first_line_lower.starts_with("sankey") {
        MermaidDiagramType::Sankey
    } else if first_line_lower.starts_with("xychart") {
        MermaidDiagramType::XYChart
    } else if first_line_lower.starts_with("block") {
        MermaidDiagramType::Block
    } else {
        MermaidDiagramType::Unknown
    }
}

fn render_mermaid_header_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    diagram_type: MermaidDiagramType,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(format!("Mermaid · {}", diagram_type.display_name())),
        style: Some(TextStyle {
            font: FontId::monospace(),
            size: theme.metric_required("metric.font.mono_size"),
            weight: FontWeight::SEMIBOLD,
            slant: Default::default(),
            line_height: Some(theme.metric_required("metric.font.mono_line_height")),
            letter_spacing_em: None,
        }),
        color: Some(theme.color_required("muted-foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

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

fn render_code_block<H: UiHost>(
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

fn markdown_mdstream_pulldown_with<H: UiHost>(
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

fn render_mdstream_block_with_events<H: UiHost>(
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
                render_pulldown_events_root(cx, theme, markdown_theme, components, events)
            }
        }
        mdstream::BlockKind::BlockQuote => {
            let info = BlockQuoteInfo {
                text: strip_blockquote_prefix(block.display_or_raw()),
            };
            if let Some(render) = &components.blockquote {
                render(cx, info)
            } else {
                render_pulldown_events_root(cx, theme, markdown_theme, components, events)
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
                render_pulldown_events_root(cx, theme, markdown_theme, components, events)
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
                render_pulldown_events_root(cx, theme, markdown_theme, components, events)
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

    if pieces.iter().any(|p| match &p.kind {
        InlinePieceKind::Text(_) => p.style.code || p.style.strikethrough,
        InlinePieceKind::Image(_) | InlinePieceKind::InlineMath(_) => true,
    }) {
        return None;
    }

    if !allow_link_color_only && pieces.iter().any(|p| p.style.link.is_some()) {
        return None;
    }

    let mut text = String::new();
    let mut runs: Vec<TextRun> = Vec::new();
    for p in pieces {
        let InlinePieceKind::Text(t) = &p.kind else {
            continue;
        };
        if t.is_empty() {
            continue;
        }
        text.push_str(t);

        let run_weight = if p.style.strong {
            Some(FontWeight::SEMIBOLD)
        } else {
            None
        };
        let run_slant = if p.style.emphasis {
            Some(TextSlant::Italic)
        } else {
            None
        };
        let run_color = if p.style.link.is_some() {
            Some(markdown_theme.link)
        } else {
            None
        };

        runs.push(TextRun {
            len: t.len(),
            color: run_color,
            weight: run_weight,
            slant: run_slant,
        });
    }

    if text.is_empty() {
        return None;
    }

    let rich = RichText::new(Arc::<str>::from(text), runs);

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

fn render_pulldown_events_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let mut cursor = 0usize;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        &mut cursor,
        None,
    );
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
    FootnoteDefinition,
}

fn stop_matches(end: &pulldown_cmark::TagEnd, stop: PulldownStop) -> bool {
    use pulldown_cmark::TagEnd;
    match (stop, end) {
        (PulldownStop::Item, TagEnd::Item) => true,
        (PulldownStop::BlockQuote, TagEnd::BlockQuote(_)) => true,
        (PulldownStop::FootnoteDefinition, TagEnd::FootnoteDefinition) => true,
        _ => false,
    }
}

fn render_pulldown_blocks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
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
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::Start(Tag::Heading { level, .. }) => out.push(render_pulldown_heading(
                cx,
                theme,
                markdown_theme,
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
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
                *start,
            )),
            Event::Start(Tag::BlockQuote(_)) => out.push(render_pulldown_blockquote(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::Start(Tag::FootnoteDefinition(label)) => {
                out.push(render_pulldown_footnote_definition(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                    cursor,
                    Arc::<str>::from(label.to_string()),
                ))
            }
            Event::Start(Tag::Table(_)) => out.push(render_pulldown_table(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::DisplayMath(latex) => {
                out.push(render_math_block(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    Arc::<str>::from(latex.to_string()),
                ));
                *cursor += 1;
            }
            Event::Rule => {
                out.push(render_thematic_break(cx, theme, markdown_theme));
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

fn render_pulldown_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    use pulldown_cmark::{Alignment, Event, Tag, TagEnd};

    let alignments = match events.get(*cursor) {
        Some(Event::Start(Tag::Table(alignments))) => alignments.clone(),
        _ => Vec::new(),
    };

    *cursor += 1;

    let mut in_head = false;
    let mut header_rows: Vec<Vec<Vec<InlinePiece>>> = Vec::new();
    let mut body_rows: Vec<Vec<Vec<InlinePiece>>> = Vec::new();

    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::TableHead) => {
                in_head = true;
                *cursor += 1;
            }
            Event::End(TagEnd::TableHead) => {
                in_head = false;
                *cursor += 1;
            }
            Event::Start(Tag::TableRow) => {
                let row = parse_pulldown_table_row(events, cursor);
                if in_head {
                    header_rows.push(row);
                } else {
                    body_rows.push(row);
                }
            }
            Event::End(TagEnd::Table) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    let mut column_count = alignments.len();
    for row in header_rows.iter().chain(body_rows.iter()) {
        column_count = column_count.max(row.len());
    }

    fn justify_for_alignment(alignment: Alignment) -> MainAlign {
        match alignment {
            Alignment::Center => MainAlign::Center,
            Alignment::Right => MainAlign::End,
            Alignment::None | Alignment::Left => MainAlign::Start,
        }
    }

    let all_rows = header_rows
        .iter()
        .map(|r| (true, r))
        .chain(body_rows.iter().map(|r| (false, r)));

    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;

    cx.scroll(scroll_props, |cx| {
        let mut table_props = ContainerProps::default();
        table_props.padding = Edges::all(Px(0.0));
        table_props.border = Edges::all(Px(1.0));
        table_props.border_color = Some(markdown_theme.table_border);
        table_props.background = None;

        vec![cx.container(table_props, |cx| {
            let mut column_props = FlexProps::default();
            column_props.direction = Axis::Vertical;
            column_props.wrap = false;
            column_props.gap = Px(0.0);
            column_props.padding = Edges::all(Px(0.0));
            column_props.justify = MainAlign::Start;
            column_props.align = CrossAlign::Start;

            vec![cx.flex(column_props, |cx| {
                let mut row_index = 0usize;
                all_rows
                    .map(|(is_header, row)| {
                        let mut row_props = FlexProps::default();
                        row_props.direction = Axis::Horizontal;
                        row_props.wrap = false;
                        row_props.gap = Px(0.0);
                        row_props.padding = Edges::all(Px(0.0));
                        row_props.justify = MainAlign::Start;
                        row_props.align = CrossAlign::Stretch;

                        let cur_row_index = row_index;
                        row_index += 1;

                        cx.flex(row_props, |cx| {
                            (0..column_count)
                                .map(|col_index| {
                                    let pieces = row.get(col_index).cloned().unwrap_or_default();
                                    let justify = alignments
                                        .get(col_index)
                                        .copied()
                                        .map(justify_for_alignment)
                                        .unwrap_or(MainAlign::Start);
                                    render_table_cell(
                                        cx,
                                        theme,
                                        markdown_theme,
                                        components,
                                        is_header,
                                        cur_row_index,
                                        col_index,
                                        pieces,
                                        justify,
                                    )
                                })
                                .collect()
                        })
                    })
                    .collect()
            })]
        })]
    })
}

fn parse_pulldown_table_row(
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> Vec<Vec<InlinePiece>> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    *cursor += 1;
    let mut cells: Vec<Vec<InlinePiece>> = Vec::new();
    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::TableCell) => cells.push(parse_pulldown_table_cell(events, cursor)),
            Event::End(TagEnd::TableRow) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }
    cells
}

fn parse_pulldown_table_cell(
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> Vec<InlinePiece> {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::TableCell)) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }
    inline_pieces_from_events_unwrapped(&events[start..*cursor])
}

fn render_table_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    is_header: bool,
    row_index: usize,
    col_index: usize,
    pieces: Vec<InlinePiece>,
    justify: MainAlign,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.flex.grow = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));
    props.layout.size.min_width = Some(Px(0.0));
    props.padding = Edges {
        top: markdown_theme.table_cell_padding_y,
        right: markdown_theme.table_cell_padding_x,
        bottom: markdown_theme.table_cell_padding_y,
        left: markdown_theme.table_cell_padding_x,
    };
    props.border = Edges {
        top: if row_index > 0 { Px(1.0) } else { Px(0.0) },
        right: Px(0.0),
        bottom: Px(0.0),
        left: if col_index > 0 { Px(1.0) } else { Px(0.0) },
    };
    props.border_color = Some(markdown_theme.table_border);
    props.background = is_header.then_some(markdown_theme.table_header_bg);

    let font_size = theme.metric_required("metric.font.size");
    let line_height = theme.metric_required("metric.font.line_height");
    let fg = theme.color_required("foreground");
    let base = InlineBaseStyle {
        font: FontId::default(),
        size: font_size,
        weight: if is_header {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        },
        line_height: Some(line_height),
        color: fg,
    };

    cx.container(props, |cx| {
        vec![render_inline_flow_with_layout(
            cx,
            theme,
            markdown_theme,
            components,
            base,
            &pieces,
            justify,
        )]
    })
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
    render_math_block_mathjax_svg(cx, theme, markdown_theme, latex)
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

#[cfg(feature = "mathjax-svg")]
fn render_math_block_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Display, latex.as_ref());

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
        vec![cx.scroll(scroll_props, |cx| match entry {
            MathJaxSvgEntry::Ready(ready) => {
                let mut icon = SvgIconProps::new(SvgSource::Bytes(ready.svg_bytes));
                icon.fit = SvgFit::Contain;
                icon.color = markdown_theme.math_block_fg;
                icon.layout.size.height = Length::Px(markdown_theme.math_block_height);
                icon.layout.aspect_ratio = ready.aspect_ratio;
                vec![cx.svg_icon_props(icon)]
            }
            MathJaxSvgEntry::Loading => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: latex.clone(),
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
            })],
            MathJaxSvgEntry::Error(err) => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(format!("{latex} (mathjax error: {err})")),
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
            })],
        })]
    })
}

fn render_pulldown_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
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
    render_paragraph_inline(
        cx,
        theme,
        markdown_theme,
        components,
        &events[start..*cursor],
    )
}

fn render_pulldown_heading<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
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
    render_heading_inline(cx, theme, markdown_theme, components, info, slice)
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

    let start = *cursor;
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

    let mut hasher = DefaultHasher::new();
    start.hash(&mut hasher);
    language.as_deref().hash(&mut hasher);
    buf.hash(&mut hasher);
    let id = BlockId(hasher.finish());

    let info = CodeBlockInfo {
        id,
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
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    *cursor += 1;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        cursor,
        Some(PulldownStop::BlockQuote),
    );
    render_blockquote_container(cx, theme, markdown_theme, children)
}

fn render_blockquote_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    markdown_theme: MarkdownTheme,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::all(markdown_theme.blockquote_padding);
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: markdown_theme.blockquote_border_width,
    };
    props.border_color = Some(markdown_theme.blockquote_border);

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
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    start: Option<u64>,
) -> AnyElement {
    use pulldown_cmark::{Event, Tag, TagEnd};

    struct ListItem {
        task: Option<bool>,
        children: Vec<AnyElement>,
    }

    let ordered = start.is_some();
    let start_no = start.unwrap_or(1) as u32;

    *cursor += 1;
    let mut items: Vec<ListItem> = Vec::new();

    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::Item) => {
                *cursor += 1;
                let task = match events.get(*cursor) {
                    Some(Event::TaskListMarker(checked)) => {
                        *cursor += 1;
                        Some(*checked)
                    }
                    _ => None,
                };
                let children = render_pulldown_blocks(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                    cursor,
                    Some(PulldownStop::Item),
                );
                items.push(ListItem { task, children });
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
            .map(|(i, item)| {
                let body = if item.children.len() == 1 {
                    item.children.into_iter().next().unwrap()
                } else {
                    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
                        item.children
                    })
                };

                let marker_el = match item.task {
                    Some(checked) => {
                        let task_el = render_task_list_marker(cx, theme, markdown_theme, checked);
                        if ordered {
                            let no =
                                Arc::<str>::from(format!("{}.", start_no.saturating_add(i as u32)));
                            let no_el = cx.text_props(TextProps {
                                layout: Default::default(),
                                text: no,
                                style: None,
                                color: Some(markdown_theme.muted),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            });
                            stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N1).items_start(),
                                |_cx| vec![no_el, task_el],
                            )
                        } else {
                            task_el
                        }
                    }
                    None => {
                        let marker = if ordered {
                            Arc::<str>::from(format!("{}.", start_no.saturating_add(i as u32)))
                        } else {
                            Arc::<str>::from("•".to_string())
                        };

                        cx.text_props(TextProps {
                            layout: Default::default(),
                            text: marker,
                            style: None,
                            color: Some(markdown_theme.muted),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })
                    }
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

fn render_task_list_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    checked: bool,
) -> AnyElement {
    let (text, color) = if checked {
        ("☑", markdown_theme.task_checked)
    } else {
        ("☐", markdown_theme.task_unchecked)
    };

    cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(text.to_string()),
        style: Some(TextStyle {
            font: FontId::default(),
            size: theme.metric_required("metric.font.size"),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: Some(theme.metric_required("metric.font.line_height")),
            letter_spacing_em: None,
        }),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn render_pulldown_footnote_definition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    label: Arc<str>,
) -> AnyElement {
    *cursor += 1;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        cursor,
        Some(PulldownStop::FootnoteDefinition),
    );

    let label_el = cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(format!("[^{}]", label)),
        style: None,
        color: Some(markdown_theme.muted),
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
        |_cx| vec![label_el, body],
    )
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
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                out.push_str(t.as_ref())
            }
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
                .collect()
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
            .collect()
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
    render_inline_math_mathjax_svg(cx, theme, markdown_theme, info)
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

#[cfg(feature = "mathjax-svg")]
fn render_inline_math_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Inline, info.latex.as_ref());
    match entry {
        MathJaxSvgEntry::Ready(ready) => render_inline_math_svg(
            cx,
            theme,
            markdown_theme,
            ready.svg_bytes,
            ready.aspect_ratio,
        ),
        MathJaxSvgEntry::Loading => render_inline_math_source(cx, theme, markdown_theme, info),
        MathJaxSvgEntry::Error(err) => render_inline_math_source(
            cx,
            theme,
            markdown_theme,
            InlineMathInfo {
                latex: Arc::<str>::from(format!("{} (mathjax error: {err})", info.latex)),
            },
        ),
    }
}

#[cfg(feature = "mathjax-svg")]
fn render_inline_math_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
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
        let mut icon = SvgIconProps::new(SvgSource::Bytes(svg_bytes));
        icon.fit = SvgFit::Contain;
        icon.color = markdown_theme.inline_math_fg;
        icon.layout.size.height = Length::Px(markdown_theme.inline_math_height);
        icon.layout.aspect_ratio = aspect_ratio;
        vec![cx.svg_icon_props(icon)]
    })
}

#[cfg(feature = "mathjax-svg")]
fn render_inline_math_source<H: UiHost>(
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
            text: Arc::<str>::from(info.latex.trim().to_string()),
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

#[cfg(feature = "mathjax-svg")]
fn mathjax_svg_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: MathJaxMode,
    latex: &str,
) -> MathJaxSvgEntry {
    let latex = latex.trim();
    if latex.is_empty() {
        return MathJaxSvgEntry::Error(Arc::<str>::from("empty latex"));
    }

    let key = MathJaxKey {
        mode,
        latex: latex.to_string(),
    };

    let mut spawn = None::<(
        Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
        MathJaxKey,
    )>;
    let entry = cx
        .app
        .with_global_mut(MathJaxSvgCache::default, |cache, host| {
            let map = cache.inner.clone();
            let mut map_guard = map.lock().expect("mathjax svg cache lock");

            match map_guard.get(&key) {
                Some(existing) => {
                    if matches!(existing, MathJaxSvgEntry::Loading) {
                        host.request_redraw(cx.window);
                    }
                    return existing.clone();
                }
                None => {
                    map_guard.insert(key.clone(), MathJaxSvgEntry::Loading);
                    host.request_redraw(cx.window);
                    spawn = Some((map.clone(), key.clone()));
                    MathJaxSvgEntry::Loading
                }
            }
        });

    if let Some((map, key)) = spawn {
        let work = MathJaxWorkItem {
            map: map.clone(),
            key: key.clone(),
        };
        if let Err(_err) = mathjax_worker().tx.send(work) {
            let mut map_guard = map.lock().expect("mathjax svg cache lock");
            map_guard.insert(
                key,
                MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg worker unavailable")),
            );
        }
    }

    entry
}

#[cfg(feature = "mathjax-svg")]
fn svg_viewbox_aspect_ratio(svg: &str) -> Option<f32> {
    let idx = svg.find("viewBox=")?;
    let rest = &svg[idx + "viewBox=".len()..];
    let mut chars = rest.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = chars.as_str();
    let end = rest.find(quote)?;
    let value = &rest[..end];

    let mut nums: [f32; 4] = [0.0; 4];
    let mut i = 0usize;
    for part in value.split(|c: char| c.is_whitespace() || c == ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if i >= 4 {
            break;
        }
        nums[i] = part.parse::<f32>().ok()?;
        i += 1;
    }
    if i < 4 {
        return None;
    }
    let w = nums[2];
    let h = nums[3];
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(w / h)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        any::{Any, TypeId},
        collections::{HashMap, HashSet},
    };

    use fret_core::{AppWindowId, ClipboardToken, ImageUploadToken, Point, TimerToken};
    use fret_runtime::{
        CommandRegistry, CommandsHost, DragHost, DragKind, DragSession, Effect, EffectSink,
        FrameId, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost, TickId, TimeHost,
    };
    use fret_ui::ThemeConfig;

    #[derive(Default)]
    struct ThemeTestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        redraw: HashSet<AppWindowId>,
        effects: Vec<Effect>,
        drag: Option<DragSession>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for ThemeTestHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
            self.globals
                .get_mut(&TypeId::of::<T>())
                .and_then(|v| v.downcast_mut::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            #[derive(Debug)]
            struct GlobalLeaseMarker;

            struct Guard<T: Any> {
                type_id: TypeId,
                value: Option<T>,
                globals: *mut HashMap<TypeId, Box<dyn Any>>,
            }

            impl<T: Any> Drop for Guard<T> {
                fn drop(&mut self) {
                    let Some(value) = self.value.take() else {
                        return;
                    };
                    unsafe {
                        (*self.globals).insert(self.type_id, Box::new(value));
                    }
                }
            }

            let type_id = TypeId::of::<T>();
            let existing = self
                .globals
                .insert(type_id, Box::new(GlobalLeaseMarker) as Box<dyn Any>);

            let existing = match existing {
                None => None,
                Some(v) => {
                    if v.is::<GlobalLeaseMarker>() {
                        panic!("global already leased: {type_id:?}");
                    }
                    Some(*v.downcast::<T>().expect("global type id must match"))
                }
            };

            let mut guard = Guard::<T> {
                type_id,
                value: Some(existing.unwrap_or_else(init)),
                globals: &mut self.globals as *mut _,
            };

            let result = {
                let value = guard.value.as_mut().expect("guard value exists");
                f(value, self)
            };

            drop(guard);
            result
        }
    }

    impl ModelHost for ThemeTestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for ThemeTestHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for ThemeTestHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for ThemeTestHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraw.insert(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for ThemeTestHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            TimerToken(self.next_timer_token)
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            ClipboardToken(self.next_clipboard_token)
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl DragHost for ThemeTestHost {
        fn drag(&self) -> Option<&DragSession> {
            self.drag.as_ref()
        }

        fn drag_mut(&mut self) -> Option<&mut DragSession> {
            self.drag.as_mut()
        }

        fn cancel_drag(&mut self) {
            self.drag = None;
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            kind: DragKind,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new(source_window, kind, start, payload));
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            kind: DragKind,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new_cross_window(
                source_window,
                kind,
                start,
                payload,
            ));
        }
    }

    fn theme_with_metrics(metrics: &[(&str, f32)]) -> Theme {
        let mut host = ThemeTestHost::default();
        let theme = Theme::global_mut(&mut host);

        let mut cfg = ThemeConfig {
            name: theme.name.clone(),
            author: theme.author.clone(),
            url: theme.url.clone(),
            colors: HashMap::new(),
            metrics: HashMap::new(),
        };
        for (k, v) in metrics {
            cfg.metrics.insert((*k).to_string(), *v);
        }

        theme.apply_config(&cfg);
        theme.clone()
    }

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
    fn detects_mermaid_diagram_type() {
        assert_eq!(
            detect_mermaid_diagram_type("flowchart TD\n  A --> B"),
            MermaidDiagramType::Flowchart
        );
        assert_eq!(
            detect_mermaid_diagram_type("%% comment\nsequenceDiagram\n  A->>B: hi"),
            MermaidDiagramType::Sequence
        );
        assert_eq!(
            detect_mermaid_diagram_type("classDiagram\n  A <|-- B"),
            MermaidDiagramType::Class
        );
        assert_eq!(detect_mermaid_diagram_type(""), MermaidDiagramType::Unknown);
    }

    #[test]
    fn is_mermaid_language_is_case_insensitive() {
        assert!(is_mermaid_language(Some("mermaid")));
        assert!(is_mermaid_language(Some("Mermaid")));
        assert!(!is_mermaid_language(Some("rust")));
        assert!(!is_mermaid_language(None));
    }

    #[test]
    fn mdstream_assigns_stable_ids_for_full_source() {
        let source = "# A\n\nB\n\n```rust\nfn main() {}\n```\n";

        let mut s1 = mdstream::MdStream::default();
        let mut st1 = MarkdownPulldownState::new();
        st1.apply_update(s1.append(source));
        let ids1: Vec<_> = st1.doc().committed().iter().map(|b| b.id).collect();

        let mut s2 = mdstream::MdStream::default();
        let mut st2 = MarkdownPulldownState::new();
        st2.apply_update(s2.append(source));
        let ids2: Vec<_> = st2.doc().committed().iter().map(|b| b.id).collect();

        assert!(!ids1.is_empty());
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn mdstream_pulldown_state_applies_incrementally() {
        let mut stream = mdstream::MdStream::default();
        let mut state = MarkdownPulldownState::new();

        let u1 = stream.append("Hello\n\n```rust\nfn main() {");
        let a1 = state.apply_update(u1);
        assert!(!a1.reset);
        assert_eq!(state.doc().committed().len(), 1);
        assert!(state.doc().pending().is_some());

        let u2 = stream.append("}\n```\n");
        let _a2 = state.apply_update(u2);
        assert_eq!(state.doc().committed().len(), 2);
        assert!(state.doc().pending().is_none());
    }

    #[test]
    fn markdown_stream_state_keeps_blocks_with_footnotes() {
        let source = r#"# A

Footnotes are supported.[^note]

[^note]: This is a footnote definition.

$$
\int_0^1 x^2\,dx = \frac{1}{3}
$$
"#;

        let mut st = MarkdownStreamState::new();
        st.append(source);
        st.finalize();

        let committed = st.state().doc().committed();
        assert!(committed.len() > 1);
        assert!(
            committed
                .iter()
                .any(|b| b.kind == mdstream::BlockKind::FootnoteDefinition)
        );
        assert!(
            committed
                .iter()
                .any(|b| b.kind == mdstream::BlockKind::MathBlock)
        );
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
    fn pulldown_parses_gfm_task_list_marker() {
        use pulldown_cmark::Event;
        let events = parse_events("- [x] done\n- [ ] todo\n");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::TaskListMarker(true)))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::TaskListMarker(false)))
        );
    }

    #[test]
    fn pulldown_parses_gfm_autolinks_when_enabled() {
        let events = parse_events("<https://example.com>\n");
        let pieces = inline_pieces_from_events_unwrapped(&events);

        assert!(pieces.iter().any(|p| {
            let InlinePieceKind::Text(text) = &p.kind else {
                return false;
            };
            text.contains("https://example.com")
                && p.style.link.as_deref() == Some("https://example.com")
        }));
    }

    #[test]
    fn pulldown_parses_strikethrough_when_enabled() {
        use pulldown_cmark::{Event, Tag};
        let events = parse_events("~~gone~~\n");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::Strikethrough)))
        );
    }

    #[test]
    fn pulldown_parses_image_and_collects_alt_text() {
        let events = parse_events("![alt **bold** `code`](https://example.com/a.png \"t\")\n");
        let pieces = inline_pieces_from_events_unwrapped(&events);

        let imgs: Vec<_> = pieces
            .iter()
            .filter_map(|p| match &p.kind {
                InlinePieceKind::Image(info) => Some(info),
                _ => None,
            })
            .collect();

        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].src.as_ref(), "https://example.com/a.png");
        assert_eq!(imgs[0].alt.as_ref(), "alt bold code");
        assert_eq!(imgs[0].title.as_deref(), Some("t"));
        assert!(!imgs[0].is_svg);
    }

    #[test]
    fn pulldown_maps_inline_br_html_to_line_break() {
        let events = parse_events("a<br>b\n");
        let pieces = inline_pieces_from_events_unwrapped(&events);
        assert!(
            pieces
                .iter()
                .any(|p| matches!(&p.kind, InlinePieceKind::Text(t) if t.contains('\n')))
        );
    }

    #[test]
    fn autolinks_bare_urls_in_plain_text() {
        let style = InlineStyle {
            strong: false,
            emphasis: false,
            strikethrough: false,
            code: false,
            link: None,
        };
        let pieces = split_piece_into_tokens("See https://example.com).", &style);

        assert!(pieces.iter().any(|p| {
            let InlinePieceKind::Text(text) = &p.kind else {
                return false;
            };
            text == "https://example.com" && p.style.link.as_deref() == Some("https://example.com")
        }));
    }

    #[test]
    fn pulldown_parses_inline_math_when_enabled() {
        use pulldown_cmark::Event;
        let events = parse_events("$x^2$\n");
        assert!(events.iter().any(|e| matches!(e, Event::InlineMath(_))));
    }

    #[test]
    fn pulldown_parses_display_math_when_enabled() {
        use pulldown_cmark::Event;
        let events = parse_events("$$x^2$$\n");
        assert!(events.iter().any(|e| matches!(e, Event::DisplayMath(_))));
    }

    #[test]
    fn pulldown_parses_multiline_display_math_when_enabled() {
        use pulldown_cmark::Event;
        let events = parse_events("$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n");
        assert!(events.iter().any(|e| matches!(e, Event::DisplayMath(_))));
    }

    #[test]
    fn mdstream_math_block_body_strips_common_delimiters() {
        let mut stream = mdstream::MdStream::default();
        let update = stream.append("$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n");

        let mut state = MarkdownPulldownState::new();
        state.apply_update(update);

        let blocks = state.doc().committed();
        let math = blocks
            .iter()
            .find(|b| matches!(b.kind, mdstream::BlockKind::MathBlock))
            .expect("math block exists");

        let body = parse_math_block_body(math.display_or_raw());
        assert!(body.contains("\\int_0^1"));
    }

    #[test]
    fn detects_display_math_only_events() {
        let events = parse_events("$$x^2$$\n");
        let latex = display_math_only_events(&events);
        assert!(latex.is_some());
        assert_eq!(latex.unwrap().as_ref(), "x^2");
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

    #[test]
    fn code_block_max_height_prefers_fret_namespace() {
        let theme = theme_with_metrics(&[
            ("markdown.code_block.max_height", 111.0),
            ("fret.markdown.code_block.max_height", 222.0),
        ]);
        let mut options = fret_code_view::CodeBlockUiOptions::default();
        options.max_height = None;
        resolve_code_block_ui(&theme, &mut options);
        assert_eq!(options.max_height, Some(Px(222.0)));
    }

    #[test]
    fn code_block_max_height_falls_back_to_markdown_namespace() {
        let theme = theme_with_metrics(&[("markdown.code_block.max_height", 123.0)]);
        let mut options = fret_code_view::CodeBlockUiOptions::default();
        options.max_height = None;
        resolve_code_block_ui(&theme, &mut options);
        assert_eq!(options.max_height, Some(Px(123.0)));
    }

    #[test]
    fn code_block_max_height_does_not_override_explicit_option() {
        let theme = theme_with_metrics(&[("fret.markdown.code_block.max_height", 999.0)]);
        let mut options = fret_code_view::CodeBlockUiOptions::default();
        options.max_height = Some(Px(321.0));
        resolve_code_block_ui(&theme, &mut options);
        assert_eq!(options.max_height, Some(Px(321.0)));
    }
}
