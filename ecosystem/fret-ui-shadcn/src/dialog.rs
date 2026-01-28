use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Point, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_icons::ids;
use fret_runtime::{Model, ModelId};
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, OpacityProps, Overflow,
    PositionStyle, PressableA11y, PressableProps, RingPlacement, RingStyle, SemanticsProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Space, ui,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

fn default_overlay_color() -> Color {
    Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.5,
    }
}

/// shadcn/ui `Dialog` (v4).
///
/// This is a modal overlay (barrier-backed) installed via the component-layer overlay manager
/// (`fret-ui-kit/overlay_controller.rs`).
///
/// Notes:
/// - Dismiss on Escape is handled by the shared dismissible root (ADR 0067).
/// - Overlay click-to-dismiss is implemented here by rendering a full-window barrier behind the
///   dialog content.
#[derive(Clone)]
pub struct Dialog {
    open: Model<bool>,
    overlay_closable: bool,
    overlay_color: Option<Color>,
    window_padding: Space,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
}

impl std::fmt::Debug for Dialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialog")
            .field("open", &"<model>")
            .field("overlay_closable", &self.overlay_closable)
            .field("overlay_color", &self.overlay_color)
            .field("window_padding", &self.window_padding)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl Dialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            overlay_closable: true,
            overlay_color: None,
            window_padding: Space::N4,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }

    /// Creates a dialog with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_dialog::DialogRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        Self::new(open)
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.overlay_closable = overlay_closable;
        self
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    pub fn window_padding(mut self, padding: Space) -> Self {
        self.window_padding = padding;
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    /// Installs an open auto-focus hook (Radix `FocusScope` `onMountAutoFocus`).
    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    /// Installs a close auto-focus hook (Radix `FocusScope` `onUnmountAutoFocus`).
    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx
                .watch_model(&self.open)
                .layout()
                .copied()
                .unwrap_or(false);
            let open_id: ModelId = self.open.id();

            #[derive(Default)]
            struct DialogA11yState {
                content_element: Option<fret_ui::elements::GlobalElementId>,
            }

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = radix_dialog::dialog_root_name(id);
            let prev_content_element =
                cx.with_state(DialogA11yState::default, |st| st.content_element);

            let motion = OverlayController::transition_with_durations_and_easing(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_200,
                overlay_motion::SHADCN_MOTION_TICKS_200,
                overlay_motion::shadcn_ease,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            #[derive(Default)]
            struct DialogFocusRestoreState {
                was_open: bool,
                restore_element: Option<fret_ui::elements::GlobalElementId>,
            }

            let focused_element = cx.focused_element();
            let restore_element = cx.with_state(DialogFocusRestoreState::default, |st| {
                if is_open && !st.was_open {
                    st.restore_element = focused_element;
                    st.was_open = true;
                } else if !overlay_presence.present {
                    st.was_open = false;
                    st.restore_element = None;
                } else if !is_open {
                    st.was_open = false;
                }
                st.restore_element
            });
            let restore_trigger = restore_element.unwrap_or(id);

            let content_element_for_trigger: std::cell::Cell<
                Option<fret_ui::elements::GlobalElementId>,
            > = std::cell::Cell::new(None);

            if overlay_presence.present {
                let on_dismiss_request_for_barrier = self.on_dismiss_request.clone();
                let on_dismiss_request_for_request = self.on_dismiss_request.clone();
                let on_open_auto_focus = self.on_open_auto_focus.clone();
                let on_close_auto_focus = self.on_close_auto_focus.clone();

                let overlay_color = self.overlay_color.unwrap_or_else(default_overlay_color);
                let overlay_closable = self.overlay_closable;
                let window_padding_px = MetricRef::space(self.window_padding).resolve(&theme);

                let opacity = motion.progress;
                let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                    let barrier_fill = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges::all(Px(0.0)),
                            background: Some(overlay_color),
                            shadow: None,
                            border: Edges::all(Px(0.0)),
                            border_color: None,
                            corner_radii: Corners::all(Px(0.0)),
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );

                    let outer = cx.bounds;
                    let available_w = Px((outer.size.width.0 - window_padding_px.0 * 2.0).max(0.0));
                    let available_h =
                        Px((outer.size.height.0 - window_padding_px.0 * 2.0).max(0.0));

                    crate::a11y_modal::begin_modal_a11y_scope(cx.app, open_id);
                    let content = content(cx);
                    let content_id = content.id;
                    content_element_for_trigger.set(Some(content_id));
                    let max_w_hint =
                        crate::a11y_modal::modal_content_max_width_for_current_scope(cx.app)
                            .unwrap_or(Px(512.0));
                    crate::a11y_modal::end_modal_a11y_scope(cx.app, open_id);
                    let last_size = cx.last_bounds_for_element(content_id).map(|r| r.size);

                    // Upstream uses `w-full` + `max-w-*`, so dialog width should not collapse to
                    // intrinsic content. We compute a width hint from the modal content and clamp
                    // it to the available viewport space (matches `max-w-[calc(100%-2rem)]`).
                    let content_w = Px(max_w_hint.0.min(available_w.0).max(0.0));

                    // Height remains content-driven; use last-frame bounds as a stable anchor for
                    // the open zoom transform origin and placement.
                    let desired_h = last_size.map(|s| s.height).unwrap_or(Px(320.0));
                    let content_h = Px(desired_h.0.max(0.0));

                    let left = Px(outer.origin.x.0
                        + window_padding_px.0
                        + ((available_w.0 - content_w.0) * 0.5).max(0.0));
                    let top = Px(outer.origin.y.0
                        + window_padding_px.0
                        + (available_h.0 - content_h.0) * 0.5);

                    let origin = Point::new(
                        Px(left.0 + content_w.0 * 0.5),
                        Px(top.0 + content_h.0 * 0.5),
                    );
                    let zoom = overlay_motion::shadcn_zoom_transform(origin, opacity);

                    let dialog_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(top),
                            left: Some(left),
                            right: None,
                            bottom: None,
                        },
                        size: SizeStyle {
                            width: Length::Px(content_w),
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };
                    let dialog_positioned = cx.container(
                        ContainerProps {
                            layout: dialog_layout,
                            ..Default::default()
                        },
                        move |_cx| vec![content],
                    );
                    let dialog = overlay_motion::wrap_opacity_and_render_transform(
                        cx,
                        opacity,
                        zoom,
                        vec![dialog_positioned],
                    );

                    let opacity_layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let barrier = cx.opacity_props(
                        OpacityProps {
                            layout: opacity_layout,
                            opacity,
                        },
                        move |_cx| vec![barrier_fill],
                    );
                    let open_for_children = self.open.clone();
                    let dialog_options = radix_dialog::DialogOptions::default()
                        .dismiss_on_overlay_press(overlay_closable)
                        .initial_focus(None)
                        .on_open_auto_focus(on_open_auto_focus.clone())
                        .on_close_auto_focus(on_close_auto_focus.clone());
                    radix_dialog::modal_dialog_layer_elements_with_dismiss_handler(
                        cx,
                        open_for_children.clone(),
                        dialog_options,
                        on_dismiss_request_for_barrier.clone(),
                        [barrier],
                        dialog,
                    )
                });

                if let Some(content_element) = content_element_for_trigger.get() {
                    cx.with_state(DialogA11yState::default, |st| {
                        st.content_element = Some(content_element)
                    });
                }

                let dialog_options = radix_dialog::DialogOptions::default()
                    .dismiss_on_overlay_press(overlay_closable)
                    .initial_focus(None)
                    .on_open_auto_focus(on_open_auto_focus)
                    .on_close_auto_focus(on_close_auto_focus);
                let request = radix_dialog::modal_dialog_request_with_options_and_dismiss_handler(
                    id,
                    restore_trigger,
                    self.open,
                    overlay_presence,
                    dialog_options,
                    on_dismiss_request_for_request,
                    overlay_children,
                );
                radix_dialog::request_modal_dialog(cx, request);
            }

            let content_element = content_element_for_trigger.get().or(prev_content_element);
            radix_dialog::apply_dialog_trigger_a11y(trigger, is_open, content_element)
        })
    }
}

