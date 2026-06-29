use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Write as _,
    rc::Rc,
    sync::Arc,
};

use fret_core::{
    AttributedText, Color, Edges, FontId, FontWeight, Px, TextOverflow, TextPaintStyle,
    TextShapingStyle, TextSpan, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, InsetStyle, LayoutStyle, Length, OpacityProps,
    Overflow, PositionStyle, ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps,
    ScrollbarStyle, SelectableTextProps, SizeStyle, StackProps, StyledTextProps, TextInkOverflow,
    TextProps, VirtualListKeyCacheMode, VirtualListOptions, WheelRegionProps,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius, Space,
    ThemeTokenRead, ui,
};

use crate::copy_button::{CopyFeedbackRef, render_copy_button, render_copy_button_overlay};
use crate::prepare::{CodeBlockPrepareMode, CodeBlockPreparedState};
use crate::syntax::syntax_color;

#[derive(Clone, Copy)]
enum CodeBlockInput<'a> {
    Borrowed(&'a str),
    Shared(&'a Arc<str>),
}

#[derive(Default)]
struct CodeBlockTextCache {
    theme_revision: u64,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
    prepared: Option<Arc<crate::prepare::PreparedCodeBlock>>,
    rich: Option<AttributedText>,
    line_numbers: Option<Arc<str>>,
}

fn code_shaping_for_code_block_flags(
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
) -> TextShapingStyle {
    let mut shaping = TextShapingStyle::default();
    if disable_ligatures {
        shaping = shaping.with_feature("liga", 0);
    }
    if disable_contextual_alternates {
        shaping = shaping.with_feature("calt", 0);
    }
    shaping
}

fn build_code_block_rich(
    theme: &Theme,
    prepared: &crate::prepare::PreparedCodeBlock,
    code_shaping: &TextShapingStyle,
) -> AttributedText {
    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    for (line_i, line) in prepared.lines.iter().enumerate() {
        for seg in &line.segments {
            if seg.text.is_empty() {
                continue;
            }
            let color = seg.highlight.and_then(|h| syntax_color(theme, h));
            text.push_str(seg.text.as_ref());
            spans.push(TextSpan {
                len: seg.text.len(),
                shaping: code_shaping.clone(),
                paint: TextPaintStyle {
                    fg: color,
                    ..Default::default()
                },
            });
        }
        if line_i + 1 < prepared.lines.len() {
            text.push('\n');
            spans.push(TextSpan {
                len: 1,
                shaping: code_shaping.clone(),
                ..Default::default()
            });
        }
    }

    AttributedText::new(Arc::<str>::from(text), spans)
}

fn build_line_numbers(prepared: &crate::prepare::PreparedCodeBlock) -> Arc<str> {
    Arc::<str>::from({
        let mut s = String::new();
        for (i, _line) in prepared.lines.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            let n = i + 1;
            s.push_str(&format!(
                "{n:>width$}",
                n = n,
                width = prepared.line_number_width
            ));
        }
        s
    })
}

