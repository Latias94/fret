use std::sync::Arc;

use fret_core::{
    AttributedText, Edges, FontId, FontWeight, Px, TextOverflow, TextPaintStyle, TextSpan,
    TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps, ScrollbarStyle,
    SelectableTextProps, SizeStyle, StackProps, StyledTextProps, TextProps,
    VirtualListKeyCacheMode, VirtualListOptions,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius, Space,
};

use crate::copy_button::{CopyFeedbackRef, render_copy_button, render_copy_button_overlay};
use crate::prepare::CodeBlockPreparedState;
use crate::syntax::syntax_color;

#[derive(Default)]
struct CodeBlockTextCache {
    theme_revision: u64,
    prepared: Option<Arc<crate::prepare::PreparedCodeBlock>>,
    rich: Option<AttributedText>,
    line_numbers: Option<Arc<str>>,
}

fn build_code_block_rich(
    theme: &Theme,
    prepared: &crate::prepare::PreparedCodeBlock,
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
                shaping: Default::default(),
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

fn resolve_code_block_cached_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: &Arc<crate::prepare::PreparedCodeBlock>,
) -> (AttributedText, Option<Arc<str>>) {
    cx.with_state(CodeBlockTextCache::default, |st| {
        let theme_revision = theme.revision();
        let needs_rebuild = st.rich.is_none()
            || st.theme_revision != theme_revision
            || st
                .prepared
                .as_ref()
                .is_none_or(|p| !Arc::ptr_eq(p, prepared));

        if needs_rebuild {
            st.theme_revision = theme_revision;
            st.prepared = Some(prepared.clone());
            st.rich = Some(build_code_block_rich(theme, prepared.as_ref()));
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
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CodeBlock {
    code: Arc<str>,
    language: Option<Arc<str>>,
    show_line_numbers: bool,
    show_header: bool,
    header_divider: bool,
    header_background: CodeBlockHeaderBackground,
    show_copy_button: bool,
    copy_button_on_hover: bool,
    copy_button_placement: CodeBlockCopyButtonPlacement,
    border: bool,
    wrap: CodeBlockWrap,
    max_height: Option<Px>,
    windowed_lines: bool,
    windowed_lines_overscan: usize,
    show_scrollbar_x: bool,
    scrollbar_x_on_hover: bool,
    show_scrollbar_y: bool,
    scrollbar_y_on_hover: bool,
}

impl CodeBlock {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_line_numbers: false,
            show_header: false,
            header_divider: false,
            header_background: CodeBlockHeaderBackground::None,
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: CodeBlockCopyButtonPlacement::Overlay,
            border: true,
            wrap: CodeBlockWrap::ScrollX,
            max_height: None,
            windowed_lines: false,
            windowed_lines_overscan: 6,
            show_scrollbar_x: false,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: false,
            scrollbar_y_on_hover: true,
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

    pub fn windowed_lines(mut self, windowed: bool) -> Self {
        self.windowed_lines = windowed;
        self
    }

    pub fn windowed_lines_overscan(mut self, overscan: usize) -> Self {
        self.windowed_lines_overscan = overscan.max(1);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        code_block_with(
            cx,
            &self.code,
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
                windowed_lines: self.windowed_lines,
                windowed_lines_overscan: self.windowed_lines_overscan,
                show_scrollbar_x: self.show_scrollbar_x,
                scrollbar_x_on_hover: self.scrollbar_x_on_hover,
                show_scrollbar_y: self.show_scrollbar_y,
                scrollbar_y_on_hover: self.scrollbar_y_on_hover,
            },
        )
    }
}

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
    pub windowed_lines: bool,
    pub windowed_lines_overscan: usize,
    pub show_scrollbar_x: bool,
    pub scrollbar_x_on_hover: bool,
    pub show_scrollbar_y: bool,
    pub scrollbar_y_on_hover: bool,
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
            windowed_lines: false,
            windowed_lines_overscan: 6,
            show_scrollbar_x: false,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: false,
            scrollbar_y_on_hover: true,
        }
    }
}

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