/// shadcn/ui `DialogContent` (v4).
#[derive(Debug, Clone)]
pub struct DialogContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl DialogContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let chrome = crate::ui_builder_ext::surfaces::dialog_style_chrome().merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_full()
            .max_w(Px(512.0))
            .merge(self.layout);

        if let Some(max_w) = layout
            .size
            .as_ref()
            .and_then(|s| s.max_width.as_ref())
            .map(|m| m.resolve(&theme))
        {
            crate::a11y_modal::register_modal_content_max_width(cx.app, max_w);
        }

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        let container = shadcn_layout::container_vstack_gap(cx, props, Space::N4, children);

        let (labelled_by_element, described_by_element) =
            crate::a11y_modal::modal_relations_for_current_scope(cx.app);
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Dialog,
                labelled_by_element,
                described_by_element,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

/// shadcn/ui `DialogClose` (v4-aligned recipe).
///
/// Upstream shadcn's `DialogContent` renders a close affordance wired to the underlying Radix
/// primitive. Fret exposes this as an explicit building block so apps can choose to include it (or
/// replace it) while keeping the modal overlay policy decoupled from visuals.
#[derive(Clone)]
pub struct DialogClose {
    open: Model<bool>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for DialogClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogClose")
            .field("open", &"<model>")
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl DialogClose {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let fg = theme
                .color_by_key("muted.foreground")
                .or_else(|| theme.color_by_key("muted-foreground"))
                .unwrap_or_else(|| theme.color_required("muted.foreground"));

            let a11y_label: Arc<str> = Arc::from("Close");
            let open = self.open.clone();

            let radius = Px(2.0);

            let base_layout = LayoutRefinement::default()
                .absolute()
                .top(Space::N4)
                .right(Space::N4)
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, base_layout);

