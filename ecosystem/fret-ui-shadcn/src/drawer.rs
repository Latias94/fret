//! shadcn/ui `Drawer` facade.
//!
//! Fret currently models drawers as a `Sheet` that defaults to the `Bottom` side.

use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, MouseButton, Point, Px, SemanticsRole, TextAlign, Transform2D,
};
use fret_runtime::{Model, ModelStore, TickId};
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, LayoutStyle, Length, MarginEdge, MarginEdges,
    PointerRegionProps, RenderTransformProps, SemanticsDecoration, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::motion::inertia::{InertiaBounds, InertiaSimulation};
use fret_ui_headless::motion::simulation::Simulation1D;
use fret_ui_headless::motion::tolerance::Tolerance;

use crate::layout as shadcn_layout;
use crate::sheet::Sheet;
pub use crate::sheet::{
    SheetDescription as DrawerDescription, SheetModalMode as DrawerModalMode,
    SheetSide as DrawerSide, SheetTitle as DrawerTitle,
};

pub type DrawerDirection = DrawerSide;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion_springs::{
    shadcn_drawer_inertia_bounce_spring_description, shadcn_drawer_settle_spring_description,
};
use fret_ui_kit::declarative::motion_value::{
    MotionKickF32, MotionToSpecF32, MotionValueF32Update, SpringSpecF32, drive_motion_value_f32,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, Space, UiPatch, UiPatchTarget,
    UiSupportsChrome, UiSupportsLayout, ui,
};

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;
type OnSnapPointChange = Arc<dyn Fn(Option<usize>) + Send + Sync + 'static>;

const DRAWER_EDGE_GAP_PX: Px = Px(96.0);
const DRAWER_MAX_HEIGHT_FRACTION: f32 = 0.8;
const DRAWER_SIDE_PANEL_WIDTH_FRACTION: f32 = 0.75;
const DRAWER_SIDE_PANEL_MAX_WIDTH_PX: Px = Px(384.0);

/// shadcn/ui `DrawerPortal` (v4).
///
/// In upstream (Vaul/Radix), `Portal` controls *where* the drawer is mounted in the DOM. In Fret,
/// overlay mounting is owned by the per-window overlay manager, so this type exists for taxonomy
/// parity only. The `Drawer` recipe always renders into an overlay root.
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawerPortal;

impl DrawerPortal {
    pub fn new() -> Self {
        Self
    }
}

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
    let mut out = Vec::new();
    for point in points {
        let Some(fraction) = (match point {
            DrawerSnapPoint::Fraction(f) if f.is_finite() && f > 0.0 => Some(f.min(1.0)),
            _ => None,
        }) else {
            continue;
        };
        if out
            .iter()
            .any(|existing: &f32| (*existing - fraction).abs() < f32::EPSILON)
        {
            continue;
        }
        out.push(fraction);
    }
    out
}

fn drawer_default_snap_point_index(points: &[f32], explicit_index: Option<usize>) -> Option<usize> {
    if points.is_empty() {
        return None;
    }
    if let Some(index) = explicit_index.filter(|index| *index < points.len()) {
        return Some(index);
    }
    points
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

fn drawer_resolve_snap_point_index(
    points: &[f32],
    explicit_default_index: Option<usize>,
    requested_index: Option<usize>,
) -> Option<usize> {
    requested_index
        .filter(|index| *index < points.len())
        .or_else(|| drawer_default_snap_point_index(points, explicit_default_index))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DrawerResolvedSnapTarget {
    model_index: usize,
    visible: Px,
    offset: Px,
}

fn drawer_resolved_snap_targets(
    points: &[f32],
    drawer_height: Px,
    window_height: Px,
) -> Vec<DrawerResolvedSnapTarget> {
    points
        .iter()
        .copied()
        .enumerate()
        .map(|(model_index, fraction)| {
            let desired_visible = Px((window_height.0 * fraction).max(0.0));
            let visible = Px(desired_visible.0.min(drawer_height.0).max(0.0));
            let offset = Px((drawer_height.0 - visible.0).max(0.0));
            DrawerResolvedSnapTarget {
                model_index,
                visible,
                offset,
            }
        })
        .collect()
}

fn drawer_resolved_snap_target_for_index(
    targets: &[DrawerResolvedSnapTarget],
    index: usize,
) -> Option<DrawerResolvedSnapTarget> {
    targets
        .iter()
        .copied()
        .find(|target| target.model_index == index)
}

fn drawer_resolved_snap_target_closest_by_offset(
    targets: &[DrawerResolvedSnapTarget],
    offset: Px,
) -> Option<DrawerResolvedSnapTarget> {
    targets.iter().copied().min_by(|a, b| {
        let da = (a.offset.0 - offset.0).abs();
        let db = (b.offset.0 - offset.0).abs();
        da.total_cmp(&db)
    })
}

fn drawer_set_snap_point_with_callback(
    models: &mut ModelStore,
    model: &Model<Option<usize>>,
    on_change: Option<&OnSnapPointChange>,
    next: Option<usize>,
) -> bool {
    let current = models.get_cloned(model).unwrap_or(None);
    if current == next {
        return false;
    }
    let _ = models.update(model, |value| *value = next);
    if let Some(on_change) = on_change {
        on_change(next);
    }
    true
}

#[derive(Debug, Clone, Copy, Default)]
struct DrawerSideProviderState {
    current: DrawerSide,
}

fn inherited_drawer_side<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<DrawerSide> {
    cx.provided::<DrawerSideProviderState>()
        .map(|st| st.current)
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
    cx.provide(DrawerSideProviderState { current: side }, f)
}

fn drawer_drag_snap_height(drawer_height: Px, window_height: Px, side: DrawerSide) -> Px {
    // Snap-point math should be based on the border-box height.
    //
    // The layout engine treats max-size constraints as border-box under Tailwind-style
    // `box-sizing: border-box`, so the measured drawer bounds already include the edge border.
    //
    // When the content subtree's intrinsic height exceeds the sheet's max-height clamp, layout
    // bounds can report a taller value than what is actually visible. For Vaul-style snap points
    // we want the *effective* drawer height, so clamp to the same max-height fraction used by the
    // recipe (`max-h-[80vh]`).
    if matches!(side, DrawerSide::Top | DrawerSide::Bottom) {
        let max_h = Px((window_height.0 * DRAWER_MAX_HEIGHT_FRACTION).max(0.0));
        return Px(drawer_height.0.min(max_h.0));
    }
    drawer_height
}

fn collect_built_drawer_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

fn apply_drawer_header_text_alignment(mut element: AnyElement, align: TextAlign) -> AnyElement {
    let apply_text = |layout: &mut LayoutStyle, text_align: &mut TextAlign| {
        if matches!(layout.size.width, Length::Auto) {
            layout.size.width = Length::Fill;
        }
        if layout.size.min_width.is_none() {
            layout.size.min_width = Some(Length::Px(Px(0.0)));
        }
        *text_align = align;
    };

    match &mut element.kind {
        ElementKind::Text(props) => apply_text(&mut props.layout, &mut props.align),
        ElementKind::StyledText(props) => apply_text(&mut props.layout, &mut props.align),
        ElementKind::SelectableText(props) => apply_text(&mut props.layout, &mut props.align),
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_drawer_header_text_alignment(child, align))
        .collect();
    element
}

/// shadcn/ui `DrawerContent` (v4).
#[derive(Debug)]
pub struct DrawerContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    drag_handle_test_id: Option<Arc<str>>,
}

impl DrawerContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            drag_handle_test_id: None,
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> DrawerContentBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        DrawerContentBuild {
            build: Some(build),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            drag_handle_test_id: None,
            test_id: None,
            _phantom: PhantomData,
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

    pub fn drag_handle_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.drag_handle_test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let side = drawer_side_in_scope(cx);

        let bg = theme.color_token("background");
        let border = theme.color_token("border");
        let muted = theme.color_token("muted");
        let radius = theme.metric_token("metric.radius.lg");

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

        let viewport_bounds = cx.environment_viewport_bounds(fret_ui::Invalidation::Layout);
        let window_height =
            fret_ui_kit::OverlayController::last_known_window_bounds(cx.app, cx.window)
                .unwrap_or(viewport_bounds)
                .size
                .height;
        let cap = (window_height.0 * DRAWER_MAX_HEIGHT_FRACTION).max(0.0);
        let by_gap = (window_height.0 - DRAWER_EDGE_GAP_PX.0).max(0.0);
        let max_height = Px(cap.min(by_gap));

        let base_layout = match side {
            DrawerSide::Left | DrawerSide::Right => LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0()
                .overflow_visible(),
            DrawerSide::Top | DrawerSide::Bottom => LayoutRefinement::default()
                .w_full()
                .max_h(max_height)
                .min_w_0()
                .min_h_0()
                .overflow_visible(),
        };
        let layout = base_layout.merge(self.layout);

        let mut props = decl_style::container_props(&theme, chrome, layout);
        props.padding = Edges::all(Px(0.0)).into();
        props.shadow = None;
        props.border = borders;
        props.corner_radii = corners;

        let children = self.children;
        let drag_handle_test_id = self.drag_handle_test_id;

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
                    padding: Edges::all(Px(0.0)).into(),
                    background: Some(muted),
                    shadow: None,
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );
            let bar = if let Some(id) = drag_handle_test_id {
                bar.test_id(id)
            } else {
                bar
            };
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
                shadcn_layout::HStackProps::default()
                    .gap(Space::N0)
                    .justify_center()
                    .items_center(),
                vec![bar],
            ));
        }
        rows.extend(children);

        let stack_layout = match side {
            DrawerSide::Left | DrawerSide::Right => LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0(),
            DrawerSide::Top | DrawerSide::Bottom => {
                LayoutRefinement::default().w_full().min_w_0().min_h_0()
            }
        };
        let content = cx.container(props, move |cx| {
            vec![
                ui::v_stack(move |_cx| rows)
                    .gap(Space::N0)
                    .layout(stack_layout)
                    .items_stretch()
                    .into_element(cx),
            ]
        });

        content.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Dialog))
    }
}

