use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::window::ColorScheme;
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
use fret_ui_kit::declarative::chrome::centered_fixed_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{
    input_chrome_container_props, resolve_input_chrome, InputTokenKeys,
};
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Size, Space};

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

/// Nearest `Snippet` context in scope.
#[derive(Debug, Clone)]
pub struct SnippetContext {
    pub code: Arc<str>,
}

#[derive(Debug, Default, Clone)]
struct SnippetLocalState {
    context: Option<SnippetContext>,
}

pub fn use_snippet_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SnippetContext> {
    cx.inherited_state::<SnippetLocalState>()
        .and_then(|st| st.context.clone())
}

/// AI Elements-aligned snippet surface (inline copyable code).
pub struct Snippet {
    code: Option<Arc<str>>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Snippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snippet")
            .field("has_code", &self.code.is_some())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Snippet {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            code: None,
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn with_code(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: Some(code.into()),
            children: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn code(mut self, code: impl Into<Arc<str>>) -> Self {
        self.code = Some(code.into());
        self
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Snippet {
            code: _,
            children,
            test_id,
            layout,
            chrome,
        } = self;
        let theme = Theme::global(&*cx.app).clone();

        let chrome = resolve_input_chrome(&theme, Size::default(), &chrome, InputTokenKeys::none());
        let mut layout = decl_style::layout_style(&theme, layout);
        layout.size.height = Length::Px(chrome.min_height);

        let mut props = input_chrome_container_props(layout, chrome, chrome.border_color);
        props.shadow = Some(decl_style::shadow_xs(&theme, chrome.radius));
        props.focus_within = true;
        props.focus_border_color = Some(chrome.border_color_focused);
        props.focus_ring = Some(decl_style::focus_ring(&theme, chrome.radius));
        let el = cx.container(props, move |cx| {
            vec![ui::h_row(move |_cx| children)
                .gap(Space::N0)
                .items(Items::Center)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx)]
        });

        let mut semantics =
            fret_ui::element::SemanticsDecoration::default().role(SemanticsRole::Group);
        if let Some(test_id) = test_id {
            semantics = semantics.test_id(test_id);
        }
        el.attach_semantics(semantics)
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        if let Some(code) = self.code.clone() {
            cx.root_state(SnippetLocalState::default, |st| {
                st.context = Some(SnippetContext { code });
            });
        }
        self.children(children(cx)).into_element(cx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnippetAddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
}

/// Inline addon wrapper aligned with AI Elements `SnippetAddon`.
#[derive(Debug)]
pub struct SnippetAddon {
    children: Vec<AnyElement>,
    align: SnippetAddonAlign,
}

impl SnippetAddon {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            align: SnippetAddonAlign::default(),
        }
    }

    pub fn align(mut self, align: SnippetAddonAlign) -> Self {
        self.align = align;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let pad_x = theme
            .metric_by_key("component.input.padding_x")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme));
        let pad_y = theme
            .metric_by_key("component.input.padding_y")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N1).resolve(&theme));

        let padding = match self.align {
            SnippetAddonAlign::InlineStart => Edges {
                top: pad_y,
                right: Px(0.0),
                bottom: pad_y,
                left: pad_x,
            },
            SnippetAddonAlign::InlineEnd => Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: Px(0.0),
            },
        };
        let children = self.children;

        cx.container(
            ContainerProps {
                padding: padding.into(),
                ..Default::default()
            },
            move |cx| {
                vec![ui::h_row(move |_cx| children)
                    .gap(Space::N0)
                    .items(Items::Center)
                    .into_element(cx)]
            },
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
        let pad_left = fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme);

        cx.container(
            ContainerProps {
                padding: Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(0.0),
                    left: pad_left,
                }
                .into(),
                ..Default::default()
            },
            move |cx| {
                vec![cx.text_props(TextProps {
                    layout: Default::default(),
                    text: self.text,
                    style: Some(typography::as_control_text(TextStyle {
                        font: FontId::monospace(),
                        size: theme.metric_token("metric.font.mono_size"),
                        weight: FontWeight::NORMAL,
                        slant: Default::default(),
                        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
                        letter_spacing_em: None,
                        ..Default::default()
                    })),
                    color: Some(theme.color_token("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,

                    ink_overflow: fret_ui::element::TextInkOverflow::None,
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
    code: Option<Arc<str>>,
}

impl SnippetInput {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: Some(code.into()),
        }
    }

    pub fn from_context() -> Self {
        Self { code: None }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let pad_x = theme
            .metric_by_key("component.input.padding_x")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme));
        let pad_y = theme
            .metric_by_key("component.input.padding_y")
            .unwrap_or_else(|| fret_ui_kit::MetricRef::space(Space::N1).resolve(&theme));

        let code = self
            .code
            .or_else(|| use_snippet_context(cx).map(|context| context.code))
            .unwrap_or_else(|| Arc::<str>::from(""));
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
        props.style = Some(typography::as_control_text(TextStyle {
            font: FontId::monospace(),
            size: theme.metric_token("metric.font.mono_size"),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(theme.metric_token("metric.font.mono_line_height")),
            letter_spacing_em: None,
            ..Default::default()
        }));
        props.color = Some(theme.color_token("foreground"));
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
                padding: Edges::symmetric(pad_x, pad_y).into(),
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
pub struct SnippetCopyButton {
    code: Option<Arc<str>>,
    children: Vec<AnyElement>,
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
            .field("code_len", &self.code.as_ref().map(|code| code.len()))
            .field("children_len", &self.children.len())
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
            code: Some(code.into()),
            children: Vec::new(),
            on_copy: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn from_context() -> Self {
        Self {
            code: None,
            children: Vec::new(),
            on_copy: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    /// Overrides the default copy/check icon with caller-owned button content.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.slot_state(CopyFeedbackRef::default, |st| st.clone());

        let code = self
            .code
            .or_else(|| use_snippet_context(cx).map(|context| context.code))
            .unwrap_or_else(|| Arc::<str>::from(""));
        let custom_children = self.children;
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            let copied = feedback.lock().copied;

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
            props.a11y.label = Some(Arc::<str>::from("Copy"));
            props.a11y.test_id = test_id.clone();

            let fg_default = theme.color_token("foreground");
            let fg_hover = theme
                .color_by_key("accent-foreground")
                .unwrap_or(fg_default);
            let fg = if st.hovered || st.pressed {
                fg_hover
            } else {
                fg_default
            };

            let bg_hover = if theme.color_scheme == Some(ColorScheme::Dark) {
                theme
                    .color_by_key("accent/50")
                    .unwrap_or_else(|| alpha(theme.color_token("accent"), 0.5))
            } else {
                theme.color_token("accent")
            };
            let bg_pressed = bg_hover;

            let bg = if st.pressed {
                bg_pressed
            } else if st.hovered {
                bg_hover
            } else {
                Color::TRANSPARENT
            };

            // AI Elements uses InputGroupButton size="icon-sm" (8x8 Tailwind => 32px).
            let size = Px(32.0);
            let radius = theme.metric_token("metric.radius.md");

            let mut chrome_props = ContainerProps::default();
            chrome_props.layout.size.width = Length::Px(size);
            chrome_props.layout.size.height = Length::Px(size);
            chrome_props.background = Some(bg);
            chrome_props.corner_radii = fret_core::Corners::all(radius);
            chrome_props.border = Edges::all(Px(0.0));
            chrome_props.padding = Edges::all(Px(0.0)).into();

            (props, chrome_props, move |cx| {
                let visual_children = if custom_children.is_empty() {
                    let icon_id = if copied {
                        fret_icons::ids::ui::CHECK
                    } else {
                        fret_icons::ids::ui::COPY
                    };
                    vec![decl_icon::icon_with(
                        cx,
                        icon_id,
                        Some(Px(14.0)),
                        Some(ColorRef::Color(fg)),
                    )]
                } else {
                    custom_children
                };

                let row = ui::h_row(move |_cx| visual_children)
                    .items(Items::Center)
                    .justify(Justify::Center)
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .into_element(cx)
                    .inherit_foreground(fg);

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
                            align: fret_core::TextAlign::Start,

                            ink_overflow: fret_ui::element::TextInkOverflow::None,
                        })
                        .attach_semantics(
                            fret_ui::element::SemanticsDecoration::default()
                                .role(SemanticsRole::Group)
                                .test_id(marker_id),
                        )
                    })
                });

                let mut children = vec![row];
                if let Some(marker) = marker {
                    children.push(marker);
                }
                children
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(160.0)),
        )
    }

    fn has_selectable_text(element: &AnyElement, text: &str) -> bool {
        if matches!(
            &element.kind,
            ElementKind::SelectableText(props) if props.rich.text.as_ref() == text
        ) {
            return true;
        }
        element
            .children
            .iter()
            .any(|child| has_selectable_text(child, text))
    }

    fn has_pressable_test_id(element: &AnyElement, test_id: &str) -> bool {
        if matches!(
            &element.kind,
            ElementKind::Pressable(props)
                if props.a11y.test_id.as_deref() == Some(test_id)
                    && props.a11y.label.as_deref() == Some("Copy")
        ) {
            return true;
        }
        element
            .children
            .iter()
            .any(|child| has_pressable_test_id(child, test_id))
    }

    fn has_text(element: &AnyElement, text: &str) -> bool {
        if matches!(&element.kind, ElementKind::Text(props) if props.text.as_ref() == text) {
            return true;
        }
        element.children.iter().any(|child| has_text(child, text))
    }

    #[test]
    fn snippet_children_can_consume_inherited_code_context() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Snippet::with_code("cargo nextest run -p fret-ui-ai")
                    .test_id("ui-ai-snippet-root")
                    .into_element_with_children(cx, |cx| {
                        vec![
                            SnippetAddon::new([SnippetText::new("$").into_element(cx)])
                                .into_element(cx),
                            SnippetInput::from_context().into_element(cx),
                            SnippetAddon::new([SnippetCopyButton::from_context()
                                .test_id("ui-ai-snippet-copy")
                                .into_element(cx)])
                            .align(SnippetAddonAlign::InlineEnd)
                            .into_element(cx),
                        ]
                    })
            });

        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-snippet-root")
        );
        assert!(has_selectable_text(
            &element,
            "cargo nextest run -p fret-ui-ai"
        ));
        assert!(has_pressable_test_id(&element, "ui-ai-snippet-copy"));
    }

    #[test]
    fn snippet_copy_button_accepts_custom_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SnippetCopyButton::new("cargo run -p fret-ui-ai")
                    .children([cx.text("X")])
                    .test_id("ui-ai-snippet-copy-custom")
                    .into_element(cx)
            });

        assert!(has_pressable_test_id(&element, "ui-ai-snippet-copy-custom"));
        assert!(has_text(&element, "X"));
    }
}