#[track_caller]
fn resolve_code_block_cached_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: &Arc<crate::prepare::PreparedCodeBlock>,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
) -> (AttributedText, Option<Arc<str>>) {
    cx.slot_state(CodeBlockTextCache::default, |st| {
        let theme_revision = theme.revision();
        let needs_rebuild = st.rich.is_none()
            || st.theme_revision != theme_revision
            || st.disable_ligatures != disable_ligatures
            || st.disable_contextual_alternates != disable_contextual_alternates
            || st
                .prepared
                .as_ref()
                .is_none_or(|p| !Arc::ptr_eq(p, prepared));

        if needs_rebuild {
            st.theme_revision = theme_revision;
            st.disable_ligatures = disable_ligatures;
            st.disable_contextual_alternates = disable_contextual_alternates;
            st.prepared = Some(prepared.clone());
            let shaping =
                code_shaping_for_code_block_flags(disable_ligatures, disable_contextual_alternates);
            st.rich = Some(build_code_block_rich(theme, prepared.as_ref(), &shaping));
            st.line_numbers = prepared
                .show_line_numbers
                .then(|| build_line_numbers(prepared.as_ref()));
        }

        (
            st.rich.clone().unwrap_or_else(|| {
                AttributedText::new(Arc::<str>::from(""), Arc::<[TextSpan]>::from([]))
            }),
            st.line_numbers.clone(),
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockWrap {
    /// Do not wrap; use horizontal scrolling for long lines.
    #[default]
    ScrollX,
    /// Wrap at word boundaries (best-effort, depends on the text system).
    Word,
    /// Wrap between grapheme clusters when needed (recommended for long identifiers/paths).
    Grapheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockCopyButtonPlacement {
    #[default]
    Overlay,
    Header,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockHeaderBackground {
    #[default]
    None,
    Secondary,
    /// Upstream AI Elements uses `bg-muted/80` for the header row.
    Muted80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeBlockWindowedOptions {
    pub overscan: usize,
    pub highlight_mode: CodeBlockWindowedHighlightMode,
}

impl CodeBlockWindowedOptions {
    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan.max(1);
        self
    }

    /// Controls how syntax highlighting is prepared for the retained/windowed renderer.
    pub fn highlight_mode(mut self, mode: CodeBlockWindowedHighlightMode) -> Self {
        self.highlight_mode = mode;
        self
    }

    fn normalized(mut self) -> Self {
        self.overscan = self.overscan.max(1);
        self
    }
}

impl Default for CodeBlockWindowedOptions {
    fn default() -> Self {
        Self {
            overscan: 6,
            highlight_mode: CodeBlockWindowedHighlightMode::Full,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockWindowedHighlightMode {
    /// Preserve syntax highlighting by preparing the full source before mounting the windowed list.
    #[default]
    Full,
    /// Skip full-source highlighting and keep an indexed plain-text line model.
    ///
    /// This is intended for very large or synthetic code surfaces where first-frame latency is more
    /// important than syntax color parity. A future async/incremental highlighter should replace
    /// this escape hatch for editor-grade surfaces.
    PlainIndexed,
}

#[derive(Debug)]
pub struct CodeBlockHeaderSlots {
    pub show_language: bool,
    pub left: Vec<AnyElement>,
    pub right: Vec<AnyElement>,
}

impl Default for CodeBlockHeaderSlots {
    fn default() -> Self {
        Self {
            show_language: true,
            left: Vec::new(),
            right: Vec::new(),
        }
    }
}

impl CodeBlockHeaderSlots {
    pub fn show_language(mut self, show: bool) -> Self {
        self.show_language = show;
        self
    }

    pub fn left(mut self, els: impl IntoIterator<Item = AnyElement>) -> Self {
        self.left.extend(els);
        self
    }

    pub fn right(mut self, els: impl IntoIterator<Item = AnyElement>) -> Self {
        self.right.extend(els);
        self
    }

    pub fn push_left(mut self, el: AnyElement) -> Self {
        self.left.push(el);
        self
    }

    pub fn push_right(mut self, el: AnyElement) -> Self {
        self.right.push(el);
        self
    }
}

#[derive(Debug)]
pub struct CodeBlock {
    code: Arc<str>,
    language: Option<Arc<str>>,
    show_line_numbers: bool,
    show_header: bool,
    show_language_in_header: bool,
    header_divider: bool,
    header_background: CodeBlockHeaderBackground,
    show_copy_button: bool,
    copy_button_on_hover: bool,
    copy_button_placement: CodeBlockCopyButtonPlacement,
    border: bool,
    wrap: CodeBlockWrap,
    max_height: Option<Px>,
    windowed: Option<CodeBlockWindowedOptions>,
    show_scrollbar_x: bool,
    scrollbar_x_on_hover: bool,
    show_scrollbar_y: bool,
    scrollbar_y_on_hover: bool,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
}

impl CodeBlock {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_line_numbers: false,
            show_header: false,
            show_language_in_header: true,
            header_divider: false,
            header_background: CodeBlockHeaderBackground::None,
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: CodeBlockCopyButtonPlacement::Overlay,
            border: true,
            wrap: CodeBlockWrap::ScrollX,
            max_height: None,
            windowed: None,
            show_scrollbar_x: false,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: false,
            scrollbar_y_on_hover: true,
            disable_ligatures: true,
            disable_contextual_alternates: true,
        }
    }

    pub fn language(mut self, language: impl Into<Arc<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    pub fn show_language_in_header(mut self, show: bool) -> Self {
        self.show_language_in_header = show;
        self
    }

    pub fn header_divider(mut self, show: bool) -> Self {
        self.header_divider = show;
        self
    }

    pub fn header_background(mut self, bg: CodeBlockHeaderBackground) -> Self {
        self.header_background = bg;
        self
    }

    pub fn show_copy_button(mut self, show: bool) -> Self {
        self.show_copy_button = show;
        self
    }

    pub fn copy_button_on_hover(mut self, on_hover: bool) -> Self {
        self.copy_button_on_hover = on_hover;
        self
    }

    pub fn copy_button_placement(mut self, placement: CodeBlockCopyButtonPlacement) -> Self {
        self.copy_button_placement = placement;
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn wrap(mut self, wrap: CodeBlockWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Enables the retained/windowed renderer.
    ///
    /// Call `into_element_windowed` to opt into the retained lane. The default `into_element`
    /// keeps the non-windowed contract so callers do not inherit a `'static` requirement unless
    /// they explicitly choose it.
    pub fn windowed(mut self, options: CodeBlockWindowedOptions) -> Self {
        self.windowed = Some(options.normalized());
        self
    }

    pub fn show_scrollbar_x(mut self, show: bool) -> Self {
        self.show_scrollbar_x = show;
        self
    }

    pub fn scrollbar_x_on_hover(mut self, on_hover: bool) -> Self {
        self.scrollbar_x_on_hover = on_hover;
        self
    }

    pub fn show_scrollbar_y(mut self, show: bool) -> Self {
        self.show_scrollbar_y = show;
        self
    }

    pub fn scrollbar_y_on_hover(mut self, on_hover: bool) -> Self {
        self.scrollbar_y_on_hover = on_hover;
        self
    }

    pub fn disable_ligatures(mut self, disable: bool) -> Self {
        self.disable_ligatures = disable;
        self
    }

    pub fn disable_contextual_alternates(mut self, disable: bool) -> Self {
        self.disable_contextual_alternates = disable;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        debug_assert!(
            self.windowed.is_none(),
            "CodeBlock::windowed(...) requires CodeBlock::into_element_windowed(...)"
        );
        code_block_with_header_slots_impl(
            cx,
            CodeBlockInput::Shared(&self.code),
            self.language.as_deref(),
            self.show_line_numbers,
            CodeBlockUiOptions {
                show_header: self.show_header,
                header_divider: self.header_divider,
                header_background: self.header_background,
                show_copy_button: self.show_copy_button,
                copy_button_on_hover: self.copy_button_on_hover,
                copy_button_placement: self.copy_button_placement,
                border: self.border,
                wrap: self.wrap,
                max_height: self.max_height,
                show_scrollbar_x: self.show_scrollbar_x,
                scrollbar_x_on_hover: self.scrollbar_x_on_hover,
                show_scrollbar_y: self.show_scrollbar_y,
                scrollbar_y_on_hover: self.scrollbar_y_on_hover,
                disable_ligatures: self.disable_ligatures,
                disable_contextual_alternates: self.disable_contextual_alternates,
            },
            CodeBlockHeaderSlots::default().show_language(self.show_language_in_header),
            CodeBlockPrepareMode::Full,
            |cx, theme, prepared, options| {
                render_code_block_body_non_windowed(
                    cx,
                    theme,
                    prepared,
                    options.wrap,
                    options.show_scrollbar_x,
                    options.scrollbar_x_on_hover,
                    options.show_scrollbar_y,
                    options.scrollbar_y_on_hover,
                    options.max_height,
                    options.disable_ligatures,
                    options.disable_contextual_alternates,
                )
            },
        )
    }

    #[track_caller]
    pub fn into_element_windowed<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let windowed = self.windowed.unwrap_or_default();
        code_block_with_header_slots_impl(
            cx,
            CodeBlockInput::Shared(&self.code),
            self.language.as_deref(),
            self.show_line_numbers,
            CodeBlockUiOptions {
                show_header: self.show_header,
                header_divider: self.header_divider,
                header_background: self.header_background,
                show_copy_button: self.show_copy_button,
                copy_button_on_hover: self.copy_button_on_hover,
                copy_button_placement: self.copy_button_placement,
                border: self.border,
                wrap: self.wrap,
                max_height: self.max_height,
                show_scrollbar_x: self.show_scrollbar_x,
                scrollbar_x_on_hover: self.scrollbar_x_on_hover,
                show_scrollbar_y: self.show_scrollbar_y,
                scrollbar_y_on_hover: self.scrollbar_y_on_hover,
                disable_ligatures: self.disable_ligatures,
                disable_contextual_alternates: self.disable_contextual_alternates,
            },
            CodeBlockHeaderSlots::default().show_language(self.show_language_in_header),
            match windowed.highlight_mode {
                CodeBlockWindowedHighlightMode::Full => CodeBlockPrepareMode::Full,
                CodeBlockWindowedHighlightMode::PlainIndexed => CodeBlockPrepareMode::LineIndexed,
            },
            move |cx, theme, prepared, options| {
                render_code_block_body(cx, theme, prepared, options, windowed)
            },
        )
    }

    #[track_caller]
    pub fn into_element_non_windowed<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        self.windowed = None;
        self.into_element(cx)
    }
}

#[track_caller]
pub fn code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) -> AnyElement {
    code_block_with(
        cx,
        code,
        language,
        show_line_numbers,
        CodeBlockUiOptions::default(),
    )
}

#[track_caller]
pub fn code_block_windowed<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) -> AnyElement {
    code_block_with_windowed(
        cx,
        code,
        language,
        show_line_numbers,
        CodeBlockUiOptions::default(),
        CodeBlockWindowedOptions::default(),
    )
}

#[track_caller]
pub fn code_block_non_windowed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) -> AnyElement {
    code_block_with_non_windowed(
        cx,
        code,
        language,
        show_line_numbers,
        CodeBlockUiOptions::default(),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct CodeBlockUiOptions {
    pub show_header: bool,
    pub header_divider: bool,
    pub header_background: CodeBlockHeaderBackground,
    pub show_copy_button: bool,
    pub copy_button_on_hover: bool,
    pub copy_button_placement: CodeBlockCopyButtonPlacement,
    pub border: bool,
    pub wrap: CodeBlockWrap,
    pub max_height: Option<Px>,
    pub show_scrollbar_x: bool,
    pub scrollbar_x_on_hover: bool,
    pub show_scrollbar_y: bool,
    pub scrollbar_y_on_hover: bool,
    /// Best-effort OpenType feature policy for code shaping.
    pub disable_ligatures: bool,
    pub disable_contextual_alternates: bool,
}

impl Default for CodeBlockUiOptions {
    fn default() -> Self {
        Self {
            show_header: false,
            header_divider: false,
            header_background: CodeBlockHeaderBackground::None,
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: CodeBlockCopyButtonPlacement::Overlay,
            border: true,
            wrap: CodeBlockWrap::ScrollX,
            max_height: None,
            show_scrollbar_x: false,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: false,
            scrollbar_y_on_hover: true,
            // Common editor baseline: disable `liga`/`calt` for code, best-effort.
            disable_ligatures: true,
            disable_contextual_alternates: true,
        }
    }
}

#[track_caller]
pub fn code_block_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
) -> AnyElement {
    code_block_with_header_slots(
        cx,
        code,
        language,
        show_line_numbers,
        options,
        CodeBlockHeaderSlots::default(),
    )
}

#[track_caller]
pub fn code_block_with_windowed<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    windowed: CodeBlockWindowedOptions,
) -> AnyElement {
    code_block_with_header_slots_windowed(
        cx,
        code,
        language,
        show_line_numbers,
        options,
        CodeBlockHeaderSlots::default(),
        windowed,
    )
}

#[track_caller]
pub fn code_block_with_non_windowed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
) -> AnyElement {
    code_block_with_header_slots_non_windowed(
        cx,
        code,
        language,
        show_line_numbers,
        options,
        CodeBlockHeaderSlots::default(),
    )
}

#[track_caller]
pub fn code_block_with_header_slots<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    header: CodeBlockHeaderSlots,
) -> AnyElement {
    code_block_with_header_slots_non_windowed(
        cx,
        code,
        language,
        show_line_numbers,
        options,
        header,
    )
}

#[track_caller]
pub fn code_block_with_header_slots_windowed<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    header: CodeBlockHeaderSlots,
    windowed: CodeBlockWindowedOptions,
) -> AnyElement {
    let windowed = windowed.normalized();
    let prepare_mode = match windowed.highlight_mode {
        CodeBlockWindowedHighlightMode::Full => CodeBlockPrepareMode::Full,
        CodeBlockWindowedHighlightMode::PlainIndexed => CodeBlockPrepareMode::LineIndexed,
    };
    code_block_with_header_slots_impl(
        cx,
        CodeBlockInput::Borrowed(code),
        language,
        show_line_numbers,
        options,
        header,
        prepare_mode,
        move |cx, theme, prepared, options| {
            render_code_block_body(cx, theme, prepared, options, windowed)
        },
    )
}

#[track_caller]
pub fn code_block_with_header_slots_non_windowed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    header: CodeBlockHeaderSlots,
) -> AnyElement {
    code_block_with_header_slots_impl(
        cx,
        CodeBlockInput::Borrowed(code),
        language,
        show_line_numbers,
        options,
        header,
        CodeBlockPrepareMode::Full,
        |cx, theme, prepared, options| {
            render_code_block_body_non_windowed(
                cx,
                theme,
                prepared,
                options.wrap,
                options.show_scrollbar_x,
                options.scrollbar_x_on_hover,
                options.show_scrollbar_y,
                options.scrollbar_y_on_hover,
                options.max_height,
                options.disable_ligatures,
                options.disable_contextual_alternates,
            )
        },
    )
}

#[track_caller]
fn code_block_with_header_slots_impl<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    code: CodeBlockInput<'_>,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    header: CodeBlockHeaderSlots,
    prepare_mode: CodeBlockPrepareMode,
    render_body: F,
) -> AnyElement
where
    F: Fn(
        &mut ElementContext<'_, H>,
        &Theme,
        Arc<crate::prepare::PreparedCodeBlock>,
        CodeBlockUiOptions,
    ) -> AnyElement,
{
    let theme = Theme::global(&*cx.app).clone();
    let bg = theme.color_token("card");
    let border = theme.color_token("border");

    let chrome = {
        let mut chrome = ChromeRefinement::default().rounded(Radius::Md);
        if options.border {
            chrome = chrome
                .border_1()
                .bg(ColorRef::Color(bg))
                .border_color(ColorRef::Color(border));
        } else {
            chrome = chrome.bg(ColorRef::Color(bg));
        }
        chrome
    };
    let mut props =
        decl_style::container_props(&theme, chrome, LayoutRefinement::default().w_full());
    props.layout.position = PositionStyle::Relative;

    let language = language.map(str::trim).filter(|s| !s.is_empty());
    let prepared = cx.slot_state(CodeBlockPreparedState::default, |st| {
        match code {
            CodeBlockInput::Borrowed(code) => {
                st.prepare(code, language, show_line_numbers, prepare_mode);
            }
            CodeBlockInput::Shared(code) => {
                st.prepare_arc(code, language, show_line_numbers, prepare_mode);
            }
        }
        st.prepared.clone()
    });

    let copy_code = options.show_copy_button.then(|| match code {
        CodeBlockInput::Borrowed(code) => Arc::<str>::from(code),
        CodeBlockInput::Shared(code) => Arc::clone(code),
    });
    let feedback = cx.slot_state(CopyFeedbackRef::default, |st| st.clone());
    let needs_hover_tracking = code_block_needs_hover_tracking(options);

    cx.container(props, move |cx| {
        if needs_hover_tracking {
            vec![
                cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
                    render_code_block_content(
                        cx,
                        &theme,
                        language,
                        header,
                        options,
                        prepared.clone(),
                        copy_code.clone(),
                        feedback.clone(),
                        &render_body,
                        hovered,
                    )
                }),
            ]
        } else {
            render_code_block_content(
                cx,
                &theme,
                language,
                header,
                options,
                prepared.clone(),
                copy_code.clone(),
                feedback.clone(),
                &render_body,
                false,
            )
        }
    })
}

