use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::{BlockId, OnLinkActivate};

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
pub type AnchorDecorator<H> =
    dyn for<'a> Fn(&mut ElementContext<'a, H>, Arc<str>, AnyElement) -> AnyElement;

#[derive(Clone)]
pub struct MarkdownComponents<H: UiHost> {
    pub heading: Option<Arc<HeadingRenderer<H>>>,
    pub paragraph: Option<Arc<ParagraphRenderer<H>>>,
    pub code_block: Option<Arc<CodeBlockRenderer<H>>>,
    /// UI policy for the default fenced code block renderer (`fret-code-view`).
    ///
    /// If you set `code_block`, you own the full rendering and this value is ignored.
    pub code_block_ui: fret_code_view::CodeBlockUiOptions,
    /// Explicit retained/windowed config for the default fenced code block renderer.
    ///
    /// This stays separate from `code_block_ui` because opting into the retained lane changes
    /// the host contract (`H: UiHost + 'static`) and should never be implicit.
    pub code_block_windowed: Option<fret_code_view::CodeBlockWindowedOptions>,
    /// Whether the default fenced code block renderer should resolve `max_height` from theme
    /// tokens when `code_block_ui.max_height` is unset.
    ///
    /// This is a policy knob on `fret-markdown` rather than `fret-code-view` because only
    /// Markdown knows which theme token names are relevant.
    pub code_block_max_height_from_theme: bool,
    /// Per-code-block UI tweaks for the default fenced code block renderer (`fret-code-view`).
    ///
    /// This is applied after theme token resolution, so the resolver can override the final
    /// `CodeBlockUiOptions` for specific blocks (expand/collapse, wrap overrides, etc.). The
    /// retained/windowed lane is configured separately via `code_block_windowed`.
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
    /// Decorate elements that represent in-document anchors (headings, footnote definitions).
    ///
    /// This runs **before** `fret-markdown` attaches the anchor `test_id`, so decorators can wrap
    /// the element tree while keeping semantics/test IDs on the outermost wrapper.
    ///
    /// Typical use cases:
    ///
    /// - Wrap anchors in `LayoutQueryRegion` so the host can implement `#fragment` scroll
    ///   navigation without relying on semantics snapshots.
    /// - Attach additional host-side metadata without changing Markdown rendering policy.
    pub anchor_decorate: Option<Arc<AnchorDecorator<H>>>,
    /// Semantics overrides for the default task list marker renderer.
    ///
    /// Notes:
    /// - This is only used by the default pulldown renderer path (GFM task list items).
    /// - It is intentionally lightweight: Markdown is read-only content, but it still benefits from
    ///   structured a11y hints in snapshots/diagnostics.
    pub task_list_marker_role: Option<SemanticsRole>,
}

impl<H: UiHost> Default for MarkdownComponents<H> {
    fn default() -> Self {
        let mut code_block_ui = fret_code_view::CodeBlockUiOptions::default();
        code_block_ui.show_header = true;
        code_block_ui.header_divider = true;
        code_block_ui.header_background = fret_code_view::CodeBlockHeaderBackground::Muted80;
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
            code_block_windowed: None,
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
            anchor_decorate: None,
            task_list_marker_role: Some(SemanticsRole::Checkbox),
        }
    }
}

impl<H: UiHost> MarkdownComponents<H> {
    pub fn with_open_url(mut self) -> Self {
        self.on_link_activate = Some(crate::on_link_activate_open_url());
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

    pub fn with_code_block_windowed(
        mut self,
        windowed: Option<fret_code_view::CodeBlockWindowedOptions>,
    ) -> Self {
        self.code_block_windowed = windowed;
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