            let user_chrome = self.chrome;
            let user_bg_override = user_chrome.background.is_some();

            control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_set_bool(&open, false);

                let hovered = st.hovered;
                let pressed = st.pressed;

                // new-york-v4: `rounded-xs opacity-70 hover:opacity-100` (no default hover bg).
                let mut chrome = ChromeRefinement::default();
                chrome.radius = Some(radius.into());
                if !user_bg_override {
                    chrome.background = Some(ColorRef::Color(Color::TRANSPARENT));
                }
                chrome = chrome.merge(user_chrome.clone());

                let mut chrome_props =
                    decl_style::container_props(&theme, chrome, LayoutRefinement::default());
                chrome_props.layout.size = pressable_layout.size;

                let ring_color = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_required("ring"));
                let ring_offset_bg = theme
                    .color_by_key("ring-offset-background")
                    .unwrap_or_else(|| theme.color_required("ring-offset-background"));

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: true,
                    focusable: true,
                    focus_ring: Some(RingStyle {
                        placement: RingPlacement::Outset,
                        width: Px(2.0),
                        offset: Px(2.0),
                        color: ring_color,
                        offset_color: Some(ring_offset_bg),
                        corner_radii: Corners::all(radius),
                    }),
                    a11y: PressableA11y {
                        label: Some(a11y_label.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let children = move |cx: &mut ElementContext<'_, H>| {
                    let opacity = if hovered || pressed { 1.0 } else { 0.7 };
                    let icon = decl_icon::icon_with(
                        cx,
                        ids::ui::CLOSE,
                        Some(Px(16.0)),
                        Some(ColorRef::Color(fg)),
                    );
                    let icon = cx.opacity(opacity, move |_cx| vec![icon]);

                    vec![fret_ui_kit::declarative::stack::hstack(
                        cx,
                        fret_ui_kit::declarative::stack::HStackProps::default()
                            .justify_center()
                            .items_center(),
                        |_cx| vec![icon],
                    )]
                };

                (pressable_props, chrome_props, children)
            })
        })
    }
}

/// shadcn/ui `DialogHeader` (v4).
#[derive(Debug, Clone)]
pub struct DialogHeader {
    children: Vec<AnyElement>,
}

impl DialogHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pb(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N1p5, children)
    }
}

/// shadcn/ui `DialogFooter` (v4).
#[derive(Debug, Clone)]
pub struct DialogFooter {
    children: Vec<AnyElement>,
}

impl DialogFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pt(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_hstack(
            cx,
            props,
            fret_ui_kit::declarative::stack::HStackProps::default()
                .gap(Space::N2)
                .justify_end()
                .items_center(),
            children,
        )
    }
}

/// shadcn/ui `DialogTitle` (v4).
#[derive(Debug, Clone)]
pub struct DialogTitle {
    text: Arc<str>,
}

impl DialogTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_required("foreground"));

        let px = theme
            .metric_by_key("component.dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.dialog.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        let title = ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .letter_spacing_em(-0.02)
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx);
        crate::a11y_modal::register_modal_title(cx.app, title.id);
        title
    }
}

/// shadcn/ui `DialogDescription` (v4).
#[derive(Debug, Clone)]
pub struct DialogDescription {
    text: Arc<str>,
}

