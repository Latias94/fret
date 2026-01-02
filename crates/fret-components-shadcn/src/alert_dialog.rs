use std::sync::Arc;

use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Radius, Space,
};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::button::{Button, ButtonVariant};

fn default_overlay_color() -> Color {
    Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.8,
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
}

impl std::fmt::Debug for AlertDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialog")
            .field("open", &"<model>")
            .field("overlay_color", &self.overlay_color)
            .field("window_padding", &self.window_padding)
            .finish()
    }
}

impl AlertDialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            overlay_color: None,
            window_padding: Space::N6,
        }
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    pub fn window_padding(mut self, padding: Space) -> Self {
        self.window_padding = padding;
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
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = OverlayController::modal_root_name(id);

            if is_open {
                let overlay_color = self.overlay_color.unwrap_or_else(default_overlay_color);
                let window_padding_px = MetricRef::space(self.window_padding).resolve(&theme);

                let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                    let barrier_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    // Non-closable overlay barrier.
                    let barrier = cx.container(
                        ContainerProps {
                            layout: barrier_layout,
                            padding: Edges::all(Px(0.0)),
                            background: Some(overlay_color),
                            shadow: None,
                            border: Edges::all(Px(0.0)),
                            border_color: None,
                            corner_radii: Corners::all(Px(0.0)),
                        },
                        |_cx| Vec::new(),
                    );

                    let content = content(cx);

                    // Center like `Dialog`, but avoid full-window wrappers that can steal hit tests.
                    let outer = cx.bounds;
                    let available_w = Px((outer.size.width.0 - window_padding_px.0 * 2.0).max(0.0));
                    let available_h =
                        Px((outer.size.height.0 - window_padding_px.0 * 2.0).max(0.0));

                    let last_size = cx.last_bounds_for_element(content.id).map(|r| r.size);
                    let desired_w = last_size.map(|s| s.width).unwrap_or(Px(420.0));
                    let desired_h = last_size.map(|s| s.height).unwrap_or(Px(240.0));

                    let content_w = Px(desired_w.0.min(available_w.0).max(0.0));
                    let content_h = Px(desired_h.0.min(available_h.0).max(0.0));

                    let left = Px(outer.origin.x.0
                        + window_padding_px.0
                        + ((available_w.0 - content_w.0) * 0.5).max(0.0));
                    let top = Px(outer.origin.y.0
                        + window_padding_px.0
                        + ((available_h.0 - content_h.0) * 0.5).max(0.0));

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

                    vec![barrier, wrapper]
                });

                let mut request = OverlayRequest::modal(
                    id,
                    Some(id),
                    self.open,
                    OverlayPresence::instant(true),
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
                OverlayController::request(cx, request);
            }

            trigger
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
    pub fn new(children: Vec<AnyElement>) -> Self {
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

        let bg = theme
            .color_by_key("background")
            .unwrap_or(theme.colors.panel_background);
        let border = theme
            .color_by_key("border")
            .unwrap_or(theme.colors.panel_border);

        let radius = theme.metrics.radius_lg;
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
            .max_w(MetricRef::Px(Px(420.0)))
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        let container = cx.container(
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            move |_cx| children,
        );

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Alert,
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
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pb(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `AlertDialogFooter` (v4).
#[derive(Debug, Clone)]
pub struct AlertDialogFooter {
    children: Vec<AnyElement>,
}

impl AlertDialogFooter {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pt(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        cx.container(props, move |_cx| children)
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
            .unwrap_or(theme.colors.text_primary);

        let px = theme
            .metric_by_key("component.alert_dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.alert_dialog.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::SEMIBOLD,
                line_height: Some(line_height),
                letter_spacing_em: Some(-0.02),
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
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
            .unwrap_or(theme.colors.text_muted);

        let px = theme
            .metric_by_key("component.alert_dialog.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.alert_dialog.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
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
        Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .disabled(self.disabled)
            .toggle_model(self.open)
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_components_ui::declarative::action_hooks::ActionHooksExt;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::element::PressableProps;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
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
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
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
                position: Point::new(Px(4.0), Px(4.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(4.0), Px(4.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
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
        for frame in 3..=6 {
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
}