pub fn code_block_with_header_slots<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    options: CodeBlockUiOptions,
    mut header: CodeBlockHeaderSlots,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let bg = theme.color_required("card");
    let border = theme.color_required("border");

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
    let prepared = cx.with_state(CodeBlockPreparedState::default, |st| {
        st.prepare(code, language, show_line_numbers);
        st.prepared.clone()
    });

    let code = Arc::<str>::from(code.to_string());
    let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

    cx.container(props, |cx| {
        vec![cx.hover_region(HoverRegionProps::default(), |cx, hovered| {
            let copied = feedback.is_copied();
            let copy_visible = !options.copy_button_on_hover || hovered || copied;
            let scrollbar_x_visible =
                options.show_scrollbar_x && (!options.scrollbar_x_on_hover || hovered);
            let scrollbar_y_visible = options.show_scrollbar_y
                && options.max_height.is_some()
                && (!options.scrollbar_y_on_hover || hovered);

            let header_visible = options.show_header
                || language.is_some()
                || !header.left.is_empty()
                || !header.right.is_empty();

            if !header_visible {
                header.show_language = false;
            }

            let content = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let mut out = Vec::new();
                    if header_visible {
                        out.push(render_code_block_header(
                            cx,
                            &theme,
                            language,
                            &header,
                            options.header_divider,
                            options.header_background,
                            if options.show_copy_button
                                && options.copy_button_placement
                                    == CodeBlockCopyButtonPlacement::Header
                            {
                                Some(CopyButtonInHeader {
                                    feedback: feedback.clone(),
                                    code: code.clone(),
                                    visible: copy_visible,
                                })
                            } else {
                                None
                            },
                        ));
                    }
                    out.push(render_code_block_body(
                        cx,
                        &theme,
                        prepared.clone(),
                        options.wrap,
                        options.windowed_lines,
                        options.windowed_lines_overscan,
                        scrollbar_x_visible,
                        scrollbar_y_visible,
                        options.max_height,
                    ));
                    out
                },
            );

            let mut out = vec![content];
            if options.show_copy_button
                && options.copy_button_placement == CodeBlockCopyButtonPlacement::Overlay
            {
                let el = render_copy_button_overlay(cx, &theme, feedback.clone(), code.clone());
                out.push(cx.opacity(if copy_visible { 1.0 } else { 0.0 }, |_cx| vec![el]));
            }
            out
        })]
    })
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
    header: &CodeBlockHeaderSlots,
    divider: bool,
    background: CodeBlockHeaderBackground,
    copy: Option<CopyButtonInHeader>,
) -> AnyElement {
    let pad_x = MetricRef::space(Space::N2).resolve(theme);
    let pad_y = MetricRef::space(Space::N1).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::symmetric(pad_x, pad_y);
    match background {
        CodeBlockHeaderBackground::None => {}
        CodeBlockHeaderBackground::Secondary => {
            props.background = Some(theme.color_required("secondary"));
        }
    }
    if divider {
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(theme.color_required("border"));
    }

    cx.container(props, |cx| {
        vec![stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .justify(Justify::Between)
                .items(Items::Center)
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let mut left = Vec::new();
                if header.show_language
                    && let Some(lang) = language
                {
                    left.push(cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::<str>::from(lang.to_string()),
                        style: Some(TextStyle {
                            font: FontId::monospace(),
                            size: theme.metric_required("metric.font.mono_size"),
                            weight: FontWeight::SEMIBOLD,
                            slant: Default::default(),
                            line_height: Some(
                                theme.metric_required("metric.font.mono_line_height"),
                            ),
                            letter_spacing_em: None,
                        }),
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }));
                }
                left.extend(header.left.iter().cloned());

                let mut right = Vec::new();
                right.extend(header.right.iter().cloned());
                if let Some(copy) = copy {
                    let el = render_copy_button(cx, theme, copy.feedback, copy.code);
                    right.push(cx.opacity(if copy.visible { 1.0 } else { 0.0 }, |_cx| vec![el]));
                }

                vec![
                    stack::hstack(cx, stack::HStackProps::default().gap(Space::N1), |_| left),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N1).justify_end(),
                        |_| right,
                    ),
                ]
            },
        )]
    })
}

