use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, MarginEdge, MarginEdges, Overflow,
    PositionStyle, SemanticsProps, SizeStyle,
};
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverlayController, OverlayPresence, Space, ui,
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

#[derive(Debug, Default)]
struct SheetSideProviderState {
    current: Option<SheetSide>,
}

fn inherited_sheet_side<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SheetSide> {
    cx.inherited_state_where::<SheetSideProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
}

#[track_caller]
fn with_sheet_side_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    side: SheetSide,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SheetSideProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(side);
        prev
    });
    let out = f(cx);
    cx.with_state(SheetSideProviderState::default, |st| {
        st.current = prev;
    });
    out
}

fn sheet_side_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> SheetSide {
    inherited_sheet_side(cx).unwrap_or_default()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// shadcn/ui `Sheet` (v4).
///
/// This is a modal overlay (barrier-backed) installed via the component-layer overlay manager.
/// The barrier provides the "overlay click to dismiss" policy when configured.
#[derive(Clone)]
pub struct Sheet {
    open: Model<bool>,
    side: SheetSide,
    size_override: Option<Px>,
    overlay_closable: bool,
    overlay_color: Option<Color>,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    vertical_edge_gap_px: Option<Px>,
    vertical_auto_max_height_fraction: Option<f32>,
}

impl std::fmt::Debug for Sheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sheet")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("size_override", &self.size_override)
            .field("overlay_closable", &self.overlay_closable)
            .field("overlay_color", &self.overlay_color)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("vertical_edge_gap_px", &self.vertical_edge_gap_px)
            .field(
                "vertical_auto_max_height_fraction",
                &self.vertical_auto_max_height_fraction,
            )
            .finish()
    }
}

