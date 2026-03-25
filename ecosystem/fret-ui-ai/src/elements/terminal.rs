//! AI Elements-aligned `Terminal` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/terminal.tsx`.

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use fret_core::{Color, Edges, Point, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SemanticsDecoration,
    SizeStyle, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::chrome::centered_fixed_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::facade::ScrollArea;

use super::Shimmer;
use super::clipboard_copy::{
    ClipboardCopyFeedbackRef, begin_request, finish_request, handle_reset_timer,
};
pub type OnTerminalClear = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

fn token(key: &'static str, fallback: Color) -> ColorRef {
    ColorRef::Token {
        key,
        fallback: ColorFallback::Color(fallback),
    }
}

fn color(theme: &Theme, key: &'static str, fallback: Color) -> Color {
    theme.color_by_key(key).unwrap_or(fallback)
}

fn zinc_950() -> Color {
    // Tailwind zinc-950 (#09090b).
    fret_ui_kit::colors::linear_from_hex_rgb(0x09_09_0B)
}

fn zinc_100() -> Color {
    // Tailwind zinc-100 (#f4f4f5).
    fret_ui_kit::colors::linear_from_hex_rgb(0xF4_F4_F5)
}

fn zinc_800() -> Color {
    // Tailwind zinc-800 (#27272a).
    fret_ui_kit::colors::linear_from_hex_rgb(0x27_27_2A)
}

fn zinc_400() -> Color {
    // Tailwind zinc-400 (#a1a1aa).
    fret_ui_kit::colors::linear_from_hex_rgb(0xA1_A1_AA)
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

fn use_terminal_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TerminalController> {
    cx.provided::<TerminalController>().cloned()
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

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, TerminalController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(token("component.terminal.bg", zinc_950()))
            .border_color(token("component.terminal.border", zinc_800()))
            .text_color(token("component.terminal.fg", zinc_100()))
            .merge(self.chrome);

        let controller = TerminalController {
            output: self.output.clone(),
            is_streaming: self.is_streaming,
            auto_scroll: self.auto_scroll,
            on_clear: self.on_clear.clone(),
        };
        let root = cx.provide(controller.clone(), |cx| {
            let children = children(cx, controller.clone());

            let inner = ui::v_stack(move |_cx| children)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            cx.container(
                decl_style::container_props(&theme, root_chrome, self.layout),
                move |_cx| vec![inner],
            )
        });

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let test_id_copy = self.test_id_copy.clone();
        let copied_marker_test_id = self.copied_marker_test_id.clone();
        let test_id_clear = self.test_id_clear.clone();
        let test_id_viewport = self.test_id_viewport.clone();

        self.into_element_with_children(cx, move |cx, _controller| {
            let header = TerminalHeader::new()
                .children([
                    TerminalTitle::new().into_element(cx),
                    ui::h_row(move |cx| {
                        vec![
                            TerminalStatus::new().into_element(cx),
                            TerminalActions::new()
                                .children([
                                    TerminalCopyButton::new()
                                        .test_id_opt(test_id_copy.clone())
                                        .copied_marker_test_id_opt(copied_marker_test_id.clone())
                                        .into_element(cx),
                                    TerminalClearButton::new()
                                        .test_id_opt(test_id_clear.clone())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                        ]
                    })
                    .items(Items::Center)
                    .gap(Space::N1)
                    .justify(Justify::End)
                    .into_element(cx),
                ])
                .into_element(cx);

            let content = TerminalContent::new()
                .viewport_test_id_opt(test_id_viewport.clone())
                .into_element(cx);

            vec![header, content]
        })
    }
}

#[derive(Default)]
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

        let hstack = ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .justify(Justify::Between)
            .items(Items::Center)
            .into_element(cx);

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
        props.border_color = Some(color(&theme, "component.terminal.border", zinc_800()));

        cx.container(props, move |_cx| vec![hstack])
    }
}

pub struct TerminalTitle {
    content: TerminalTitleContent,
    icon: IconId,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

enum TerminalTitleContent {
    Label(Arc<str>),
    Children(Vec<AnyElement>),
}

impl std::fmt::Debug for TerminalTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("TerminalTitle");
        match &self.content {
            TerminalTitleContent::Label(label) => debug.field("label", &label.as_ref()),
            TerminalTitleContent::Children(children) => {
                debug.field("children_len", &children.len())
            }
        };
        debug
            .field("icon", &self.icon)
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Default for TerminalTitle {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalTitle {
    pub fn new() -> Self {
        Self {
            content: TerminalTitleContent::Label(Arc::<str>::from("Terminal")),
            icon: IconId::new_static("lucide.terminal"),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: TerminalTitleContent::Children(children.into_iter().collect()),
            icon: IconId::new_static("lucide.terminal"),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.content = TerminalTitleContent::Label(label.into());
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
        let fg = color(&theme, "component.terminal.muted_fg", zinc_400());
        let refinement = typography::composable_preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        );

        let icon = decl_icon::icon_with(cx, self.icon, Some(Px(16.0)), Some(ColorRef::Color(fg)));
        let content = match self.content {
            TerminalTitleContent::Label(label) => vec![
                icon,
                fret_ui_kit::ui::raw_text(label)
                    .wrap(TextWrap::None)
                    .overflow(TextOverflow::Clip)
                    .into_element(cx),
            ],
            TerminalTitleContent::Children(children) => {
                let mut row = Vec::with_capacity(children.len() + 1);
                row.push(icon);
                row.extend(children);
                row
            }
        };

        ui::h_row(move |_cx| content)
            .layout(self.layout)
            .items(Items::Center)
            .gap(Space::N2)
            .into_element(cx)
            .inherit_foreground(fg)
            .inherit_text_style(refinement)
    }
}

#[derive(Default)]
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

        let theme = Theme::global(&*cx.app).clone();
        let status_fg = color(&theme, "component.terminal.muted_fg", zinc_400());
        let children = if self.children.is_empty() {
            vec![fret_ui_kit::typography::scope_text_style_with_color(
                Shimmer::new(Arc::<str>::from("Streaming"))
                    .use_resolved_passive_text()
                    .refine_layout(LayoutRefinement::default().w_px(Px(64.0)))
                    .into_element(cx),
                typography::preset_text_refinement(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                ),
                status_fg,
            )]
        } else {
            self.children
        };

        ui::h_row(move |_cx| children)
            .layout(self.layout)
            .items(Items::Center)
            .gap(Space::N2)
            .into_element(cx)
    }
}

#[derive(Default)]
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
        ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .items(Items::Center)
            .gap(Space::N1)
            .into_element(cx)
    }
}

