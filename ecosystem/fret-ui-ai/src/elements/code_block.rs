use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ColorRef, LayoutRefinement};

/// AI Elements-aligned code block surface backed by `ecosystem/fret-code-view`.
///
/// This is a policy/composition layer:
/// - apps own effects (except for local clipboard copy),
/// - the component exposes stable selectors for diag automation.
#[derive(Clone)]
pub struct CodeBlock {
    code: Arc<str>,
    language: Option<Arc<str>>,
    show_line_numbers: bool,
    show_language: bool,
    max_height: Option<Px>,
    header_left: Vec<AnyElement>,
    header_right: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBlock")
            .field("code_len", &self.code.len())
            .field("language", &self.language.as_deref())
            .field("show_line_numbers", &self.show_line_numbers)
            .field("show_language", &self.show_language)
            .field("max_height", &self.max_height)
            .field("header_left_len", &self.header_left.len())
            .field("header_right_len", &self.header_right.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl CodeBlock {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_line_numbers: false,
            show_language: true,
            max_height: None,
            header_left: Vec::new(),
            header_right: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    pub fn show_language(mut self, show: bool) -> Self {
        self.show_language = show;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(Px(max_height.0.max(0.0)));
        self
    }

    pub fn header_left<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.header_left.extend(children);
        self
    }

    pub fn header_right<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.header_right.extend(children);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut header =
            fret_code_view::CodeBlockHeaderSlots::default().show_language(self.show_language);
        header.left = self.header_left;
        header.right = self.header_right;

        let options = fret_code_view::CodeBlockUiOptions {
            show_header: true,
            header_divider: true,
            header_background: fret_code_view::CodeBlockHeaderBackground::Secondary,
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: fret_code_view::CodeBlockCopyButtonPlacement::Overlay,
            border: true,
            wrap: fret_code_view::CodeBlockWrap::ScrollX,
            max_height: self.max_height,
            windowed_lines: false,
            windowed_lines_overscan: 6,
            show_scrollbar_x: false,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: false,
            scrollbar_y_on_hover: true,
        };

        let code = self.code;
        let language = self.language;
        let show_line_numbers = self.show_line_numbers;
        let content = fret_code_view::code_block_with_header_slots(
            cx,
            &code,
            language.as_deref(),
            show_line_numbers,
            options,
            header,
        );

        let el = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
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

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

/// Copy button aligned with AI Elements `CodeBlockCopyButton`.
#[derive(Clone)]
pub struct CodeBlockCopyButton {
    code: Arc<str>,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CodeBlockCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBlockCopyButton")
            .field("code_len", &self.code.len())
            .field("timeout_ms", &self.timeout.as_millis())
            .field("test_id", &self.test_id.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .finish()
    }
}

impl CodeBlockCopyButton {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Optional marker that only exists while the button is in the "copied" state.
    ///
    /// This is intended for `fretboard diag` scripts.
    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

        let code = self.code;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        cx.pressable_with_id_props(move |cx, st, id| {
            let copied = feedback.lock().copied;
            let label = if copied { "Copied" } else { "Copy" };

            cx.timer_on_timer_for(
                id,
                Arc::new({
                    let feedback = feedback.clone();
                    move |host, action_cx, token| {
                        let mut feedback = feedback.lock();
                        if feedback.token != Some(token) {
                            return false;
                        }
                        feedback.token = None;
                        feedback.copied = false;
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
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
                        after: timeout,
                        repeat: None,
                    });
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                })
            });

            let mut props = PressableProps::default();
            props.enabled = true;
            props.focusable = true;
            props.a11y.role = Some(SemanticsRole::Button);
            props.a11y.label = Some(Arc::<str>::from(label));
            props.a11y.test_id = test_id.clone();

            let fg = theme.color_required("muted-foreground");
            let bg_hover = theme
                .color_by_key("color.menu.item.hover")
                .unwrap_or_else(|| theme.color_required("secondary"));
            let bg_pressed = theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_required("secondary"));

            let bg = if st.pressed {
                alpha(bg_pressed, 0.9)
            } else if st.hovered {
                alpha(bg_hover, 0.9)
            } else {
                Color::TRANSPARENT
            };

            let size = Px(28.0);
            let radius = theme.metric_required("metric.radius.sm");

            let icon_id = if copied {
                fret_icons::ids::ui::CHECK
            } else {
                fret_icons::ids::ui::COPY
            };

            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg)));
            let mut content_props = ContainerProps::default();
            content_props.layout.size.width = Length::Px(size);
            content_props.layout.size.height = Length::Px(size);
            content_props.background = Some(bg);
            content_props.corner_radii = fret_core::Corners::all(radius);
            content_props.border = Edges::all(Px(0.0));
            content_props.padding = Edges::all(Px(0.0));

            let content = cx.container(content_props, move |cx| {
                let row = fret_ui_kit::declarative::stack::hstack(
                    cx,
                    fret_ui_kit::declarative::stack::HStackProps::default()
                        .items_center()
                        .justify_center()
                        .layout(LayoutRefinement::default().w_full().h_full()),
                    move |_cx| vec![icon],
                );
                vec![row]
            });

            let marker = copied_marker_test_id.clone().and_then(|marker_id| {
                copied.then(|| {
                    cx.text_props(TextProps {
                        layout: LayoutStyle {
                            // Keep it out of layout flow.
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(Px(0.0)),
                                height: Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Arc::<str>::from(""),
                        style: None,
                        color: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    })
                    .attach_semantics(
                        fret_ui::element::SemanticsDecoration::default()
                            .role(SemanticsRole::Group)
                            .test_id(marker_id),
                    )
                })
            });

            let mut children = vec![content];
            if let Some(marker) = marker {
                children.push(marker);
            }
            (props, children)
        })
    }
}

/// Monospace filename label for `CodeBlock` headers.
#[derive(Clone)]
pub struct CodeBlockFilename {
    text: Arc<str>,
}

impl CodeBlockFilename {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}
