use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, MarginEdge, MarginEdges, Overflow,
    PositionStyle, SemanticsDecoration, SizeStyle,
};
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, Invalidation, Theme, ThemeNamedColorKey, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::{
    occlusion_insets_or_zero, safe_area_insets_or_zero, viewport_queries,
};
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::primitives::portal_inherited;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverlayController, OverlayPresence, Space,
    UiChildIntoElement, UiHostBoundIntoElement, ui,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;
use fret_ui_kit::typography::scope_description_text;

fn default_overlay_color(theme: &ThemeSnapshot) -> Color {
    let mut scrim = theme.named_color(ThemeNamedColorKey::Black);
    scrim.a = 0.5;
    scrim
}

/// shadcn/ui `SheetPortal` (v4).
///
/// Fret installs sheets through the overlay controller, which implies a portal-like boundary
/// already. This type is a no-op marker that exists to align the shadcn part surface and leave room
/// for future portal configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct SheetPortal;

impl SheetPortal {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `SheetOverlay` (v4).
///
/// Upstream exposes the overlay (scrim/backdrop) as a distinct part with styling concerns.
/// Fret's sheet surface currently owns the overlay policy knobs on [`Sheet`]. This type is a thin
/// adapter so part-based call sites can keep configuration in a shadcn-like location.
#[derive(Debug, Clone, Default)]
pub struct SheetOverlay {
    closable: Option<bool>,
    color: Option<Color>,
}

impl SheetOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Controls whether outside pointer press dismisses the sheet.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = Some(closable);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    fn apply_to(self, mut sheet: Sheet) -> Sheet {
        if let Some(v) = self.closable {
            sheet.overlay_closable = v;
        }
        if let Some(v) = self.color {
            sheet.overlay_color = Some(v);
        }
        sheet
    }
}

/// shadcn/ui `SheetTrigger` (v4).
///
/// In the upstream DOM implementation this is a Radix primitive part. In Fret, the trigger element
/// itself is still authored by the caller; this wrapper exists to align the part surface with
/// shadcn docs/examples and to keep room for future trigger-specific defaults.
#[derive(Debug)]
pub struct SheetTrigger {
    child: AnyElement,
}

pub struct SheetTriggerBuild<H, T> {
    child: Option<T>,
    _phantom: PhantomData<fn() -> H>,
}

impl SheetTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    /// Builder-first variant that late-lands the trigger child at `into_element(cx)` time.
    pub fn build<H: UiHost, T>(child: T) -> SheetTriggerBuild<H, T>
    where
        T: UiChildIntoElement<H>,
    {
        SheetTriggerBuild {
            child: Some(child),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

impl<H: UiHost, T> SheetTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        SheetTrigger::new(
            self.child
                .expect("expected sheet trigger child")
                .into_child_element(cx),
        )
        .into_element(cx)
    }
}

impl<H: UiHost, T> UiHostBoundIntoElement<H> for SheetTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        SheetTriggerBuild::into_element(self, cx)
    }
}

impl<H: UiHost, T> UiChildIntoElement<H> for SheetTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    fn into_child_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        SheetTriggerBuild::into_element(self, cx)
    }
}

#[doc(hidden)]
pub trait SheetCompositionTriggerArg<H: UiHost> {
    fn into_sheet_trigger(self, cx: &mut ElementContext<'_, H>) -> SheetTrigger;
}

impl<H: UiHost> SheetCompositionTriggerArg<H> for SheetTrigger {
    fn into_sheet_trigger(self, _cx: &mut ElementContext<'_, H>) -> SheetTrigger {
        self
    }
}

impl<H: UiHost, T> SheetCompositionTriggerArg<H> for SheetTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    fn into_sheet_trigger(self, cx: &mut ElementContext<'_, H>) -> SheetTrigger {
        SheetTrigger::new(
            self.child
                .expect("expected sheet trigger child")
                .into_child_element(cx),
        )
    }
}

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default)]
struct SheetOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn sheet_open_change_events(
    state: &mut SheetOpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> (Option<bool>, Option<bool>) {
    let mut changed = None;
    let mut completed = None;

    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
        changed = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        completed = Some(open);
    }

    (changed, completed)
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

#[derive(Debug, Default)]
struct SheetOpenProviderState {
    current: Option<Model<bool>>,
}

#[track_caller]
fn with_sheet_open_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SheetOpenProviderState::default, |st| {
        let prev = st.current.clone();
        st.current = Some(open);
        prev
    });
    let out = f(cx);
    cx.with_state(SheetOpenProviderState::default, |st| {
        st.current = prev;
    });
    out
}

