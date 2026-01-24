use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Point, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    RenderTransformProps, SemanticsProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::alert_dialog as radix_alert_dialog;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Radius, Space, ui,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

use crate::button::{Button, ButtonVariant};

fn default_overlay_color() -> Color {
    Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.5,
    }
}

/// shadcn/ui `AlertDialog` (v4).
///
/// This is a modal overlay (barrier-backed). Unlike `Dialog`, the overlay is not closable by
/// default (Radix/shadcn behavior).
#[derive(Clone)]
pub struct AlertDialog {
    open: Model<bool>,
    overlay_color: Option<Color>,
    window_padding: Space,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
}

impl std::fmt::Debug for AlertDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialog")
            .field("open", &"<model>")
            .field("overlay_color", &self.overlay_color)
            .field("window_padding", &self.window_padding)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl AlertDialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            overlay_color: None,
            window_padding: Space::N6,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }

    /// Creates an alert dialog with a controlled/uncontrolled open model (Radix `open` /
    /// `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_alert_dialog::AlertDialogRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        Self::new(open)
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    pub fn window_padding(mut self, padding: Space) -> Self {
        self.window_padding = padding;
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
            let open_id = self.open.id();

            #[derive(Default)]
            struct AlertDialogA11yState {
                content_element: Option<fret_ui::elements::GlobalElementId>,
            }

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = radix_alert_dialog::alert_dialog_root_name(id);
            let prev_content_element =
                cx.with_state(AlertDialogA11yState::default, |st| st.content_element);

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

            let content_element_for_trigger: std::cell::Cell<
                Option<fret_ui::elements::GlobalElementId>,
            > = std::cell::Cell::new(None);

            if overlay_presence.present {
                if is_open {
                    radix_alert_dialog::clear_cancel_for_open_model(cx, open_id);
                }

                let overlay_color = self.overlay_color.unwrap_or_else(default_overlay_color);
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

                    crate::a11y_modal::begin_modal_a11y_scope(cx.app, open_id);
                    let content = content(cx);
                    content_element_for_trigger.set(Some(content.id));
                    crate::a11y_modal::end_modal_a11y_scope(cx.app, open_id);

                    // Center like `Dialog`, but avoid full-window wrappers that can steal hit tests.
                    let outer = cx.bounds;
                    let available_w = Px((outer.size.width.0 - window_padding_px.0 * 2.0).max(0.0));
                    let available_h =
                        Px((outer.size.height.0 - window_padding_px.0 * 2.0).max(0.0));

                    let last_size = cx.last_bounds_for_element(content.id).map(|r| r.size);
                    let desired_w = last_size.map(|s| s.width).unwrap_or(Px(512.0));
                    let desired_h = last_size.map(|s| s.height).unwrap_or(Px(240.0));

                    let content_w = Px(desired_w.0.min(available_w.0).max(0.0));
                    let content_h = Px(desired_h.0.min(available_h.0).max(0.0));

                    let left = Px(outer.origin.x.0
                        + window_padding_px.0
                        + ((available_w.0 - content_w.0) * 0.5).max(0.0));
                    let top = Px(outer.origin.y.0
                        + window_padding_px.0
                        + ((available_h.0 - content_h.0) * 0.5).max(0.0));

                    let origin = Point::new(
                        Px(left.0 + content_w.0 * 0.5),
                        Px(top.0 + content_h.0 * 0.5),
                    );
                    let zoom = overlay_motion::shadcn_zoom_transform(origin, opacity);

                    let wrapper = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(top),
                                    left: Some(left),
                                    right: None,
                                    bottom: None,
                                },
                                size: SizeStyle {
                                    width: Length::Px(content_w),
                                    height: Length::Px(content_h),
                                    ..Default::default()
                                },
                                overflow: Overflow::Visible,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![content],
                    );

                    let opacity_layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let content_layout = opacity_layout.clone();
                    let barrier_children = vec![barrier_fill];
                    let open_for_children = self.open.clone();

                    let content = overlay_motion::wrap_opacity_and_render_transform_with_layouts(
                        cx,
                        opacity_layout,
                        opacity,
                        RenderTransformProps {
                            layout: content_layout,
                            transform: zoom,
                        },
                        vec![wrapper],
                    );
                    radix_alert_dialog::alert_dialog_modal_layer_children(
                        cx,
                        open_for_children.clone(),
                        barrier_children,
                        content,
                    )
                });

                if let Some(content_element) = content_element_for_trigger.get() {
                    cx.with_state(AlertDialogA11yState::default, |st| {
                        st.content_element = Some(content_element)
                    });
                }

                let options = radix_alert_dialog::dialog_options_for_alert_dialog(
                    cx,
                    open_id,
                    radix_alert_dialog::AlertDialogOptions::default()
                        .on_open_auto_focus(self.on_open_auto_focus.clone())
                        .on_close_auto_focus(self.on_close_auto_focus.clone()),
                );
                let initial_focus = is_open.then_some(options.initial_focus).flatten();
                let options = options.initial_focus(initial_focus);

                let request = radix_alert_dialog::alert_dialog_modal_request_with_options(
                    id,
                    id,
                    self.open.clone(),
                    overlay_presence,
                    options,
                    overlay_children,
                );
                radix_alert_dialog::request_alert_dialog(cx, request);
            } else {
                radix_alert_dialog::clear_cancel_for_open_model(cx, open_id);
            }

            let content_element = content_element_for_trigger.get().or(prev_content_element);
            radix_alert_dialog::apply_alert_dialog_trigger_a11y(trigger, is_open, content_element)
        })
    }
}