fn code_block_needs_hover_tracking(options: CodeBlockUiOptions) -> bool {
    let copy_depends_on_hover = options.show_copy_button && options.copy_button_on_hover;
    let scrollbar_x_depends_on_hover = options.show_scrollbar_x && options.scrollbar_x_on_hover;
    let scrollbar_y_depends_on_hover =
        options.show_scrollbar_y && options.max_height.is_some() && options.scrollbar_y_on_hover;

    copy_depends_on_hover || scrollbar_x_depends_on_hover || scrollbar_y_depends_on_hover
}

#[allow(clippy::too_many_arguments)]
fn render_code_block_content<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    language: Option<&str>,
    mut header: CodeBlockHeaderSlots,
    options: CodeBlockUiOptions,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    copy_code: Option<Arc<str>>,
    feedback: CopyFeedbackRef,
    render_body: &F,
    hovered: bool,
) -> Vec<AnyElement>
where
    F: Fn(
        &mut ElementContext<'_, H>,
        &Theme,
        Arc<crate::prepare::PreparedCodeBlock>,
        CodeBlockUiOptions,
    ) -> AnyElement,
{
    let copied = feedback.is_copied();
    let copy_visible = !options.copy_button_on_hover || hovered || copied;
    let scrollbar_x_enabled = options.show_scrollbar_x;
    let scrollbar_x_visible = scrollbar_x_enabled && (!options.scrollbar_x_on_hover || hovered);
    let scrollbar_y_enabled = options.show_scrollbar_y && options.max_height.is_some();
    let scrollbar_y_visible = scrollbar_y_enabled && (!options.scrollbar_y_on_hover || hovered);

    let header_visible = options.show_header
        || !header.left.is_empty()
        || !header.right.is_empty()
        || (header.show_language && language.is_some());

    if !header_visible {
        header.show_language = false;
    }

    let body_options = CodeBlockUiOptions {
        show_scrollbar_x: scrollbar_x_enabled,
        scrollbar_x_on_hover: scrollbar_x_visible,
        show_scrollbar_y: scrollbar_y_enabled,
        scrollbar_y_on_hover: scrollbar_y_visible,
        ..options
    };

    let content = if !header_visible {
        render_body(cx, theme, prepared.clone(), body_options)
    } else {
        ui::v_flex(|cx| {
            let mut out = Vec::new();
            out.push(render_code_block_header(
                cx,
                theme,
                language,
                header,
                options.header_divider,
                options.header_background,
                if options.show_copy_button
                    && options.copy_button_placement == CodeBlockCopyButtonPlacement::Header
                {
                    Some(CopyButtonInHeader {
                        feedback: feedback.clone(),
                        code: copy_code
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| Arc::<str>::from("")),
                        visible: copy_visible,
                    })
                } else {
                    None
                },
            ));
            out.push(render_body(cx, theme, prepared.clone(), body_options));
            out
        })
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let mut out = vec![content];
    if options.show_copy_button
        && options.copy_button_placement == CodeBlockCopyButtonPlacement::Overlay
        && let Some(code) = copy_code.clone()
    {
        let el = render_copy_button_overlay(cx, theme, feedback.clone(), code);
        out.push(cx.opacity(if copy_visible { 1.0 } else { 0.0 }, |_cx| vec![el]));
    }
    out
}

#[derive(Clone)]
struct CopyButtonInHeader {
    feedback: CopyFeedbackRef,
    code: Arc<str>,
    visible: bool,
}

fn render_code_block_header<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    language: Option<&str>,
    header: CodeBlockHeaderSlots,
    divider: bool,
    background: CodeBlockHeaderBackground,
    copy: Option<CopyButtonInHeader>,
) -> AnyElement {
    // shadcn/AI Elements baseline: `px-3 py-2`.
    let pad_x = MetricRef::space(Space::N3).resolve(theme);
    let pad_y = MetricRef::space(Space::N2).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::symmetric(pad_x, pad_y).into();
    match background {
        CodeBlockHeaderBackground::None => {}
        CodeBlockHeaderBackground::Secondary => {
            props.background = Some(theme.color_token("secondary"));
        }
        CodeBlockHeaderBackground::Muted80 => {
            let muted = theme.color_token("muted");
            props.background = Some(Color { a: 0.8, ..muted });
        }
    }
    if divider {
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(theme.color_token("border"));
    }

    cx.container(props, |cx| {
        vec![
            ui::h_flex(|cx| {
                let mut left = Vec::new();
                if header.show_language
                    && let Some(lang) = language
                {
                    left.push(cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::<str>::from(lang.to_string()),
                        style: Some(typography::as_control_text(TextStyle {
                            font: FontId::monospace(),
                            size: theme.metric_token("metric.font.mono_size"),
                            weight: FontWeight::SEMIBOLD,
                            slant: Default::default(),
                            line_height: Some(theme.metric_token("metric.font.mono_line_height")),
                            letter_spacing_em: None,
                            ..Default::default()
                        })),
                        color: Some(theme.color_token("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: TextInkOverflow::None,
                    }));
                }
                left.extend(header.left);

                let mut right = Vec::new();
                right.extend(header.right);
                if let Some(copy) = copy {
                    let el = render_copy_button(cx, theme, copy.feedback, copy.code);
                    right.push(cx.opacity(if copy.visible { 1.0 } else { 0.0 }, |_cx| vec![el]));
                }

                vec![
                    ui::h_row(move |_cx| left).gap(Space::N1).into_element(cx),
                    ui::h_row(move |_cx| right)
                        .gap(Space::N1)
                        .justify_end()
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .justify(Justify::Between)
            .items(Items::Center)
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
}

#[allow(clippy::too_many_arguments)]
#[track_caller]
fn render_code_block_body<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    options: CodeBlockUiOptions,
    windowed: CodeBlockWindowedOptions,
) -> AnyElement {
    // shadcn/AI Elements baseline: `p-4`.
    let pad = MetricRef::space(Space::N4).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.overflow = Overflow::Clip;
    props.padding = Edges::all(pad).into();

    cx.container(props, |cx| {
        let wrap = if prepared.show_line_numbers {
            debug_assert!(
                !matches!(options.wrap, CodeBlockWrap::Word | CodeBlockWrap::Grapheme),
                "wrapping with line numbers is not supported yet"
            );
            CodeBlockWrap::ScrollX
        } else {
            options.wrap
        };

        let scrollbar_w = theme.metric_token("metric.scrollbar.width");

        let content = if options.max_height.is_some() && matches!(wrap, CodeBlockWrap::ScrollX) {
            let reserved_right_for_x_scrollbar = if options.show_scrollbar_y {
                scrollbar_w
            } else {
                Px(0.0)
            };
            render_code_block_windowed_lines(
                cx,
                theme,
                prepared.clone(),
                windowed.overscan,
                options.show_scrollbar_x,
                options.scrollbar_x_on_hover,
                reserved_right_for_x_scrollbar,
                options.show_scrollbar_y,
                options.scrollbar_y_on_hover,
                options.max_height,
                options.disable_ligatures,
                options.disable_contextual_alternates,
            )
        } else {
            render_code_block_body_non_windowed(
                cx,
                theme,
                prepared.clone(),
                wrap,
                options.show_scrollbar_x,
                options.scrollbar_x_on_hover,
                options.show_scrollbar_y,
                options.scrollbar_y_on_hover,
                options.max_height,
                options.disable_ligatures,
                options.disable_contextual_alternates,
            )
        };

        vec![content]
    })
}

#[allow(clippy::too_many_arguments)]
fn render_code_block_body_non_windowed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    wrap: CodeBlockWrap,
    scrollbar_x_enabled: bool,
    scrollbar_x_visible: bool,
    scrollbar_y_enabled: bool,
    scrollbar_y_visible: bool,
    max_height: Option<Px>,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
) -> AnyElement {
    let (rich, line_numbers) = resolve_code_block_cached_text(
        cx,
        theme,
        &prepared,
        disable_ligatures,
        disable_contextual_alternates,
    );
    let line_count = prepared.lines.len();

    let needs_scroll_y = match max_height {
        None => false,
        Some(max_height) => match text_wrap_for_code_block_wrap(wrap) {
            TextWrap::None => {
                let line_height = theme.metric_token("metric.font.mono_line_height");
                let est_h = Px(line_height.0 * (line_count.max(1) as f32));
                est_h.0 > max_height.0
            }
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme => true,
        },
    };
    let scrollbar_w = theme.metric_token("metric.scrollbar.width");
    let reserved_right_for_x_scrollbar = if needs_scroll_y && scrollbar_y_enabled {
        scrollbar_w
    } else {
        Px(0.0)
    };

    let content = if !prepared.show_line_numbers {
        render_code_block_text(
            cx,
            theme,
            rich,
            wrap,
            scrollbar_x_enabled,
            scrollbar_x_visible,
            reserved_right_for_x_scrollbar,
            line_count,
        )
    } else {
        let code = render_code_block_text(
            cx,
            theme,
            rich,
            wrap,
            scrollbar_x_enabled,
            scrollbar_x_visible,
            reserved_right_for_x_scrollbar,
            line_count,
        );
        let line_numbers = line_numbers.unwrap_or_else(|| Arc::<str>::from(""));
        render_code_block_with_line_numbers(cx, theme, line_numbers, code)
    };

    if let (Some(max_height), true) = (max_height, needs_scroll_y) {
        let thumb = theme.color_token("scrollbar.thumb.background");
        let thumb_hover = theme.color_token("scrollbar.thumb.hover.background");
        let handle = cx.slot_state(ScrollHandle::default, |h| h.clone());

        let outer_layout = {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            // `Scroll` children frequently use `Length::Fill` so they need a definite
            // viewport height. Using `max_height` alone yields an indefinite height
            // (auto) which can collapse or produce inconsistent layout.
            layout.size.height = Length::Px(max_height);
            layout.size.min_height = Some(Length::Px(max_height));
            layout.size.max_height = Some(Length::Px(max_height));
            layout.overflow = Overflow::Clip;
            layout
        };

        let scroll = cx.scroll(
            ScrollProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout.overflow = Overflow::Clip;
                    layout
                },
                axis: ScrollAxis::Y,
                scroll_handle: Some(handle.clone()),
                ..Default::default()
            },
            |_cx| vec![content],
        );

        let scroll_id = scroll.id;
        return cx.stack_props(
            StackProps {
                layout: outer_layout,
            },
            move |cx| {
                let mut out = vec![scroll];

                if scrollbar_y_enabled {
                    let scrollbar_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(Px(0.0)).into(),
                            right: Some(Px(0.0)).into(),
                            bottom: Some(if scrollbar_x_enabled {
                                scrollbar_w
                            } else {
                                Px(0.0)
                            })
                            .into(),
                            left: None.into(),
                        },
                        size: SizeStyle {
                            width: Length::Px(scrollbar_w),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(ScrollbarProps {
                        layout: fill_layout(),
                        axis: ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll_id),
                        scroll_handle: handle,
                        style: ScrollbarStyle {
                            thumb,
                            thumb_hover,
                            ..Default::default()
                        },
                    });

                    out.push(overlay_chrome(
                        cx,
                        scrollbar_layout,
                        scrollbar_y_visible,
                        scrollbar,
                    ));
                }

                out
            },
        );
    }

    content
}

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn overlay_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    visible: bool,
    child: AnyElement,
) -> AnyElement {
    let opacity = if visible { 1.0 } else { 0.0 };
    cx.opacity_props(OpacityProps { layout, opacity }, |_cx| vec![child])
}

