//! AI Elements-aligned `Terminal` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/terminal.tsx`.

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Color, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SemanticsDecoration,
    SizeStyle, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::ScrollArea;

use super::Shimmer;
pub type OnTerminalClear = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

fn zinc_950() -> Color {
    Color {
        r: 9.0 / 255.0,
        g: 9.0 / 255.0,
        b: 11.0 / 255.0,
        a: 1.0,
    }
}

fn zinc_100() -> Color {
    Color {
        r: 244.0 / 255.0,
        g: 244.0 / 255.0,
        b: 245.0 / 255.0,
        a: 1.0,
    }
}

fn zinc_800() -> Color {
    Color {
        r: 39.0 / 255.0,
        g: 39.0 / 255.0,
        b: 42.0 / 255.0,
        a: 1.0,
    }
}

fn zinc_400() -> Color {
    Color {
        r: 161.0 / 255.0,
        g: 161.0 / 255.0,
        b: 170.0 / 255.0,
        a: 1.0,
    }
}

#[derive(Debug, Default, Clone)]
struct TerminalProviderState {
    controller: Option<TerminalController>,
}

#[derive(Clone)]
pub struct TerminalController {
    pub output: Model<String>,
    pub is_streaming: bool,
    pub auto_scroll: bool,
    pub on_clear: Option<OnTerminalClear>,
}

impl std::fmt::Debug for TerminalController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalController")
            .field("output", &"<model>")
            .field("is_streaming", &self.is_streaming)
            .field("auto_scroll", &self.auto_scroll)
            .field("has_on_clear", &self.on_clear.is_some())
            .finish()
    }
}

fn use_terminal_controller<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Option<TerminalController> {
    cx.with_state(TerminalProviderState::default, |st| st.controller.clone())
}