#[derive(Clone)]
pub struct TerminalCopyButton {
    on_copy: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    on_error: Option<
        Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, fret_core::ClipboardAccessError) + 'static>,
    >,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl TerminalCopyButton {
    pub fn new() -> Self {
        Self {
            on_copy: None,
            on_error: None,
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

    pub fn on_error(
        mut self,
        on_error: Arc<
            dyn Fn(&mut dyn UiActionHost, ActionCx, fret_core::ClipboardAccessError) + 'static,
        >,
    ) -> Self {
        self.on_error = Some(on_error);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.slot_state(ClipboardCopyFeedbackRef::default, |st| st.clone());

        let output = controller.output;
        let on_copy = self.on_copy;
        let on_error = self.on_error;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            let copied = feedback.is_copied();
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
                        if !handle_reset_timer(&feedback, token) {
                            return false;
                        }
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }
                }),
            );

            cx.pressable_on_clipboard_write_completed({
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                let on_error = on_error.clone();
                Arc::new(move |host, action_cx, token, outcome| {
                    let Some(result) =
                        finish_request(&feedback, token, outcome, || host.next_timer_token())
                    else {
                        return false;
                    };

                    if let Some(prev_reset) = result.prev_reset {
                        host.push_effect(Effect::CancelTimer { token: prev_reset });
                    }
                    if let Some(reset_token) = result.next_reset {
                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token: reset_token,
                            after: timeout,
                            repeat: None,
                        });
                    }
                    if let Some(error) = result.error {
                        if let Some(on_error) = on_error.as_ref() {
                            on_error(host, action_cx, error);
                        }
                    } else if let Some(on_copy) = on_copy.as_ref() {
                        on_copy(host, action_cx);
                    }
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                    true
                })
            });

            cx.pressable_on_activate({
                let feedback = feedback.clone();
                let output = output.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let text = host
                        .models_mut()
                        .read(&output, |v| v.clone())
                        .unwrap_or_default();

                    let Some(request) = begin_request(&feedback, || host.next_clipboard_token())
                    else {
                        return;
                    };

                    if let Some(prev_reset) = request.prev_reset {
                        host.push_effect(Effect::CancelTimer { token: prev_reset });
                    }
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token: request.clipboard_token,
                        text,
                    });
                })
            });

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(label);
            pressable.a11y.test_id = test_id.clone();

            let fg = if st.hovered || st.pressed {
                color(&theme, "component.terminal.fg", zinc_100())
            } else {
                color(&theme, "component.terminal.muted_fg", zinc_400())
            };
            let bg = if st.pressed || st.hovered {
                color(&theme, "component.terminal.border", zinc_800())
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

            let mut chrome_props = ContainerProps::default();
            chrome_props.layout.size.width = Length::Px(size);
            chrome_props.layout.size.height = Length::Px(size);
            chrome_props.layout.flex.shrink = 0.0;
            chrome_props.background = Some(bg);
            chrome_props.corner_radii =
                fret_core::Corners::all(theme.metric_token("metric.radius.sm"));
            chrome_props.border = Edges::all(Px(0.0));
            chrome_props.padding = Edges::all(Px(0.0)).into();

            (pressable, chrome_props, move |cx| {
                let row = ui::h_row(move |_cx| vec![icon])
                    .items(Items::Center)
                    .justify(Justify::Center)
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .into_element(cx);

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
                            ink_overflow: Default::default(),
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_terminal_controller(cx) else {
            return cx.text("");
        };
        let Some(on_clear) = controller.on_clear else {
            return cx.text("");
        };
        let theme = Theme::global(&*cx.app).clone();

        let test_id = self.test_id;
        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
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
                color(&theme, "component.terminal.fg", zinc_100())
            } else {
                color(&theme, "component.terminal.muted_fg", zinc_400())
            };
            let bg = if st.pressed || st.hovered {
                color(&theme, "component.terminal.border", zinc_800())
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
            let mut chrome_props = ContainerProps::default();
            chrome_props.layout.size.width = Length::Px(size);
            chrome_props.layout.size.height = Length::Px(size);
            chrome_props.layout.flex.shrink = 0.0;
            chrome_props.background = Some(bg);
            chrome_props.corner_radii =
                fret_core::Corners::all(theme.metric_token("metric.radius.sm"));
            chrome_props.border = Edges::all(Px(0.0));
            chrome_props.padding = Edges::all(Px(0.0)).into();

            (pressable, chrome_props, move |cx| {
                vec![
                    ui::h_row(move |_cx| vec![icon])
                        .items(Items::Center)
                        .justify(Justify::Center)
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .into_element(cx),
                ]
            })
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

        let handle = cx.root_state(TerminalContentState::default, |st| st.handle.clone());

        cx.root_state(TerminalContentState::default, |st| {
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
            style: Some(
                typography::TypographyPreset::control_monospace(typography::UiTextSize::Sm)
                    .resolve(&theme),
            ),
            color: Some(color(&theme, "component.terminal.fg", zinc_100())),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    fn find_text_by_content<'a>(element: &'a AnyElement, content: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Text(props) = &element.kind
            && props.text.as_ref() == content
        {
            return Some(element);
        }

        element
            .children
            .iter()
            .find_map(|child| find_text_by_content(child, content))
    }

    fn has_scoped_text_style(
        element: &AnyElement,
        refinement: &fret_core::TextStyleRefinement,
        foreground: fret_core::Color,
    ) -> bool {
        if element.inherited_text_style.as_ref() == Some(refinement)
            && element.inherited_foreground == Some(foreground)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_scoped_text_style(child, refinement, foreground))
    }

    fn has_test_id(element: &AnyElement, expected: &str) -> bool {
        element
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.test_id.as_deref())
            .is_some_and(|test_id| test_id == expected)
            || match &element.kind {
                ElementKind::Pressable(props) => props.a11y.test_id.as_deref() == Some(expected),
                ElementKind::Semantics(props) => props.test_id.as_deref() == Some(expected),
                _ => false,
            }
            || element
                .children
                .iter()
                .any(|child| has_test_id(child, expected))
    }

    #[test]
    fn terminal_title_children_scope_inherited_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                TerminalTitle::new_children([cx.text("Build Output")]).into_element(cx)
            });

        let theme = fret_ui::Theme::global(&app).clone();
        let expected_refinement = typography::composable_preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        );
        let expected_fg = color(&theme, "component.terminal.muted_fg", zinc_400());
        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_fg
        ));

        let text =
            find_text_by_content(&element, "Build Output").expect("expected terminal title text");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected terminal title leaf to be text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
    }

    #[test]
    fn terminal_root_provides_controller_to_default_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let output = app.models_mut().insert(String::from("cargo check"));

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Terminal::new(output.clone())
                    .is_streaming(true)
                    .on_clear(Arc::new(|_host, _action_cx| {}))
                    .test_id_clear("terminal-clear")
                    .test_id_viewport("terminal-viewport")
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "terminal-clear"));
        assert!(has_test_id(&element, "terminal-viewport"));
        assert!(find_text_by_content(&element, "Streaming").is_some());
    }

    #[test]
    fn terminal_status_default_streaming_message_scopes_inherited_typography_for_shimmer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let output = app.models_mut().insert(String::new());

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                cx.provide(
                    TerminalController {
                        output: output.clone(),
                        is_streaming: true,
                        auto_scroll: true,
                        on_clear: None,
                    },
                    |cx| TerminalStatus::new().into_element(cx),
                )
            });

        let theme = fret_ui::Theme::global(&app).clone();
        let expected_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
        );
        let expected_fg = color(&theme, "component.terminal.muted_fg", zinc_400());

        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_fg
        ));

        let text = find_text_by_content(&element, "Streaming")
            .expect("expected default terminal streaming label");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected text leaf under the scoped terminal status");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
    }
}