impl Sheet {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            side: SheetSide::default(),
            size_override: None,
            overlay_closable: true,
            overlay_color: None,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            vertical_edge_gap_px: None,
            vertical_auto_max_height_fraction: None,
        }
    }

    /// Creates a sheet with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
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

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Sets the sheet size:
    /// - width for `Left` / `Right`
    /// - height for `Top` / `Bottom`
    pub fn size(mut self, size: Px) -> Self {
        self.size_override = Some(size);
        self
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.overlay_closable = overlay_closable;
        self
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    /// Installs an extra edge gap for vertical (`Top` / `Bottom`) sheets.
    ///
    /// This exists to support strict shadcn Drawer parity (`mt-24` / `mb-24`) while still using
    /// the shared Sheet overlay scaffolding.
    pub(crate) fn vertical_edge_gap_px(mut self, gap: Px) -> Self {
        self.vertical_edge_gap_px = Some(gap);
        self
    }

    /// Caps vertical (`Top` / `Bottom`) auto-sized sheets to a fraction of the viewport height.
    ///
    /// This exists to support strict shadcn Drawer parity (`max-h-[80vh]`) while still using the
    /// shared Sheet overlay scaffolding.
    pub(crate) fn vertical_auto_max_height_fraction(mut self, fraction: f32) -> Self {
        self.vertical_auto_max_height_fraction = Some(fraction);
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape dismissals (overlay root) and overlay-click dismissals (barrier press) are
    /// routed through this handler. To prevent default dismissal, call `req.prevent_default()`.
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

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = radix_dialog::dialog_root_name(id);

            let motion = OverlayController::transition_with_durations_and_easing(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_500,
                overlay_motion::SHADCN_MOTION_TICKS_300,
                overlay_motion::shadcn_ease,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            if overlay_presence.present {
                let on_dismiss_request_for_barrier = self.on_dismiss_request.clone();
                let on_dismiss_request_for_request = self.on_dismiss_request.clone();

                let open = self.open;
                let open_for_children = open.clone();
                let overlay_color = self.overlay_color.unwrap_or_else(default_overlay_color);
                let overlay_closable = self.overlay_closable;
                let sheet_side = self.side;
                let dialog_options = radix_dialog::DialogOptions::default()
                    .dismiss_on_overlay_press(overlay_closable)
                    .initial_focus(None)
                    .on_open_auto_focus(self.on_open_auto_focus.clone())
                    .on_close_auto_focus(self.on_close_auto_focus.clone());

                let size_override = self.size_override;
                let vertical_edge_gap_px = self.vertical_edge_gap_px.unwrap_or(Px(0.0));
                let vertical_auto_max_height_fraction =
                    self.vertical_auto_max_height_fraction.unwrap_or(1.0);
                let default_size = theme
                    .metric_by_key("component.sheet.size")
                    .or_else(|| theme.metric_by_key("component.sheet.width"))
                    .unwrap_or(Px(350.0));
                let size = size_override.unwrap_or(default_size);

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

                    let content = with_sheet_side_provider(cx, sheet_side, |cx| content(cx));

                    let outer = cx.bounds;
                    let max_w = outer.size.width;
                    let max_h = outer.size.height;

                    let sheet_w = Px(size.0.min(max_w.0).max(0.0));
                    let sheet_h = Px(size.0.min(max_h.0).max(0.0));

                    let vertical_auto_max_h = if size_override.is_some() {
                        None
                    } else {
                        let cap = Px((max_h.0 * vertical_auto_max_height_fraction).max(0.0));
                        let by_gap = Px((max_h.0 - vertical_edge_gap_px.0).max(0.0));
                        Some(Px(cap.0.min(by_gap.0)))
                    };

                    let (inset, size, estimated_motion_distance) = match sheet_side {
                        SheetSide::Right => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                left: None,
                            },
                            SizeStyle {
                                width: Length::Px(sheet_w),
                                height: Length::Fill,
                                ..Default::default()
                            },
                            sheet_w,
                        ),
                        SheetSide::Left => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: None,
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Px(sheet_w),
                                height: Length::Fill,
                                ..Default::default()
                            },
                            sheet_w,
                        ),
                        SheetSide::Top => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: Some(Px(0.0)),
                                bottom: None,
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Fill,
                                height: if size_override.is_some() {
                                    Length::Px(sheet_h)
                                } else {
                                    Length::Auto
                                },
                                max_height: vertical_auto_max_h,
                                ..Default::default()
                            },
                            sheet_h,
                        ),
                        SheetSide::Bottom => (
                            InsetStyle {
                                top: None,
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Fill,
                                height: if size_override.is_some() {
                                    Length::Px(sheet_h)
                                } else {
                                    Length::Auto
                                },
                                max_height: vertical_auto_max_h,
                                ..Default::default()
                            },
                            sheet_h,
                        ),
                    };

                    let motion_side = match sheet_side {
                        SheetSide::Left => Side::Left,
                        SheetSide::Right => Side::Right,
                        SheetSide::Top => Side::Top,
                        SheetSide::Bottom => Side::Bottom,
                    };

                    let wrapper = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset,
                                margin: match sheet_side {
                                    SheetSide::Top if vertical_edge_gap_px.0 > 0.0 => MarginEdges {
                                        bottom: MarginEdge::Px(vertical_edge_gap_px),
                                        ..Default::default()
                                    },
                                    SheetSide::Bottom if vertical_edge_gap_px.0 > 0.0 => {
                                        MarginEdges {
                                            top: MarginEdge::Px(vertical_edge_gap_px),
                                            ..Default::default()
                                        }
                                    }
                                    _ => Default::default(),
                                },
                                size,
                                overflow: Overflow::Visible,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![content],
                    );

                    let motion_distance =
                        if matches!(sheet_side, SheetSide::Top | SheetSide::Bottom)
                            && size_override.is_none()
                        {
                            cx.last_bounds_for_element(wrapper.id)
                                .map(|r| r.size.height)
                                .unwrap_or(estimated_motion_distance)
                        } else {
                            estimated_motion_distance
                        };
                    let slide = overlay_motion::shadcn_modal_slide_transform(
                        motion_side,
                        motion_distance,
                        opacity,
                    );

                    let content = overlay_motion::wrap_opacity_and_render_transform(
                        cx,
                        opacity,
                        slide,
                        vec![wrapper],
                    );

                    radix_dialog::modal_dialog_layer_children_with_dismiss_handler(
                        cx,
                        open_for_children.clone(),
                        dialog_options.clone(),
                        on_dismiss_request_for_barrier.clone(),
                        vec![barrier_fill],
                        content,
                    )
                });

                let request = radix_dialog::modal_dialog_request_with_options_and_dismiss_handler(
                    id,
                    id,
                    open,
                    overlay_presence,
                    dialog_options,
                    on_dismiss_request_for_request,
                    overlay_children,
                );
                radix_dialog::request_modal_dialog(cx, request);
            }

            trigger
        })
    }
}

/// shadcn/ui `SheetContent` (v4).
#[derive(Debug, Clone)]
pub struct SheetContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SheetContent {
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
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .p(Space::N6)
            .merge(self.chrome);

        let side = sheet_side_in_scope(cx);
        let base_layout = match side {
            SheetSide::Left | SheetSide::Right => LayoutRefinement::default()
                .w_full()
                .h_full()
                .overflow_hidden(),
            SheetSide::Top | SheetSide::Bottom => {
                // Auto height by default for top/bottom sheets, matching upstream intent.
                LayoutRefinement::default().w_full().overflow_hidden()
            }
        };
        let layout = base_layout.merge(self.layout);