#[derive(Clone)]
pub struct Terminal {
    output: Model<String>,
    is_streaming: bool,
    auto_scroll: bool,
    on_clear: Option<OnTerminalClear>,
    test_id_root: Option<Arc<str>>,
    test_id_copy: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
    test_id_clear: Option<Arc<str>>,
    test_id_viewport: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terminal")
            .field("output", &"<model>")
            .field("is_streaming", &self.is_streaming)
            .field("auto_scroll", &self.auto_scroll)
            .field("has_on_clear", &self.on_clear.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_copy", &self.test_id_copy.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .field("test_id_clear", &self.test_id_clear.as_deref())
            .field("test_id_viewport", &self.test_id_viewport.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Terminal {
    pub fn new(output: Model<String>) -> Self {
        Self {
            output,
            is_streaming: false,
            auto_scroll: true,
            on_clear: None,
            test_id_root: None,
            test_id_copy: None,
            copied_marker_test_id: None,
            test_id_clear: None,
            test_id_viewport: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn is_streaming(mut self, is_streaming: bool) -> Self {
        self.is_streaming = is_streaming;
        self
    }

    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    pub fn on_clear(mut self, on_clear: OnTerminalClear) -> Self {
        self.on_clear = Some(on_clear);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_copy(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_copy = Some(id.into());
        self
    }

    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn test_id_clear(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_clear = Some(id.into());
        self
    }

    pub fn test_id_viewport(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_viewport = Some(id.into());
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

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, TerminalController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(zinc_950()))
            .border_color(ColorRef::Color(zinc_800()))
            .text_color(ColorRef::Color(zinc_100()))
            .merge(self.chrome);

        let controller = TerminalController {
            output: self.output.clone(),
            is_streaming: self.is_streaming,
            auto_scroll: self.auto_scroll,
            on_clear: self.on_clear.clone(),
        };
        cx.with_state(TerminalProviderState::default, |st| {
            st.controller = Some(controller.clone());
        });

        let children = children(cx, controller.clone());

        let root = cx.container(
            decl_style::container_props(&theme, root_chrome, self.layout),
            move |_cx| children,
        );

        if let Some(test_id) = self.test_id_root {
            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            root
        }
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let test_id_copy = self.test_id_copy.clone();
        let copied_marker_test_id = self.copied_marker_test_id.clone();
        let test_id_clear = self.test_id_clear.clone();
        let test_id_viewport = self.test_id_viewport.clone();

        self.into_element_with_children(cx, move |cx, _controller| {
            let header = TerminalHeader::new()
                .children([
                    TerminalTitle::new().into_element(cx),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .items_center()
                            .gap(Space::N1)
                            .justify(Justify::End),
                        move |cx| {
                            vec![
                                TerminalStatus::new().into_element(cx),
                                TerminalActions::new()
                                    .children([
                                        TerminalCopyButton::new()
                                            .test_id_opt(test_id_copy.clone())
                                            .copied_marker_test_id_opt(
                                                copied_marker_test_id.clone(),
                                            )
                                            .into_element(cx),
                                        TerminalClearButton::new()
                                            .test_id_opt(test_id_clear.clone())
                                            .into_element(cx),
                                    ])
                                    .into_element(cx),
                            ]
                        },
                    ),
                ])
                .into_element(cx);

            let content = TerminalContent::new()
                .viewport_test_id_opt(test_id_viewport.clone())
                .into_element(cx);

            vec![header, content]
        })
    }
}

#[derive(Clone, Default)]
pub struct TerminalHeader {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TerminalHeader {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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
        let theme = Theme::global(&*cx.app).clone();

        let hstack = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .justify(Justify::Between)
                .items_center(),
            move |_cx| self.children,
        );

        let chrome = ChromeRefinement::default()
            .px(Space::N4)
            .py(Space::N2)
            .merge(self.chrome);

        let mut props = decl_style::container_props(
            &theme,
            chrome,
            LayoutRefinement::default().w_full().min_w_0(),
        );
        props.border = Edges {
            left: Px(0.0),
            right: Px(0.0),
            top: Px(0.0),
            bottom: Px(1.0),
        };
        props.border_color = Some(zinc_800());

        cx.container(props, move |_cx| vec![hstack])
    }
}

#[derive(Clone)]
pub struct TerminalTitle {
    label: Arc<str>,
    icon: IconId,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Default for TerminalTitle {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalTitle {
    pub fn new() -> Self {
        Self {
            label: Arc::<str>::from("Terminal"),
            icon: IconId::new_static("lucide.terminal"),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = icon;
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
        let theme = Theme::global(&*cx.app).clone();
        let fg = zinc_400();

        let icon = decl_icon::icon_with(cx, self.icon, Some(Px(16.0)), Some(ColorRef::Color(fg)));
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.label,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required("component.text.sm_px"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("component.text.sm_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .items_center()
                .gap(Space::N2),
            move |_cx| vec![icon, text],
        )
    }
}

#[derive(Clone, Default)]
pub struct TerminalStatus {
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
    children: Vec<AnyElement>,
}

impl TerminalStatus {
    pub fn new() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        if !controller.is_streaming {
            return cx.text("");
        }

        let children = if self.children.is_empty() {
            vec![
                Shimmer::new(Arc::<str>::from("Streaming"))
                    .refine_layout(LayoutRefinement::default().w_px(Px(64.0)))
                    .into_element(cx),
            ]
        } else {
            self.children
        };

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .items_center()
                .gap(Space::N2),
            move |_cx| children,
        )
    }
}

#[derive(Clone, Default)]
pub struct TerminalActions {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TerminalActions {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .items_center()
                .gap(Space::N1),
            move |_cx| self.children,
        )
    }
}

#[derive(Debug, Default, Clone)]
struct CopyFeedback {
    copied: bool,
    token: Option<fret_runtime::TimerToken>,
}

#[derive(Debug, Default, Clone)]
struct CopyFeedbackRef(Arc<Mutex<CopyFeedback>>);

impl CopyFeedbackRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, CopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Clone)]
pub struct TerminalCopyButton {
    on_copy: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl TerminalCopyButton {
    pub fn new() -> Self {
        Self {
            on_copy: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn on_copy(
        mut self,
        on_copy: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>,
    ) -> Self {
        self.on_copy = Some(on_copy);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    fn copied_marker_test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.copied_marker_test_id = id;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

        let output = controller.output;
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        cx.pressable_with_id_props(move |cx, st, id| {
            let copied = feedback.lock().copied;
            let label: Arc<str> = if copied {
                Arc::<str>::from("Copied")
            } else {
                Arc::<str>::from("Copy output")
            };

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
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                let output = output.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let text = host
                        .models_mut()
                        .read(&output, |v| v.clone())
                        .unwrap_or_default();

                    host.push_effect(Effect::ClipboardSetText { text });
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

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(label);
            pressable.a11y.test_id = test_id.clone();

            let fg = if st.hovered || st.pressed {
                zinc_100()
            } else {
                zinc_400()
            };
            let bg = if st.pressed || st.hovered {
                zinc_800()
            } else {
                Color::TRANSPARENT
            };

            let size = Px(28.0);
            let icon_id = if copied {
                fret_icons::ids::ui::CHECK
            } else {
                fret_icons::ids::ui::COPY
            };
            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg)));

            let mut content_props = ContainerProps::default();
            content_props.layout.size.width = Length::Px(size);
            content_props.layout.size.height = Length::Px(size);
            content_props.layout.flex.shrink = 0.0;
            content_props.background = Some(bg);
            content_props.corner_radii =
                fret_core::Corners::all(theme.metric_required("metric.radius.sm"));
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
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Group)
                            .test_id(marker_id),
                    )
                })
            });

            let mut children = vec![content];
            if let Some(marker) = marker {
                children.push(marker);
            }
            (pressable, children)
        })
    }
}

