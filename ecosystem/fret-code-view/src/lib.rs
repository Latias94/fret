//! Code view component(s) for Fret.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Color, Edges, FontId, FontWeight, Px, RichText, SemanticsRole, TextOverflow, TextRun,
    TextStyle, TextWrap, TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, LayoutStyle, Length, Overflow, PositionStyle,
    PressableProps, ScrollAxis, ScrollProps, SelectableTextProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius, Space,
};

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
            let copied = feedback.lock().copied;
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
    prepared: &PreparedCodeBlock,
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

        match wrap {
            CodeBlockWrap::ScrollX => vec![decl_scroll::overflow_scroll_x_vstack(
                cx,
                LayoutRefinement::default().w_full(),
                false,
                stack::VStackProps::default().gap(Space::N0),
                |cx| {
                    prepared
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(i, line)| {
                            render_code_line_with_number(
                                cx,
                                theme,
                                i + 1,
                                prepared.line_number_width,
                                line,
                                wrap,
                            )
                        })
                        .collect::<Vec<_>>()
                },
            )],
            CodeBlockWrap::Word => {
                let mut scroll_layout = LayoutStyle::default();
                scroll_layout.size.width = Length::Fill;
                scroll_layout.size.height = Length::Auto;
                scroll_layout.overflow = Overflow::Clip;

                vec![cx.scroll(
                    ScrollProps {
                        layout: scroll_layout,
                        axis: ScrollAxis::X,
                        probe_unbounded: false,
                        ..Default::default()
                    },
                    |cx| {
                        vec![stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N0)
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                prepared
                                    .lines
                                    .iter()
                                    .enumerate()
                                    .map(|(i, line)| {
                                        render_code_line_with_number(
                                            cx,
                                            theme,
                                            i + 1,
                                            prepared.line_number_width,
                                            line,
                                            wrap,
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            },
                        )]
                    },
                )]
            }
        }
    })
}

fn render_code_block_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prepared: &PreparedCodeBlock,
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

#[derive(Debug, Default)]
struct CopyFeedback {
    copied: bool,
    token: Option<TimerToken>,
}

#[derive(Clone, Default)]
struct CopyFeedbackRef(Arc<Mutex<CopyFeedback>>);

impl CopyFeedbackRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, CopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn render_copy_button_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    feedback: CopyFeedbackRef,
    code: Arc<str>,
) -> AnyElement {
    let inset = MetricRef::space(Space::N1p5).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.top = Some(inset);
    props.layout.inset.right = Some(inset);
    props.layout.size.width = Length::Auto;

    cx.container(props, |cx| {
        vec![render_copy_button(cx, theme, feedback, code)]
    })
}

fn render_copy_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    feedback: CopyFeedbackRef,
    code: Arc<str>,
) -> AnyElement {
    let copied = feedback.lock().copied;
    let label = if copied { "Copied" } else { "Copy" };

    cx.pressable_with_id_props(move |cx, st, id| {
        let mut props = PressableProps::default();
        props.a11y.role = Some(SemanticsRole::Button);
        props.a11y.label = Some(Arc::<str>::from(label));
        props.focusable = false;

        cx.timer_on_timer_for(
            id,
            Arc::new({
                let feedback = feedback.clone();
                move |_host, _cx, token| {
                    let mut feedback = feedback.lock();
                    if feedback.token != Some(token) {
                        return false;
                    }
                    feedback.token = None;
                    feedback.copied = false;
                    true
                }
            }),
        );

        cx.pressable_on_activate({
            let code = code.clone();
            let feedback = feedback.clone();
            Arc::new(move |host, action_cx, _reason| {
                host.push_effect(Effect::ClipboardSetText {
                    text: code.to_string(),
                });

                let (prev, token) = {
                    let mut feedback = feedback.lock();
                    let prev = feedback.token.take();
                    let token = host.next_timer_token();
                    feedback.copied = true;
                    feedback.token = Some(token);
                    (prev, token)
                };

                if let Some(prev) = prev {
                    host.push_effect(Effect::CancelTimer { token: prev });
                }
                host.push_effect(Effect::SetTimer {
                    window: Some(action_cx.window),
                    token,
                    after: Duration::from_secs(2),
                    repeat: None,
                });
                host.request_redraw(action_cx.window);
            })
        });

        let bg_pressed = theme.color_required("accent");
        let bg_hover = theme.color_required("color.menu.item.hover");
        let bg_idle = theme.color_required("secondary");
        let radius_sm = theme.metric_required("metric.radius.sm");
        let font_size = theme.metric_required("metric.font.size");
        let line_height = theme.metric_required("metric.font.line_height");
        let fg = theme.color_required("foreground");

        let bg = if st.pressed {
            bg_pressed
        } else if st.hovered {
            bg_hover
        } else {
            bg_idle
        };

        let pad_y = MetricRef::space(Space::N0p5).resolve(theme);
        let pad_x = MetricRef::space(Space::N1p5).resolve(theme);

        let mut container = ContainerProps::default();
        container.padding = Edges {
            top: pad_y,
            right: pad_x,
            bottom: pad_y,
            left: pad_x,
        };
        container.corner_radii = fret_core::Corners::all(radius_sm);
        container.background = Some(bg);
        container.border = Edges::all(Px(0.0));

        (
            props,
            vec![cx.container(container, |cx| {
                vec![cx.text_props(TextProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Auto;
                        layout
                    },
                    text: Arc::<str>::from(label),
                    style: Some(TextStyle {
                        font: FontId::default(),
                        size: font_size,
                        weight: FontWeight::SEMIBOLD,
                        slant: Default::default(),
                        line_height: Some(line_height),
                        letter_spacing_em: None,
                    }),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            })],
        )
    })
}

fn render_code_line_with_number<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line_no: usize,
    width: usize,
    line: &PreparedLine,
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

    stack::hstack(cx, stack::HStackProps::default().gap(Space::N2), |cx| {
        let number = Arc::<str>::from(format!("{line_no:>width$}", width = width));
        let muted = theme.color_required("muted-foreground");
        let number_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: number,
            style: Some(number_style),
            color: Some(muted),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        vec![number_el, render_code_line(cx, theme, line, wrap)]
    })
}

