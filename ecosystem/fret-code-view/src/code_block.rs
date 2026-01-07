use std::sync::Arc;

use fret_core::{
    Edges, FontId, FontWeight, Px, RichText, TextOverflow, TextRun, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, LayoutStyle, Length, Overflow, PositionStyle,
    ScrollAxis, ScrollProps, SelectableTextProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius, Space,
};

use crate::copy_button::{CopyFeedbackRef, render_copy_button, render_copy_button_overlay};
use crate::prepare::CodeBlockPreparedState;
use crate::syntax::syntax_color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeBlockWrap {
    /// Do not wrap; use horizontal scrolling for long lines.
    ScrollX,
    /// Wrap at word boundaries (best-effort, depends on the text system).
    Word,
}

impl Default for CodeBlockWrap {
    fn default() -> Self {
        Self::ScrollX
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeBlockCopyButtonPlacement {
    Overlay,
    Header,
}

impl Default for CodeBlockCopyButtonPlacement {
    fn default() -> Self {
        Self::Overlay
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeBlockHeaderBackground {
    None,
    Secondary,
}

impl Default for CodeBlockHeaderBackground {
    fn default() -> Self {
        Self::None
    }
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
            let show_copy = options.show_copy_button && copy_visible;

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
                    out.push(render_code_block_body(cx, &theme, &prepared, options.wrap));
                    out
                },
            );

            let mut out = vec![content];
            if show_copy && options.copy_button_placement == CodeBlockCopyButtonPlacement::Overlay {
                out.push(render_copy_button_overlay(
                    cx,
                    &theme,
                    feedback.clone(),
                    code.clone(),
                ));
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
                if header.show_language {
                    if let Some(lang) = language {
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
                }
                left.extend(header.left.iter().cloned());

                let mut right = Vec::new();
                right.extend(header.right.iter().cloned());
                if let Some(copy) = copy {
                    let el = render_copy_button(cx, theme, copy.feedback, copy.code);
                    right.push(cx.opacity(if copy.visible { 1.0 } else { 0.0 }, |cx| {
                        vec![cx.interactivity_gate(true, copy.visible, |_cx| vec![el])]
                    }));
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
    prepared: &crate::prepare::PreparedCodeBlock,
    wrap: CodeBlockWrap,
) -> AnyElement {
    let pad = MetricRef::space(Space::N2).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.overflow = Overflow::Clip;
    props.padding = Edges::all(pad);

    cx.container(props, |cx| {
        if !prepared.show_line_numbers {
            return vec![render_code_block_text(cx, theme, prepared, wrap)];
        }

        vec![render_code_block_with_line_numbers(
            cx, theme, prepared, wrap,
        )]
    })
}

fn render_code_block_with_line_numbers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: &crate::prepare::PreparedCodeBlock,
    wrap: CodeBlockWrap,
) -> AnyElement {
    let number_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };

    let numbers = Arc::<str>::from({
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
    });

    let line_numbers_text = cx.text_props(TextProps {
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Auto;
            layout
        },
        text: numbers,
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
                layout.size.height = Length::Fill;
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
        },
        |_cx| vec![line_numbers_text],
    );

    let code = render_code_block_text(cx, theme, prepared, wrap);

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
    prepared: &crate::prepare::PreparedCodeBlock,
    wrap: CodeBlockWrap,
) -> AnyElement {
    let mut text = String::new();
    let mut runs: Vec<TextRun> = Vec::new();
    for (line_i, line) in prepared.lines.iter().enumerate() {
        for seg in &line.segments {
            if seg.text.is_empty() {
                continue;
            }
            let color = seg.highlight.and_then(|h| syntax_color(theme, h));
            text.push_str(seg.text.as_ref());
            runs.push(TextRun {
                len: seg.text.len(),
                color,
                weight: None,
                slant: None,
            });
        }
        if line_i + 1 < prepared.lines.len() {
            text.push('\n');
            runs.push(TextRun {
                len: 1,
                color: None,
                weight: None,
                slant: None,
            });
        }
    }

    let rich = RichText::new(Arc::<str>::from(text), runs);

    let text_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metric_required("metric.font.mono_size"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    };
    let fg = theme.color_required("foreground");

    let (wrap, overflow) = match wrap {
        CodeBlockWrap::ScrollX => (TextWrap::None, TextOverflow::Clip),
        CodeBlockWrap::Word => (TextWrap::Word, TextOverflow::Clip),
    };

    let mut scroll_layout = LayoutStyle::default();
    scroll_layout.size.width = Length::Fill;
    scroll_layout.size.height = Length::Auto;
    scroll_layout.overflow = Overflow::Clip;

    let text_layout = {
        let mut layout = LayoutStyle::default();
        layout.size.width = match wrap {
            TextWrap::None => Length::Auto,
            TextWrap::Word => Length::Fill,
        };
        layout
    };

    cx.scroll(
        ScrollProps {
            layout: scroll_layout,
            axis: ScrollAxis::X,
            probe_unbounded: matches!(wrap, TextWrap::None),
            ..Default::default()
        },
        |cx| {
            vec![cx.selectable_text_props(SelectableTextProps {
                layout: text_layout,
                rich,
                style: Some(text_style),
                color: Some(fg),
                wrap,
                overflow,
            })]
        },
    )
}