        let props = {
            let mut props = decl_style::container_props(&theme, chrome, layout);
            let border_w = props.border.top;
            props.border = match side {
                SheetSide::Left => Edges {
                    right: border_w,
                    ..Edges::all(Px(0.0))
                },
                SheetSide::Right => Edges {
                    left: border_w,
                    ..Edges::all(Px(0.0))
                },
                SheetSide::Top => Edges {
                    bottom: border_w,
                    ..Edges::all(Px(0.0))
                },
                SheetSide::Bottom => Edges {
                    top: border_w,
                    ..Edges::all(Px(0.0))
                },
            };
            props
        };
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

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Dialog,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

/// shadcn/ui `SheetHeader` (v4).
#[derive(Debug, Clone)]
pub struct SheetHeader {
    children: Vec<AnyElement>,
}

impl SheetHeader {
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

/// shadcn/ui `SheetFooter` (v4).
#[derive(Debug, Clone)]
pub struct SheetFooter {
    children: Vec<AnyElement>,
}

impl SheetFooter {
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

/// shadcn/ui `SheetTitle` (v4).
#[derive(Debug, Clone)]
pub struct SheetTitle {
    text: Arc<str>,
}

impl SheetTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_required("foreground"));

        let px = theme
            .metric_by_key("component.sheet.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.sheet.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .letter_spacing_em(-0.02)
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx)
    }
}

/// shadcn/ui `SheetDescription` (v4).
#[derive(Debug, Clone)]
pub struct SheetDescription {
    text: Arc<str>,
}

impl SheetDescription {
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
            .metric_by_key("component.sheet.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.sheet.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::PressableProps;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    #[test]
    fn sheet_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let sheet = Sheet::new_controllable(cx, Some(controlled.clone()), false);
            assert_eq!(sheet.open, controlled);
        });
    }

    #[test]
    fn sheet_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let sheet = Sheet::new_controllable(cx, None, true);
            let open = cx
                .watch_model(&sheet.open)
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

    fn render_sheet_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
        overlay_closable: bool,
        side: SheetSide,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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

                let sheet = Sheet::new(open)
                    .side(side)
                    .overlay_closable(overlay_closable)
                    .size(Px(300.0))
                    .on_dismiss_request(on_dismiss_request.clone())
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

                            let content = SheetContent::new(vec![focusable]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![sheet]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn sheet_overlay_click_closes_when_overlay_closable() {
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
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            true,
            SheetSide::Right,
            content_id.clone(),
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
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            true,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert!(content_id.get().is_some());

        // Click inside sheet should not close.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(780.0), Px(50.0)),
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
                position: Point::new(Px(780.0), Px(50.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Click outside sheet should close via barrier.
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
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn sheet_overlay_click_does_not_close_when_not_overlay_closable() {
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

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            false,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

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
    }

    #[test]
    fn sheet_escape_closes() {
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

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            true,
            SheetSide::Right,
            content_id.clone(),
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
    fn sheet_escape_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let reason_cell: Arc<std::sync::Mutex<Option<DismissReason>>> =
            Arc::new(std::sync::Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            *reason_cell_for_handler.lock().expect("reason lock") = Some(req.reason);
            req.prevent_default();
        });

        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler.clone()),
            true,
            SheetSide::Right,
            content_id.clone(),
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

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            *reason_cell.lock().expect("reason lock"),
            Some(DismissReason::Escape)
        );
    }

    #[test]
    fn sheet_overlay_click_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let reason_cell: Arc<std::sync::Mutex<Option<DismissReason>>> =
            Arc::new(std::sync::Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            *reason_cell_for_handler.lock().expect("reason lock") = Some(req.reason);
            req.prevent_default();
        });

        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler.clone()),
            true,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click the underlay area: this should hit the modal barrier behind the sheet content.
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
        let reason = *reason_cell.lock().expect("reason lock");
        let Some(DismissReason::OutsidePress { pointer }) = reason else {
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
    fn sheet_focuses_first_focusable_on_open_and_restores_trigger_on_close() {
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
        let trigger = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            true,
            SheetSide::Right,
            content_id.clone(),
            initial_focus_cell.clone(),
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
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            None,
            true,
            SheetSide::Right,
            content_id.clone(),
            initial_focus_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus_element_id = initial_focus_cell.get().expect("initial focus element id");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus_element_id)
                .expect("initial focus node");
        assert_eq!(ui.focus(), Some(initial_focus_node));

        // Close via Escape and render a few frames to allow the close animation to finish and the
        // overlay manager to restore focus when the layer is uninstalled.
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

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_300 as usize + 1;
        for _ in 0..settle_frames {
            let _ = render_sheet_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                None,
                true,
                SheetSide::Right,
                content_id.clone(),
                initial_focus_cell.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        assert_eq!(ui.focus(), Some(trigger_node));
    }
}
