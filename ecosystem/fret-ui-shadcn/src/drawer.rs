//! shadcn/ui `Drawer` facade.
//!
//! Fret currently models drawers as a `Sheet` that defaults to the `Bottom` side.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, MouseButton, Point, Px, SemanticsRole, Transform2D};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, MarginEdge, MarginEdges, PointerRegionProps,
    RenderTransformProps, SemanticsProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::Sheet;
use crate::layout as shadcn_layout;
pub use crate::sheet::{
    SheetDescription as DrawerDescription, SheetSide as DrawerSide, SheetTitle as DrawerTitle,
};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, LayoutRefinement, MetricRef, Space};

const DRAWER_EDGE_GAP_PX: Px = Px(96.0);
const DRAWER_MAX_HEIGHT_FRACTION: f32 = 0.8;
const DRAWER_SIDE_PANEL_WIDTH_FRACTION: f32 = 0.75;
const DRAWER_SIDE_PANEL_MAX_WIDTH_PX: Px = Px(384.0);
const DRAWER_SNAP_SETTLE_TICKS: u64 = 18;

/// shadcn/ui `DrawerPortal` (v4).
///
/// In upstream (Vaul/Radix), `Portal` controls *where* the drawer is mounted in the DOM. In Fret,
/// overlay mounting is owned by the per-window overlay manager, so this type exists for taxonomy
/// parity only. The `Drawer` recipe always renders into an overlay root.
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawerPortal;

/// shadcn/ui `DrawerOverlay` (v4).
///
/// In upstream, `DrawerOverlay` is a styled overlay element rendered inside the portal. In Fret the
/// barrier is authored by the recipe layer (`Drawer` -> `Sheet`), but we expose this type so callers
/// can configure overlay defaults using shadcn-aligned naming.
#[derive(Debug, Clone, Default)]
pub struct DrawerOverlay {
    color: Option<Color>,
}

impl DrawerOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawerSnapPoint {
    /// A fraction of the window height (Vaul-style).
    Fraction(f32),
}

fn normalize_snap_points(points: Vec<DrawerSnapPoint>) -> Vec<f32> {
    let mut out: Vec<f32> = points
        .into_iter()
        .filter_map(|p| match p {
            DrawerSnapPoint::Fraction(f) if f.is_finite() && f > 0.0 => Some(f.min(1.0)),
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.total_cmp(b));
    out.dedup_by(|a, b| (*a - *b).abs() < f32::EPSILON);
    out
}

#[derive(Debug, Default)]
struct DrawerSideProviderState {
    current: Option<DrawerSide>,
}

fn inherited_drawer_side<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<DrawerSide> {
    cx.inherited_state_where::<DrawerSideProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
}

fn drawer_side_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> DrawerSide {
    inherited_drawer_side(cx).unwrap_or(DrawerSide::Bottom)
}

#[track_caller]
fn with_drawer_side_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    side: DrawerSide,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(DrawerSideProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(side);
        prev
    });
    let out = f(cx);
    cx.with_state(DrawerSideProviderState::default, |st| {
        st.current = prev;
    });
    out
}

fn drawer_vertical_max_height(viewport_height: Px) -> Px {
    let cap = (viewport_height.0 * DRAWER_MAX_HEIGHT_FRACTION).max(0.0);
    let by_gap = (viewport_height.0 - DRAWER_EDGE_GAP_PX.0).max(0.0);
    Px(cap.min(by_gap))
}

fn drawer_drag_snap_height(drawer_height: Px, side: DrawerSide) -> Px {
    // DrawerContent applies a 1px border on the edge that meets the viewport.
    // The snap-point math should be based on the border-box height, so account for that inset.
    match side {
        DrawerSide::Bottom | DrawerSide::Top => Px(drawer_height.0 + 1.0),
        DrawerSide::Left | DrawerSide::Right => drawer_height,
    }
}

/// shadcn/ui `DrawerContent` (v4).
#[derive(Debug, Clone)]
pub struct DrawerContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl DrawerContent {
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
        let side = drawer_side_in_scope(cx);

        let bg = theme.color_required("background");
        let border = theme.color_required("border");
        let muted = theme.color_required("muted");
        let radius = theme.metric_required("metric.radius.lg");