fn render_code_block_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    wrap: CodeBlockWrap,
    windowed_lines: bool,
    windowed_lines_overscan: usize,
    scrollbar_x_visible: bool,
    scrollbar_y_visible: bool,
    max_height: Option<Px>,
) -> AnyElement {
    let pad = MetricRef::space(Space::N2).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.overflow = Overflow::Clip;
    props.padding = Edges::all(pad);

    cx.container(props, |cx| {
        let wrap = if prepared.show_line_numbers {
            debug_assert!(
                !matches!(wrap, CodeBlockWrap::Word),
                "word wrap with line numbers is not supported yet"
            );
            CodeBlockWrap::ScrollX
        } else {
            wrap
        };

        let scrollbar_w = theme.metric_required("metric.scrollbar.width");
        let reserved_right_for_x_scrollbar = if scrollbar_y_visible {
            scrollbar_w
        } else {
            Px(0.0)
        };

        let content =
            if windowed_lines && max_height.is_some() && matches!(wrap, CodeBlockWrap::ScrollX) {
                render_code_block_windowed_lines(
                    cx,
                    theme,
                    prepared.clone(),
                    windowed_lines_overscan,
                    scrollbar_x_visible,
                    reserved_right_for_x_scrollbar,
                    scrollbar_y_visible,
                    max_height,
                )
            } else {
                let (rich, line_numbers) = resolve_code_block_cached_text(cx, theme, &prepared);
                let line_count = prepared.lines.len();

                let content = if !prepared.show_line_numbers {
                    render_code_block_text(
                        cx,
                        theme,
                        rich,
                        wrap,
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
                        scrollbar_x_visible,
                        reserved_right_for_x_scrollbar,
                        line_count,
                    );
                    let line_numbers = line_numbers.unwrap_or_else(|| Arc::<str>::from(""));
                    render_code_block_with_line_numbers(cx, theme, line_numbers, code)
                };

                if let Some(max_height) = max_height {
                    let thumb = theme.color_required("scrollbar.thumb.background");
                    let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");
                    let handle = cx.with_state(ScrollHandle::default, |h| h.clone());

                    let outer_layout = {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Auto;
                        layout.size.max_height = Some(max_height);
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

                    if !scrollbar_y_visible {
                        return vec![scroll];
                    }

                    let scroll_id = scroll.id;
                    return vec![cx.stack_props(
                        StackProps {
                            layout: outer_layout,
                        },
                        move |cx| {
                            let scrollbar_layout = LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(if scrollbar_x_visible {
                                        scrollbar_w
                                    } else {
                                        Px(0.0)
                                    }),
                                    left: None,
                                },
                                size: SizeStyle {
                                    width: Length::Px(scrollbar_w),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            vec![
                                scroll,
                                cx.scrollbar(ScrollbarProps {
                                    layout: scrollbar_layout,
                                    axis: ScrollbarAxis::Vertical,
                                    scroll_target: Some(scroll_id),
                                    scroll_handle: handle,
                                    style: ScrollbarStyle {
                                        thumb,
                                        thumb_hover,
                                        ..Default::default()
                                    },
                                }),
                            ]
                        },
                    )];
                } else {
                    return vec![content];
                }
            };

        vec![content]
    })
}

fn build_code_block_line_rich(
    theme: &Theme,
    prepared: &crate::prepare::PreparedCodeBlock,
    line_i: usize,
) -> AttributedText {
    let Some(line) = prepared.lines.get(line_i) else {
        return AttributedText::new(Arc::<str>::from(""), Arc::<[TextSpan]>::from([]));
    };

    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    for seg in &line.segments {
        if seg.text.is_empty() {
            continue;
        }
        let color = seg.highlight.and_then(|h| syntax_color(theme, h));
        text.push_str(seg.text.as_ref());
        spans.push(TextSpan {
            len: seg.text.len(),
            shaping: Default::default(),
            paint: TextPaintStyle {
                fg: color,
                ..Default::default()
            },
        });
    }

    AttributedText::new(Arc::<str>::from(text), spans)
}

fn render_code_block_line_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: &crate::prepare::PreparedCodeBlock,
    line_i: usize,
) -> AnyElement {
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };

    let fg = theme.color_required("foreground");
    let muted_fg = theme.color_required("muted-foreground");
    let border = theme.color_required("border");

    let code = cx.styled_text_props(StyledTextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Auto;
            layout
        },
        rich: build_code_block_line_rich(theme, prepared, line_i),
        style: Some(text_style),
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    });

    if !prepared.show_line_numbers {
        return code;
    }

    let number = prepared
        .line_numbers
        .get(line_i)
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from(""));

    let number_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };

    let number = cx.text_props(TextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Auto;
            layout
        },
        text: number,
        style: Some(number_style),
        color: Some(muted_fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    });

    let gutter = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Auto;
                layout.size.height = Length::Auto;
                layout
            },
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges {
                top: Px(0.0),
                right: Px(1.0),
                bottom: Px(0.0),
                left: Px(0.0),
            },
            border_color: Some(border),
            corner_radii: fret_core::Corners::all(Px(0.0)),
            ..Default::default()
        },
        |_cx| vec![number],
    );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items(Items::Center)
            .layout(LayoutRefinement::default()),
        |_cx| vec![gutter, code],
    )
}