pub struct DrawerContentBuild<H, B> {
    build: Option<B>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    drag_handle_test_id: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> DrawerContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn drag_handle_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.drag_handle_test_id = Some(id.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut content = DrawerContent::new(collect_built_drawer_children(
            cx,
            self.build.expect("expected drawer content build closure"),
        ))
        .refine_style(self.chrome)
        .refine_layout(self.layout);
        if let Some(id) = self.drag_handle_test_id {
            content = content.drag_handle_test_id(id);
        }
        let content = content.into_element(cx);
        if let Some(id) = self.test_id {
            content.test_id(id)
        } else {
            content
        }
    }
}

impl<H: UiHost, B> UiPatchTarget for DrawerContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for DrawerContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for DrawerContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for DrawerContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerContentBuild::into_element(self, cx)
    }
}

/// shadcn/ui `DrawerHeader` (v4).
#[derive(Debug)]
pub struct DrawerHeader {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    text_align: Option<TextAlign>,
}

impl DrawerHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            text_align: None,
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> DrawerHeaderBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        DrawerHeaderBuild {
            build: Some(build),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            text_align: None,
            _phantom: PhantomData,
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

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let side = drawer_side_in_scope(cx);
        let md_breakpoint = cx
            .environment_viewport_width(fret_ui::Invalidation::Layout)
            .0
            >= fret_ui_kit::declarative::viewport_tailwind::MD.0;
        let centered = matches!(side, DrawerSide::Top | DrawerSide::Bottom) && !md_breakpoint;
        let text_align = self.text_align.unwrap_or_else(|| {
            if centered {
                TextAlign::Center
            } else {
                TextAlign::Start
            }
        });
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4).merge(self.chrome),
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .merge(self.layout),
        );
        let gap = if md_breakpoint {
            Space::N1p5
        } else {
            Space::N0p5
        };
        let children = self
            .children
            .into_iter()
            .map(|child| apply_drawer_header_text_alignment(child, text_align))
            .collect();
        shadcn_layout::container_vstack(
            cx,
            props,
            shadcn_layout::VStackProps::default()
                .gap(gap)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_stretch(),
            children,
        )
    }
}

pub struct DrawerHeaderBuild<H, B> {
    build: Option<B>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    text_align: Option<TextAlign>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> DrawerHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut header = DrawerHeader::new(collect_built_drawer_children(
            cx,
            self.build.expect("expected drawer header build closure"),
        ))
        .refine_style(self.chrome)
        .refine_layout(self.layout);
        if let Some(align) = self.text_align {
            header = header.text_align(align);
        }
        header.into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for DrawerHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for DrawerHeaderBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for DrawerHeaderBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for DrawerHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerHeaderBuild::into_element(self, cx)
    }
}

/// shadcn/ui `DrawerFooter` (v4).
#[derive(Debug)]
pub struct DrawerFooter {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl DrawerFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> DrawerFooterBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        DrawerFooterBuild {
            build: Some(build),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4).merge(self.chrome),
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .mt_auto()
                .merge(self.layout),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            shadcn_layout::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_stretch(),
            children,
        )
    }
}

pub struct DrawerFooterBuild<H, B> {
    build: Option<B>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> DrawerFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerFooter::new(collect_built_drawer_children(
            cx,
            self.build.expect("expected drawer footer build closure"),
        ))
        .refine_style(self.chrome)
        .refine_layout(self.layout)
        .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for DrawerFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for DrawerFooterBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for DrawerFooterBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for DrawerFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerFooterBuild::into_element(self, cx)
    }
}

#[derive(Clone)]
pub struct Drawer {
    open: Model<bool>,
    side: DrawerSide,
    inner: Sheet,
    drag_to_dismiss: bool,
    snap_points: Option<Vec<f32>>,
    snap_point: Option<Model<Option<usize>>>,
    default_snap_point_index: Option<usize>,
    snap_to_sequential_points: bool,
    on_snap_point_change: Option<OnSnapPointChange>,
}