fn inherited_sheet_open<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Model<bool>> {
    cx.inherited_state_where::<SheetOpenProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current.clone())
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
    size_override: Option<SheetSizeOverride>,
    max_size_override: Option<Px>,
    overlay_closable: bool,
    overlay_color: Option<Color>,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
    vertical_edge_gap_px: Option<Px>,
    vertical_auto_max_height_fraction: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SheetSizeOverride {
    Px(Px),
    Fraction(f32),
}

impl SheetSizeOverride {
    fn as_length(self) -> Length {
        match self {
            Self::Px(px) => Length::Px(px),
            Self::Fraction(fraction) => Length::Fraction(fraction),
        }
    }
}

impl std::fmt::Debug for Sheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sheet")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("size_override", &self.size_override)
            .field("max_size_override", &self.max_size_override)
            .field("overlay_closable", &self.overlay_closable)
            .field("overlay_color", &self.overlay_color)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
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
            max_size_override: None,
            overlay_closable: true,
            overlay_color: None,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
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
        let max_size = self.max_size_override.unwrap_or(Px(f32::INFINITY));
        let size = Px(size.0.max(0.0).min(max_size.0.max(0.0)));
        self.size_override = Some(SheetSizeOverride::Px(size));
        self
    }

    /// Sets the sheet size as a fraction of the viewport:
    /// - width for `Left` / `Right`
    /// - height for `Top` / `Bottom`
    pub fn size_fraction(mut self, fraction: f32) -> Self {
        let fraction = if fraction.is_finite() && fraction > 0.0 {
            fraction.min(1.0)
        } else {
            0.0
        };
        self.size_override = Some(SheetSizeOverride::Fraction(fraction));
        self
    }

    /// Caps the sheet size on its primary axis:
    /// - width for `Left` / `Right`
    /// - height for `Top` / `Bottom`
    pub fn max_size(mut self, max_size: Px) -> Self {
        let max_size = Px(max_size.0.max(0.0));
        self.max_size_override = Some(max_size);
        if let Some(SheetSizeOverride::Px(size_px)) = self.size_override {
            if size_px.0 > max_size.0 {
                self.size_override = Some(SheetSizeOverride::Px(max_size));
            }
        }
        self
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.overlay_closable = overlay_closable;
        self
    }

    /// Base UI-compatible alias.
    ///
    /// When `true`, outside pointer press does not dismiss the sheet.
    /// This is equivalent to `overlay_closable(false)`.
    pub fn disable_pointer_dismissal(mut self, disable: bool) -> Self {
        self.overlay_closable = !disable;
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

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.on_open_change_complete = on_open_change_complete;
        self
    }

    /// Returns a recipe-level composition builder for shadcn-style part assembly.
    ///
    /// This bridges Fret's closure-root authoring model with the nested part mental model used by
    /// shadcn/Radix while keeping the underlying mechanism surface unchanged.
    pub fn compose<H: UiHost>(self) -> SheetComposition<H> {
        SheetComposition::new(self)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let is_open = cx.watch_model(&self.open).paint().copied().unwrap_or(false);

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = radix_dialog::dialog_root_name(id);

            let motion = OverlayController::transition_with_durations_and_cubic_bezier_duration(
                cx,
                is_open,
                overlay_motion::shadcn_motion_duration_500(cx),
                overlay_motion::shadcn_motion_duration_300(cx),
                overlay_motion::shadcn_motion_ease_bezier(cx),
            );
            let (open_change, open_change_complete) = cx
                .with_state(SheetOpenChangeCallbackState::default, |state| {
                    sheet_open_change_events(state, is_open, motion.present, motion.animating)
                });
            if let (Some(open), Some(on_open_change)) = (open_change, self.on_open_change.as_ref())
            {
                on_open_change(open);
            }
            if let (Some(open), Some(on_open_change_complete)) =
                (open_change_complete, self.on_open_change_complete.as_ref())
            {
                on_open_change_complete(open);
            }
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            if overlay_presence.present {
                let on_dismiss_request_for_barrier = self.on_dismiss_request.clone();
                let policy = radix_dialog::DialogCloseAutoFocusGuardPolicy::for_modal(true);
                let (on_dismiss_request_for_request, on_close_auto_focus) =
                    radix_dialog::dialog_close_auto_focus_guard_hooks(
                        cx,
                        policy,
                        self.open.clone(),
                        self.on_dismiss_request.clone(),
                        self.on_close_auto_focus.clone(),
                    );

                let open = self.open.clone();
                let open_for_children = open.clone();
                let overlay_color = self
                    .overlay_color
                    .unwrap_or_else(|| default_overlay_color(&theme));
                let overlay_closable = self.overlay_closable;
                let sheet_side = self.side;
                let dialog_options = radix_dialog::DialogOptions::default()
                    .dismiss_on_overlay_press(overlay_closable)
                    .initial_focus(None)
                    .on_open_auto_focus(self.on_open_auto_focus.clone())
                    .on_close_auto_focus(on_close_auto_focus);

                let size_override = self.size_override;
                let max_size_override = self.max_size_override;
                let vertical_edge_gap_px = self.vertical_edge_gap_px.unwrap_or(Px(0.0));
                let vertical_auto_max_height_fraction =
                    self.vertical_auto_max_height_fraction.unwrap_or(1.0);
                let opacity = motion.progress;
                let viewport_is_sm = viewport_queries::viewport_width_at_least(
                    cx,
                    Invalidation::Layout,
                    viewport_queries::tailwind::SM,
                    viewport_queries::ViewportQueryHysteresis::default(),
                );
                let shadcn_default_side_fraction = 0.75_f32;
                let shadcn_sm_max_width = theme
                    .metric_by_key("component.sheet.max_width_sm")
                    .or_else(|| theme.metric_by_key("component.sheet.size"))
                    .or_else(|| theme.metric_by_key("component.sheet.width"))
                    .unwrap_or(Px(384.0));
                let viewport_bounds = cx.environment_viewport_bounds(Invalidation::Layout);
                let viewport_width = viewport_bounds.size.width;
                let viewport_height = viewport_bounds.size.height;
                let portal_inherited = portal_inherited::PortalInherited::capture(cx);
                let overlay_children = portal_inherited::with_root_name_inheriting(
                    cx,
                    &overlay_root_name,
                    portal_inherited,
                    |cx| {
                        let mut barrier_overlay_color = overlay_color;
                        barrier_overlay_color.a =
                            (barrier_overlay_color.a * opacity).max(0.0).min(1.0);
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
                                padding: Edges::all(Px(0.0)).into(),
                                background: Some(barrier_overlay_color),
                                shadow: None,
                                border: Edges::all(Px(0.0)),
                                border_color: None,
                                corner_radii: Corners::all(Px(0.0)),
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        );

                        let content =
                            with_sheet_open_provider(cx, open_for_children.clone(), |cx| {
                                with_sheet_side_provider(cx, sheet_side, |cx| content(cx))
                            });
                        let vertical_auto_max_height_fraction = if vertical_auto_max_height_fraction
                            .is_finite()
                            && vertical_auto_max_height_fraction > 0.0
                        {
                            vertical_auto_max_height_fraction.min(1.0)
                        } else {
                            0.0
                        };

                        let (inset, size, estimated_motion_distance) = match sheet_side {
                            SheetSide::Right => (
                                InsetStyle {
                                    top: Some(Px(0.0)).into(),
                                    right: Some(Px(0.0)).into(),
                                    bottom: Some(Px(0.0)).into(),
                                    left: None.into(),
                                },
                                SizeStyle {
                                    width: size_override
                                        .map(|spec| spec.as_length())
                                        .unwrap_or(Length::Fraction(shadcn_default_side_fraction)),
                                    max_width: match (size_override, max_size_override) {
                                        (Some(SheetSizeOverride::Px(_)), _) => Some(Length::Fill),
                                        (_, Some(max_px)) => Some(Length::Px(max_px)),
                                        (None, None) if viewport_is_sm => {
                                            Some(Length::Px(shadcn_sm_max_width))
                                        }
                                        _ => None,
                                    },
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                match size_override {
                                    Some(SheetSizeOverride::Px(px)) => px,
                                    Some(SheetSizeOverride::Fraction(fraction)) => {
                                        let base = Px((viewport_width.0 * fraction).max(0.0));
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                    None => {
                                        let base = Px((viewport_width.0
                                            * shadcn_default_side_fraction)
                                            .max(0.0));
                                        let base = if viewport_is_sm {
                                            Px(base.0.min(shadcn_sm_max_width.0))
                                        } else {
                                            base
                                        };
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                },
                            ),
                            SheetSide::Left => (
                                InsetStyle {
                                    top: Some(Px(0.0)).into(),
                                    right: None.into(),
                                    bottom: Some(Px(0.0)).into(),
                                    left: Some(Px(0.0)).into(),
                                },
                                SizeStyle {
                                    width: size_override
                                        .map(|spec| spec.as_length())
                                        .unwrap_or(Length::Fraction(shadcn_default_side_fraction)),
                                    max_width: match (size_override, max_size_override) {
                                        (Some(SheetSizeOverride::Px(_)), _) => Some(Length::Fill),
                                        (_, Some(max_px)) => Some(Length::Px(max_px)),
                                        (None, None) if viewport_is_sm => {
                                            Some(Length::Px(shadcn_sm_max_width))
                                        }
                                        _ => None,
                                    },
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                match size_override {
                                    Some(SheetSizeOverride::Px(px)) => px,
                                    Some(SheetSizeOverride::Fraction(fraction)) => {
                                        let base = Px((viewport_width.0 * fraction).max(0.0));
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                    None => {
                                        let base = Px((viewport_width.0
                                            * shadcn_default_side_fraction)
                                            .max(0.0));
                                        let base = if viewport_is_sm {
                                            Px(base.0.min(shadcn_sm_max_width.0))
                                        } else {
                                            base
                                        };
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                },
                            ),
                            SheetSide::Top => (
                                InsetStyle {
                                    top: Some(Px(0.0)).into(),
                                    right: Some(Px(0.0)).into(),
                                    bottom: None.into(),
                                    left: Some(Px(0.0)).into(),
                                },
                                SizeStyle {
                                    width: Length::Fill,
                                    height: if let Some(spec) = size_override {
                                        spec.as_length()
                                    } else {
                                        Length::Auto
                                    },
                                    max_height: if size_override.is_some() {
                                        Some(Length::Fill)
                                    } else if vertical_auto_max_height_fraction < 1.0 {
                                        Some(Length::Fraction(vertical_auto_max_height_fraction))
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                },
                                match size_override {
                                    Some(SheetSizeOverride::Px(px)) => px,
                                    Some(SheetSizeOverride::Fraction(fraction)) => {
                                        let base = Px((viewport_height.0 * fraction).max(0.0));
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                    None => Px(350.0),
                                },
                            ),
                            SheetSide::Bottom => (
                                InsetStyle {
                                    top: None.into(),
                                    right: Some(Px(0.0)).into(),
                                    bottom: Some(Px(0.0)).into(),
                                    left: Some(Px(0.0)).into(),
                                },
                                SizeStyle {
                                    width: Length::Fill,
                                    height: if let Some(spec) = size_override {
                                        spec.as_length()
                                    } else {
                                        Length::Auto
                                    },
                                    max_height: if size_override.is_some() {
                                        Some(Length::Fill)
                                    } else if vertical_auto_max_height_fraction < 1.0 {
                                        Some(Length::Fraction(vertical_auto_max_height_fraction))
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                },
                                match size_override {
                                    Some(SheetSizeOverride::Px(px)) => px,
                                    Some(SheetSizeOverride::Fraction(fraction)) => {
                                        let base = Px((viewport_height.0 * fraction).max(0.0));
                                        if let Some(max_px) = max_size_override {
                                            Px(base.0.min(max_px.0))
                                        } else {
                                            base
                                        }
                                    }
                                    None => Px(350.0),
                                },
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
                                        SheetSide::Top if vertical_edge_gap_px.0 > 0.0 => {
                                            MarginEdges {
                                                bottom: MarginEdge::Px(vertical_edge_gap_px),
                                                ..Default::default()
                                            }
                                        }
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

                        let motion_distance = match sheet_side {
                            SheetSide::Left | SheetSide::Right => cx
                                .last_bounds_for_element(wrapper.id)
                                .map(|r| r.size.width)
                                .unwrap_or(estimated_motion_distance),
                            SheetSide::Top | SheetSide::Bottom if size_override.is_none() => cx
                                .last_bounds_for_element(wrapper.id)
                                .map(|r| r.size.height)
                                .unwrap_or(estimated_motion_distance),
                            SheetSide::Top | SheetSide::Bottom => estimated_motion_distance,
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

                        radix_dialog::modal_dialog_layer_elements_with_dismiss_handler(
                            cx,
                            open_for_children.clone(),
                            dialog_options.clone(),
                            on_dismiss_request_for_barrier.clone(),
                            [barrier_fill],
                            content,
                        )
                    },
                );

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

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over [`Sheet::into_element`] that accepts shadcn-style parts
    /// (`SheetTrigger`, `SheetPortal`, `SheetOverlay`).
    ///
    /// It also installs a default "open on activate" behavior on the trigger element when the
    /// trigger is a `Pressable` (e.g. shadcn `Button`), matching the upstream Radix trigger
    /// contract.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> SheetTrigger,
        _portal: SheetPortal,
        overlay: SheetOverlay,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let sheet = overlay.apply_to(self);
        let open_for_trigger = sheet.open.clone();
        sheet.into_element(
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
}

/// Recipe-level builder for composing a sheet from shadcn-style parts.
type SheetDeferredContent<H> = Box<dyn FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static>;

enum SheetCompositionContent<H: UiHost> {
    Eager(AnyElement),
    Deferred(SheetDeferredContent<H>),
}

pub struct SheetComposition<H: UiHost, TTrigger = SheetTrigger> {
    sheet: Sheet,
    trigger: Option<TTrigger>,
    portal: SheetPortal,
    overlay: SheetOverlay,
    content: Option<SheetCompositionContent<H>>,
}

impl<H: UiHost, TTrigger> std::fmt::Debug for SheetComposition<H, TTrigger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SheetComposition")
            .field("sheet", &self.sheet)
            .field("trigger", &self.trigger.is_some())
            .field("portal", &self.portal)
            .field("overlay", &self.overlay)
            .field("content", &self.content.is_some())
            .finish()
    }
}

impl<H: UiHost> SheetComposition<H> {
    pub fn new(sheet: Sheet) -> Self {
        Self {
            sheet,
            trigger: None,
            portal: SheetPortal::new(),
            overlay: SheetOverlay::new(),
            content: None,
        }
    }
}

impl<H: UiHost, TTrigger> SheetComposition<H, TTrigger> {
    pub fn trigger<TNextTrigger>(self, trigger: TNextTrigger) -> SheetComposition<H, TNextTrigger> {
        SheetComposition {
            sheet: self.sheet,
            trigger: Some(trigger),
            portal: self.portal,
            overlay: self.overlay,
            content: self.content,
        }
    }

    pub fn portal(mut self, portal: SheetPortal) -> Self {
        self.portal = portal;
        self
    }

    pub fn overlay(mut self, overlay: SheetOverlay) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(SheetCompositionContent::Eager(content));
        self
    }

    pub fn content_with(
        mut self,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(SheetCompositionContent::Deferred(Box::new(content)));
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TTrigger: SheetCompositionTriggerArg<H>,
    {
        let trigger = self
            .trigger
            .expect("Sheet::compose().trigger(...) must be provided before into_element()")
            .into_sheet_trigger(cx);
        let content = self
            .content
            .expect("Sheet::compose().content(...) must be provided before into_element()");

        let portal = self.portal;
        let overlay = self.overlay;

        match content {
            SheetCompositionContent::Eager(content) => self.sheet.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |_cx| content,
            ),
            SheetCompositionContent::Deferred(content) => self.sheet.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |cx| content(cx),
            ),
        }
    }
}

/// shadcn/ui `SheetClose` (v4).
///
/// Upstream `SheetClose` is a thin wrapper around the underlying primitive's `Close` component.
/// In Fret, sheets are backed by modal overlays, so this delegates to `DialogClose`.
#[derive(Clone)]
pub struct SheetClose {
    open: Option<Model<bool>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for SheetClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SheetClose").finish()
    }
}

impl SheetClose {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open: Some(open),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().relative().inset(Space::N0),
        }
    }

    /// Creates a close affordance that resolves the current sheet/dialog scope at render time.
    pub fn from_scope() -> Self {
        Self {
            open: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().relative().inset(Space::N0),
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
        let open = self.open.unwrap_or_else(|| {
            inherited_sheet_open(cx).unwrap_or_else(|| {
                panic!("SheetClose::from_scope() must be used while rendering Sheet content")
            })
        });

        crate::DialogClose::new(open)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

/// shadcn/ui `SheetContent` (v4).
#[derive(Debug)]
pub struct SheetContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    show_close_button: bool,
}

impl SheetContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            show_close_button: true,
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

    /// Controls whether the default top-right close affordance is rendered.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let bg = theme.color_token("background");
        let border = theme.color_token("border");

        let radius = theme.metric_token("metric.radius.lg");
        let shadow = decl_style::shadow_lg(&theme, radius);

        let chrome = ChromeRefinement::default()
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .merge(self.chrome);

        let side = sheet_side_in_scope(cx);
        let base_layout = match side {
            SheetSide::Left | SheetSide::Right => LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0()
                .overflow_hidden(),
            SheetSide::Top | SheetSide::Bottom => {
                // Auto height by default for top/bottom sheets, matching upstream intent.
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
            }
        };
        let layout = base_layout.merge(self.layout);

        let props = {
            let mut props = decl_style::container_props(&theme, chrome, layout);

            // Apply environment-driven window insets to avoid system UI and virtual keyboard
            // occlusion on future mobile targets (ADR 0232).
            let safe = safe_area_insets_or_zero(cx, Invalidation::Layout);
            let occlusion = occlusion_insets_or_zero(cx, Invalidation::Layout);
            let max_px = |a: Px, b: Px| if a.0 > b.0 { a } else { b };
            let insets = Edges {
                top: max_px(safe.top, occlusion.top),
                right: max_px(safe.right, occlusion.right),
                bottom: max_px(safe.bottom, occlusion.bottom),
                left: max_px(safe.left, occlusion.left),
            };

            let add_inset = |edge: &mut fret_ui::element::SpacingLength, delta: Px| {
                if let fret_ui::element::SpacingLength::Px(px) = edge {
                    px.0 = (px.0 + delta.0).max(0.0);
                }
            };
            match side {
                SheetSide::Left => {
                    add_inset(&mut props.padding.left, insets.left);
                    add_inset(&mut props.padding.top, insets.top);
                    add_inset(&mut props.padding.bottom, insets.bottom);
                }
                SheetSide::Right => {
                    add_inset(&mut props.padding.right, insets.right);
                    add_inset(&mut props.padding.top, insets.top);
                    add_inset(&mut props.padding.bottom, insets.bottom);
                }
                SheetSide::Top => {
                    add_inset(&mut props.padding.top, insets.top);
                    add_inset(&mut props.padding.left, insets.left);
                    add_inset(&mut props.padding.right, insets.right);
                }
                SheetSide::Bottom => {
                    add_inset(&mut props.padding.bottom, insets.bottom);
                    add_inset(&mut props.padding.left, insets.left);
                    add_inset(&mut props.padding.right, insets.right);
                }
            }

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

        let mut children = self.children;
        if self.show_close_button {
            let open = inherited_sheet_open(cx)
                .expect("SheetContent close button must be rendered inside Sheet content");
            let close = crate::dialog::DialogClose::new(open).into_element(cx);
            children.push(close);
        }
        let container = shadcn_layout::container_vstack(
            cx,
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            shadcn_layout::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0()),
            children,
        );

        container
            .attach_semantics(SemanticsDecoration::default().role(fret_core::SemanticsRole::Dialog))
    }
}

/// shadcn/ui `SheetHeader` (v4).
#[derive(Debug)]
pub struct SheetHeader {
    children: Vec<AnyElement>,
}

impl SheetHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N1p5, children)
    }
}

/// shadcn/ui `SheetFooter` (v4).
#[derive(Debug)]
pub struct SheetFooter {
    children: Vec<AnyElement>,
}

impl SheetFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().p(Space::N4),
            LayoutRefinement::default().mt_auto(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            shadcn_layout::VStackProps::default()
                .gap(Space::N2)
                .justify_start()
                .items_stretch(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));

        let px = theme
            .metric_by_key("component.sheet.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.sheet.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        ui::text(self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .letter_spacing_em(-0.02)
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        scope_description_text(
            ui::raw_text(self.text)
                .wrap(TextWrap::Word)
                .overflow(TextOverflow::Clip)
                .into_element(cx),
            &theme,
            "component.sheet.description",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fret_app::App;
    use fret_core::{AppWindowId, Edges, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::{ContainerProps, ElementKind, PressableProps};
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
    use fret_ui_kit::ui::UiElementSinkExt as _;

    #[test]
    fn sheet_trigger_build_push_ui_accepts_late_landed_child() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let mut out = Vec::new();
            out.push_ui(cx, SheetTrigger::build(crate::Card::build(|_cx, _out| {})));

            assert_eq!(out.len(), 1);
            assert!(matches!(out[0].kind, ElementKind::Container(_)));
            assert!(out[0].inherited_foreground.is_some());
        });
    }

    #[test]
    fn sheet_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            SheetDescription::new("Description").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected SheetDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.sheet.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

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

    #[test]
    fn sheet_disable_pointer_dismissal_alias_maps_overlay_closable() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let a = Sheet::new(open.clone()).disable_pointer_dismissal(true);
        assert!(!a.overlay_closable);

        let b = Sheet::new(open).disable_pointer_dismissal(false);
        assert!(b.overlay_closable);
    }

    #[test]
    fn sheet_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = SheetOpenChangeCallbackState::default();

        let (changed, completed) = sheet_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = sheet_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = sheet_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn sheet_open_change_events_complete_without_animation() {
        let mut state = SheetOpenChangeCallbackState::default();

        let _ = sheet_open_change_events(&mut state, false, false, false);
        let (changed, completed) = sheet_open_change_events(&mut state, true, true, false);

        assert_eq!(changed, Some(true));
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn sheet_content_padding_includes_window_insets_for_bottom_side() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_safe_area_insets(window, None);
            rt.set_window_occlusion_insets(window, None);
        });
        let base_padding =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
                let element = with_sheet_side_provider(cx, SheetSide::Bottom, |cx| {
                    SheetContent::new([cx.text("child")]).into_element(cx)
                });
                match element.kind {
                    ElementKind::Container(ContainerProps { padding, .. }) => padding,
                    other => panic!("expected container root, got {other:?}"),
                }
            });

        // Safe area contributes left/right, occlusion contributes bottom and should win via max().
        app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_safe_area_insets(
                window,
                Some(Edges {
                    top: Px(0.0),
                    right: Px(8.0),
                    bottom: Px(20.0),
                    left: Px(6.0),
                }),
            );
            rt.set_window_occlusion_insets(
                window,
                Some(Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(48.0),
                    left: Px(0.0),
                }),
            );
        });
        let inset_padding =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
                let element = with_sheet_side_provider(cx, SheetSide::Bottom, |cx| {
                    SheetContent::new([cx.text("child")]).into_element(cx)
                });
                match element.kind {
                    ElementKind::Container(ContainerProps { padding, .. }) => padding,
                    other => panic!("expected container root, got {other:?}"),
                }
            });

        let px = |l: fret_ui::element::SpacingLength| match l {
            fret_ui::element::SpacingLength::Px(px) => px.0,
            other => panic!("expected px spacing length, got {other:?}"),
        };

        assert_eq!(px(inset_padding.left) - px(base_padding.left), 6.0);
        assert_eq!(px(inset_padding.right) - px(base_padding.right), 8.0);
        assert_eq!(px(inset_padding.bottom) - px(base_padding.bottom), 48.0);
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
    fn sheet_into_element_parts_trigger_opens_on_activate() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices::default();

        let open = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sheet-into-element-parts-trigger-opens",
            |cx| {
                vec![Sheet::new(open.clone()).into_element_parts(
                    cx,
                    |cx| SheetTrigger::new(crate::Button::new("Open").into_element(cx)),
                    SheetPortal::new(),
                    SheetOverlay::new(),
                    |cx| SheetContent::new([cx.text("Content")]).into_element(cx),
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
    fn sheet_composition_trigger_accepts_late_landed_child() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices::default();

        let open = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sheet-composition-trigger-accepts-late-child",
            |cx| {
                vec![
                    Sheet::new(open.clone())
                        .compose()
                        .trigger(SheetTrigger::build(
                            crate::Button::new("Open").test_id("sheet-compose-trigger-late-child"),
                        ))
                        .portal(SheetPortal::new())
                        .overlay(SheetOverlay::new())
                        .content(
                            SheetContent::new([cx.text("Content")])
                                .show_close_button(false)
                                .into_element(cx),
                        )
                        .into_element(cx),
                ]
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
    fn sheet_compose_content_with_supports_from_scope_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices::default();
        let open = app.models_mut().insert(true);

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sheet-compose-content-with-from-scope",
            |cx| {
                let trigger = SheetTrigger::new(crate::Button::new("Open").into_element(cx));

                vec![
                    Sheet::new(open.clone())
                        .compose()
                        .trigger(trigger)
                        .portal(SheetPortal::new())
                        .overlay(SheetOverlay::new())
                        .content_with(|cx| {
                            let close = SheetClose::from_scope().into_element(cx);
                            SheetContent::new(vec![close]).into_element(cx)
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
    fn sheet_bottom_auto_max_height_fraction_clamps_tall_content_with_edge_gap() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui = UiTree::new();
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(400.0)),
        );

        let open = app.models_mut().insert(true);
        let content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        OverlayController::begin_frame(&mut app, window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let trigger = cx.container(ContainerProps::default(), |_cx| Vec::new());

                let content_id_out = content_id_out.clone();
                let sheet = Sheet::new(open.clone())
                    .side(SheetSide::Bottom)
                    .vertical_edge_gap_px(Px(96.0))
                    .vertical_auto_max_height_fraction(0.8)
                    .into_element(
                        cx,
                        move |_cx| trigger,
                        move |cx| {
                            let tall = cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Px(Px(1200.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            );

                            let content = SheetContent::new([tall]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![sheet]
            },
        );

        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);

        let content_id = content_id_out.get().expect("sheet content id");
        let content_node =
            fret_ui::elements::node_for_element(&mut app, window, content_id).expect("node");
        let content_bounds = ui.debug_node_bounds(content_node).expect("bounds");

        let max_h = bounds.size.height.0 - 96.0;
        assert!(
            content_bounds.size.height.0 <= max_h + 0.5,
            "expected sheet content height <= viewport-height - edge-gap ({max_h}), got {}",
            content_bounds.size.height.0
        );
    }

    #[test]
    fn sheet_right_fractional_width_clamps_to_max_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui = UiTree::new();
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(1200.0), Px(400.0)),
        );

        let open = app.models_mut().insert(true);
        let content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        OverlayController::begin_frame(&mut app, window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let trigger = cx.container(ContainerProps::default(), |_cx| Vec::new());

                let content_id_out = content_id_out.clone();
                let sheet = Sheet::new(open.clone())
                    .side(SheetSide::Right)
                    .size_fraction(0.75)
                    .max_size(Px(384.0))
                    .into_element(
                        cx,
                        move |_cx| trigger,
                        move |cx| {
                            let content = SheetContent::new([cx.text("child")]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![sheet]
            },
        );

        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);

        let content_id = content_id_out.get().expect("sheet content id");
        let content_node =
            fret_ui::elements::node_for_element(&mut app, window, content_id).expect("node");
        let content_bounds = ui.debug_node_bounds(content_node).expect("bounds");

        assert!(
            content_bounds.size.width.0 <= 384.0 + 0.5,
            "expected right sheet width <= max-size, got {}",
            content_bounds.size.width.0
        );
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

    fn render_sheet_frame_with_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
        side: SheetSide,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);
        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

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

                let sheet = Sheet::new(open.clone())
                    .side(side)
                    .overlay_closable(overlay_closable)
                    .size(Px(300.0))
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
                                    initial_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content = SheetContent::new(vec![focusable]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![underlay, sheet]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_sheet_frame_with_open_auto_focus_redirect_target(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
        side: SheetSide,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);
        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

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
                let sheet = Sheet::new(open.clone())
                    .side(side)
                    .overlay_closable(overlay_closable)
                    .size(Px(300.0))
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

                            SheetContent::new(vec![initial, redirect]).into_element(cx)
                        },
                    );

                vec![underlay, sheet]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_sheet_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_activated: Model<bool>,
    ) {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "sheet-underlay",
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

                let sheet = Sheet::new(open.clone())
                    .overlay_closable(true)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let content =
                                SheetContent::new(vec![ui::raw_text("sheet").into_element(cx)])
                                    .into_element(cx);
                            content
                        },
                    );

                vec![underlay, sheet]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
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

        // Let the enter transition settle so hit-testing lands inside the sheet content for
        // deterministic pointer tests.
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
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
                Rc::new(Cell::new(None)),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

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
    fn sheet_close_button_closes() {
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

        // Render open + let the enter transition settle so hit-testing is deterministic.
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_500,
        ) as usize
            + 4;
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
                Rc::new(Cell::new(None)),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        ui.request_semantics_snapshot();
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

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let close = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("Close")
            })
            .expect("close button semantics node");
        let close_center = Point::new(
            Px(close.bounds.origin.x.0 + close.bounds.size.width.0 / 2.0),
            Px(close.bounds.origin.y.0 + close.bounds.size.height.0 / 2.0),
        );

        // Click the default top-right close affordance (same positioning as shadcn/Radix).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: close_center,
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
                position: close_center,
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
    fn sheet_escape_closes_by_default_when_handler_allows() {
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

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(
            *reason_cell.lock().expect("reason lock"),
            Some(DismissReason::Escape)
        );
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_300,
        ) as usize
            + 1;
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

    #[test]
    fn sheet_close_transition_keeps_modal_barrier_blocking_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed.
        render_sheet_frame_with_underlay(
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
        render_sheet_frame_with_underlay(
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
            "expected sheet to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> barrier must remain active.
        render_sheet_frame_with_underlay(
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
            .expect("expected barrier root to remain while the sheet is closing");
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
            "underlay should remain inert while the sheet is closing"
        );

        // After the exit transition settles, the barrier must drop and the underlay becomes
        // interactive again.
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_300,
        ) + 2;
        for _ in 0..settle_frames {
            render_sheet_frame_with_underlay(
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
    fn sheet_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(fret_runtime::FrameId(1));
        let trigger = render_sheet_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            underlay_id.clone(),
            None,
            content_id,
            initial_focus_id.clone(),
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(fret_runtime::FrameId(2));
        let _ = render_sheet_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            underlay_id,
            None,
            Rc::new(Cell::new(None)),
            initial_focus_id.clone(),
            Some(handler),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let initial_focus = initial_focus_id.get().expect("initial focus element");
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
            "expected focus containment to keep focus within the sheet layer"
        );
    }

    #[test]
    fn sheet_open_auto_focus_can_be_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
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

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(fret_runtime::FrameId(1));
        let trigger = render_sheet_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
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

        app.set_frame_id(fret_runtime::FrameId(2));
        let _ = render_sheet_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
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
    fn sheet_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(Mutex::new(None));

        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
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

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(fret_runtime::FrameId(1));
        let trigger = render_sheet_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
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

        app.set_frame_id(fret_runtime::FrameId(2));
        let _ = render_sheet_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
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
            "expected focus containment to clamp focus within the sheet layer"
        );
    }

    #[test]
    fn sheet_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let underlay_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
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

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(fret_runtime::FrameId(1));
        let _trigger = render_sheet_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            underlay_id_out.clone(),
            Some(underlay_id_cell.clone()),
            content_id,
            initial_focus_id.clone(),
            None,
            Some(handler.clone()),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus = initial_focus_id.get().expect("initial focus element");
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
            app.set_frame_id(fret_runtime::FrameId(2 + i as u64));
            let _ = render_sheet_frame_with_auto_focus_hooks(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
                SheetSide::Right,
                underlay_id_out.clone(),
                Some(underlay_id_cell.clone()),
                Rc::new(Cell::new(None)),
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