const WINDOWED_LINE_NUMBER_SEPARATOR: &str = "  ";
const WINDOWED_LINE_CHUNK_SIZE: usize = 4;

fn decimal_digits(mut n: usize) -> usize {
    let mut digits = 1;
    while n >= 10 {
        n /= 10;
        digits += 1;
    }
    digits
}

fn write_windowed_line_number_prefix(
    out: &mut String,
    prepared: &crate::prepare::PreparedCodeBlock,
    line_i: usize,
) {
    let n = line_i + 1;
    let width = prepared.line_number_width.max(decimal_digits(n));
    let _ = write!(out, "{n:>width$}");
    out.push_str(WINDOWED_LINE_NUMBER_SEPARATOR);
}

fn build_code_block_line_rich(
    row_theme: &CodeBlockLineRowTheme,
    prepared: &crate::prepare::PreparedCodeBlock,
    line_i: usize,
) -> AttributedText {
    let Some(line) = prepared.lines.get(line_i) else {
        return AttributedText::new(Arc::<str>::from(""), Arc::<[TextSpan]>::from([]));
    };
    if !prepared.show_line_numbers
        && line.segments.len() == 1
        && let Some(seg) = line.segments.first()
        && !seg.text.is_empty()
    {
        return AttributedText::new(
            Arc::clone(&seg.text),
            Arc::<[TextSpan]>::from([TextSpan {
                len: seg.text.len(),
                paint: TextPaintStyle {
                    fg: seg.highlight.and_then(|h| row_theme.syntax_color(h)),
                    ..Default::default()
                },
                ..Default::default()
            }]),
        );
    }

    let code_len = line
        .segments
        .iter()
        .map(|seg| seg.text.len())
        .sum::<usize>();
    let prefix_len = if prepared.show_line_numbers {
        prepared
            .line_number_width
            .max(decimal_digits(line_i.saturating_add(1)))
            + WINDOWED_LINE_NUMBER_SEPARATOR.len()
    } else {
        0
    };
    let mut text = String::with_capacity(prefix_len + code_len);
    let mut spans: Vec<TextSpan> = Vec::with_capacity(
        line.segments
            .iter()
            .filter(|seg| !seg.text.is_empty())
            .count()
            + usize::from(prepared.show_line_numbers),
    );

    if prepared.show_line_numbers {
        write_windowed_line_number_prefix(&mut text, prepared, line_i);
        spans.push(TextSpan {
            len: prefix_len,
            paint: TextPaintStyle {
                fg: Some(row_theme.muted_fg),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    for seg in &line.segments {
        if seg.text.is_empty() {
            continue;
        }
        let color = seg.highlight.and_then(|h| row_theme.syntax_color(h));
        text.push_str(seg.text.as_ref());
        spans.push(TextSpan {
            len: seg.text.len(),
            paint: TextPaintStyle {
                fg: color,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    AttributedText::new(Arc::<str>::from(text), spans)
}

fn windowed_line_chunk_count(line_count: usize) -> usize {
    line_count.div_ceil(WINDOWED_LINE_CHUNK_SIZE)
}

fn windowed_line_chunk_start(chunk_i: usize) -> usize {
    chunk_i.saturating_mul(WINDOWED_LINE_CHUNK_SIZE)
}

fn windowed_line_chunk_len(line_count: usize, chunk_i: usize) -> usize {
    let start = windowed_line_chunk_start(chunk_i);
    line_count
        .saturating_sub(start)
        .min(WINDOWED_LINE_CHUNK_SIZE)
}

fn windowed_line_chunk_height(row_h: Px, line_count: usize, chunk_i: usize) -> Px {
    Px(row_h.0 * windowed_line_chunk_len(line_count, chunk_i).max(1) as f32)
}

fn windowed_line_chunk_overscan(line_overscan: usize) -> usize {
    line_overscan
        .max(1)
        .div_ceil(WINDOWED_LINE_CHUNK_SIZE)
        .max(1)
}

fn build_code_block_line_chunk_rich(
    row_theme: &CodeBlockLineRowTheme,
    prepared: &crate::prepare::PreparedCodeBlock,
    chunk_i: usize,
) -> AttributedText {
    let start = windowed_line_chunk_start(chunk_i);
    let len = windowed_line_chunk_len(prepared.lines.len(), chunk_i);
    if len <= 1 {
        return build_code_block_line_rich(row_theme, prepared, start);
    }

    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    for line_i in start..start + len {
        if line_i > start {
            text.push('\n');
            spans.push(TextSpan {
                len: 1,
                ..Default::default()
            });
        }

        let Some(line) = prepared.lines.get(line_i) else {
            continue;
        };

        if prepared.show_line_numbers {
            let prefix_start = text.len();
            write_windowed_line_number_prefix(&mut text, prepared, line_i);
            spans.push(TextSpan {
                len: text.len().saturating_sub(prefix_start),
                paint: TextPaintStyle {
                    fg: Some(row_theme.muted_fg),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        for seg in &line.segments {
            if seg.text.is_empty() {
                continue;
            }
            let color = seg.highlight.and_then(|h| row_theme.syntax_color(h));
            text.push_str(seg.text.as_ref());
            spans.push(TextSpan {
                len: seg.text.len(),
                paint: TextPaintStyle {
                    fg: color,
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }

    AttributedText::new(Arc::<str>::from(text), spans)
}

fn estimate_monospace_content_width_px(
    prepared: &crate::prepare::PreparedCodeBlock,
    row_theme: &CodeBlockLineRowTheme,
    scrollbar_x_right_inset: Px,
) -> Px {
    // `ScrollProps::known_content_size` is only used as scroll extent metadata. A conservative
    // monospace estimate avoids an unbounded text measurement of the longest line on mount.
    let char_advance = Px((row_theme.mono_size.0 * 0.7).max(1.0));
    let code_width = Px(char_advance.0 * prepared.max_line_columns.max(1) as f32);
    let gutter_width = if prepared.show_line_numbers {
        let number_and_separator_columns =
            prepared.line_number_width.max(1) + WINDOWED_LINE_NUMBER_SEPARATOR.len();
        Px(char_advance.0 * number_and_separator_columns as f32)
    } else {
        Px(0.0)
    };

    Px(code_width.0 + gutter_width.0 + scrollbar_x_right_inset.0)
}

#[derive(Default)]
struct CodeBlockWindowedChunkRichCache {
    theme_revision: u64,
    prepared_revision: u64,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
    tick: u64,
    max_entries: usize,
    entries: HashMap<usize, (AttributedText, u64)>,
    queue: VecDeque<(usize, u64)>,
}

impl CodeBlockWindowedChunkRichCache {
    #[allow(clippy::too_many_arguments)]
    fn resolve(
        &mut self,
        theme_revision: u64,
        prepared_revision: u64,
        disable_ligatures: bool,
        disable_contextual_alternates: bool,
        row_theme: &CodeBlockLineRowTheme,
        prepared: &crate::prepare::PreparedCodeBlock,
        chunk_i: usize,
        max_entries: usize,
    ) -> AttributedText {
        let max_entries = max_entries.max(1);
        if self.theme_revision != theme_revision
            || self.prepared_revision != prepared_revision
            || self.disable_ligatures != disable_ligatures
            || self.disable_contextual_alternates != disable_contextual_alternates
            || self.max_entries != max_entries
        {
            self.theme_revision = theme_revision;
            self.prepared_revision = prepared_revision;
            self.disable_ligatures = disable_ligatures;
            self.disable_contextual_alternates = disable_contextual_alternates;
            self.tick = 0;
            self.max_entries = max_entries;
            self.entries.clear();
            self.queue.clear();
        }

        self.tick = self.tick.saturating_add(1);
        let tick = self.tick;

        if let Some((rich, last_used)) = self.entries.get_mut(&chunk_i) {
            *last_used = tick;
            self.queue.push_back((chunk_i, tick));
            return rich.clone();
        }

        let rich = build_code_block_line_chunk_rich(row_theme, prepared, chunk_i);
        self.entries.insert(chunk_i, (rich.clone(), tick));
        self.queue.push_back((chunk_i, tick));

        while self.entries.len() > max_entries {
            let Some((victim, victim_tick)) = self.queue.pop_front() else {
                break;
            };
            let Some((_, last_used)) = self.entries.get(&victim) else {
                continue;
            };
            if *last_used == victim_tick {
                self.entries.remove(&victim);
            }
        }

        rich
    }
}

#[derive(Default)]
struct CodeBlockLineRowThemeCache {
    theme_revision: u64,
    prepared_revision: u64,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
    row_theme: Option<Arc<CodeBlockLineRowTheme>>,
}

impl CodeBlockLineRowThemeCache {
    fn resolve<T: ThemeTokenRead + ?Sized>(
        &mut self,
        theme_revision: u64,
        prepared_revision: u64,
        disable_ligatures: bool,
        disable_contextual_alternates: bool,
        theme: &T,
        prepared: &crate::prepare::PreparedCodeBlock,
    ) -> Arc<CodeBlockLineRowTheme> {
        let needs_rebuild = self.row_theme.is_none()
            || self.theme_revision != theme_revision
            || self.prepared_revision != prepared_revision
            || self.disable_ligatures != disable_ligatures
            || self.disable_contextual_alternates != disable_contextual_alternates;

        if needs_rebuild {
            self.theme_revision = theme_revision;
            self.prepared_revision = prepared_revision;
            self.disable_ligatures = disable_ligatures;
            self.disable_contextual_alternates = disable_contextual_alternates;
            self.row_theme = Some(Arc::new(CodeBlockLineRowTheme::new(
                theme,
                prepared,
                disable_ligatures,
                disable_contextual_alternates,
            )));
        }

        self.row_theme
            .as_ref()
            .expect("row theme cache should always hold a value after resolve")
            .clone()
    }
}

#[derive(Debug, Clone)]
struct CodeBlockLineRowTheme {
    mono_size: Px,
    mono_line_height: Px,
    text_style: TextStyle,
    fg: fret_core::Color,
    muted_fg: fret_core::Color,
    syntax_colors: HashMap<&'static str, Option<fret_core::Color>>,
}

impl CodeBlockLineRowTheme {
    fn new<T: ThemeTokenRead + ?Sized>(
        theme: &T,
        prepared: &crate::prepare::PreparedCodeBlock,
        disable_ligatures: bool,
        disable_contextual_alternates: bool,
    ) -> Self {
        let mono_size = theme.metric_token("metric.font.mono_size");
        let mono_line_height = theme.metric_token("metric.font.mono_line_height");
        let syntax_colors = prepared
            .syntax_highlights
            .iter()
            .copied()
            .map(|highlight| (highlight, syntax_color(theme, highlight)))
            .collect::<HashMap<_, _>>();
        let code_shaping =
            code_shaping_for_code_block_flags(disable_ligatures, disable_contextual_alternates);
        let mut text_style = typography::as_control_text(TextStyle {
            font: FontId::monospace(),
            size: mono_size,
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(mono_line_height),
            letter_spacing_em: None,
            ..Default::default()
        });
        text_style.features = code_shaping.features.clone();
        text_style.axes = code_shaping.axes.clone();

        Self {
            mono_size,
            mono_line_height,
            text_style,
            fg: theme.color_token("foreground"),
            muted_fg: theme.color_token("muted-foreground"),
            syntax_colors,
        }
    }

    fn syntax_color(&self, highlight: &'static str) -> Option<fret_core::Color> {
        self.syntax_colors.get(highlight).copied().flatten()
    }
}

fn render_code_block_line_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    row_theme: &CodeBlockLineRowTheme,
    rich: AttributedText,
    line_count: usize,
) -> AnyElement {
    let code = cx.styled_text_props(StyledTextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height =
                Length::Px(Px(row_theme.mono_line_height.0 * line_count.max(1) as f32));
            layout
        },
        rich,
        style: Some(row_theme.text_style.clone()),
        color: Some(row_theme.fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    });

    code
}

#[allow(clippy::too_many_arguments)]
#[track_caller]
fn render_code_block_windowed_lines<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    overscan: usize,
    scrollbar_x_enabled: bool,
    scrollbar_x_visible: bool,
    scrollbar_x_right_inset: Px,
    scrollbar_y_enabled: bool,
    scrollbar_y_visible: bool,
    max_height: Option<Px>,
    disable_ligatures: bool,
    disable_contextual_alternates: bool,
) -> AnyElement {
    let Some(max_height) = max_height else {
        return cx.text("windowed code block requires max_height");
    };

    let row_h = theme.metric_token("metric.font.mono_line_height");
    let theme_revision = theme.revision();
    let prepared_revision = prepared.revision;

    let line_rich_cache = cx.slot_state(
        || Rc::new(RefCell::new(CodeBlockWindowedChunkRichCache::default())),
        |h| h.clone(),
    );

    let scroll_y_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());
    let line_len = prepared.lines.len();
    let len = windowed_line_chunk_count(line_len);
    let chunk_overscan = windowed_line_chunk_overscan(overscan);
    let mut list_options = VirtualListOptions::known(
        Px(row_h.0 * WINDOWED_LINE_CHUNK_SIZE as f32),
        chunk_overscan,
        move |chunk_i| windowed_line_chunk_height(row_h, line_len, chunk_i),
    );
    list_options.items_revision = prepared.revision;
    list_options.key_cache = VirtualListKeyCacheMode::VisibleOnly;
    list_options.keep_alive = chunk_overscan.saturating_mul(8).max(32);

    let prepared_for_rows = prepared.clone();
    let row_theme = cx.slot_state(CodeBlockLineRowThemeCache::default, |cache| {
        cache.resolve(
            theme_revision,
            prepared_revision,
            disable_ligatures,
            disable_contextual_alternates,
            theme,
            prepared.as_ref(),
        )
    });
    let known_content_size = fret_core::Size::new(
        estimate_monospace_content_width_px(
            prepared.as_ref(),
            row_theme.as_ref(),
            scrollbar_x_right_inset,
        ),
        Px(row_h.0 * line_len.max(1) as f32),
    );
    let row_theme_for_rows = Arc::clone(&row_theme);
    let line_rich_cache_for_rows = line_rich_cache.clone();
    let max_cache_entries = (chunk_overscan.max(1)).saturating_mul(16).max(128);

    let list_layout = {
        let mut layout = LayoutStyle::default();
        // Ensure wheel events over the "empty" horizontal gutter still target the VirtualList.
        //
        // When the list is `Auto` width inside the X-scroll viewport, short lines can shrink the
        // list's hit-test bounds. This causes wheel scrolling in the right-side gutter to hit the
        // ancestor X-scroll container instead of the VirtualList, preventing vertical scrolling.
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        layout.overflow = Overflow::Clip;
        layout
    };

    let list = cx.virtual_list_keyed_retained_with_layout_fn(
        list_layout,
        len,
        list_options,
        &scroll_y_handle,
        |i| windowed_line_chunk_start(i) as u64,
        move |cx, chunk_i| {
            let line_count = windowed_line_chunk_len(prepared_for_rows.lines.len(), chunk_i);
            let rich = line_rich_cache_for_rows.borrow_mut().resolve(
                theme_revision,
                prepared_revision,
                disable_ligatures,
                disable_contextual_alternates,
                row_theme_for_rows.as_ref(),
                prepared_for_rows.as_ref(),
                chunk_i,
                max_cache_entries,
            );
            render_code_block_line_row(cx, row_theme_for_rows.as_ref(), rich, line_count)
        },
    );

    let list_id = list.id;

    let scroll_x_handle = cx.slot_state(ScrollHandle::default, |h| h.clone());
    let scroll_x_layout = {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        layout.overflow = Overflow::Clip;
        layout
    };

    let scroll_x_el = cx.scroll(
        ScrollProps {
            layout: scroll_x_layout,
            axis: ScrollAxis::X,
            scroll_handle: Some(scroll_x_handle.clone()),
            known_content_size: Some(known_content_size),
            probe_unbounded: true,
            ..Default::default()
        },
        |_cx| vec![list],
    );

    let scrollbar_w = theme.metric_token("metric.scrollbar.width");
    let thumb = theme.color_token("scrollbar.thumb.background");
    let thumb_hover = theme.color_token("scrollbar.thumb.hover.background");

    let scroll_x_id = scroll_x_el.id;

    cx.stack_props(
        StackProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                // Same rationale as the non-windowed path: nested scrollables need a definite
                // viewport height, otherwise `Length::Fill` has no base to resolve against.
                layout.size.height = Length::Px(max_height);
                layout.size.min_height = Some(Length::Px(max_height));
                layout.size.max_height = Some(Length::Px(max_height));
                layout.overflow = Overflow::Clip;
                layout
            },
        },
        move |cx| {
            let mut out = Vec::new();

            let thumb_x = thumb;
            let thumb_hover_x = thumb_hover;
            let thumb_y = thumb;
            let thumb_hover_y = thumb_hover;

            let scroll_x_and_bar = cx.stack_props(
                StackProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout.overflow = Overflow::Clip;
                        layout
                    },
                },
                move |cx| {
                    let scrollbar_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: None.into(),
                            right: Some(scrollbar_x_right_inset).into(),
                            bottom: Some(Px(0.0)).into(),
                            left: Some(Px(0.0)).into(),
                        },
                        size: SizeStyle {
                            height: Length::Px(scrollbar_w),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let mut out = vec![scroll_x_el];

                    if scrollbar_x_enabled {
                        let scrollbar = cx.scrollbar(ScrollbarProps {
                            layout: fill_layout(),
                            axis: ScrollbarAxis::Horizontal,
                            scroll_target: Some(scroll_x_id),
                            scroll_handle: scroll_x_handle.clone(),
                            style: ScrollbarStyle {
                                thumb: thumb_x,
                                thumb_hover: thumb_hover_x,
                                ..Default::default()
                            },
                        });
                        out.push(overlay_chrome(
                            cx,
                            scrollbar_layout,
                            scrollbar_x_visible,
                            scrollbar,
                        ));
                    }

                    out
                },
            );

            // Windowed code blocks wrap the VirtualList inside a horizontal `Scroll` for X
            // overflow. That `Scroll` can end up capturing wheel events, preventing the VirtualList
            // from receiving vertical wheel deltas. A WheelRegion drives the VirtualList's shared
            // scroll handle so vertical wheel scrolling always works.
            let scroll_y_base_handle = scroll_y_handle.base_handle().clone();
            let wheel_region = cx.wheel_region(
                WheelRegionProps {
                    layout: fill_layout(),
                    axis: ScrollAxis::Y,
                    scroll_target: Some(list_id),
                    scroll_handle: scroll_y_base_handle,
                },
                |_cx| vec![scroll_x_and_bar],
            );
            out.push(wheel_region);

            if scrollbar_y_enabled {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(if scrollbar_x_enabled {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        })
                        .into(),
                        left: None.into(),
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let scrollbar = cx.scrollbar(ScrollbarProps {
                    layout: fill_layout(),
                    axis: ScrollbarAxis::Vertical,
                    scroll_target: Some(list_id),
                    scroll_handle: scroll_y_handle.base_handle().clone(),
                    style: ScrollbarStyle {
                        thumb: thumb_y,
                        thumb_hover: thumb_hover_y,
                        ..Default::default()
                    },
                });

                out.push(overlay_chrome(
                    cx,
                    scrollbar_layout,
                    scrollbar_y_visible,
                    scrollbar,
                ));
            }

            out
        },
    )
}

fn render_code_block_with_line_numbers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line_numbers: Arc<str>,
    code: AnyElement,
) -> AnyElement {
    let number_style = typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size: theme.metric_token("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    });

    let line_numbers_text = cx.text_props(TextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Auto;
            layout
        },
        text: line_numbers,
        style: Some(number_style),
        color: Some(theme.color_token("muted-foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    });

    let gutter = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Auto;
                layout.size.height = Length::Auto;
                layout
            },
            padding: Edges::all(Px(0.0)).into(),
            background: None,
            shadow: None,
            border: Edges {
                top: Px(0.0),
                right: Px(1.0),
                bottom: Px(0.0),
                left: Px(0.0),
            },
            border_color: Some(theme.color_token("border")),
            corner_radii: fret_core::Corners::all(Px(0.0)),
            ..Default::default()
        },
        |_cx| vec![line_numbers_text],
    );

    ui::h_flex(|_cx| vec![gutter, code])
        .gap(Space::N2)
        .items_stretch()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}

#[track_caller]
fn render_code_block_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    rich: AttributedText,
    wrap: CodeBlockWrap,
    scrollbar_x_enabled: bool,
    scrollbar_x_visible: bool,
    scrollbar_x_right_inset: Px,
    line_count: usize,
) -> AnyElement {
    let text_style = typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size: theme.metric_token("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    });
    let fg = theme.color_token("foreground");

    let text_wrap = text_wrap_for_code_block_wrap(wrap);
    let overflow = TextOverflow::Clip;

    let mut scroll_layout = LayoutStyle::default();
    scroll_layout.size.width = Length::Fill;
    scroll_layout.size.height = match text_wrap {
        TextWrap::None => {
            let line_height = theme.metric_token("metric.font.mono_line_height");
            let lines = line_count.max(1) as f32;
            Length::Px(Px(line_height.0 * lines))
        }
        TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme => {
            Length::Auto
        }
    };
    scroll_layout.overflow = Overflow::Clip;

    let text_layout = {
        let mut layout = LayoutStyle::default();
        layout.size.width = match text_wrap {
            TextWrap::None => Length::Auto,
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme => {
                Length::Fill
            }
        };
        layout
    };

    let handle = cx.slot_state(ScrollHandle::default, |h| h.clone());
    let scroll = cx.scroll(
        ScrollProps {
            layout: scroll_layout,
            axis: ScrollAxis::X,
            scroll_handle: Some(handle.clone()),
            probe_unbounded: matches!(text_wrap, TextWrap::None),
            ..Default::default()
        },
        |cx| {
            vec![cx.selectable_text_props(SelectableTextProps {
                layout: text_layout,
                rich,
                style: Some(text_style),
                color: Some(fg),
                wrap: text_wrap,
                overflow,
                align: fret_core::TextAlign::Start,
                ink_overflow: TextInkOverflow::None,
                interactive_spans: std::sync::Arc::from([]),
            })]
        },
    );

    let scrollbar_w = theme.metric_token("metric.scrollbar.width");
    let thumb = theme.color_token("scrollbar.thumb.background");
    let thumb_hover = theme.color_token("scrollbar.thumb.hover.background");

    let scroll_id = scroll.id;
    cx.stack_props(
        StackProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Auto;
                layout.overflow = Overflow::Clip;
                layout
            },
        },
        move |cx| {
            let scrollbar_layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    top: None.into(),
                    right: Some(scrollbar_x_right_inset).into(),
                    bottom: Some(Px(0.0)).into(),
                    left: Some(Px(0.0)).into(),
                },
                size: SizeStyle {
                    height: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut out = vec![scroll];
            if scrollbar_x_enabled {
                let scrollbar = cx.scrollbar(ScrollbarProps {
                    layout: fill_layout(),
                    axis: ScrollbarAxis::Horizontal,
                    scroll_target: Some(scroll_id),
                    scroll_handle: handle,
                    style: ScrollbarStyle {
                        thumb,
                        thumb_hover,
                        ..Default::default()
                    },
                });
                out.push(overlay_chrome(
                    cx,
                    scrollbar_layout,
                    scrollbar_x_visible,
                    scrollbar,
                ));
            }

            out
        },
    )
}

fn text_wrap_for_code_block_wrap(wrap: CodeBlockWrap) -> TextWrap {
    match wrap {
        CodeBlockWrap::ScrollX => TextWrap::None,
        CodeBlockWrap::Word => TextWrap::Word,
        CodeBlockWrap::Grapheme => TextWrap::Grapheme,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE_BLOCK_RS: &str = include_str!("code_block.rs");

    #[test]
    fn code_block_wrap_maps_to_text_wrap() {
        assert_eq!(
            text_wrap_for_code_block_wrap(CodeBlockWrap::ScrollX),
            TextWrap::None
        );
        assert_eq!(
            text_wrap_for_code_block_wrap(CodeBlockWrap::Word),
            TextWrap::Word
        );
        assert_eq!(
            text_wrap_for_code_block_wrap(CodeBlockWrap::Grapheme),
            TextWrap::Grapheme
        );
    }

    #[test]
    fn windowed_line_numbers_are_folded_into_single_rich_line() {
        let mut prepared = crate::prepare::PreparedCodeBlock {
            show_line_numbers: true,
            line_number_width: 3,
            ..Default::default()
        };
        prepared.lines.push(crate::prepare::PreparedLine {
            segments: vec![crate::prepare::PreparedSegment {
                text: Arc::<str>::from("let value = 1;"),
                highlight: None,
            }],
        });

        let row_theme = CodeBlockLineRowTheme {
            mono_size: Px(10.0),
            mono_line_height: Px(14.0),
            text_style: TextStyle::default(),
            fg: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            muted_fg: fret_core::Color::from_srgb_hex_rgb(0x808080),
            syntax_colors: HashMap::new(),
        };

        let rich = build_code_block_line_rich(&row_theme, &prepared, 0);

        assert_eq!(rich.text.as_ref(), "  1  let value = 1;");
        assert_eq!(rich.spans.len(), 2);
        assert_eq!(rich.spans[0].len, 3 + WINDOWED_LINE_NUMBER_SEPARATOR.len());
        assert_eq!("let value = 1;".len(), rich.spans[1].len);
        assert_eq!(rich.spans[0].paint.fg, Some(row_theme.muted_fg));
        assert_eq!(rich.spans[1].paint.fg, None);
    }

    #[test]
    fn windowed_plain_single_segment_rows_reuse_prepared_text() {
        let text = Arc::<str>::from("let value = 1;");
        let mut prepared = crate::prepare::PreparedCodeBlock {
            show_line_numbers: false,
            ..Default::default()
        };
        prepared.lines.push(crate::prepare::PreparedLine {
            segments: vec![crate::prepare::PreparedSegment {
                text: Arc::clone(&text),
                highlight: None,
            }],
        });

        let row_theme = CodeBlockLineRowTheme {
            mono_size: Px(10.0),
            mono_line_height: Px(14.0),
            text_style: TextStyle::default(),
            fg: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            muted_fg: fret_core::Color::from_srgb_hex_rgb(0x808080),
            syntax_colors: HashMap::new(),
        };

        let rich = build_code_block_line_rich(&row_theme, &prepared, 0);

        assert!(Arc::ptr_eq(&rich.text, &text));
        assert_eq!(rich.text.as_ref(), "let value = 1;");
        assert_eq!(rich.spans.len(), 1);
        assert_eq!("let value = 1;".len(), rich.spans[0].len);
        assert_eq!(
            rich.spans[0].shaping,
            TextShapingStyle::default(),
            "shared row shaping should live on the base TextStyle, not every span"
        );
    }

    #[test]
    fn windowed_line_chunks_merge_contiguous_lines_into_one_rich_text() {
        let mut prepared = crate::prepare::PreparedCodeBlock {
            show_line_numbers: true,
            line_number_width: 2,
            ..Default::default()
        };
        for text in ["fn main() {", "    let value = 1;", "}"] {
            prepared.lines.push(crate::prepare::PreparedLine {
                segments: vec![crate::prepare::PreparedSegment {
                    text: Arc::<str>::from(text),
                    highlight: None,
                }],
            });
        }

        let row_theme = CodeBlockLineRowTheme {
            mono_size: Px(10.0),
            mono_line_height: Px(14.0),
            text_style: TextStyle::default(),
            fg: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            muted_fg: fret_core::Color::from_srgb_hex_rgb(0x808080),
            syntax_colors: HashMap::new(),
        };

        let rich = build_code_block_line_chunk_rich(&row_theme, &prepared, 0);

        assert_eq!(
            rich.text.as_ref(),
            " 1  fn main() {\n 2      let value = 1;\n 3  }"
        );
        assert_eq!(
            rich.text.matches('\n').count(),
            2,
            "a 3-line chunk should keep newline spans inside one StyledText blob"
        );
        assert_eq!(
            rich.spans
                .iter()
                .filter(|span| span.paint.fg == Some(row_theme.muted_fg))
                .count(),
            3,
            "each line number prefix should keep muted paint"
        );
    }

    #[test]
    fn windowed_line_chunks_keep_precise_count_overscan_and_tail_height() {
        assert_eq!(WINDOWED_LINE_CHUNK_SIZE, 4);
        assert_eq!(windowed_line_chunk_count(0), 0);
        assert_eq!(windowed_line_chunk_count(1), 1);
        assert_eq!(windowed_line_chunk_count(4), 1);
        assert_eq!(windowed_line_chunk_count(5), 2);
        assert_eq!(windowed_line_chunk_overscan(1), 1);
        assert_eq!(windowed_line_chunk_overscan(4), 1);
        assert_eq!(windowed_line_chunk_overscan(5), 2);

        let row_h = Px(14.0);
        assert_eq!(windowed_line_chunk_height(row_h, 10, 0), Px(56.0));
        assert_eq!(windowed_line_chunk_height(row_h, 10, 1), Px(56.0));
        assert_eq!(
            windowed_line_chunk_height(row_h, 10, 2),
            Px(28.0),
            "tail chunk height should match the remaining real line count"
        );
    }

    #[test]
    fn windowed_row_theme_cache_reuses_same_revision() {
        use fret_ui::ThemeSnapshot;
        use fret_ui::theme::{ThemeColors, ThemeMetrics};

        let colors = ThemeColors {
            surface_background: fret_core::Color::from_srgb_hex_rgb(0x24272e),
            panel_background: fret_core::Color::from_srgb_hex_rgb(0x2b3038),
            panel_border: fret_core::Color::from_srgb_hex_rgb(0x3a424d),
            text_primary: fret_core::Color::from_srgb_hex_rgb(0xd7dee9),
            text_muted: fret_core::Color::from_srgb_hex_rgb(0xaab3c2),
            text_disabled: fret_core::Color::from_srgb_hex_rgb(0x7d8798),
            accent: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            selection_background: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            selection_inactive_background: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            selection_window_inactive_background: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            hover_background: fret_core::Color::from_srgb_hex_rgb(0x363c46),
            focus_ring: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            menu_background: fret_core::Color::from_srgb_hex_rgb(0x2b3038),
            menu_border: fret_core::Color::from_srgb_hex_rgb(0x3a424d),
            menu_item_hover: fret_core::Color::from_srgb_hex_rgb(0x363c46),
            menu_item_selected: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            list_background: fret_core::Color::from_srgb_hex_rgb(0x2b3038),
            list_border: fret_core::Color::from_srgb_hex_rgb(0x3a424d),
            list_row_hover: fret_core::Color::from_srgb_hex_rgb(0x363c46),
            list_row_selected: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            scrollbar_track: fret_core::Color::from_srgb_hex_rgb(0x1c1f25),
            scrollbar_thumb: fret_core::Color::from_srgb_hex_rgb(0x4c5666),
            scrollbar_thumb_hover: fret_core::Color::from_srgb_hex_rgb(0x5a687d),
            viewport_selection_fill: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            viewport_selection_stroke: fret_core::Color::from_srgb_hex_rgb(0x3d8bff),
            viewport_marker: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            viewport_drag_line_pan: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            viewport_drag_line_orbit: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            viewport_gizmo_x: fret_core::Color::from_srgb_hex_rgb(0xff0000),
            viewport_gizmo_y: fret_core::Color::from_srgb_hex_rgb(0x00ff00),
            viewport_gizmo_handle_background: fret_core::Color::from_srgb_hex_rgb(0x000000),
            viewport_gizmo_handle_border: fret_core::Color::from_srgb_hex_rgb(0xffffff),
            viewport_rotate_gizmo: fret_core::Color::from_srgb_hex_rgb(0x0000ff),
        };
        let metrics = ThemeMetrics {
            radius_sm: Px(6.0),
            radius_md: Px(8.0),
            radius_lg: Px(10.0),
            padding_sm: Px(8.0),
            padding_md: Px(10.0),
            scrollbar_width: Px(10.0),
            font_size: Px(13.0),
            mono_font_size: Px(13.0),
            font_line_height: Px(16.0),
            mono_font_line_height: Px(16.0),
        };
        let theme = ThemeSnapshot::from_baseline(colors, metrics, 7);
        let mut prepared = crate::prepare::PreparedCodeBlock {
            revision: 11,
            ..Default::default()
        };
        prepared.syntax_highlights = vec!["keyword", "string"];
        let prepared = &prepared;

        let mut cache = CodeBlockLineRowThemeCache::default();
        let first = cache.resolve(7, prepared.revision, true, true, &theme, prepared);
        let same = cache.resolve(7, prepared.revision, true, true, &theme, prepared);
        assert!(
            Arc::ptr_eq(&first, &same),
            "same theme/prepared revision should reuse the cached row theme"
        );
        assert!(
            first
                .text_style
                .features
                .iter()
                .any(|f| f.tag.as_ref() == "liga" && f.value == 0),
            "windowed row theme should move code ligature policy into the shared base TextStyle"
        );
        assert!(
            first
                .text_style
                .features
                .iter()
                .any(|f| f.tag.as_ref() == "calt" && f.value == 0),
            "windowed row theme should move code contextual alternate policy into the shared base TextStyle"
        );

        let rebuilt = cache.resolve(8, prepared.revision, true, true, &theme, prepared);
        assert!(
            !Arc::ptr_eq(&first, &rebuilt),
            "theme revision changes should rebuild the cached row theme"
        );
    }

    #[test]
    fn code_block_hover_tracking_is_only_required_for_hover_chrome() {
        let mut options = CodeBlockUiOptions::default();
        assert!(
            !code_block_needs_hover_tracking(options),
            "plain code blocks should not pay for a hover tracking root"
        );

        options.show_scrollbar_y = true;
        options.max_height = Some(Px(100.0));
        assert!(
            code_block_needs_hover_tracking(options),
            "hover-only Y scrollbars need hover tracking"
        );
        options.scrollbar_y_on_hover = false;
        assert!(
            !code_block_needs_hover_tracking(options),
            "always-visible Y scrollbars should not require hover tracking"
        );

        options = CodeBlockUiOptions::default();
        options.show_scrollbar_x = true;
        assert!(
            code_block_needs_hover_tracking(options),
            "hover-only X scrollbars need hover tracking"
        );
        options.scrollbar_x_on_hover = false;
        assert!(
            !code_block_needs_hover_tracking(options),
            "always-visible X scrollbars should not require hover tracking"
        );

        options = CodeBlockUiOptions::default();
        options.show_copy_button = true;
        assert!(
            code_block_needs_hover_tracking(options),
            "copy buttons hidden until hover need hover tracking"
        );
        options.copy_button_on_hover = false;
        assert!(
            !code_block_needs_hover_tracking(options),
            "always-visible copy buttons should not require hover tracking"
        );
    }

    #[test]
    fn code_block_public_surface_defaults_to_non_windowed_host_lane() {
        for marker in [
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
            "pub fn into_element_non_windowed<H: UiHost>(",
            "pub fn code_block<H: UiHost>(",
            "pub fn code_block_with<H: UiHost>(",
            "pub fn code_block_with_non_windowed<H: UiHost>(",
            "pub fn code_block_with_header_slots<H: UiHost>(",
            "pub fn code_block_with_header_slots_non_windowed<H: UiHost>(",
        ] {
            assert!(
                CODE_BLOCK_RS.contains(marker),
                "code_block.rs should keep non-windowed public surface marker `{marker}`"
            );
        }
    }

    #[test]
    fn code_block_builder_can_hide_header_language_without_dropping_prepare_language() {
        for marker in [
            "show_language_in_header: bool,",
            "show_language_in_header: true,",
            "pub fn show_language_in_header(mut self, show: bool) -> Self {",
            "CodeBlockHeaderSlots::default().show_language(self.show_language_in_header),",
        ] {
            assert!(
                CODE_BLOCK_RS.contains(marker),
                "CodeBlock builder should keep header-language display independent from prepare language marker `{marker}`"
            );
        }
    }

    #[test]
    fn code_block_windowed_lane_keeps_explicit_static_host_boundary() {
        for marker in [
            "pub struct CodeBlockWindowedOptions {",
            "pub enum CodeBlockWindowedHighlightMode {",
            "pub fn highlight_mode(mut self, mode: CodeBlockWindowedHighlightMode) -> Self {",
            "pub fn windowed(mut self, options: CodeBlockWindowedOptions) -> Self {",
            "pub fn into_element_windowed<H: UiHost + 'static>(",
            "pub fn code_block_windowed<H: UiHost + 'static>(",
            "pub fn code_block_with_windowed<H: UiHost + 'static>(",
            "pub fn code_block_with_header_slots_windowed<H: UiHost + 'static>(",
            "fn render_code_block_windowed_lines<H: UiHost + 'static>(",
            "CodeBlock::windowed(...) requires CodeBlock::into_element_windowed(...)",
        ] {
            assert!(
                CODE_BLOCK_RS.contains(marker),
                "code_block.rs should keep explicit retained-lane marker `{marker}`"
            );
        }
    }

    #[test]
    fn code_block_windowed_chunks_keep_known_line_box_text_layout() {
        let render_row_section = CODE_BLOCK_RS
            .split("fn render_code_block_line_row")
            .nth(1)
            .and_then(|section| section.split("#[allow(clippy::too_many_arguments)]").next())
            .expect("render_code_block_line_row section should exist");
        for marker in [
            "layout.size.width = Length::Fill;",
            "row_theme.mono_line_height.0 * line_count.max(1) as f32",
        ] {
            assert!(
                render_row_section.contains(marker),
                "windowed code chunks should keep a fixed line-count-based StyledText layout marker `{marker}`"
            );
        }

        let windowed_section = CODE_BLOCK_RS
            .split("fn render_code_block_windowed_lines")
            .nth(1)
            .and_then(|section| {
                section
                    .split("let prepared_for_rows = prepared.clone();")
                    .next()
            })
            .expect("render_code_block_windowed_lines options section should exist");
        assert!(
            windowed_section.contains("Px(row_h.0 * WINDOWED_LINE_CHUNK_SIZE as f32),"),
            "windowed code chunks should use fixed-size chunk estimates"
        );
        assert!(
            windowed_section.contains("windowed_line_chunk_height(row_h, line_len, chunk_i)"),
            "windowed code chunks should preserve precise tail chunk height"
        );
    }

    #[test]
    fn code_block_windowed_rows_reuse_theme_text_style() {
        let row_theme_section = CODE_BLOCK_RS
            .split("struct CodeBlockLineRowTheme {")
            .nth(1)
            .and_then(|section| section.split("impl CodeBlockLineRowTheme").next())
            .expect("row theme section should exist");
        assert!(
            row_theme_section.contains("text_style: TextStyle,"),
            "windowed row theme should own the shared StyledText style"
        );
        assert!(
            !row_theme_section.contains("code_shaping: TextShapingStyle,"),
            "windowed rows should carry shared code shaping on TextStyle rather than every span"
        );

        let render_row_section = CODE_BLOCK_RS
            .split("fn render_code_block_line_row")
            .nth(1)
            .and_then(|section| section.split("#[allow(clippy::too_many_arguments)]").next())
            .expect("render_code_block_line_row section should exist");
        assert!(
            render_row_section.contains("style: Some(row_theme.text_style.clone()),"),
            "windowed rows should reuse the style cached on CodeBlockLineRowTheme"
        );
        assert!(
            !render_row_section.contains("typography::as_control_text(TextStyle {"),
            "windowed rows should not rebuild the same TextStyle for every mounted line"
        );

        let chunk_cache_section = CODE_BLOCK_RS
            .split("impl CodeBlockWindowedChunkRichCache")
            .nth(1)
            .and_then(|section| section.split("#[derive(Debug, Clone)]").next())
            .expect("windowed chunk rich cache section should exist");
        assert!(
            !chunk_cache_section.contains("&row_theme.code_shaping"),
            "windowed chunk rich cache should not copy shared code shaping into every span"
        );
        assert!(
            !chunk_cache_section.contains("code_shaping_for_code_block_flags("),
            "windowed chunk rich cache should not rebuild the same shaping style for every missed chunk"
        );
    }

    #[test]
    fn code_block_windowed_list_keeps_visible_keys_with_limited_chunk_keep_alive() {
        let windowed_section = CODE_BLOCK_RS
            .split("fn render_code_block_windowed_lines")
            .nth(1)
            .and_then(|section| {
                section
                    .split("let prepared_for_rows = prepared.clone();")
                    .next()
            })
            .expect("render_code_block_windowed_lines options section should exist");
        assert!(
            windowed_section
                .contains("list_options.key_cache = VirtualListKeyCacheMode::VisibleOnly;"),
            "windowed code view should keep visible-only key caching instead of rebuilding all chunk keys"
        );
        assert!(
            windowed_section.contains("let len = windowed_line_chunk_count(line_len);"),
            "windowed code view should virtualize fixed-size line chunks instead of one text blob per line"
        );
        assert!(
            windowed_section.contains("VirtualListOptions::known("),
            "windowed chunks should use known heights so the tail chunk still preserves scroll extent"
        );
        assert!(
            windowed_section
                .contains("list_options.keep_alive = chunk_overscan.saturating_mul(8).max(32);"),
            "windowed code view should retain a bounded off-window chunk pool for scroll reuse"
        );
    }

    #[test]
    fn code_block_headerless_content_path_skips_the_empty_vertical_flex_shell() {
        let normalized = CODE_BLOCK_RS.split_whitespace().collect::<String>();
        for marker in [
            "letcontent=if!header_visible{render_body(cx,theme,prepared.clone(),body_options)}else{",
            "render_body(cx,theme,prepared.clone(),body_options)",
            "ui::v_flex(|cx|{",
        ] {
            assert!(
                normalized.contains(marker),
                "headerless code block content path should keep the direct-body shortcut marker `{marker}`"
            );
        }
    }

    #[test]
    fn code_block_common_ui_options_do_not_leak_windowed_knobs() {
        let source_without_tests = CODE_BLOCK_RS
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(CODE_BLOCK_RS);
        let ui_options_section = source_without_tests
            .split("pub struct CodeBlockUiOptions {")
            .nth(1)
            .and_then(|section| section.split("impl Default for CodeBlockUiOptions").next())
            .unwrap_or("");

        for marker in [
            "pub struct CodeBlockUiOptions {",
            "pub wrap: CodeBlockWrap,",
            "pub max_height: Option<Px>,",
            "pub show_scrollbar_x: bool,",
            "pub show_scrollbar_y: bool,",
        ] {
            assert!(
                source_without_tests.contains(marker),
                "code_block.rs should keep common UI option marker `{marker}`"
            );
        }
        assert!(
            !ui_options_section.contains("windowed_lines"),
            "code_block.rs should keep `windowed_lines` out of CodeBlockUiOptions"
        );
        assert!(
            !ui_options_section.contains("windowed_lines_overscan"),
            "code_block.rs should keep `windowed_lines_overscan` out of CodeBlockUiOptions"
        );
        assert!(
            !source_without_tests.contains("options.windowed_lines"),
            "code_block.rs should not branch on hidden windowed flags inside the common options lane"
        );
    }
}