#[derive(Clone, Default)]
pub struct TerminalClearButton {
    test_id: Option<Arc<str>>,
}

impl TerminalClearButton {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        let Some(on_clear) = controller.on_clear else {
            return cx.text("");
        };
        let theme = Theme::global(&*cx.app).clone();

        let test_id = self.test_id;
        cx.pressable_with_id_props(move |cx, st, _id| {
            cx.pressable_on_activate({
                let on_clear = on_clear.clone();
                Arc::new(move |host, action_cx, _reason| {
                    on_clear(host, action_cx);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                })
            });

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(Arc::<str>::from("Clear"));
            pressable.a11y.test_id = test_id.clone();

            let fg = if st.hovered || st.pressed {
                zinc_100()
            } else {
                zinc_400()
            };
            let bg = if st.pressed || st.hovered {
                zinc_800()
            } else {
                Color::TRANSPARENT
            };

            let icon = decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.trash-2"),
                Some(Px(14.0)),
                Some(ColorRef::Color(fg)),
            );

            let size = Px(28.0);
            let mut content_props = ContainerProps::default();
            content_props.layout.size.width = Length::Px(size);
            content_props.layout.size.height = Length::Px(size);
            content_props.layout.flex.shrink = 0.0;
            content_props.background = Some(bg);
            content_props.corner_radii =
                fret_core::Corners::all(theme.metric_required("metric.radius.sm"));
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

            (pressable, vec![content])
        })
    }
}

#[derive(Debug, Default, Clone)]
struct TerminalContentState {
    handle: ScrollHandle,
    last_hash: u64,
    pending_scroll_frames: u8,
    initialized: bool,
}

#[derive(Clone, Default)]
pub struct TerminalContent {
    viewport_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TerminalContent {
    pub fn new() -> Self {
        Self {
            viewport_test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_h(Px(384.0)),
            chrome: ChromeRefinement::default().p(Space::N4),
        }
    }

    pub fn viewport_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(id.into());
        self
    }

    fn viewport_test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.viewport_test_id = id;
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
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        let theme = Theme::global(&*cx.app).clone();

        let output = cx
            .get_model_cloned(&controller.output, Invalidation::Layout)
            .unwrap_or_default();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        output.hash(&mut hasher);
        let output_hash = hasher.finish();

        let handle = cx.with_state(TerminalContentState::default, |st| st.handle.clone());

        cx.with_state(TerminalContentState::default, |st| {
            if !st.initialized {
                st.initialized = true;
                st.last_hash = output_hash;
                if controller.auto_scroll {
                    st.pending_scroll_frames = 2;
                }
            } else if controller.auto_scroll && st.last_hash != output_hash {
                st.pending_scroll_frames = 2;
                st.last_hash = output_hash;
            }

            if controller.auto_scroll && st.pending_scroll_frames > 0 {
                let next = Point::new(handle.offset().x, handle.max_offset().y);
                handle.scroll_to_offset(next);
                st.pending_scroll_frames = st.pending_scroll_frames.saturating_sub(1);
            }
        });

        let mut display = output;
        if controller.is_streaming {
            display.push('█');
        }

        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(display),
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("component.text.sm_px"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("component.text.sm_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(zinc_100()),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
        });

        let chrome = ChromeRefinement::default().p(Space::N4).merge(self.chrome);
        let mut content_props = decl_style::container_props(
            &theme,
            chrome,
            LayoutRefinement::default().w_full().min_w_0(),
        );
        content_props.background = None;
        content_props.border = Edges::all(Px(0.0));
        content_props.border_color = None;

        let content = cx.container(content_props, move |_cx| vec![text]);

        let mut scroll = ScrollArea::new(vec![content])
            .scroll_handle(handle)
            .refine_layout(self.layout);
        if let Some(test_id) = self.viewport_test_id.clone() {
            scroll = scroll.viewport_test_id(test_id);
        }
        scroll.into_element(cx)
    }
}
