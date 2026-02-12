use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    AttributedText, Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextSpan,
    TextStyle, TextWrap, TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SelectableTextProps,
    SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{
    InputTokenKeys, input_chrome_container_props, resolve_input_chrome,
};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Size, Space};

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

/// AI Elements-aligned snippet surface (inline copyable code).
#[derive(Clone)]
pub struct Snippet {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Snippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snippet")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Snippet {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let chrome = resolve_input_chrome(
            &theme,
            Size::default(),
            &self.chrome,
            InputTokenKeys::none(),
        );
        let mut layout = decl_style::layout_style(&theme, self.layout);
        layout.size.height = Length::Auto;

        let props = input_chrome_container_props(layout, chrome, chrome.border_color);
        let el = cx.container(props, move |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N0)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| self.children.clone(),
            )]
        });

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

/// Muted label segment aligned with AI Elements `SnippetText`.
#[derive(Clone)]
pub struct SnippetText {
    text: Arc<str>,
}

impl SnippetText {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let pad_x = theme
            .metric_by_key("component.input.padding_x")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme));
        let pad_y = theme
            .metric_by_key("component.input.padding_y")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N1).resolve(&theme));

        cx.container(
            ContainerProps {
                padding: Edges {
                    top: pad_y,
                    right: pad_x,
                    bottom: pad_y,
                    left: Px(pad_x.0.max(0.0) * 0.75),
                },
                ..Default::default()
            },
            move |cx| {
                vec![cx.text_props(TextProps {
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
                })]
            },
        )
    }
}

/// Read-only code segment aligned with AI Elements `SnippetInput`.
///
/// Note: Fret's text inputs are model-backed and currently do not support `readOnly` directly.
/// This uses `SelectableText` to preserve selection/copy outcomes.
#[derive(Clone)]
pub struct SnippetInput {
    code: Arc<str>,
}

impl SnippetInput {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self { code: code.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let pad_x = theme
            .metric_by_key("component.input.padding_x")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme));
        let pad_y = theme
            .metric_by_key("component.input.padding_y")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N1).resolve(&theme));

        let code = self.code;
        let rich = AttributedText::new(
            code.clone(),
            Arc::<[TextSpan]>::from([TextSpan {
                len: code.len(),
                shaping: Default::default(),
                paint: Default::default(),
            }]),
        );
        let mut props = SelectableTextProps::new(rich);
        props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        };
        props.style = Some(TextStyle {
            font: FontId::monospace(),
            size: theme.metric_required("metric.font.mono_size"),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(theme.metric_required("metric.font.mono_line_height")),
            letter_spacing_em: None,
        });
        props.color = Some(theme.color_required("foreground"));
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;

        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Auto;
                    layout
                },
                padding: Edges::symmetric(pad_x, pad_y),
                ..Default::default()
            },
            move |cx| vec![cx.selectable_text_props(props)],
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

/// Copy button aligned with AI Elements `SnippetCopyButton`.
#[derive(Clone)]
pub struct SnippetCopyButton {
    code: Arc<str>,
    on_copy: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>,
    >,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SnippetCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnippetCopyButton")
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

impl SnippetCopyButton {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            on_copy: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    /// Called after the copy intent is issued.
    ///
    /// Note: this callback does not currently model "copy failed" (platform effects are
    /// best-effort).
    pub fn on_copy(
        mut self,
        on_copy: Arc<
            dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static,
        >,
    ) -> Self {
        self.on_copy = Some(on_copy);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

        let code = self.code;
        let on_copy = self.on_copy;
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
                let on_copy = on_copy.clone();
                Arc::new(move |host, action_cx, _reason| {
                    if feedback.lock().copied {
                        return;
                    }

                    host.push_effect(Effect::ClipboardSetText {
                        text: code.to_string(),
                    });
                    if let Some(on_copy) = on_copy.as_ref() {
                        on_copy(host, action_cx);
                    }

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
                vec![stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .items_center()
                        .justify_center()
                        .layout(LayoutRefinement::default().w_full().h_full()),
                    move |_cx| vec![icon],
                )]
            });

            let marker = copied_marker_test_id.clone().and_then(|marker_id| {
                copied.then(|| {
                    cx.text_props(TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
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