        let (borders, corners) = match side {
            DrawerSide::Bottom => (
                Edges {
                    top: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                Corners {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: Px(0.0),
                    bottom_left: Px(0.0),
                },
            ),
            DrawerSide::Top => (
                Edges {
                    bottom: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                Corners {
                    top_left: Px(0.0),
                    top_right: Px(0.0),
                    bottom_right: radius,
                    bottom_left: radius,
                },
            ),
            DrawerSide::Left => (
                Edges {
                    right: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                Corners::all(Px(0.0)),
            ),
            DrawerSide::Right => (
                Edges {
                    left: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                Corners::all(Px(0.0)),
            ),
        };

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(self.chrome);

        let base_layout = match side {
            DrawerSide::Left | DrawerSide::Right => LayoutRefinement::default()
                .w_full()
                .h_full()
                .overflow_visible(),
            DrawerSide::Top | DrawerSide::Bottom => LayoutRefinement::default()
                .w_full()
                .max_h(MetricRef::Px(drawer_vertical_max_height(
                    cx.bounds.size.height,
                )))
                .overflow_visible(),
        };
        let layout = base_layout.merge(self.layout);

        let mut props = decl_style::container_props(&theme, chrome, layout);
        props.padding = Edges::all(Px(0.0));
        props.shadow = None;
        props.border = borders;
        props.corner_radii = corners;

        let children = self.children;

        let mut rows: Vec<AnyElement> = Vec::new();
        if side == DrawerSide::Bottom {
            let bar = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(100.0)),
                            height: Length::Px(Px(8.0)),
                            ..Default::default()
                        },
                        margin: MarginEdges {
                            top: MarginEdge::Px(Px(16.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(0.0)),
                    background: Some(muted),
                    shadow: None,
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );
            rows.push(shadcn_layout::container_hstack(
                cx,
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                stack::HStackProps::default()
                    .gap(Space::N0)
                    .justify_center()
                    .items_center(),
                vec![bar],
            ));
        }
        rows.extend(children);

        let stack_layout = match side {
            DrawerSide::Left | DrawerSide::Right => LayoutRefinement::default().w_full().h_full(),
            DrawerSide::Top | DrawerSide::Bottom => LayoutRefinement::default().w_full(),
        };
        let content = cx.container(props, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(stack_layout)
                    .items_stretch(),
                move |_cx| rows,
            )]
        });

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Dialog,
                ..Default::default()
            },
            move |_cx| vec![content],
        )
    }
}

/// shadcn/ui `DrawerHeader` (v4).
#[derive(Debug, Clone)]
pub struct DrawerHeader {
    children: Vec<AnyElement>,
}

impl DrawerHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let side = drawer_side_in_scope(cx);
        let items = match side {
            DrawerSide::Top | DrawerSide::Bottom => Items::Center,
            DrawerSide::Left | DrawerSide::Right => Items::Start,
        };
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default()
                .gap(Space::N1)
                .layout(LayoutRefinement::default().w_full())
                .items(items),
            children,
        )
    }
}

/// shadcn/ui `DrawerFooter` (v4).
#[derive(Debug, Clone)]
pub struct DrawerFooter {
    children: Vec<AnyElement>,
}

impl DrawerFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4),
            LayoutRefinement::default().mt_auto(),
        );
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N2, children)
    }
}

#[derive(Clone)]
pub struct Drawer {
    open: Model<bool>,
    side: DrawerSide,
    inner: Sheet,
    drag_to_dismiss: bool,
    snap_points: Option<Vec<f32>>,
    default_snap_point_index: Option<usize>,
}

impl std::fmt::Debug for Drawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawer")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("drag_to_dismiss", &self.drag_to_dismiss)
            .field("snap_points", &self.snap_points.as_ref().map(|v| v.len()))
            .field("default_snap_point_index", &self.default_snap_point_index)
            .finish()
    }
}