impl DialogDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or_else(|| theme.color_required("muted.foreground"));

        let px = theme
            .metric_by_key("component.dialog.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.dialog.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        let description = ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx);
        crate::a11y_modal::register_modal_description(cx.app, description.id);
        description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{
        KeyCode, Modifiers, Px, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::Effect;
    use fret_ui::UiTree;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    #[test]
    fn dialog_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let dialog = Dialog::new_controllable(cx, Some(controlled.clone()), false);
            assert_eq!(dialog.open, controlled);
        });
    }

    #[test]
    fn dialog_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let dialog = Dialog::new_controllable(cx, None, true);
            let open = cx
                .watch_model(&dialog.open)
                .layout()
                .copied()
                .unwrap_or(false);
            assert!(open);
        });
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_dialog_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        close_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let dialog = Dialog::new(open.clone())
                    .overlay_closable(overlay_closable)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let focusable = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(200.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    initial_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let close = DialogClose::new(open.clone()).into_element(cx);
                            close_id_out.set(Some(close.id));

                            let content =
                                DialogContent::new(vec![focusable, close]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![dialog]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn apply_command_effects(ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices) {
        let effects = app.flush_effects();
        for effect in effects {
            let Effect::Command { window: _, command } = effect else {
                continue;
            };
            let _ = ui.dispatch_command(app, services, &command);
        }
    }

    #[test]
    fn dialog_overlay_click_closes_when_overlay_closable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // First frame: render closed.
        let trigger = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Second frame: render open + overlay.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert!(content_id.get().is_some());

        // Click inside content should not close.
        let content_rect = content_id
            .get()
            .and_then(|id| fret_ui::elements::node_for_element(&mut app, window, id))
            .and_then(|node| ui.debug_node_bounds(node))
            .expect("content bounds");
        let inside = Point::new(
            Px(content_rect.origin.x.0 + content_rect.size.width.0 * 0.5),
            Px(content_rect.origin.y.0 + content_rect.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: inside,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: inside,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Click outside content should close via barrier.
        let mut outside = Point::new(Px(bounds.origin.x.0 + 4.0), Px(bounds.origin.y.0 + 4.0));
        if content_rect.contains(outside) {
            outside = Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 - 4.0),
                Px(bounds.origin.y.0 + bounds.size.height.0 - 4.0),
            );
        }
        assert!(
            !content_rect.contains(outside),
            "expected an outside point that is not inside the dialog content"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        let _ = trigger;
    }

    #[test]
    fn dialog_overlay_click_does_not_close_when_not_overlay_closable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Render open.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            content_id.clone(),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click outside content should not close.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(4.0), Px(4.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(4.0), Px(4.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn dialog_escape_closes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn dialog_escape_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let dismiss_reason: Rc<Cell<Option<fret_ui::action::DismissReason>>> =
            Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            dismiss_reason_cell.set(Some(req.reason));
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st| {
                        cx.pressable_toggle_bool(&open);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let dialog = Dialog::new(open.clone())
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            DialogContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            dismiss_reason.get(),
            Some(fret_ui::action::DismissReason::Escape)
        );
    }

    #[test]
    fn dialog_overlay_click_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let dismiss_reason: Rc<Cell<Option<fret_ui::action::DismissReason>>> =
            Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            dismiss_reason_cell.set(Some(req.reason));
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.top = Some(Px(0.0));
                            layout.inset.left = Some(Px(0.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        Vec::new()
                    },
                );

                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.top = Some(Px(200.0));
                            layout.inset.left = Some(Px(200.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let dialog = Dialog::new(open.clone())
                    .overlay_closable(true)
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            DialogContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![underlay, dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click the underlay area. The modal barrier should catch the click and route it through
        // the dismiss handler without closing.
        let point = Point::new(Px(4.0), Px(4.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should not activate while modal dialog is open"
        );
        let reason = dismiss_reason.get();
        let Some(fret_ui::action::DismissReason::OutsidePress { pointer }) = reason else {
            panic!("expected outside-press dismissal, got {reason:?}");
        };
        let Some(cx) = pointer else {
            panic!("expected pointer payload for outside-press dismissal");
        };
        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
        assert_eq!(cx.button, fret_core::MouseButton::Left);
        assert_eq!(cx.modifiers, fret_core::Modifiers::default());
        assert_eq!(cx.click_count, 1);
    }

    #[test]
    fn dialog_focuses_first_focusable_on_open_and_restores_trigger_on_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // First frame: closed.
        let trigger = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            initial_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Second frame: open.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            initial_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus_element_id = initial_focus_cell.get().expect("initial focus element id");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus_element_id)
                .expect("initial focus node");
        assert_eq!(ui.focus(), Some(initial_focus_node));

        // Close via Escape and render one more frame to apply focus restore policy.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Render a few frames to allow the close animation to finish and the overlay manager to
        // apply focus restore when the layer is uninstalled.
        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 1;
        for _ in 0..settle_frames {
            let _ = render_dialog_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
                content_id.clone(),
                initial_focus_cell.clone(),
                Rc::new(Cell::new(None)),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dialog_close_button_closes_and_restores_trigger_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let close_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed.
        let trigger = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            initial_focus_cell.clone(),
            close_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open (capture close bounds).
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            initial_focus_cell.clone(),
            close_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let close_id = close_cell.get().expect("close element id");
        let close_node =
            fret_ui::elements::node_for_element(&mut app, window, close_id).expect("close node");
        let close_bounds = ui.debug_node_bounds(close_node).expect("close bounds");
        let click = Point::new(
            Px(close_bounds.origin.x.0 + close_bounds.size.width.0 * 0.5),
            Px(close_bounds.origin.y.0 + close_bounds.size.height.0 * 0.5),
        );

        // Click close and ensure model closes.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Render a few frames to allow presence to complete and focus restore to apply.
        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 1;
        for _ in 0..settle_frames {
            let _ = render_dialog_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
                content_id.clone(),
                initial_focus_cell.clone(),
                close_cell.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dialog_tab_traversal_wraps_within_modal_layer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let first_focusable_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let second_focusable_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed.
        let first_focusable_id_frame1 = first_focusable_id.clone();
        let second_focusable_id_frame1 = second_focusable_id.clone();
        OverlayController::begin_frame(&mut app, window);
        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let first_focusable_id = first_focusable_id_frame1.clone();
                let second_focusable_id = second_focusable_id_frame1.clone();

                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let dialog = Dialog::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let first = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(200.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                first_focusable_id.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let second = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(200.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                second_focusable_id.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        DialogContent::new(vec![first, second]).into_element(cx)
                    },
                );

                vec![dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        let trigger_element = trigger_id.expect("trigger id");
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open.
        let first_focusable_id_frame2 = first_focusable_id.clone();
        let second_focusable_id_frame2 = second_focusable_id.clone();
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let first_focusable_id = first_focusable_id_frame2.clone();
                let second_focusable_id = second_focusable_id_frame2.clone();

                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        let _ = id;
                        cx.pressable_toggle_bool(&open);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let dialog = Dialog::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let first = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(200.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                first_focusable_id.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let second = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(200.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                second_focusable_id.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        DialogContent::new(vec![first, second]).into_element(cx)
                    },
                );
                vec![dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let first_element_id = first_focusable_id
            .get()
            .expect("first focusable element id");
        let second_element_id = second_focusable_id
            .get()
            .expect("second focusable element id");
        let first_node =
            fret_ui::elements::node_for_element(&mut app, window, first_element_id).expect("first");
        let second_node = fret_ui::elements::node_for_element(&mut app, window, second_element_id)
            .expect("second");

        assert_eq!(ui.focus(), Some(first_node));

        // Tab -> next
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_eq!(ui.focus(), Some(second_node));

        // Tab -> wrap
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_eq!(ui.focus(), Some(first_node));

        // Shift+Tab -> previous (wrap)
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_eq!(ui.focus(), Some(second_node));

        // Sanity: focus must never escape to the trigger while modal is open.
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger");
        assert_ne!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dialog_content_exposes_labelled_by_and_described_by_relations() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                OverlayController::begin_frame(app, window);

                let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "dialog-a11y-relations",
                    |cx| {
                        let trigger = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                cx.pressable_toggle_bool(&open);
                                trigger_id = Some(id);
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let dialog = Dialog::new(open.clone()).into_element(
                            cx,
                            |_cx| trigger,
                            |cx| {
                                let title = DialogTitle::new("Dialog Title").into_element(cx);
                                let description =
                                    DialogDescription::new("Dialog Description").into_element(cx);
                                DialogContent::new(vec![title, description]).into_element(cx)
                            },
                        );

                        vec![dialog]
                    },
                );

                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                trigger_id.expect("trigger id")
            };

        // Frame 1: closed.
        let _trigger = render_frame(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open + semantics snapshot.
        let _ = render_frame(&mut ui, &mut app, &mut services);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let dialog = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Dialog)
            .expect("dialog semantics node");
        let title = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref() == Some("Dialog Title")
            })
            .expect("title semantics node");
        let description = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref() == Some("Dialog Description")
            })
            .expect("description semantics node");

        assert!(
            dialog.labelled_by.iter().any(|id| *id == title.id),
            "dialog should be labelled by its title"
        );
        assert!(
            dialog.described_by.iter().any(|id| *id == description.id),
            "dialog should be described by its description"
        );
    }
}