impl std::fmt::Debug for Drawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawer")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("drag_to_dismiss", &self.drag_to_dismiss)
            .field("snap_points", &self.snap_points.as_ref().map(|v| v.len()))
            .field("snap_point", &self.snap_point.as_ref().map(|_| "<model>"))
            .field("default_snap_point_index", &self.default_snap_point_index)
            .field("snap_to_sequential_points", &self.snap_to_sequential_points)
            .field("on_snap_point_change", &self.on_snap_point_change.is_some())
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
            snap_point: None,
            default_snap_point_index: None,
            snap_to_sequential_points: false,
            on_snap_point_change: None,
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

    /// Base UI-compatible alias.
    ///
    /// When `true`, outside pointer press does not dismiss the drawer.
    /// This is equivalent to `overlay_closable(false)`.
    pub fn disable_pointer_dismissal(mut self, disable: bool) -> Self {
        self.inner = self.inner.disable_pointer_dismissal(disable);
        self
    }

    /// Enables or disables the modal overlay barrier.
    ///
    /// Default: `true`.
    pub fn modal(mut self, modal: bool) -> Self {
        self.inner = self.inner.modal(modal);
        self
    }

    /// Sets the drawer modal policy in one call.
    ///
    /// This is the closest Fret equivalent to Base UI's `modal={true | false | 'trap-focus'}`
    /// root prop while keeping the policy in the recipe layer.
    pub fn modal_mode(mut self, mode: DrawerModalMode) -> Self {
        self.inner = self.inner.modal_mode(mode);
        self
    }

    /// Base UI-style trap-focus mode.
    ///
    /// This traps Tab focus inside the drawer while leaving outside pointer interaction enabled.
    pub fn modal_trap_focus(mut self, trap: bool) -> Self {
        self.inner = self.inner.modal_trap_focus(trap);
        self
    }

    pub fn overlay_color(mut self, overlay_color: fret_core::Color) -> Self {
        self.inner = self.inner.overlay_color(overlay_color);
        self
    }

    /// Returns a recipe-level composition builder for shadcn-style part assembly.
    ///
    /// This bridges Fret's closure-root authoring model with the nested part mental model used by
    /// shadcn/Vaul while keeping the underlying mechanism surface unchanged.
    pub fn compose<H: UiHost>(self) -> DrawerComposition<H> {
        DrawerComposition::new(self)
    }

    /// Returns a part-children builder for root-level authoring that is closer to upstream nested
    /// children composition while still lowering into the existing recipe-layer slots.
    pub fn children<H: UiHost, I, P>(self, parts: I) -> DrawerChildren<H>
    where
        I: IntoIterator<Item = P>,
        P: Into<DrawerPart<H>>,
    {
        DrawerChildren::new(self, parts)
    }

    /// Host-bound builder-first helper that late-lands the trigger/content at the root call site.
    #[track_caller]
    pub fn build<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl IntoUiElement<H>,
        content: impl IntoUiElement<H>,
    ) -> AnyElement {
        self.into_element(
            cx,
            move |cx| trigger.into_element(cx),
            move |cx| content.into_element(cx),
        )
    }

    pub fn overlay_component(mut self, overlay: DrawerOverlay) -> Self {
        if let Some(color) = overlay.color {
            self.inner = self.inner.overlay_color(color);
        }
        self
    }

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over [`Drawer::into_element`] that accepts shadcn-style parts
    /// (`DrawerTrigger`, `DrawerPortal`, `DrawerOverlay`).
    ///
    /// It also installs a default "open on activate" behavior on the trigger element when the
    /// trigger is a `Pressable` (e.g. shadcn `Button`), matching the upstream trigger contract.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> DrawerTrigger,
        _portal: DrawerPortal,
        overlay: DrawerOverlay,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let drawer = self.overlay_component(overlay);
        let open_for_trigger = drawer.open.clone();
        drawer.into_element(
            cx,
            move |cx| {
                let trigger_el = trigger(cx).into_element(cx);
                let open = open_for_trigger.clone();
                cx.pressable_add_on_activate_for(
                    trigger_el.id,
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiActionHost,
                              acx: fret_ui::action::ActionCx,
                              _reason: fret_ui::action::ActivateReason| {
                            let _ = host.models_mut().update(&open, |v| *v = true);
                            host.request_redraw(acx.window);
                        },
                    ),
                );
                trigger_el
            },
            content,
        )
    }

    /// Enables Vaul-style snap points for bottom drawers.
    ///
    /// Notes:
    /// - Only modeled for `Bottom` drawers today.
    /// - Points are expressed as fractions of the window height, matching Vaul's default authoring
    ///   style.
    /// - When enabled, releasing a drag will settle to the nearest snap point; dragging far enough
    ///   down will still close the drawer.
    pub fn snap_points(mut self, points: impl IntoIterator<Item = DrawerSnapPoint>) -> Self {
        let points: Vec<DrawerSnapPoint> = points.into_iter().collect();
        let points = normalize_snap_points(points);
        self.snap_points = if points.is_empty() {
            None
        } else {
            Some(points)
        };
        self
    }

    /// Sets the active snap point model (Base UI `snapPoint`).
    ///
    /// The model stores an authored snap-point index into the `snap_points(...)` list. When the
    /// model is `None` or out of range, the drawer falls back to `default_snap_point(...)` or the
    /// largest authored snap point.
    pub fn snap_point(mut self, snap_point: Model<Option<usize>>) -> Self {
        self.snap_point = Some(snap_point);
        self
    }

    /// Overrides which snap point is used as the initial "open" position.
    ///
    /// When unset, the largest snap point is used (most open), matching typical Vaul usage.
    pub fn default_snap_point(mut self, index: usize) -> Self {
        self.default_snap_point_index = Some(index);
        self
    }

    /// Disables velocity-based snap skipping so a drag only advances one snap point at a time.
    pub fn snap_to_sequential_points(mut self, enabled: bool) -> Self {
        self.snap_to_sequential_points = enabled;
        self
    }

    /// Called when the active snap point changes due to drawer-owned interactions or close reset.
    pub fn on_snap_point_change(
        mut self,
        on_snap_point_change: impl Fn(Option<usize>) + Send + Sync + 'static,
    ) -> Self {
        self.on_snap_point_change = Some(Arc::new(on_snap_point_change));
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

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.inner = self.inner.on_open_change(on_open_change);
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.inner = self.inner.on_open_change_complete(on_open_change_complete);
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

    /// Upstream-aligned placement setter (`direction` in shadcn/ui / Vaul docs).
    pub fn direction(self, direction: DrawerDirection) -> Self {
        self.side(direction)
    }

    /// Optional compatibility escape hatch: allow non-bottom drawers by forwarding to `Sheet`.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self.inner = self.inner.side(side);
        self
    }

    #[track_caller]
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
        let snap_point_model = snap_points.as_ref().map(|points| {
            controllable_state::use_controllable_model(cx, self.snap_point.clone(), || {
                drawer_default_snap_point_index(points, self.default_snap_point_index)
            })
            .model()
        });
        let default_snap_point_index = self.default_snap_point_index;
        let snap_to_sequential_points = self.snap_to_sequential_points;
        let on_snap_point_change = self.on_snap_point_change.clone();

        let mut inner = self
            .inner
            .vertical_edge_gap_px(DRAWER_EDGE_GAP_PX)
            .vertical_auto_max_height_fraction(DRAWER_MAX_HEIGHT_FRACTION);
        match side {
            DrawerSide::Left | DrawerSide::Right => {
                inner = inner
                    .size_fraction(DRAWER_SIDE_PANEL_WIDTH_FRACTION)
                    .max_size(DRAWER_SIDE_PANEL_MAX_WIDTH_PX);
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
            let viewport_bounds = cx.environment_viewport_bounds(fret_ui::Invalidation::Layout);
            let window_height =
                fret_ui_kit::OverlayController::last_known_window_bounds(cx.app, cx.window)
                    .unwrap_or(viewport_bounds)
                    .size
                    .height;
            let _ = cx.app.models_mut().update(&runtime, |st| {
                st.window_height = window_height;
                st.viewport_height = viewport_bounds.size.height;
            });
            let has_snap_points = snap_points.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
            let requested_snap_point_index = snap_point_model
                .as_ref()
                .and_then(|model| cx.watch_model(model).layout().cloned().flatten());

            if let Some(bounds) = cx.last_bounds_for_element(content.id) {
                let drawer_h = drawer_drag_snap_height(bounds.size.height, window_height, side);
                let _ = cx.app.models_mut().update(&runtime, |st| {
                    if st.drawer_height != drawer_h {
                        st.drawer_height = drawer_h;
                    }
                });
            }

            if !is_open && was_open {
                if let (Some(points), Some(model)) =
                    (snap_points.as_ref(), snap_point_model.as_ref())
                {
                    let default_index =
                        drawer_default_snap_point_index(points, default_snap_point_index);
                    let _ = drawer_set_snap_point_with_callback(
                        cx.app.models_mut(),
                        model,
                        on_snap_point_change.as_ref(),
                        default_index,
                    );
                }
            }

            if is_open && !was_open {
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = Px(0.0));
                if has_snap_points {
                    let _ = cx.app.models_mut().update(&runtime, |st| {
                        st.needs_snap_init = true;
                        st.settling = false;
                        st.applied_snap_point_index = None;
                    });
                }
            }
            drawer_drag_set_was_open(cx, is_open);

            if !is_open {
                let _ = cx.app.models_mut().update(&runtime, |st| {
                    st.needs_snap_init = false;
                    st.settling = false;
                    st.applied_snap_point_index = None;
                });
            }

            if is_open && has_snap_points {
                let points = snap_points.as_ref().expect("snap points");
                let resolved_snap_point_index = drawer_resolve_snap_point_index(
                    points,
                    default_snap_point_index,
                    requested_snap_point_index,
                );
                let needs_init = cx
                    .app
                    .models()
                    .get_copied(&runtime)
                    .map(|st| st.needs_snap_init)
                    .unwrap_or(false);

                if needs_init {
                    if let Some(bounds) = cx.last_bounds_for_element(content.id) {
                        let drawer_h =
                            drawer_drag_snap_height(bounds.size.height, window_height, side);
                        let targets = drawer_resolved_snap_targets(points, drawer_h, window_height);
                        let _ = cx.app.models_mut().update(&runtime, |st| {
                            st.drawer_height = drawer_h;
                        });

                        if let Some(target) = resolved_snap_point_index.and_then(|index| {
                            drawer_resolved_snap_target_for_index(&targets, index)
                        }) {
                            let _ = cx.app.models_mut().update(&offset_model, |v| {
                                *v = target.offset;
                            });
                            let _ = cx.app.models_mut().update(&runtime, |st| {
                                st.needs_snap_init = false;
                                st.applied_snap_point_index = Some(target.model_index);
                            });
                        } else {
                            let _ = cx.app.models_mut().update(&runtime, |st| {
                                st.needs_snap_init = false;
                                st.applied_snap_point_index = None;
                            });
                        }
                    }
                } else if let Some(bounds) = cx.last_bounds_for_element(content.id) {
                    let drawer_h = drawer_drag_snap_height(bounds.size.height, window_height, side);
                    let targets = drawer_resolved_snap_targets(points, drawer_h, window_height);
                    let runtime_snapshot = cx.app.models().get_copied(&runtime).unwrap_or_default();
                    if !runtime_snapshot.dragging
                        && resolved_snap_point_index != runtime_snapshot.applied_snap_point_index
                    {
                        if let Some(target) = resolved_snap_point_index.and_then(|index| {
                            drawer_resolved_snap_target_for_index(&targets, index)
                        }) {
                            let current_offset =
                                cx.app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
                            if (current_offset.0 - target.offset.0).abs() > 0.5 {
                                let _ = cx.app.models_mut().update(&runtime, |st| {
                                    st.settling = true;
                                    st.settle_to = target.offset;
                                    st.settle_seq = st.settle_seq.saturating_add(1).max(1);
                                    st.settle_velocity = 0.0;
                                    st.applied_snap_point_index = Some(target.model_index);
                                });
                            } else {
                                let _ = cx.app.models_mut().update(&offset_model, |v| {
                                    *v = target.offset;
                                });
                                let _ = cx.app.models_mut().update(&runtime, |st| {
                                    st.settling = false;
                                    st.settle_velocity = 0.0;
                                    st.applied_snap_point_index = Some(target.model_index);
                                });
                            }
                        }
                    }
                }
            }

            let mut offset = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let runtime_snapshot = cx.app.models().get_copied(&runtime);
            if let Some(st) = runtime_snapshot {
                let spring = shadcn_drawer_settle_spring_description(&*cx.app);

                let update = if st.settling {
                    MotionValueF32Update::To {
                        target: st.settle_to.0,
                        spec: MotionToSpecF32::Spring(SpringSpecF32 {
                            spring,
                            tolerance: Tolerance::default(),
                            snap_to_target: true,
                        }),
                        kick: Some(MotionKickF32 {
                            id: st.settle_seq,
                            velocity: st.settle_velocity,
                        }),
                    }
                } else {
                    MotionValueF32Update::Snap(offset.0)
                };

                let out = drive_motion_value_f32(cx, offset.0, update);
                let next = Px(out.value.max(0.0).min(window_height.0));
                if next != offset {
                    offset = next;
                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                }

                if st.settling {
                    let _ = cx.app.models_mut().update(&runtime, |st| {
                        st.settling = out.animating;
                        if !out.animating {
                            st.settle_velocity = 0.0;
                        }
                    });
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
                    st.start = down.position;
                    st.start_offset = start_offset;
                    st.settling = false;
                    st.last_offset = start_offset;
                    st.last_tick = down.tick_id;
                    st.velocity = 0.0;
                });
                host.request_redraw(_cx.window);
                true
            });

            let runtime_for_move = runtime.clone();
            let offset_for_move = offset_model.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let Ok((dragging, start, start_offset, last_tick, last_offset)) =
                    host.models_mut().read(&runtime_for_move, |st| {
                        (
                            st.dragging,
                            st.start,
                            st.start_offset,
                            st.last_tick,
                            st.last_offset,
                        )
                    })
                else {
                    return false;
                };
                if !dragging {
                    return false;
                }

                let dy = mv.position.y.0 - start.y.0;
                let next = Px((start_offset.0 + dy).max(0.0).min(window_height.0));
                let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                let velocity = mv.velocity_window.map(|v| v.y.0).unwrap_or_else(|| {
                    let dt_ticks = mv.tick_id.0.saturating_sub(last_tick.0);
                    if dt_ticks == 0 {
                        return 0.0;
                    }
                    let dt_secs = dt_ticks as f32 / 60.0;
                    (next.0 - last_offset.0) / dt_secs
                });
                let _ = host.models_mut().update(&runtime_for_move, |st| {
                    st.last_offset = next;
                    st.last_tick = mv.tick_id;
                    st.velocity = velocity.clamp(-5000.0, 5000.0);
                });
                host.request_redraw(_cx.window);
                true
            });

            let open_for_up = open.clone();
            let runtime_for_up = runtime.clone();
            let offset_for_up = offset_model.clone();
            let snap_points_for_up = snap_points.clone();
            let snap_point_model_for_up = snap_point_model.clone();
            let on_snap_point_change_for_up = on_snap_point_change.clone();
            let inertia_bounce_spring = shadcn_drawer_inertia_bounce_spring_description(&*cx.app);
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, _cx, up| {
                let Ok((
                    dragging,
                    stored_velocity,
                    stored_drawer_h,
                    stored_start_offset,
                    applied_snap_point_index,
                )) = host.models_mut().read(&runtime_for_up, |st| {
                    (
                        st.dragging,
                        st.velocity,
                        st.drawer_height,
                        st.start_offset,
                        st.applied_snap_point_index,
                    )
                })
                else {
                    return false;
                };
                if !dragging {
                    return false;
                }
                let velocity_window = up.velocity_window.map(|v| v.y.0);
                let velocity_is_measured = velocity_window.is_some();
                let velocity = velocity_window.unwrap_or(stored_velocity);

                let bounds = host.bounds();
                let drawer_h = if stored_drawer_h.0 > 0.0 {
                    stored_drawer_h
                } else {
                    drawer_drag_snap_height(bounds.size.height, window_height, side)
                };
                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let projected_offset = if velocity_is_measured
                    && velocity.abs() >= DRAWER_DRAG_FLING_MIN_VELOCITY_PX_PER_SEC
                {
                    let sim = InertiaSimulation::new(
                        offset.0 as f64,
                        velocity as f64,
                        DRAWER_DRAG_FLING_DRAG,
                        Some(InertiaBounds {
                            min: 0.0,
                            max: window_height.0 as f64,
                        }),
                        inertia_bounce_spring,
                        Tolerance::default(),
                    );
                    Px(sim.x(DRAWER_DRAG_FLING_PROJECTION_TIME) as f32)
                } else {
                    offset
                };
                let projected_offset = Px(projected_offset.0.clamp(0.0, window_height.0));

                let has_snap_points = snap_points_for_up
                    .as_ref()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                if has_snap_points {
                    let points = snap_points_for_up.as_ref().expect("snap points");
                    let targets = drawer_resolved_snap_targets(points, drawer_h, window_height);

                    let min_visible = targets
                        .iter()
                        .map(|target| target.visible)
                        .filter(|visible| visible.0 > 0.0)
                        .min_by(|a, b| a.0.total_cmp(&b.0));

                    let close_threshold = min_visible
                        .map(|v| Px((drawer_h.0 - v.0 * 0.5).max(DRAWER_DRAG_DISMISS_MIN_PX)))
                        .unwrap_or_else(|| Px((drawer_h.0 * 0.25).max(DRAWER_DRAG_DISMISS_MIN_PX)));
                    let close_threshold = Px(close_threshold.0.max(
                        (drawer_h.0 * DRAWER_DRAG_SNAP_DISMISS_MIN_DRAWER_FRACTION).max(
                            (window_height.0 * DRAWER_DRAG_SNAP_DISMISS_MIN_VIEWPORT_FRACTION)
                                .max(DRAWER_DRAG_DISMISS_MIN_PX),
                        ),
                    ));

                    if projected_offset.0 > close_threshold.0 {
                        let _ = host.models_mut().update(&offset_for_up, |v| *v = Px(0.0));
                        let _ = host.models_mut().update(&open_for_up, |v| *v = false);
                    } else {
                        let nearest = if snap_to_sequential_points {
                            let mut ordered_targets = targets.clone();
                            ordered_targets.sort_by(|a, b| a.offset.0.total_cmp(&b.offset.0));

                            let current_target = applied_snap_point_index
                                .and_then(|index| {
                                    drawer_resolved_snap_target_for_index(&ordered_targets, index)
                                })
                                .or_else(|| {
                                    drawer_resolved_snap_target_closest_by_offset(
                                        &ordered_targets,
                                        stored_start_offset,
                                    )
                                });
                            let mut target = drawer_resolved_snap_target_closest_by_offset(
                                &ordered_targets,
                                projected_offset,
                            )
                            .or_else(|| ordered_targets.first().copied())
                            .expect("snap target");
                            let drag_delta = offset.0 - stored_start_offset.0;
                            let drag_direction = if drag_delta > 0.5 {
                                1isize
                            } else if drag_delta < -0.5 {
                                -1isize
                            } else {
                                0
                            };
                            let velocity_direction =
                                if velocity >= DRAWER_DRAG_FLING_MIN_VELOCITY_PX_PER_SEC {
                                    1isize
                                } else if velocity <= -DRAWER_DRAG_FLING_MIN_VELOCITY_PX_PER_SEC {
                                    -1isize
                                } else {
                                    0
                                };

                            if drag_direction != 0 && drag_direction == velocity_direction {
                                if let Some(current_target) = current_target {
                                    if let Some(current_sorted_index) = ordered_targets
                                        .iter()
                                        .position(|candidate| *candidate == current_target)
                                    {
                                        let adjacent_index = if drag_direction > 0 {
                                            current_sorted_index
                                                .saturating_add(1)
                                                .min(ordered_targets.len().saturating_sub(1))
                                        } else {
                                            current_sorted_index.saturating_sub(1)
                                        };
                                        if adjacent_index != current_sorted_index {
                                            let adjacent_target = ordered_targets[adjacent_index];
                                            let should_force_adjacent = if drag_direction > 0 {
                                                offset.0 < adjacent_target.offset.0
                                            } else {
                                                offset.0 > adjacent_target.offset.0
                                            };
                                            if should_force_adjacent {
                                                target = adjacent_target;
                                            }
                                        } else if drag_direction > 0 {
                                            let _ = host
                                                .models_mut()
                                                .update(&offset_for_up, |v| *v = Px(0.0));
                                            let _ = host
                                                .models_mut()
                                                .update(&open_for_up, |v| *v = false);
                                            let _ =
                                                host.models_mut().update(&runtime_for_up, |st| {
                                                    st.dragging = false;
                                                });
                                            host.release_pointer_capture();
                                            host.request_redraw(_cx.window);
                                            return true;
                                        }
                                    }
                                }
                            }
                            target
                        } else {
                            drawer_resolved_snap_target_closest_by_offset(
                                &targets,
                                projected_offset,
                            )
                            .expect("snap target")
                        };

                        if let Some(model) = snap_point_model_for_up.as_ref() {
                            let _ = drawer_set_snap_point_with_callback(
                                host.models_mut(),
                                model,
                                on_snap_point_change_for_up.as_ref(),
                                Some(nearest.model_index),
                            );
                        }

                        let _ = host.models_mut().update(&runtime_for_up, |st| {
                            st.settling = true;
                            st.settle_to = nearest.offset;
                            st.settle_seq = st.settle_seq.saturating_add(1).max(1);
                            st.settle_velocity = velocity;
                            st.dragging = false;
                            st.applied_snap_point_index = Some(nearest.model_index);
                        });
                        host.release_pointer_capture();
                        host.request_redraw(_cx.window);
                        return true;
                    }
                } else {
                    let threshold = Px((drawer_h.0 * 0.25).max(DRAWER_DRAG_DISMISS_MIN_PX));
                    let should_close = projected_offset.0 >= threshold.0;
                    if should_close {
                        let _ = host.models_mut().update(&open_for_up, |v| *v = false);
                    } else {
                        let _ = host.models_mut().update(&runtime_for_up, |st| {
                            st.settling = true;
                            st.settle_to = Px(0.0);
                            st.settle_seq = st.settle_seq.saturating_add(1).max(1);
                            st.settle_velocity = velocity;
                        });
                    }
                }

                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.dragging = false;
                });
                host.release_pointer_capture();
                host.request_redraw(_cx.window);
                true
            });

            let content_root = cx.pointer_region(
                PointerRegionProps {
                    layout: LayoutStyle::default(),
                    enabled: is_open,
                    ..Default::default()
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

/// Recipe-level builder for composing a drawer from shadcn-style parts.
type DrawerDeferredContent<H> = Box<dyn FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static>;
type DrawerDeferredTrigger<H> =
    Box<dyn FnOnce(&mut ElementContext<'_, H>) -> DrawerTrigger + 'static>;

/// Root-level part adapter for shadcn-style `Drawer` children composition.
///
/// This stays in the recipe layer. It does not widen the mechanism contract in `fret-ui`; it only
/// collects authored parts and lowers them into the existing trigger/content slot surface.
pub enum DrawerPart<H: UiHost> {
    Trigger(DrawerDeferredTrigger<H>),
    Portal(DrawerPortal),
    Overlay(DrawerOverlay),
    Content(DrawerDeferredContent<H>),
}

impl<H: UiHost> std::fmt::Debug for DrawerPart<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(_) => f.write_str("DrawerPart::Trigger(<deferred>)"),
            Self::Portal(portal) => f.debug_tuple("DrawerPart::Portal").field(portal).finish(),
            Self::Overlay(overlay) => f.debug_tuple("DrawerPart::Overlay").field(overlay).finish(),
            Self::Content(_) => f.write_str("DrawerPart::Content(<deferred>)"),
        }
    }
}