impl Drawer {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open: open.clone(),
            side: DrawerSide::Bottom,
            inner: Sheet::new(open)
                .side(DrawerSide::Bottom)
                .vertical_edge_gap_px(DRAWER_EDGE_GAP_PX)
                .vertical_auto_max_height_fraction(DRAWER_MAX_HEIGHT_FRACTION),
            drag_to_dismiss: true,
            snap_points: None,
            default_snap_point_index: None,
        }
    }

    /// Creates a drawer with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
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
        self.inner = self.inner.overlay_closable(overlay_closable);
        self
    }

    pub fn overlay_color(mut self, overlay_color: fret_core::Color) -> Self {
        self.inner = self.inner.overlay_color(overlay_color);
        self
    }

    pub fn overlay_component(mut self, overlay: DrawerOverlay) -> Self {
        if let Some(color) = overlay.color {
            self.inner = self.inner.overlay_color(color);
        }
        self
    }

    /// Enables Vaul-style snap points for bottom drawers.
    ///
    /// Notes:
    /// - Only modeled for `Bottom` drawers today.
    /// - Points are expressed as fractions of the window height, matching Vaul's default authoring
    ///   style.
    /// - When enabled, releasing a drag will settle to the nearest snap point; dragging far enough
    ///   down will still close the drawer.
    pub fn snap_points(mut self, points: Vec<DrawerSnapPoint>) -> Self {
        let points = normalize_snap_points(points);
        self.snap_points = if points.is_empty() {
            None
        } else {
            Some(points)
        };
        self
    }

    /// Overrides which snap point is used as the initial "open" position.
    ///
    /// When unset, the largest snap point is used (most open), matching typical Vaul usage.
    pub fn default_snap_point(mut self, index: usize) -> Self {
        self.default_snap_point_index = Some(index);
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape dismissals (overlay root) and overlay-click dismissals (barrier press) are
    /// routed through this handler. To prevent default dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.inner = self.inner.on_dismiss_request(on_dismiss_request);
        self
    }

    /// Installs an open auto-focus hook (Radix `FocusScope` `onMountAutoFocus`).
    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.inner = self.inner.on_open_auto_focus(hook);
        self
    }

    /// Installs a close auto-focus hook (Radix `FocusScope` `onUnmountAutoFocus`).
    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.inner = self.inner.on_close_auto_focus(hook);
        self
    }

    /// Enables Vaul-style drag-to-dismiss (shadcn Drawer behavior).
    ///
    /// This is intentionally Drawer-only policy and is not part of the Radix primitives boundary.
    pub fn drag_to_dismiss(mut self, enabled: bool) -> Self {
        self.drag_to_dismiss = enabled;
        self
    }

    /// Sets the drawer size (height by default, since drawers default to `Bottom`).
    pub fn size(mut self, size: fret_core::Px) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    /// Optional escape hatch: allow non-bottom drawers by forwarding to `Sheet`.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self.inner = self.inner.side(side);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = self.open.clone();
        let side = self.side;
        let drag_to_dismiss = self.drag_to_dismiss;
        let snap_points = self.snap_points.clone();
        let default_snap_point_index = self.default_snap_point_index;

        let mut inner = self
            .inner
            .vertical_edge_gap_px(DRAWER_EDGE_GAP_PX)
            .vertical_auto_max_height_fraction(DRAWER_MAX_HEIGHT_FRACTION);
        match side {
            DrawerSide::Left | DrawerSide::Right => {
                let viewport_w = cx.bounds.size.width;
                let desired = Px((viewport_w.0 * DRAWER_SIDE_PANEL_WIDTH_FRACTION)
                    .min(DRAWER_SIDE_PANEL_MAX_WIDTH_PX.0)
                    .max(0.0));
                inner = inner.size(desired);
            }
            DrawerSide::Top | DrawerSide::Bottom => {}
        }

        inner.into_element(cx, trigger, move |cx| {
            let content = with_drawer_side_provider(cx, side, |cx| content(cx));
            if !drag_to_dismiss || side != DrawerSide::Bottom {
                return content;
            }

            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
            let (runtime, offset_model, was_open) = drawer_drag_models(cx);
            let window_height = cx.bounds.size.height;
            let has_snap_points = snap_points.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

            if is_open && !was_open {
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = Px(0.0));
                if has_snap_points {
                    let _ = cx.app.models_mut().update(&runtime, |st| {
                        st.needs_snap_init = true;
                        st.settling = false;
                        st.settle_tick = 0;
                    });
                }
            }
            drawer_drag_set_was_open(cx, is_open);

            if !is_open {
                let _ = cx.app.models_mut().update(&runtime, |st| {
                    st.needs_snap_init = false;
                    st.settling = false;
                    st.settle_tick = 0;
                });
            }

            if is_open && has_snap_points {
                let needs_init = cx
                    .app
                    .models()
                    .get_copied(&runtime)
                    .map(|st| st.needs_snap_init)
                    .unwrap_or(false);

                if needs_init {
                    if let Some(bounds) = cx.last_bounds_for_element(content.id) {
                        let drawer_h = drawer_drag_snap_height(bounds.size.height, side);
                        let points = snap_points.as_ref().expect("snap points");
                        let mut idx = default_snap_point_index
                            .unwrap_or_else(|| points.len().saturating_sub(1));
                        if idx >= points.len() {
                            idx = points.len().saturating_sub(1);
                        }
                        let fraction = points.get(idx).copied().unwrap_or(1.0);
                        let desired_visible = Px((window_height.0 * fraction).max(0.0));
                        let visible = Px(desired_visible.0.min(drawer_h.0).max(0.0));
                        let desired_offset = Px((drawer_h.0 - visible.0).max(0.0));

                        let _ = cx.app.models_mut().update(&offset_model, |v| {
                            *v = desired_offset;
                        });
                        let _ = cx.app.models_mut().update(&runtime, |st| {
                            st.needs_snap_init = false;
                        });
                    }
                }
            }

            let mut offset = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let settling = cx
                .app
                .models()
                .get_copied(&runtime)
                .map(|st| st.settling)
                .unwrap_or(false);
            if settling {
                let runtime_snapshot = cx.app.models().get_copied(&runtime);
                if let Some(st) = runtime_snapshot {
                    let tick = st.settle_tick.saturating_add(1);
                    let t = (tick as f32 / DRAWER_SNAP_SETTLE_TICKS as f32).min(1.0);
                    let eased = crate::overlay_motion::shadcn_ease(t);
                    let next = Px(st.settle_from.0 + (st.settle_to.0 - st.settle_from.0) * eased);
                    offset = next;

                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                    let _ = cx.app.models_mut().update(&runtime, |st| {
                        st.settle_tick = tick;
                        if t >= 1.0 {
                            st.settling = false;
                        }
                    });
                    cx.request_frame();
                }
            }

            let transform = Transform2D::translation(Point::new(Px(0.0), offset));

            let runtime_for_down = runtime.clone();
            let offset_for_down = offset_model.clone();
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, _cx, down| {
                if !is_open || down.button != MouseButton::Left {
                    return false;
                }

                let bounds = host.bounds();
                if !drawer_drag_hit_test(bounds, down.position) {
                    return false;
                }

                host.capture_pointer();
                let start_offset = host
                    .models_mut()
                    .read(&offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let _ = host.models_mut().update(&runtime_for_down, |st| {
                    st.dragging = true;
                    st.start = Point::new(down.position.x, Px(down.position.y.0 + start_offset.0));
                    st.start_offset = start_offset;
                    st.settling = false;
                    st.settle_tick = 0;
                });
                host.request_redraw(_cx.window);
                true
            });

            let runtime_for_move = runtime.clone();
            let offset_for_move = offset_model.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let Ok((dragging, start, start_offset)) =
                    host.models_mut().read(&runtime_for_move, |st| {
                        (st.dragging, st.start, st.start_offset)
                    })
                else {
                    return false;
                };
                if !dragging {
                    return false;
                }

                let current_offset = host
                    .models_mut()
                    .read(&offset_for_move, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let global_y = mv.position.y.0 + current_offset.0;
                let dy = global_y - start.y.0;
                let next = Px((start_offset.0 + dy).max(0.0).min(window_height.0));
                let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                host.request_redraw(_cx.window);
                true
            });

            let open_for_up = open.clone();
            let runtime_for_up = runtime.clone();
            let offset_for_up = offset_model.clone();
            let snap_points_for_up = snap_points.clone();
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, _cx, _up| {
                let dragging = host
                    .models_mut()
                    .read(&runtime_for_up, |st| st.dragging)
                    .ok()
                    .unwrap_or(false);
                if !dragging {
                    return false;
                }

                host.release_pointer_capture();
                let bounds = host.bounds();
                let drawer_h = drawer_drag_snap_height(bounds.size.height, side);
                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let has_snap_points = snap_points_for_up
                    .as_ref()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                if has_snap_points {
                    let points = snap_points_for_up.as_ref().expect("snap points");

                    let mut min_visible = None::<Px>;
                    let mut targets: Vec<Px> = Vec::new();
                    for fraction in points {
                        let desired_visible = Px((window_height.0 * *fraction).max(0.0));
                        let visible = Px(desired_visible.0.min(drawer_h.0).max(0.0));
                        if visible.0 > 0.0 {
                            min_visible = Some(match min_visible {
                                Some(prev) => Px(prev.0.min(visible.0)),
                                None => visible,
                            });
                        }
                        let target_offset = Px((drawer_h.0 - visible.0).max(0.0));
                        targets.push(target_offset);
                    }
                    targets.push(Px(0.0));

                    let close_threshold = min_visible
                        .map(|v| Px((drawer_h.0 - v.0 * 0.5).max(DRAWER_DRAG_DISMISS_MIN_PX)))
                        .unwrap_or_else(|| Px((drawer_h.0 * 0.25).max(DRAWER_DRAG_DISMISS_MIN_PX)));

                    if offset.0 >= close_threshold.0 {
                        let _ = host.models_mut().update(&offset_for_up, |v| *v = Px(0.0));
                        let _ = host.models_mut().update(&open_for_up, |v| *v = false);
                    } else {
                        let nearest = targets
                            .iter()
                            .copied()
                            .min_by(|a, b| {
                                let da = (a.0 - offset.0).abs();
                                let db = (b.0 - offset.0).abs();
                                da.total_cmp(&db)
                            })
                            .unwrap_or(Px(0.0));

                        let _ = host.models_mut().update(&runtime_for_up, |st| {
                            st.settling = true;
                            st.settle_from = offset;
                            st.settle_to = nearest;
                            st.settle_tick = 0;
                            st.dragging = false;
                        });
                        host.request_redraw(_cx.window);
                        return true;
                    }
                } else {
                    let threshold = Px((drawer_h.0 * 0.25).max(DRAWER_DRAG_DISMISS_MIN_PX));
                    let should_close = offset.0 >= threshold.0;
                    if should_close {
                        let _ = host.models_mut().update(&open_for_up, |v| *v = false);
                    } else {
                        let _ = host.models_mut().update(&offset_for_up, |v| *v = Px(0.0));
                    }
                }

                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.dragging = false;
                });
                host.request_redraw(_cx.window);
                true
            });

            let content_root = cx.pointer_region(
                PointerRegionProps {
                    layout: LayoutStyle::default(),
                    enabled: is_open,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    vec![content]
                },
            );

            cx.render_transform_props(
                RenderTransformProps {
                    layout: LayoutStyle::default(),
                    transform,
                },
                move |_cx| vec![content_root],
            )
        })
    }
}