fn render_code_block_windowed_lines<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: Arc<crate::prepare::PreparedCodeBlock>,
    overscan: usize,
    scrollbar_x_visible: bool,
    scrollbar_x_right_inset: Px,
    scrollbar_y_visible: bool,
    max_height: Option<Px>,
) -> AnyElement {
    let Some(max_height) = max_height else {
        return cx.text("windowed_lines requires max_height");
    };

    let row_h = theme.metric_required("metric.font.mono_line_height");

    let scroll_y_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
    let mut list_options = VirtualListOptions::fixed(row_h, overscan.max(1));
    list_options.items_revision = prepared.revision;
    list_options.key_cache = VirtualListKeyCacheMode::VisibleOnly;

    let len = prepared.lines.len();
    let prepared_for_rows = prepared.clone();

    let list_layout = {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Auto;
        layout.size.height = Length::Fill;
        layout.overflow = Overflow::Clip;
        layout
    };

    let list = cx.virtual_list_keyed_with_layout(
        list_layout,
        len,
        list_options,
        &scroll_y_handle,
        |i| i as u64,
        |cx, i| render_code_block_line_row(cx, theme, prepared_for_rows.as_ref(), i),
    );

    let list_id = list.id;

    let scroll_x_handle = cx.with_state(ScrollHandle::default, |h| h.clone());
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
            probe_unbounded: true,
            ..Default::default()
        },
        |_cx| vec![list],
    );

    let scrollbar_w = theme.metric_required("metric.scrollbar.width");
    let thumb = theme.color_required("scrollbar.thumb.background");
    let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

    let scroll_x_id = scroll_x_el.id;

    cx.stack_props(
        StackProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Auto;
                layout.size.max_height = Some(max_height);
                layout.overflow = Overflow::Clip;
                layout
            },
        },
        move |cx| {
            let mut out = Vec::new();

            let thumb_x = thumb.clone();
            let thumb_hover_x = thumb_hover.clone();
            let thumb_y = thumb.clone();
            let thumb_hover_y = thumb_hover.clone();

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
                            top: None,
                            right: Some(scrollbar_x_right_inset),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: SizeStyle {
                            height: Length::Px(scrollbar_w),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll_x_id),
                        scroll_handle: scroll_x_handle.clone(),
                        style: ScrollbarStyle {
                            thumb: thumb_x.clone(),
                            thumb_hover: thumb_hover_x.clone(),
                            ..Default::default()
                        },
                    });

                    vec![
                        scroll_x_el,
                        cx.opacity(if scrollbar_x_visible { 1.0 } else { 0.0 }, move |_cx| {
                            vec![scrollbar]
                        }),
                    ]
                },
            );

            out.push(scroll_x_and_bar);

            if scrollbar_y_visible {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        bottom: Some(if scrollbar_x_visible {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        }),
                        left: None,
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                out.push(cx.scrollbar(ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: ScrollbarAxis::Vertical,
                    scroll_target: Some(list_id),
                    scroll_handle: scroll_y_handle.base_handle().clone(),
                    style: ScrollbarStyle {
                        thumb: thumb_y.clone(),
                        thumb_hover: thumb_hover_y.clone(),
                        ..Default::default()
                    },
                }));
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
    let number_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };

    let line_numbers_text = cx.text_props(TextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Auto;
            layout
        },
        text: line_numbers,
        style: Some(number_style),
        color: Some(theme.color_required("muted-foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    });

    let gutter = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Auto;
                layout.size.height = Length::Auto;
                layout
            },
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges {
                top: Px(0.0),
                right: Px(1.0),
                bottom: Px(0.0),
                left: Px(0.0),
            },
            border_color: Some(theme.color_required("border")),
            corner_radii: fret_core::Corners::all(Px(0.0)),
            ..Default::default()
        },
        |_cx| vec![line_numbers_text],
    );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items_stretch()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![gutter, code],
    )
}