impl<H: UiHost> DrawerPart<H> {
    pub fn trigger<T>(trigger: T) -> Self
    where
        T: DrawerCompositionTriggerArg<H> + 'static,
    {
        Self::Trigger(Box::new(move |cx| trigger.into_drawer_trigger(cx)))
    }

    pub fn portal(portal: DrawerPortal) -> Self {
        Self::Portal(portal)
    }

    pub fn overlay(overlay: DrawerOverlay) -> Self {
        Self::Overlay(overlay)
    }

    pub fn content<T>(content: T) -> Self
    where
        T: IntoUiElement<H> + 'static,
    {
        Self::Content(Box::new(move |cx| content.into_element(cx)))
    }

    pub fn content_with(
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static,
    ) -> Self {
        Self::Content(Box::new(content))
    }
}

impl<H: UiHost> From<DrawerTrigger> for DrawerPart<H> {
    fn from(value: DrawerTrigger) -> Self {
        Self::trigger(value)
    }
}

impl<H: UiHost + 'static, T> From<DrawerTriggerBuild<H, T>> for DrawerPart<H>
where
    T: IntoUiElement<H> + 'static,
{
    fn from(value: DrawerTriggerBuild<H, T>) -> Self {
        Self::trigger(value)
    }
}

impl<H: UiHost> From<DrawerPortal> for DrawerPart<H> {
    fn from(value: DrawerPortal) -> Self {
        Self::portal(value)
    }
}

impl<H: UiHost> From<DrawerOverlay> for DrawerPart<H> {
    fn from(value: DrawerOverlay) -> Self {
        Self::overlay(value)
    }
}

impl<H: UiHost> From<DrawerContent> for DrawerPart<H> {
    fn from(value: DrawerContent) -> Self {
        Self::content(value)
    }
}