const DRAWER_DRAG_HANDLE_HIT_HEIGHT: f32 = 32.0;
const DRAWER_DRAG_HANDLE_HIT_HALF_WIDTH: f32 = 80.0;
const DRAWER_DRAG_DISMISS_MIN_PX: f32 = 30.0;

#[derive(Debug, Clone, Copy, Default)]
struct DrawerDragRuntime {
    dragging: bool,
    start: Point,
    start_offset: Px,
    needs_snap_init: bool,
    settling: bool,
    settle_from: Px,
    settle_to: Px,
    settle_tick: u64,
}

#[derive(Default)]
struct DrawerDragState {
    runtime: Option<Model<DrawerDragRuntime>>,
    offset: Option<Model<Px>>,
    was_open: bool,
}

fn drawer_drag_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<DrawerDragRuntime>, Model<Px>, bool) {
    let needs_init = cx.with_state(DrawerDragState::default, |state| {
        state.runtime.is_none() || state.offset.is_none()
    });

    if needs_init {
        let runtime = cx.app.models_mut().insert(DrawerDragRuntime::default());
        let offset = cx.app.models_mut().insert(Px(0.0));
        cx.with_state(DrawerDragState::default, |state| {
            state.runtime = Some(runtime);
            state.offset = Some(offset);
        });
    }

    cx.with_state(DrawerDragState::default, |state| {
        let runtime = state.runtime.clone().expect("drawer runtime model");
        let offset = state.offset.clone().expect("drawer offset model");
        (runtime, offset, state.was_open)
    })
}