fn render_code_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line: &PreparedLine,
    wrap: CodeBlockWrap,
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

    let mut text = String::new();
    let mut runs: Vec<TextRun> = Vec::with_capacity(line.segments.len());
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

    let rich = RichText::new(Arc::<str>::from(text), runs);
    let (wrap, overflow) = match wrap {
        CodeBlockWrap::ScrollX => (TextWrap::None, TextOverflow::Clip),
        CodeBlockWrap::Word => (TextWrap::Word, TextOverflow::Clip),
    };
    cx.selectable_text_props(SelectableTextProps {
        layout: Default::default(),
        rich,
        style: Some(text_style),
        color: Some(fg),
        wrap,
        overflow,
    })
}

fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let key = format!("color.syntax.{highlight}");
    if let Some(c) = theme.color_by_key(&key) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    match fallback {
        "comment" => Some(theme.color_required("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_required("primary")),
        "property" | "variable" => Some(theme.color_required("foreground")),
        "punctuation" => Some(theme.color_required("muted-foreground")),

        // These are still treated as editor-ish baseline tokens until a dedicated SyntaxTheme
        // subsystem lands.
        "string" => Some(theme.color_required("color.viewport.gizmo.y")),
        "number" | "boolean" | "constant" => {
            Some(theme.color_required("color.viewport.rotate_gizmo"))
        }
        "type" | "constructor" => Some(theme.color_required("color.viewport.marker")),
        "function" => Some(theme.color_required("color.viewport.drag_line.orbit")),
        _ => None,
    }
}

#[derive(Default)]
struct CodeBlockPreparedState {
    key: CodeBlockKey,
    prepared: Arc<PreparedCodeBlock>,
}

impl CodeBlockPreparedState {
    fn prepare(&mut self, code: &str, language: Option<&str>, show_line_numbers: bool) {
        let key = CodeBlockKey::new(code, language, show_line_numbers);
        if self.key == key {
            return;
        }
        self.key = key;
        self.prepared = Arc::new(prepare_code_block(code, language, show_line_numbers));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodeBlockKey {
    code_hash: u64,
    code_len: usize,
    language_hash: u64,
    language_len: usize,
    show_line_numbers: bool,
}

impl Default for CodeBlockKey {
    fn default() -> Self {
        Self {
            code_hash: 0,
            code_len: 0,
            language_hash: 0,
            language_len: 0,
            show_line_numbers: false,
        }
    }
}

impl CodeBlockKey {
    fn new(code: &str, language: Option<&str>, show_line_numbers: bool) -> Self {
        let language = language.unwrap_or("");
        Self {
            code_hash: hash_value(code),
            code_len: code.len(),
            language_hash: hash_value(language),
            language_len: language.len(),
            show_line_numbers,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct PreparedCodeBlock {
    show_line_numbers: bool,
    line_number_width: usize,
    lines: Vec<PreparedLine>,
}

#[derive(Debug, Clone, Default)]
struct PreparedLine {
    segments: Vec<PreparedSegment>,
}

#[derive(Debug, Clone)]
struct PreparedSegment {
    text: Arc<str>,
    highlight: Option<&'static str>,
}

fn prepare_code_block(
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) -> PreparedCodeBlock {
    let spans = match language {
        Some(language) => fret_syntax::highlight(code, language).unwrap_or_default(),
        None => Vec::new(),
    };

    let mut lines = split_lines(code);
    let line_number_width = line_number_width(lines.len());

    let mut prepared_lines = Vec::with_capacity(lines.len());
    let mut span_i = 0usize;

    for line in &mut lines {
        let line_text = line.text;
        let global_range = line.range.clone();

        while span_i < spans.len() && spans[span_i].range.end <= global_range.start {
            span_i += 1;
        }

        let mut segments: Vec<(String, Option<&'static str>)> = Vec::new();
        let mut cursor = global_range.start;
        let mut j = span_i;
        while j < spans.len() {
            let span = &spans[j];
            if span.range.start >= global_range.end {
                break;
            }
            let start = span.range.start.max(global_range.start);
            let end = span.range.end.min(global_range.end);
            if cursor < start {
                let rel = cursor - global_range.start;
                let rel_end = start - global_range.start;
                segments.push((safe_slice(line_text, rel, rel_end), None));
            }
            let rel = start - global_range.start;
            let rel_end = end - global_range.start;
            segments.push((safe_slice(line_text, rel, rel_end), span.highlight));
            cursor = end;
            j += 1;
        }
        if cursor < global_range.end {
            let rel = cursor - global_range.start;
            let rel_end = global_range.end - global_range.start;
            segments.push((safe_slice(line_text, rel, rel_end), None));
        }

        if segments.is_empty() {
            segments.push((line_text.to_string(), None));
        }

        let segments = coalesce_segments(segments)
            .into_iter()
            .map(|(text, highlight)| PreparedSegment {
                text: Arc::<str>::from(text),
                highlight,
            })
            .collect();

        prepared_lines.push(PreparedLine { segments });
    }

    PreparedCodeBlock {
        show_line_numbers,
        line_number_width,
        lines: prepared_lines,
    }
}

fn coalesce_segments(
    segments: Vec<(String, Option<&'static str>)>,
) -> Vec<(String, Option<&'static str>)> {
    let mut out: Vec<(String, Option<&'static str>)> = Vec::with_capacity(segments.len());
    for (text, highlight) in segments {
        if text.is_empty() {
            continue;
        }
        if let Some((last_text, last_highlight)) = out.last_mut() {
            if *last_highlight == highlight {
                last_text.push_str(&text);
                continue;
            }
        }
        out.push((text, highlight));
    }
    out
}

fn hash_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

fn safe_slice(text: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    if start >= text.len() {
        return String::new();
    }
    let end = end.min(text.len());
    match text.get(start..end) {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(&text.as_bytes()[start..end]).into_owned(),
    }
}

#[derive(Debug, Clone)]
struct LineSlice<'a> {
    range: Range<usize>,
    text: &'a str,
}

fn split_lines(text: &str) -> Vec<LineSlice<'_>> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let mut end = i;
            if end > start && bytes[end - 1] == b'\r' {
                end -= 1;
            }
            out.push(LineSlice {
                range: start..end,
                text: &text[start..end],
            });
            start = i + 1;
        }
        i += 1;
    }
    out.push(LineSlice {
        range: start..text.len(),
        text: &text[start..],
    });
    out
}

fn line_number_width(lines: usize) -> usize {
    let mut n = lines.max(1);
    let mut digits = 0usize;
    while n > 0 {
        digits += 1;
        n /= 10;
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesces_adjacent_segments() {
        let segments = vec![
            ("a".to_string(), None),
            ("b".to_string(), None),
            ("c".to_string(), Some("keyword")),
            ("d".to_string(), Some("keyword")),
            ("".to_string(), Some("keyword")),
            ("e".to_string(), None),
        ];
        let out = coalesce_segments(segments);
        assert_eq!(
            out,
            vec![
                ("ab".to_string(), None),
                ("cd".to_string(), Some("keyword")),
                ("e".to_string(), None)
            ]
        );
    }

    #[test]
    fn splits_crlf_lines_without_carriage_returns() {
        let lines = split_lines("a\r\nb\r\n");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "a");
        assert_eq!(lines[1].text, "b");
        assert_eq!(lines[2].text, "");
    }
}