impl<H: UiHost + 'static, B> From<DrawerContentBuild<H, B>> for DrawerPart<H>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>) + 'static,
{
    fn from(value: DrawerContentBuild<H, B>) -> Self {
        Self::content(value)
    }
}

/// Part-children builder for root-level `Drawer` composition.
///
/// This is the closest Fret recipe surface to upstream nested children today:
/// collect `DrawerPart`s, then late-land them at the root call site.
pub struct DrawerChildren<H: UiHost> {
    drawer: Drawer,
    parts: Vec<DrawerPart<H>>,
}

impl<H: UiHost> std::fmt::Debug for DrawerChildren<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut trigger = 0usize;
        let mut portal = 0usize;
        let mut overlay = 0usize;
        let mut content = 0usize;

        for part in &self.parts {
            match part {
                DrawerPart::Trigger(_) => trigger += 1,
                DrawerPart::Portal(_) => portal += 1,
                DrawerPart::Overlay(_) => overlay += 1,
                DrawerPart::Content(_) => content += 1,
            }
        }

        f.debug_struct("DrawerChildren")
            .field("drawer", &self.drawer)
            .field("trigger_parts", &trigger)
            .field("portal_parts", &portal)
            .field("overlay_parts", &overlay)
            .field("content_parts", &content)
            .finish()
    }
}

impl<H: UiHost> DrawerChildren<H> {
    pub fn new<I, P>(drawer: Drawer, parts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<DrawerPart<H>>,
    {
        Self {
            drawer,
            parts: parts.into_iter().map(Into::into).collect(),
        }
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut trigger: Option<DrawerDeferredTrigger<H>> = None;
        let mut portal = DrawerPortal::new();
        let mut overlay = DrawerOverlay::new();
        let mut content: Option<DrawerDeferredContent<H>> = None;

        for part in self.parts {
            match part {
                DrawerPart::Trigger(next) => {
                    assert!(
                        trigger.replace(next).is_none(),
                        "Drawer::children(...) accepts at most one DrawerTrigger"
                    );
                }
                DrawerPart::Portal(next) => {
                    portal = next;
                }
                DrawerPart::Overlay(next) => {
                    overlay = next;
                }
                DrawerPart::Content(next) => {
                    assert!(
                        content.replace(next).is_none(),
                        "Drawer::children(...) accepts at most one DrawerContent"
                    );
                }
            }
        }

        let trigger =
            trigger.expect("Drawer::children(...) requires one DrawerTrigger-compatible part");
        let content =
            content.expect("Drawer::children(...) requires one DrawerContent-compatible part");

        self.drawer.into_element_parts(
            cx,
            move |cx| trigger(cx),
            portal,
            overlay,
            move |cx| content(cx),
        )
    }
}

impl<H: UiHost> IntoUiElement<H> for DrawerChildren<H> {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerChildren::into_element(self, cx)
    }
}

enum DrawerCompositionContent<H: UiHost> {
    Eager(AnyElement),
    Deferred(DrawerDeferredContent<H>),
}

pub struct DrawerComposition<H: UiHost, TTrigger = DrawerTrigger> {
    drawer: Drawer,
    trigger: Option<TTrigger>,
    portal: DrawerPortal,
    overlay: DrawerOverlay,
    content: Option<DrawerCompositionContent<H>>,
}

impl<H: UiHost, TTrigger> std::fmt::Debug for DrawerComposition<H, TTrigger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawerComposition")
            .field("drawer", &self.drawer)
            .field("trigger", &self.trigger.is_some())
            .field("portal", &self.portal)
            .field("overlay", &self.overlay)
            .field("content", &self.content.is_some())
            .finish()
    }
}

impl<H: UiHost> DrawerComposition<H> {
    pub fn new(drawer: Drawer) -> Self {
        Self {
            drawer,
            trigger: None,
            portal: DrawerPortal::new(),
            overlay: DrawerOverlay::new(),
            content: None,
        }
    }
}

impl<H: UiHost, TTrigger> DrawerComposition<H, TTrigger> {
    pub fn trigger<TNextTrigger>(
        self,
        trigger: TNextTrigger,
    ) -> DrawerComposition<H, TNextTrigger> {
        DrawerComposition {
            drawer: self.drawer,
            trigger: Some(trigger),
            portal: self.portal,
            overlay: self.overlay,
            content: self.content,
        }
    }

    pub fn portal(mut self, portal: DrawerPortal) -> Self {
        self.portal = portal;
        self
    }

    pub fn overlay(mut self, overlay: DrawerOverlay) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(DrawerCompositionContent::Eager(content));
        self
    }

    pub fn content_with(
        mut self,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(DrawerCompositionContent::Deferred(Box::new(content)));
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TTrigger: DrawerCompositionTriggerArg<H>,
    {
        let trigger = self
            .trigger
            .expect("Drawer::compose().trigger(...) must be provided before into_element()")
            .into_drawer_trigger(cx);
        let content = self
            .content
            .expect("Drawer::compose().content(...) must be provided before into_element()");

        let portal = self.portal;
        let overlay = self.overlay;

        match content {
            DrawerCompositionContent::Eager(content) => self.drawer.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |_cx| content,
            ),
            DrawerCompositionContent::Deferred(content) => self.drawer.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |cx| content(cx),
            ),
        }
    }
}

impl<H: UiHost, TTrigger> IntoUiElement<H> for DrawerComposition<H, TTrigger>
where
    TTrigger: DrawerCompositionTriggerArg<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerComposition::into_element(self, cx)
    }
}

const DRAWER_DRAG_HANDLE_HIT_HEIGHT: f32 = 32.0;
const DRAWER_DRAG_HANDLE_HIT_HALF_WIDTH: f32 = 80.0;
const DRAWER_DRAG_DISMISS_MIN_PX: f32 = 30.0;
const DRAWER_DRAG_SNAP_DISMISS_MIN_DRAWER_FRACTION: f32 = 0.8;
const DRAWER_DRAG_SNAP_DISMISS_MIN_VIEWPORT_FRACTION: f32 = 0.25;
const DRAWER_DRAG_FLING_DRAG: f64 = 0.135;
const DRAWER_DRAG_FLING_MIN_VELOCITY_PX_PER_SEC: f32 = 450.0;
const DRAWER_DRAG_FLING_PROJECTION_TIME: std::time::Duration =
    std::time::Duration::from_millis(200);

#[derive(Debug, Clone, Copy)]
struct DrawerDragRuntime {
    dragging: bool,
    start: Point,
    start_offset: Px,
    drawer_height: Px,
    window_height: Px,
    viewport_height: Px,
    last_tick: TickId,
    last_offset: Px,
    velocity: f32,
    needs_snap_init: bool,
    applied_snap_point_index: Option<usize>,
    settling: bool,
    settle_to: Px,
    settle_seq: u64,
    settle_velocity: f32,
}

impl Default for DrawerDragRuntime {
    fn default() -> Self {
        Self {
            dragging: false,
            start: Point::new(Px(0.0), Px(0.0)),
            start_offset: Px(0.0),
            drawer_height: Px(0.0),
            window_height: Px(0.0),
            viewport_height: Px(0.0),
            last_tick: TickId(0),
            last_offset: Px(0.0),
            velocity: 0.0,
            needs_snap_init: false,
            applied_snap_point_index: None,
            settling: false,
            settle_to: Px(0.0),
            settle_seq: 0,
            settle_velocity: 0.0,
        }
    }
}

fn drawer_drag_was_open_state_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_ui::GlobalElementId {
    cx.keyed_slot_id("drawer_drag_was_open")
}

fn drawer_drag_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<DrawerDragRuntime>, Model<Px>, bool) {
    let runtime = cx.local_model_keyed("drawer_drag_runtime", DrawerDragRuntime::default);
    let offset = cx.local_model_keyed("drawer_drag_offset", || Px(0.0));
    let was_open_state = drawer_drag_was_open_state_id(cx);
    let was_open = cx.state_for(was_open_state, || false, |state| *state);
    (runtime, offset, was_open)
}

fn drawer_drag_set_was_open<H: UiHost>(cx: &mut ElementContext<'_, H>, was_open: bool) {
    let was_open_state = drawer_drag_was_open_state_id(cx);
    cx.state_for(
        was_open_state,
        || false,
        |state| {
            *state = was_open;
        },
    );
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
#[derive(Debug)]
pub struct DrawerTrigger {
    child: AnyElement,
}

pub struct DrawerTriggerBuild<H, T> {
    child: Option<T>,
    _phantom: PhantomData<fn() -> H>,
}

impl DrawerTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    /// Builder-first variant that late-lands the trigger child at `into_element(cx)` time.
    pub fn build<H: UiHost, T>(child: T) -> DrawerTriggerBuild<H, T>
    where
        T: IntoUiElement<H>,
    {
        DrawerTriggerBuild {
            child: Some(child),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

impl<H: UiHost, T> DrawerTriggerBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerTrigger::new(
            self.child
                .expect("expected drawer trigger child")
                .into_element(cx),
        )
        .into_element(cx)
    }
}

impl<H: UiHost, T> IntoUiElement<H> for DrawerTriggerBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DrawerTriggerBuild::into_element(self, cx)
    }
}

#[doc(hidden)]
pub trait DrawerCompositionTriggerArg<H: UiHost> {
    fn into_drawer_trigger(self, cx: &mut ElementContext<'_, H>) -> DrawerTrigger;
}

impl<H: UiHost> DrawerCompositionTriggerArg<H> for DrawerTrigger {
    fn into_drawer_trigger(self, _cx: &mut ElementContext<'_, H>) -> DrawerTrigger {
        self
    }
}

impl<H: UiHost, T> DrawerCompositionTriggerArg<H> for DrawerTriggerBuild<H, T>
where
    T: IntoUiElement<H>,
{
    fn into_drawer_trigger(self, cx: &mut ElementContext<'_, H>) -> DrawerTrigger {
        DrawerTrigger::new(
            self.child
                .expect("expected drawer trigger child")
                .into_element(cx),
        )
    }
}

/// shadcn/ui `DrawerClose` (v4).
///
/// Upstream `DrawerClose` is a thin wrapper around the underlying primitive's `Close` component.
/// In Fret, drawers are backed by modal overlays, so this delegates to `DialogClose`.
#[derive(Clone)]
pub struct DrawerClose {
    inner: crate::sheet::SheetClose,
}

impl std::fmt::Debug for DrawerClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawerClose").finish()
    }
}