/// shadcn/ui `AlertDialogTrigger` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogTrigger {
    child: AnyElement,
}

impl AlertDialogTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `AlertDialogContent` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AlertDialogContent {
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

        let bg = theme.color_required("background");
        let border = theme.color_required("border");

        let radius = theme.metric_required("metric.radius.lg");
        let shadow = decl_style::shadow_lg(&theme, radius);

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .p(Space::N6)
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_full()
            .max_w(MetricRef::Px(Px(512.0)))
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        let container = shadcn_layout::container_vstack_gap(
            cx,
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            Space::N4,
            children,
        );

        let (labelled_by_element, described_by_element) =
            crate::a11y_modal::modal_relations_for_current_scope(cx.app);
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::AlertDialog,
                labelled_by_element,
                described_by_element,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

/// shadcn/ui `AlertDialogHeader` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogHeader {
    children: Vec<AnyElement>,
}

impl AlertDialogHeader {
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

/// shadcn/ui `AlertDialogFooter` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogFooter {
    children: Vec<AnyElement>,
}

impl AlertDialogFooter {
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

/// shadcn/ui `AlertDialogTitle` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogTitle {
    text: Arc<str>,
}

impl AlertDialogTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_required("foreground"));

        let px = theme
            .metric_by_key("component.alert_dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.alert_dialog.title_line_height")
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

/// shadcn/ui `AlertDialogDescription` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogDescription {
    text: Arc<str>,
}

impl AlertDialogDescription {
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
            .metric_by_key("component.alert_dialog.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.alert_dialog.description_line_height")
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

/// shadcn/ui `AlertDialogAction` (v4).
///
/// This is a convenience wrapper that closes the dialog on click.
#[derive(Clone)]
pub struct AlertDialogAction {
    label: Arc<str>,
    open: Model<bool>,
    variant: ButtonVariant,
    disabled: bool,
}

impl std::fmt::Debug for AlertDialogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogAction")
            .field("label", &self.label)
            .field("open", &"<model>")
            .field("variant", &self.variant)
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl AlertDialogAction {
    pub fn new(label: impl Into<Arc<str>>, open: Model<bool>) -> Self {
        Self {
            label: label.into(),
            open,
            variant: ButtonVariant::Default,
            disabled: false,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Button::new(self.label)
            .variant(self.variant)
            .disabled(self.disabled)
            .toggle_model(self.open)
            .into_element(cx)
    }
}

/// shadcn/ui `AlertDialogCancel` (v4).
///
/// This is a convenience wrapper that closes the dialog on click.
#[derive(Clone)]
pub struct AlertDialogCancel {
    label: Arc<str>,
    open: Model<bool>,
    disabled: bool,
}

impl std::fmt::Debug for AlertDialogCancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogCancel")
            .field("label", &self.label)
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl AlertDialogCancel {
    pub fn new(label: impl Into<Arc<str>>, open: Model<bool>) -> Self {
        Self {
            label: label.into(),
            open,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open_id = self.open.id();
        let element = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .disabled(self.disabled)
            .toggle_model(self.open)
            .into_element(cx);

        radix_alert_dialog::register_cancel_for_open_model(cx, open_id, element.id);

        element
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::element::PressableProps;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    #[test]
    fn alert_dialog_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let alert = AlertDialog::new_controllable(cx, Some(controlled.clone()), false);
            assert_eq!(alert.open, controlled);
        });
    }

    #[test]
    fn alert_dialog_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let alert = AlertDialog::new_controllable(cx, None, true);
            let open = cx
                .watch_model(&alert.open)
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

    fn render_alert_dialog_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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

                let open_for_dialog = open.clone();
                let open_for_cancel = open.clone();

                let alert = AlertDialog::new(open_for_dialog).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        // One focusable element (cancel-like) to make initial focus deterministic.
                        let cancel = cx.pressable_with_id(
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
                                cx.pressable_set_bool(&open_for_cancel, false);
                                cancel_id_out.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        AlertDialogContent::new(vec![cancel]).into_element(cx)
                    },
                );