fn drawer_drag_set_was_open<H: UiHost>(cx: &mut ElementContext<'_, H>, was_open: bool) {
    cx.with_state(DrawerDragState::default, |state| {
        state.was_open = was_open;
    });
}

fn drawer_drag_hit_test(bounds: fret_core::Rect, position: Point) -> bool {
    let local_y = position.y.0 - bounds.origin.y.0;
    if local_y < 0.0 {
        return false;
    }
    if local_y > DRAWER_DRAG_HANDLE_HIT_HEIGHT {
        return false;
    }

    let center_x = bounds.origin.x.0 + bounds.size.width.0 * 0.5;
    let dx = (position.x.0 - center_x).abs();
    dx <= DRAWER_DRAG_HANDLE_HIT_HALF_WIDTH
}

/// shadcn/ui `DrawerTrigger` (v4).
#[derive(Debug, Clone)]
pub struct DrawerTrigger {
    child: AnyElement,
}

impl DrawerTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `DrawerClose` (v4).
///
/// Upstream `DrawerClose` is a thin wrapper around the underlying primitive's `Close` component.
/// In Fret, drawers are backed by modal overlays, so this delegates to `DialogClose`.
#[derive(Clone)]
pub struct DrawerClose {
    inner: crate::DialogClose,
}

impl std::fmt::Debug for DrawerClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawerClose").finish()
    }
}