impl DrawerClose {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            inner: crate::sheet::SheetClose::new(open),
        }
    }

    /// Creates a close affordance that resolves the current drawer/dialog scope at render time.
    pub fn from_scope() -> Self {
        Self {
            inner: crate::sheet::SheetClose::from_scope(),
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

    #[track_caller]
    pub fn build<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        child: impl IntoUiElement<H>,
    ) -> AnyElement {
        self.inner.build(cx, child)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, TextAlign};
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{Effect, FrameId};
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PressableProps, SizeStyle};
    use fret_ui::elements::{
        GlobalElementId, current_bounds_for_element, visual_bounds_for_element,
    };
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
    use fret_ui_kit::ui::UiElementSinkExt as _;

    fn scene_contains_full_window_solid_quad(
        scene: &fret_core::Scene,
        bounds: Rect,
        color: fret_core::Color,
    ) -> bool {
        scene.ops().iter().any(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    rect, background, ..
                } if *rect == bounds && background.paint == fret_core::Paint::Solid(color).into()
            )
        })
    }

    #[test]
    fn drawer_trigger_build_push_ui_accepts_late_landed_child() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let mut out = Vec::new();
            out.push_ui(
                cx,
                DrawerTrigger::build(crate::card::Card::build(|_cx, _out| {})),
            );

            assert_eq!(out.len(), 1);
            assert!(matches!(
                out[0].kind,
                fret_ui::element::ElementKind::Container(_)
            ));
            assert!(out[0].inherited_foreground.is_some());
        });
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
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

    fn find_text_element<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(el),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_element(child, needle)),
        }
    }

    fn find_text<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a fret_ui::element::TextProps> {
        let node = find_text_element(el, needle)?;
        match &node.kind {
            ElementKind::Text(props) => Some(props),
            _ => None,
        }
    }

    #[derive(Clone, Default)]
    struct DrawerSnapHarnessState {
        runtime_model: Rc<RefCell<Option<Model<DrawerDragRuntime>>>>,
        offset_model: Rc<RefCell<Option<Model<Px>>>>,
        drawer_content_id: Rc<RefCell<Option<GlobalElementId>>>,
    }

    fn drawer_snap_test_points() -> Vec<DrawerSnapPoint> {
        vec![
            DrawerSnapPoint::Fraction(0.25),
            DrawerSnapPoint::Fraction(0.5),
            DrawerSnapPoint::Fraction(0.75),
        ]
    }

    fn render_snap_drawer_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        open: &Model<bool>,
        snap_point: Option<&Model<Option<usize>>>,
        default_snap_point_index: Option<usize>,
        snap_to_sequential_points: bool,
        on_snap_point_change: Option<OnSnapPointChange>,
        state: &DrawerSnapHarnessState,
    ) {
        let open_for_drawer = open.clone();
        let snap_point_for_drawer = snap_point.cloned();
        let on_snap_point_change_for_drawer = on_snap_point_change.clone();
        let runtime_model_cell = state.runtime_model.clone();
        let offset_model_cell = state.offset_model.clone();
        let drawer_content_id_cell = state.drawer_content_id.clone();

        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
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

                let mut drawer =
                    Drawer::new(open_for_drawer).snap_points(drawer_snap_test_points());
                if let Some(snap_point) = snap_point_for_drawer.clone() {
                    drawer = drawer.snap_point(snap_point);
                }
                if let Some(index) = default_snap_point_index {
                    drawer = drawer.default_snap_point(index);
                }
                if snap_to_sequential_points {
                    drawer = drawer.snap_to_sequential_points(true);
                }
                if let Some(on_snap_point_change) = on_snap_point_change_for_drawer.clone() {
                    drawer = drawer.on_snap_point_change(move |index| on_snap_point_change(index));
                }

                let drawer = drawer.into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let (runtime, offset_model, _was_open) = drawer_drag_models(cx);
                        *runtime_model_cell.borrow_mut() = Some(runtime);
                        *offset_model_cell.borrow_mut() = Some(offset_model);

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
                        *drawer_content_id_cell.borrow_mut() = Some(content.id);
                        content
                    },
                );

                vec![drawer]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.layout_all(app, services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, 1.0);
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

    #[test]
    fn drawer_open_change_handlers_forward_to_sheet() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let drawer = Drawer::new(open)
            .on_open_change(Some(Arc::new(|_open| {})))
            .on_open_change_complete(Some(Arc::new(|_open| {})));

        let inner_debug = format!("{:?}", drawer.inner);
        assert!(inner_debug.contains("on_open_change: true"));
        assert!(inner_debug.contains("on_open_change_complete: true"));
    }

    #[test]
    fn drawer_header_root_fills_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            with_drawer_side_provider(cx, DrawerSide::Bottom, |cx| {
                DrawerHeader::new([
                    DrawerTitle::new("Title").into_element(cx),
                    DrawerDescription::new("Description").into_element(cx),
                ])
                .into_element(cx)
            })
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!(
                "expected DrawerHeader root to be a Container, got {:?}",
                el.kind
            );
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn drawer_header_centers_text_below_md_breakpoint_for_bottom_drawer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            with_drawer_side_provider(cx, DrawerSide::Bottom, |cx| {
                DrawerHeader::new([
                    DrawerTitle::new("Title").into_element(cx),
                    DrawerDescription::new("Description").into_element(cx),
                ])
                .into_element(cx)
            })
        });

        for label in ["Title", "Description"] {
            let text = find_text(&el, label).expect("expected drawer header text node");
            assert_eq!(text.align, TextAlign::Center);
            assert_eq!(text.layout.size.width, Length::Fill);
            assert_eq!(text.layout.size.min_width, Some(Length::Px(Px(0.0))));
        }
    }

    #[test]
    fn drawer_header_left_aligns_text_for_side_drawers() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            with_drawer_side_provider(cx, DrawerSide::Left, |cx| {
                DrawerHeader::new([
                    DrawerTitle::new("Title").into_element(cx),
                    DrawerDescription::new("Description").into_element(cx),
                ])
                .into_element(cx)
            })
        });

        for label in ["Title", "Description"] {
            let text = find_text(&el, label).expect("expected drawer header text node");
            assert_eq!(text.align, TextAlign::Start);
            assert_eq!(text.layout.size.width, Length::Fill);
            assert_eq!(text.layout.size.min_width, Some(Length::Px(Px(0.0))));
        }
    }

    #[test]
    fn drawer_header_text_align_override_applies_to_bottom_drawer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            with_drawer_side_provider(cx, DrawerSide::Bottom, |cx| {
                DrawerHeader::new([
                    DrawerTitle::new("Title").into_element(cx),
                    DrawerDescription::new("Description").into_element(cx),
                ])
                .text_align(TextAlign::Start)
                .into_element(cx)
            })
        });

        for label in ["Title", "Description"] {
            let text = find_text(&el, label).expect("expected drawer header text node");
            assert_eq!(text.align, TextAlign::Start);
        }
    }

    #[test]
    fn drawer_footer_layout_refinement_applies_to_root_container() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            DrawerFooter::new([crate::button::Button::new("Close").into_element(cx)])
                .refine_layout(LayoutRefinement::default().max_w(Px(320.0)))
                .into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!(
                "expected DrawerFooter root to be a Container, got {:?}",
                el.kind
            );
        };
        assert_eq!(props.layout.size.max_width, Some(Length::Px(Px(320.0))));
    }

    #[test]
    fn drawer_content_max_height_fraction_clamps_tall_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(400.0)),
        );

        let content_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let content_id_out = content_id.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "drawer-content-max-height-fraction",
            move |cx| {
                let tall = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(2000.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                );

                let content = DrawerContent::new(vec![tall]).into_element(cx);
                content_id_out.set(Some(content.id));
                vec![content]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_bounds = current_bounds_for_element(
            &mut app,
            window,
            content_id.get().expect("drawer content element id"),
        )
        .expect("drawer content bounds");
        let viewport_h = bounds.size.height.0;
        let cap = viewport_h * DRAWER_MAX_HEIGHT_FRACTION;
        let by_gap = (viewport_h - DRAWER_EDGE_GAP_PX.0).max(0.0);
        let expected = cap.min(by_gap);
        assert!(
            (content_bounds.size.height.0 - expected).abs() < 2.0,
            "expected content max-height fraction clamp near {expected}px, got {content_bounds:?}"
        );
    }

    fn render_drawer_frame_with_real_content(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        side: DrawerSide,
        content_id_out: Rc<Cell<Option<GlobalElementId>>>,
        description_id_out: Rc<Cell<Option<GlobalElementId>>>,
        cancel_id_out: Rc<Cell<Option<GlobalElementId>>>,
        action_id_out: Rc<Cell<Option<GlobalElementId>>>,
    ) {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "test",
            |cx| {
                let content_id_out = content_id_out.clone();
                let description_id_out = description_id_out.clone();
                let cancel_id_out = cancel_id_out.clone();
                let action_id_out = action_id_out.clone();

                let content = with_drawer_side_provider(cx, side, |cx| {
                    let title = DrawerTitle::new("Delete the production project?").into_element(cx);
                    let description = DrawerDescription::new(
                        "This drawer should keep its header text and footer actions inside the panel bounds on narrow viewports instead of measuring against unconstrained content width.",
                    )
                    .into_element(cx);
                    description_id_out.set(Some(description.id));

                    let header = DrawerHeader::new(vec![title, description]).into_element(cx);

                    let action = crate::button::Button::new("Submit")
                        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx);
                    action_id_out.set(Some(action.id));

                    let cancel = crate::button::Button::new("Cancel")
                        .variant(crate::button::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx);
                    cancel_id_out.set(Some(cancel.id));

                    let footer = DrawerFooter::new(vec![action, cancel]).into_element(cx);
                    let content = DrawerContent::new(vec![header, footer]).into_element(cx);
                    content_id_out.set(Some(content.id));
                    content
                });

                vec![content]
            },
        );

        ui.set_root(root);
    }

    #[test]
    fn drawer_real_content_stays_within_panel_bounds_on_narrow_bottom_viewport() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let content_id = Rc::new(Cell::new(None));
        let description_id = Rc::new(Cell::new(None));
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(600.0)),
        );

        render_drawer_frame_with_real_content(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            DrawerSide::Bottom,
            content_id.clone(),
            description_id.clone(),
            cancel_id.clone(),
            action_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_bounds = current_bounds_for_element(
            &mut app,
            window,
            content_id.get().expect("content element id"),
        )
        .expect("content bounds");
        let description_bounds = current_bounds_for_element(
            &mut app,
            window,
            description_id.get().expect("description element id"),
        )
        .expect("description bounds");
        let cancel_bounds = current_bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = current_bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");

        let content_left = content_bounds.origin.x.0 - 0.5;
        let content_right = content_bounds.origin.x.0 + content_bounds.size.width.0 + 0.5;

        assert!(
            description_bounds.origin.x.0 >= content_left
                && description_bounds.origin.x.0 + description_bounds.size.width.0 <= content_right,
            "expected description to stay inside drawer content; content={content_bounds:?} description={description_bounds:?}"
        );
        assert!(
            cancel_bounds.origin.x.0 >= content_left
                && cancel_bounds.origin.x.0 + cancel_bounds.size.width.0 <= content_right,
            "expected cancel button to stay inside drawer content; content={content_bounds:?} cancel={cancel_bounds:?}"
        );
        assert!(
            action_bounds.origin.x.0 >= content_left
                && action_bounds.origin.x.0 + action_bounds.size.width.0 <= content_right,
            "expected action button to stay inside drawer content; content={content_bounds:?} action={action_bounds:?}"
        );
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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn drawer_into_element_parts_trigger_opens_on_activate() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let open = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-drawer-into-element-parts-trigger-opens",
            |cx| {
                vec![Drawer::new(open.clone()).into_element_parts(
                    cx,
                    |cx| DrawerTrigger::new(crate::button::Button::new("Open").into_element(cx)),
                    DrawerPortal::default(),
                    DrawerOverlay::new(),
                    |cx| DrawerContent::new([cx.text("Content")]).into_element(cx),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get_copied(&open), Some(false));

        let trigger_node = ui.children(root)[0];
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
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
                position,
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
    fn drawer_compose_content_with_supports_from_scope_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;
        let open = app.models_mut().insert(true);

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-drawer-compose-content-with-from-scope",
            |cx| {
                let trigger =
                    DrawerTrigger::new(crate::button::Button::new("Open").into_element(cx));

                vec![
                    Drawer::new(open.clone())
                        .compose()
                        .trigger(trigger)
                        .portal(DrawerPortal::new())
                        .overlay(DrawerOverlay::new())
                        .content_with(|cx| {
                            let close = DrawerClose::from_scope().into_element(cx);
                            DrawerContent::new(vec![close]).into_element(cx)
                        })
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn drawer_children_content_with_supports_from_scope_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;
        let open = app.models_mut().insert(true);

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-drawer-children-content-with-from-scope",
            |cx| {
                let trigger =
                    DrawerTrigger::new(crate::button::Button::new("Open").into_element(cx));

                vec![
                    Drawer::new(open.clone())
                        .children([
                            DrawerPart::trigger(trigger),
                            DrawerPart::portal(DrawerPortal::new()),
                            DrawerPart::overlay(DrawerOverlay::new()),
                            DrawerPart::content_with(|cx| {
                                let close = DrawerClose::from_scope().into_element(cx);
                                DrawerContent::new(vec![close]).into_element(cx)
                            }),
                        ])
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get_copied(&open), Some(true));
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
                            layout.inset.left = Some(Px(100.0)).into();
                            layout.inset.top = Some(Px(100.0)).into();
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

    fn render_drawer_frame_with_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_id_out: Rc<Cell<Option<GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<GlobalElementId>>>>,
        drawer_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
    ) -> GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<GlobalElementId> = None;
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay_id_out = underlay_id_out.clone();
                let underlay_id_cell = underlay_id_cell.clone();
                let underlay = cx.pressable_with_id(
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
                    move |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        if let Some(underlay_id_cell) = underlay_id_cell.as_ref() {
                            *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                        }
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

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

                let drawer = Drawer::new(open.clone())
                    .on_open_auto_focus(on_open_auto_focus.clone())
                    .on_close_auto_focus(on_close_auto_focus.clone())
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
                                    drawer_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            DrawerContent::new(vec![focusable]).into_element(cx)
                        },
                    );

                vec![underlay, drawer]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_drawer_frame_with_open_auto_focus_redirect_target(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_id_out: Rc<Cell<Option<GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<GlobalElementId>>>>,
        initial_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
        redirect_focus_id_cell: Arc<Mutex<Option<GlobalElementId>>>,
        redirect_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
    ) -> GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<GlobalElementId> = None;
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay_id_out = underlay_id_out.clone();
                let underlay_id_cell = underlay_id_cell.clone();
                let underlay = cx.pressable_with_id(
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
                    move |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        if let Some(underlay_id_cell) = underlay_id_cell.as_ref() {
                            *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                        }
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

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

                let redirect_focus_id_cell = redirect_focus_id_cell.clone();
                let drawer = Drawer::new(open.clone())
                    .on_open_auto_focus(on_open_auto_focus.clone())
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let initial = cx.pressable_with_id(
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

                            let redirect = cx.pressable_with_id(
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
                                    redirect_focus_id_out.set(Some(id));
                                    *redirect_focus_id_cell
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner()) = Some(id);
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            DrawerContent::new(vec![initial, redirect]).into_element(cx)
                        },
                    );

                vec![underlay, drawer]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
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
                                layout.inset.top = Some(Px(100.0)).into();
                                layout.inset.left = Some(Px(100.0)).into();
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
                            layout.inset.top = Some(Px(100.0)).into();
                            layout.inset.left = Some(Px(100.0)).into();
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
    fn drawer_disable_pointer_dismissal_alias_maps_overlay_closable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices::default();
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
                let underlay = {
                    let underlay_activated = underlay_activated.clone();
                    cx.pressable(
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
                    )
                };

                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.top = Some(Px(100.0)).into();
                            layout.inset.left = Some(Px(100.0)).into();
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let drawer = Drawer::new(open.clone())
                    .disable_pointer_dismissal(true)
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
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

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
            "underlay should remain inert while pointer dismissal is disabled"
        );
    }

    #[test]
    fn drawer_modal_trap_focus_traps_tab_but_keeps_outside_click_through() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let drawer_content_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let focusable_a_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let focusable_b_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let trigger_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let scrim_color = fret_core::Color {
            r: 0.09,
            g: 0.41,
            b: 0.32,
            a: 0.34,
        };

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frame: u64| {
                app.set_frame_id(FrameId(frame));
                OverlayController::begin_frame(app, window);

                let open = open.clone();
                let underlay_id = underlay_id.clone();
                let drawer_content_id = drawer_content_id.clone();
                let focusable_a_id = focusable_a_id.clone();
                let focusable_b_id = focusable_b_id.clone();
                let trigger_id = trigger_id.clone();
                let underlay_activated = underlay_activated.clone();

                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "drawer-trap-focus",
                    |cx| {
                        let underlay_id = underlay_id.clone();
                        let underlay_activated = underlay_activated.clone();
                        let underlay = cx.pressable_with_id(
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
                            move |cx, _st, id| {
                                underlay_id.set(Some(id));
                                cx.pressable_set_bool(&underlay_activated, true);
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let trigger_id = trigger_id.clone();
                        let open_for_trigger = open.clone();
                        let open_for_drawer = open.clone();
                        let trigger = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout.inset.left = Some(Px(100.0)).into();
                                    layout.inset.top = Some(Px(100.0)).into();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            move |cx, _st, id| {
                                trigger_id.set(Some(id));
                                cx.pressable_toggle_bool(&open_for_trigger);
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let drawer_content_id = drawer_content_id.clone();
                        let focusable_a_id = focusable_a_id.clone();
                        let focusable_b_id = focusable_b_id.clone();
                        let drawer = Drawer::new(open_for_drawer)
                            .overlay_color(scrim_color)
                            .modal_trap_focus(true)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                move |cx| {
                                    let a = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(180.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            focusable_a_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let b = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(180.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            focusable_b_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let content = DrawerContent::new(vec![a, b]).into_element(cx);
                                    drawer_content_id.set(Some(content.id));
                                    content
                                },
                            );

                        vec![underlay, drawer]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

        let trigger_element = trigger_id.get().expect("trigger id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
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
                position: trigger_center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        assert_eq!(app.models().get_copied(&open), Some(true));

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 2;
        for i in 0..settle_frames {
            render_frame(&mut ui, &mut app, &mut services, 3 + i as u64);
        }

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene_contains_full_window_solid_quad(&scene, bounds, scrim_color),
            "expected trap-focus drawer to keep painting the configured overlay color as a visual scrim"
        );

        let underlay_element = underlay_id.get().expect("underlay id");
        let content_element = drawer_content_id.get().expect("drawer content id");
        let a_id = focusable_a_id.get().expect("focusable a id");
        let b_id = focusable_b_id.get().expect("focusable b id");
        let underlay_node = fret_ui::elements::node_for_element(&mut app, window, underlay_element)
            .expect("underlay node");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let a_node = fret_ui::elements::node_for_element(&mut app, window, a_id).expect("a node");
        let b_node = fret_ui::elements::node_for_element(&mut app, window, b_id).expect("b node");
        let focused = ui.focus().expect("focused popup root");
        let mut focused_is_content_ancestor = false;
        let mut current = Some(content_node);
        while let Some(node) = current {
            if node == focused {
                focused_is_content_ancestor = true;
                break;
            }
            current = ui.node_parent(node);
        }
        assert!(
            focused_is_content_ancestor,
            "expected trap-focus drawer to focus a popup-root ancestor of the content"
        );
        assert_ne!(Some(focused), Some(a_node));
        assert_ne!(Some(focused), Some(b_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_ne!(ui.focus(), Some(underlay_node));
        assert!(
            ui.focus() == Some(a_node) || ui.focus() == Some(b_node),
            "trap-focus drawer should keep focus inside content"
        );

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
        assert_eq!(app.models().get_copied(&underlay_activated), Some(true));
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
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
        let snap_point = app.models_mut().insert(Some(2usize));
        let state = DrawerSnapHarnessState::default();

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(600.0)),
        );

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                None,
                false,
                None,
                &state,
            );
        }

        let drawer_content_id = state
            .drawer_content_id
            .borrow()
            .clone()
            .expect("drawer content id captured");

        let offset_model = state
            .offset_model
            .borrow()
            .clone()
            .expect("offset model captured");
        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        let runtime_model = state
            .runtime_model
            .borrow()
            .clone()
            .expect("runtime model captured");
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

        let expected = Px(180.0);

        let mut settled = false;
        for _ in 0..120 {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                None,
                false,
                None,
                &state,
            );

            let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
            if (offset.0 - expected.0).abs() < 1.0 {
                settled = true;
                break;
            }
        }

        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        let runtime = app
            .models()
            .get_copied(&runtime_model)
            .expect("runtime snapshot");
        assert!(
            settled,
            "expected offset to settle near {expected:?}, got {offset:?} (window_height={:?}, viewport_height={:?}, drawer_height={:?})",
            runtime.window_height, runtime.viewport_height, runtime.drawer_height,
        );
        assert_eq!(
            app.models().get_cloned(&snap_point).unwrap_or(None),
            Some(1),
            "expected drag release to update the snap-point model to the nearest authored index",
        );
    }

    #[test]
    fn drawer_snap_point_model_initializes_to_controlled_index_on_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let snap_point = app.models_mut().insert(Some(1usize));
        let state = DrawerSnapHarnessState::default();

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(600.0)),
        );

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                None,
                false,
                None,
                &state,
            );
        }

        let offset_model = state
            .offset_model
            .borrow()
            .clone()
            .expect("offset model captured");
        let runtime_model = state
            .runtime_model
            .borrow()
            .clone()
            .expect("runtime model captured");
        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        let runtime = app
            .models()
            .get_copied(&runtime_model)
            .expect("runtime snapshot");

        assert!(
            (offset.0 - 180.0).abs() < 1.0,
            "expected controlled snap point index 1 to initialize near 180px offset, got {offset:?}",
        );
        assert_eq!(
            runtime.applied_snap_point_index,
            Some(1),
            "expected runtime to track the controlled active snap-point index",
        );
    }

    #[test]
    fn drawer_close_resets_snap_point_model_to_default_index() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let snap_point = app.models_mut().insert(Some(2usize));
        let state = DrawerSnapHarnessState::default();
        let events: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
        let on_snap_point_change: OnSnapPointChange = {
            let events = events.clone();
            Arc::new(move |index| {
                events.lock().expect("snap-point events").push(index);
            })
        };

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(600.0)),
        );

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                Some(1),
                false,
                Some(on_snap_point_change.clone()),
                &state,
            );
        }

        let _ = app.models_mut().update(&open, |value| *value = false);
        app.set_frame_id(frame);
        render_snap_drawer_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            &open,
            Some(&snap_point),
            Some(1),
            false,
            Some(on_snap_point_change.clone()),
            &state,
        );

        assert_eq!(
            app.models().get_cloned(&snap_point).unwrap_or(None),
            Some(1),
            "expected close transition to restore the default snap-point index",
        );
        assert_eq!(
            events.lock().expect("snap-point events").as_slice(),
            &[Some(1)],
            "expected close reset to emit a single snap-point change callback",
        );
    }

    #[test]
    fn drawer_snap_to_sequential_points_advances_one_step_per_drag() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let snap_point = app.models_mut().insert(Some(0usize));
        let state = DrawerSnapHarnessState::default();

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(600.0)),
        );

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                None,
                true,
                None,
                &state,
            );
        }

        let drawer_content_id = state
            .drawer_content_id
            .borrow()
            .clone()
            .expect("drawer content id captured");
        let offset_model = state
            .offset_model
            .borrow()
            .clone()
            .expect("offset model captured");
        let dialog =
            visual_bounds_for_element(&mut app, window, drawer_content_id).expect("drawer visual");
        let start = Point::new(
            Px(dialog.origin.x.0 + dialog.size.width.0 * 0.5),
            Px(dialog.origin.y.0 + 10.0),
        );
        let end = Point::new(start.x, Px(start.y.0 - 220.0));

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

        assert_eq!(
            app.models().get_cloned(&snap_point).unwrap_or(None),
            Some(1),
            "expected sequential snap mode to advance only to the adjacent snap-point index",
        );

        let expected = Px(180.0);
        let mut settled = false;
        for _ in 0..120 {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));
            render_snap_drawer_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                &open,
                Some(&snap_point),
                None,
                true,
                None,
                &state,
            );

            let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
            if (offset.0 - expected.0).abs() < 1.0 {
                settled = true;
                break;
            }
        }

        let offset = app.models().get_copied(&offset_model).unwrap_or(Px(0.0));
        assert!(
            settled,
            "expected sequential snap mode to settle near {expected:?}, got {offset:?}",
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
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
                                        .w_px(Px(24.0))
                                        .h_px(Px(24.0)),
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
    fn drawer_close_build_with_child_closes_open_model() {
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
        let mut frame = FrameId(1);
        for _ in 0..settle_frames {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));

            let open_for_drawer = open.clone();

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
                            let close = DrawerClose::from_scope().build(
                                cx,
                                crate::button::Button::new("Cancel")
                                    .variant(crate::button::ButtonVariant::Outline)
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .relative()
                                            .w_px(Px(96.0))
                                            .h_px(Px(36.0)),
                                    ),
                            );
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
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_300,
        ) + 2;
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

    #[test]
    fn drawer_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let drawer_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_drawer_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id.clone(),
            None,
            drawer_focus_id.clone(),
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_drawer_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id,
            None,
            drawer_focus_id.clone(),
            Some(handler),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let initial_focus = drawer_focus_id.get().expect("drawer focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("focusable");
        assert_ne!(
            ui.focus(),
            Some(initial_focus_node),
            "expected preventDefault to suppress focusing the first focusable"
        );
        let focused = ui.focus().expect("expected focus to be set");
        assert_eq!(
            ui.node_layer(focused),
            ui.node_layer(initial_focus_node),
            "expected focus containment to keep focus within the drawer layer"
        );
    }

    #[test]
    fn drawer_open_auto_focus_can_be_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let redirect_focus_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let redirect_focus_id_cell: Arc<Mutex<Option<GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let redirect_focus_id_for_handler = redirect_focus_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = redirect_focus_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(id) = id {
                host.request_focus(id);
            }
            req.prevent_default();
        });

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_drawer_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id.clone(),
            None,
            initial_focus_id.clone(),
            redirect_focus_id_cell.clone(),
            redirect_focus_id_out.clone(),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_drawer_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id,
            None,
            initial_focus_id.clone(),
            redirect_focus_id_cell,
            redirect_focus_id_out.clone(),
            Some(handler),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let initial_focus = initial_focus_id.get().expect("initial focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("initial focus");
        let redirect_focus = redirect_focus_id_out.get().expect("redirect focus element");
        let redirect_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, redirect_focus)
                .expect("redirect focus");
        assert_ne!(
            ui.focus(),
            Some(initial_focus_node),
            "expected redirect to override the default initial focus"
        );
        assert_eq!(
            ui.focus(),
            Some(redirect_focus_node),
            "expected open autofocus redirect to win when preventDefault is set"
        );
    }

    #[test]
    fn drawer_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let underlay_id_cell: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
        let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let redirect_focus_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let redirect_focus_id_cell: Arc<Mutex<Option<GlobalElementId>>> =
            Arc::new(Mutex::new(None));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let underlay_id_for_handler = underlay_id_cell.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = underlay_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(id) = id {
                host.request_focus(id);
            }
            req.prevent_default();
        });

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_drawer_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id_out.clone(),
            Some(underlay_id_cell.clone()),
            initial_focus_id.clone(),
            redirect_focus_id_cell.clone(),
            redirect_focus_id_out.clone(),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_drawer_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id_out.clone(),
            Some(underlay_id_cell),
            initial_focus_id.clone(),
            redirect_focus_id_cell,
            redirect_focus_id_out,
            Some(handler),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let underlay = underlay_id_out.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay");
        let initial_focus = initial_focus_id.get().expect("initial focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("initial focus node");

        let focused = ui.focus().expect("expected focus after open");
        assert_ne!(
            focused, underlay_node,
            "expected modal focus containment to prevent focusing the underlay on open"
        );
        assert_eq!(
            ui.node_layer(focused),
            ui.node_layer(initial_focus_node),
            "expected focus containment to clamp focus within the drawer layer"
        );
    }

    #[test]
    fn drawer_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let drawer_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let underlay_id_cell: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
        let underlay_id_for_handler = underlay_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = underlay_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(id) = id {
                host.request_focus(id);
            }
            req.prevent_default();
        });

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let _trigger = render_drawer_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id_out.clone(),
            Some(underlay_id_cell.clone()),
            drawer_focus_id.clone(),
            None,
            Some(handler.clone()),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus = drawer_focus_id.get().expect("drawer focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("focusable");
        ui.set_focus(Some(initial_focus_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_300,
        ) as usize
            + 2;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(2 + i as u64));
            let _ = render_drawer_frame_with_auto_focus_hooks(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                underlay_id_out.clone(),
                Some(underlay_id_cell.clone()),
                Rc::new(Cell::new(None)),
                None,
                Some(handler.clone()),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let underlay_id = underlay_id_out.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected preventDefault close autofocus to allow redirecting focus"
        );
    }
}