                vec![alert]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn alert_dialog_is_not_overlay_closable_and_restores_focus_to_trigger_on_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let cancel_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed.
        app.set_frame_id(FrameId(1));
        let trigger = render_alert_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            cancel_id.clone(),
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

        // Frame 2: open, initial focus should go to the cancel element.
        app.set_frame_id(FrameId(2));
        let _ = render_alert_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            cancel_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let cancel_element_id = cancel_id.get().expect("cancel element id");
        let cancel_node = fret_ui::elements::node_for_element(&mut app, window, cancel_element_id)
            .expect("cancel node");
        assert_eq!(ui.focus(), Some(cancel_node));

        // Clicking the overlay should NOT close.
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

        // Close via Escape.
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
        // restore focus when the layer is uninstalled.
        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for frame in 3..=(2 + settle_frames) {
            app.set_frame_id(FrameId(frame));
            let _ = render_alert_dialog_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                cancel_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn alert_dialog_prefers_cancel_as_initial_focus_even_when_action_is_first() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let cancel_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>,
                            app: &mut App,
                            services: &mut dyn fret_core::UiServices,
                            frame: u64| {
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);

            let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
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

                    let open_for_dialog = open.clone();
                    let open_for_action = open.clone();
                    let open_for_cancel = open.clone();
                    let cancel_id_out = cancel_id.clone();

                    let alert = AlertDialog::new(open_for_dialog).into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let action =
                                AlertDialogAction::new("Delete", open_for_action).into_element(cx);
                            let cancel =
                                AlertDialogCancel::new("Cancel", open_for_cancel).into_element(cx);
                            cancel_id_out.set(Some(cancel.id));

                            AlertDialogContent::new(vec![action, cancel]).into_element(cx)
                        },
                    );

                    vec![alert]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            trigger_id.expect("trigger id")
        };

        // Frame 1: closed.
        let trigger = render_frame(&mut ui, &mut app, &mut services, 1);
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

        // Frame 2: open, initial focus should prefer Cancel, not the first Action.
        let _ = render_frame(&mut ui, &mut app, &mut services, 2);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let cancel = cancel_id.get().expect("cancel id");
        let cancel_node =
            fret_ui::elements::node_for_element(&mut app, window, cancel).expect("cancel node");
        assert_eq!(ui.focus(), Some(cancel_node));

        // Close and ensure focus restores to trigger.
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        let _ = app.models_mut().update(&open, |v| *v = false);

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for frame in 3..=(2 + settle_frames) {
            let _ = render_frame(&mut ui, &mut app, &mut services, frame);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn alert_dialog_content_exposes_labelled_by_and_described_by_relations() {
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

        let render_frame = |ui: &mut UiTree<App>,
                            app: &mut App,
                            services: &mut dyn fret_core::UiServices,
                            frame| {
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);

            let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "alert-dialog-a11y-relations",
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

                    let alert = AlertDialog::new(open.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            let title = AlertDialogTitle::new("AlertDialog Title").into_element(cx);
                            let description =
                                AlertDialogDescription::new("AlertDialog Description")
                                    .into_element(cx);
                            AlertDialogContent::new(vec![title, description]).into_element(cx)
                        },
                    );

                    vec![alert]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            trigger_id.expect("trigger id")
        };

        // Frame 1: closed.
        let _trigger = render_frame(&mut ui, &mut app, &mut services, 1);
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
        let _ = render_frame(&mut ui, &mut app, &mut services, 2);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alert_dialog = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::AlertDialog)
            .expect("alert dialog semantics node");
        let title = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref() == Some("AlertDialog Title")
            })
            .expect("title semantics node");
        let description = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref() == Some("AlertDialog Description")
            })
            .expect("description semantics node");

        assert!(
            alert_dialog.labelled_by.iter().any(|id| *id == title.id),
            "alert dialog should be labelled by its title"
        );
        assert!(
            alert_dialog
                .described_by
                .iter()
                .any(|id| *id == description.id),
            "alert dialog should be described by its description"
        );
    }
}