fn render_code_block_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    rich: AttributedText,
    wrap: CodeBlockWrap,
    scrollbar_x_visible: bool,
    scrollbar_x_right_inset: Px,
    line_count: usize,
) -> AnyElement {
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };
    let fg = theme.color_required("foreground");

    let (text_wrap, overflow) = match wrap {
        CodeBlockWrap::ScrollX => (TextWrap::None, TextOverflow::Clip),
        CodeBlockWrap::Word => (TextWrap::Word, TextOverflow::Clip),
    };

    let mut scroll_layout = LayoutStyle::default();
    scroll_layout.size.width = Length::Fill;
    scroll_layout.size.height = match text_wrap {
        TextWrap::None => {
            let line_height = theme.metric_required("metric.font.mono_line_height");
            let lines = line_count.max(1) as f32;
            Length::Px(Px(line_height.0 * lines))
        }
        TextWrap::Word | TextWrap::Grapheme => Length::Auto,
    };
    scroll_layout.overflow = Overflow::Clip;

    let text_layout = {
        let mut layout = LayoutStyle::default();
        layout.size.width = match text_wrap {
            TextWrap::None => Length::Auto,
            TextWrap::Word | TextWrap::Grapheme => Length::Fill,
        };
        layout
    };

    let handle = cx.with_state(ScrollHandle::default, |h| h.clone());
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
            })]
        },
    );

    let scrollbar_w = theme.metric_required("metric.scrollbar.width");
    let thumb = theme.color_required("scrollbar.thumb.background");
    let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

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
                    top: None,
                    right: Some(scrollbar_x_right_inset),
                    bottom: Some(Px(0.0)),
                    left: Some(Px(0.0)),
                },
                size: SizeStyle {
                    height: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            let scrollbar = cx.scrollbar(ScrollbarProps {
                layout: scrollbar_layout,
                axis: ScrollbarAxis::Horizontal,
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    ..Default::default()
                },
            });

            vec![
                scroll,
                cx.opacity(if scrollbar_x_visible { 1.0 } else { 0.0 }, move |_cx| {
                    vec![scrollbar]
                }),
            ]
        },
    )
}