impl DrawerClose {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            // DrawerClose should behave like a primitive close affordance (no forced positioning).
            // Delegate visuals to `DialogClose`, but override the default absolute positioning.
            inner: crate::DialogClose::new(open)
                .refine_layout(LayoutRefinement::default().relative().inset(Space::N0)),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

pub fn drawer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    Drawer::new(open).into_element(cx, trigger, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PressableProps, SizeStyle};
    use fret_ui::elements::{GlobalElementId, visual_bounds_for_element};
    use fret_ui_kit::MetricRef;
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn drawer_new_controllable_can_build_with_or_without_controlled_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(false);

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let _ = Drawer::new_controllable(cx, None, false);
            let _ = Drawer::new_controllable(cx, Some(controlled.clone()), true);
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

    fn render_drawer_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_activated: Model<bool>,
    ) {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "drawer-underlay",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
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
                            layout.inset.left = Some(Px(20.0));
                            layout.inset.top = Some(Px(20.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st| {
                        cx.pressable_toggle_bool(&open);
                        Vec::new()
                    },
                );

                let drawer = Drawer::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        DrawerContent::new(vec![
                            cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        ])
                        .into_element(cx)
                    },
                );

                vec![underlay, drawer]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn drawer_overlay_click_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let dismiss_reason: Rc<Cell<Option<DismissReason>>> = Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            dismiss_reason_cell.set(Some(req.reason));
            req.prevent_default();
        });

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                let underlay = {
                    let underlay_activated = underlay_activated.clone();
                    cx.pressable(
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
                    )
                };

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

                let drawer = Drawer::new(open.clone())
                    .overlay_closable(true)
                    .overlay_component(DrawerOverlay::new())
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(20.0)),
                                            height: Length::Px(Px(20.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            )
                        },
                    );

                vec![underlay, drawer]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, b, &mut scene, 1.0);

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
            "underlay should not activate while drawer is open"
        );
        let reason = dismiss_reason.get();
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
    fn drawer_drag_dismiss_closes_open_model_when_past_threshold() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let drawer_content_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));

            OverlayController::begin_frame(&mut app, window);
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                "test",
                |cx| {
                    let drawer_content_id = drawer_content_id.clone();
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
                        |_cx, _st| Vec::new(),
                    );

                    let drawer = Drawer::new(open.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            let content = DrawerContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx);
                            drawer_content_id.set(Some(content.id));
                            content
                        },
                    );

                    vec![drawer]
                },
            );
            ui.set_root(root);
            OverlayController::render(&mut ui, &mut app, &mut services, window, b);
            ui.layout_all(&mut app, &mut services, b, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, b, &mut scene, 1.0);
        }

        let dialog_element = drawer_content_id.get().expect("drawer content element id");
        let dialog =
            visual_bounds_for_element(&mut app, window, dialog_element).expect("drawer visual");
        let start = Point::new(
            Px(dialog.origin.x.0 + dialog.size.width.0 * 0.5),
            Px(dialog.origin.y.0 + 10.0),
        );
        let end = Point::new(start.x, Px(start.y.0 + 80.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: start,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: end,
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
    fn drawer_drag_dismiss_keeps_open_model_when_under_threshold() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let drawer_content_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));

            OverlayController::begin_frame(&mut app, window);
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                "test",
                |cx| {
                    let drawer_content_id = drawer_content_id.clone();
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
                        |_cx, _st| Vec::new(),
                    );

                    let drawer = Drawer::new(open.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            let content = DrawerContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx);
                            drawer_content_id.set(Some(content.id));
                            content
                        },
                    );

                    vec![drawer]
                },
            );
            ui.set_root(root);
            OverlayController::render(&mut ui, &mut app, &mut services, window, b);
            ui.layout_all(&mut app, &mut services, b, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, b, &mut scene, 1.0);
        }

        let dialog_element = drawer_content_id.get().expect("drawer content element id");
        let dialog =
            visual_bounds_for_element(&mut app, window, dialog_element).expect("drawer visual");
        let start = Point::new(
            Px(dialog.origin.x.0 + dialog.size.width.0 * 0.5),
            Px(dialog.origin.y.0 + 10.0),
        );
        let end = Point::new(start.x, Px(start.y.0 + 20.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: start,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: end,
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
    fn drawer_snap_points_settle_to_nearest_point_on_release() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let offset_model_cell: Rc<RefCell<Option<Model<Px>>>> = Rc::new(RefCell::new(None));
        let drawer_content_id_cell: Rc<RefCell<Option<GlobalElementId>>> =
            Rc::new(RefCell::new(None));

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let open_for_drawer = open.clone();
            let offset_model_cell_for_drawer = offset_model_cell.clone();
            let drawer_content_id_for_drawer = drawer_content_id_cell.clone();

            OverlayController::begin_frame(app, window);
            let root =
                fret_ui::declarative::render_root(ui, app, services, window, b, "test", |cx| {
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
                        |_cx, _st| Vec::new(),
                    );

                    let drawer = Drawer::new(open_for_drawer)
                        .snap_points(vec![
                            DrawerSnapPoint::Fraction(0.25),
                            DrawerSnapPoint::Fraction(0.5),
                            DrawerSnapPoint::Fraction(0.75),
                        ])
                        .into_element(
                            cx,
                            |_cx| trigger,
                            move |cx| {
                                let (_runtime, offset_model, _was_open) = drawer_drag_models(cx);
                                *offset_model_cell_for_drawer.borrow_mut() = Some(offset_model);

                                let content = DrawerContent::new(vec![cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Px(Px(800.0)),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                )])
                                .into_element(cx);
                                *drawer_content_id_for_drawer.borrow_mut() = Some(content.id);
                                content
                            },
                        );

                    vec![drawer]
                });
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, b);
            ui.layout_all(app, services, b, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(app, services, b, &mut scene, 1.0);
        };

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_frame(&mut ui, &mut app, &mut services);
        }

        let drawer_content_id = drawer_content_id_cell
            .borrow()
            .clone()
            .expect("drawer content id captured");

        let offset_model = offset_model_cell
            .borrow()
            .clone()
            .expect("offset model captured");
        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        let dialog =
            visual_bounds_for_element(&mut app, window, drawer_content_id).expect("drawer visual");
        let start = Point::new(
            Px(dialog.origin.x.0 + dialog.size.width.0 * 0.5),
            Px(dialog.origin.y.0 + 10.0),
        );
        let end = Point::new(start.x, Px(start.y.0 + 220.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: start,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let offset_after_move = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        assert!(
            offset_after_move.0 > offset.0 + 1.0,
            "expected move to increase offset from {offset:?}, got {offset_after_move:?}"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: end,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        for _ in 0..(DRAWER_SNAP_SETTLE_TICKS + 4) {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_frame(&mut ui, &mut app, &mut services);
        }

        let offset_model = offset_model_cell
            .borrow()
            .clone()
            .expect("offset model captured");
        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        let expected = Px(180.0);
        assert!(
            (offset.0 - expected.0).abs() < 1.0,
            "expected offset near {expected:?}, got {offset:?}"
        );
    }

    #[test]
    fn drawer_close_closes_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let close_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_200 as usize + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));

            let open_for_drawer = open.clone();
            let open_for_content = open.clone();

            OverlayController::begin_frame(&mut app, window);
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
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
                        |_cx, _st| Vec::new(),
                    );

                    let close_id_out = close_id.clone();
                    let drawer = Drawer::new(open_for_drawer.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let close = DrawerClose::new(open_for_content.clone())
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .relative()
                                        .w_px(MetricRef::Px(Px(24.0)))
                                        .h_px(MetricRef::Px(Px(24.0))),
                                )
                                .into_element(cx);
                            close_id_out.set(Some(close.id));
                            DrawerContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                                close,
                            ])
                            .into_element(cx)
                        },
                    );

                    vec![drawer]
                },
            );
            ui.set_root(root);
            OverlayController::render(&mut ui, &mut app, &mut services, window, b);
            ui.layout_all(&mut app, &mut services, b, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, b, &mut scene, 1.0);
        }

        let close_element = close_id.get().expect("close element id");
        let close_node = fret_ui::elements::node_for_element(&mut app, window, close_element)
            .expect("close node");
        let close_bounds = visual_bounds_for_element(&mut app, window, close_element)
            .expect("close visual bounds");
        let point = Point::new(
            Px(close_bounds.origin.x.0 + 2.0),
            Px(close_bounds.origin.y.0 + 2.0),
        );
        assert!(
            close_bounds.contains(point),
            "expected click point inside close bounds; point={point:?} bounds={close_bounds:?}"
        );
        assert!(
            b.contains(point),
            "expected click point inside window bounds; point={point:?} window={b:?}"
        );

        let pre_hit = ui.debug_hit_test(point).hit.unwrap_or_else(|| {
            panic!("pre-hit missing; point={point:?} close_bounds={close_bounds:?} window={b:?}")
        });
        let pre_path = ui.debug_node_path(pre_hit);
        assert!(
            pre_path.contains(&close_node),
            "expected click point to hit close subtree; point={point:?} hit={pre_hit:?} close={close_node:?} path={pre_path:?}"
        );

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

        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn drawer_close_transition_keeps_modal_barrier_blocking_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed.
        render_drawer_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_drawer_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected drawer to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> barrier must remain active.
        render_drawer_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected barrier root to remain while the drawer is closing");
        let barrier_layer = ui.node_layer(barrier_root).expect("barrier layer");
        let barrier = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == barrier_layer)
            .expect("barrier debug layer info");
        assert!(barrier.visible);
        assert!(barrier.hit_testable);
        assert!(
            barrier.blocks_underlay_input,
            "expected modal barrier layer to block underlay input"
        );

        // Click the underlay. The modal barrier should block the click-through while closing.
        let click = Point::new(Px(10.0), Px(10.0));
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
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should remain inert while the drawer is closing"
        );

        // After the exit transition settles, the barrier must drop and the underlay becomes
        // interactive again.
        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_300 + 2;
        for _ in 0..settle_frames {
            render_drawer_frame_with_underlay(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                underlay_activated.clone(),
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_none(),
            "expected barrier root to clear once the exit transition completes"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
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
                pointer_id: fret_core::PointerId(1),
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(true),
            "underlay should activate once the barrier is removed"
        );
    }
}
