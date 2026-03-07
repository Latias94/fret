use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, Point, Px, SemanticsRole, TextAlign, TextOverflow, TextWrap,
};
use fret_icons::ids;
use fret_runtime::{Model, ModelId};
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, LayoutStyle, Length, MainAlign,
    OpacityProps, PressableA11y, PressableProps, RingPlacement, RingStyle, SemanticFlexProps,
    SemanticsDecoration, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeNamedColorKey, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::glass::{GlassPanelProps, glass_panel};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::primitives::portal_inherited;
use fret_ui_kit::recipes::glass::GlassEffectRefinement;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Space, UiChildIntoElement, UiHostBoundIntoElement, ui,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;
use crate::typography_scope::scope_description;

#[derive(Debug, Default)]
struct DialogOpenProviderState {
    current: Option<Model<bool>>,
}

fn inherited_dialog_open<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Model<bool>> {
    cx.inherited_state_where::<DialogOpenProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current.clone())
}

#[track_caller]
fn with_dialog_open_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(DialogOpenProviderState::default, |st| {
        let prev = st.current.clone();
        st.current = Some(open);
        prev
    });
    let out = f(cx);
    cx.with_state(DialogOpenProviderState::default, |st| {
        st.current = prev;
    });
    out
}

fn default_overlay_color(theme: &ThemeSnapshot) -> Color {
    let mut scrim = theme.named_color(ThemeNamedColorKey::Black);
    scrim.a = 0.5;
    scrim
}

/// Overlay backdrop visual style for shadcn `Dialog`.
///
/// Note: This is a policy/recipe surface (ecosystem layer). Mechanism-level overlay contracts
/// remain in `crates/fret-ui` and `ecosystem/fret-ui-kit`.
#[derive(Debug, Clone)]
pub enum DialogOverlayBackdrop {
    /// A solid scrim behind the dialog content (default shadcn/Radix baseline).
    Solid,
    /// A blurred "glass" backdrop (reduced-transparency aware; implemented via `fret-ui-kit` glass).
    Glass(DialogGlassBackdropRefinement),
}

/// shadcn/ui `DialogPortal` (v4).
///
/// Fret installs modal dialogs through the overlay controller, which implies a portal-like
/// boundary already. This type is a no-op marker that exists to align the shadcn part surface and
/// leave room for future portal configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct DialogPortal;

impl DialogPortal {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `DialogOverlay` (v4).
///
/// Upstream exposes the overlay (scrim/backdrop) as a distinct part with styling concerns.
/// Fret's dialog surface currently owns the overlay policy knobs on [`Dialog`]. This type is a
/// thin adapter so part-based call sites can keep configuration in a shadcn-like location.
#[derive(Debug, Clone, Default)]
pub struct DialogOverlay {
    closable: Option<bool>,
    color: Option<Color>,
    backdrop: Option<DialogOverlayBackdrop>,
}

impl DialogOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Controls whether outside pointer press dismisses the dialog.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = Some(closable);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn backdrop(mut self, backdrop: DialogOverlayBackdrop) -> Self {
        self.backdrop = Some(backdrop);
        self
    }

    fn apply_to(self, mut dialog: Dialog) -> Dialog {
        if let Some(v) = self.closable {
            dialog.overlay_closable = v;
        }
        if let Some(v) = self.color {
            dialog.overlay_color = Some(v);
        }
        if let Some(v) = self.backdrop {
            dialog.overlay_backdrop = v;
        }
        dialog
    }
}

/// shadcn/ui `DialogTrigger` (v4).
///
/// In the upstream DOM implementation this is a Radix primitive part. In Fret, the trigger element
/// itself is still authored by the caller; this wrapper exists to align the part surface with
/// shadcn docs/examples and to keep room for future trigger-specific defaults.
#[derive(Debug)]
pub struct DialogTrigger {
    child: AnyElement,
}

pub struct DialogTriggerBuild<H, T> {
    child: Option<T>,
    _phantom: PhantomData<fn() -> H>,
}

impl DialogTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    /// Builder-first variant that late-lands the trigger child at `into_element(cx)` time.
    pub fn build<H: UiHost, T>(child: T) -> DialogTriggerBuild<H, T>
    where
        T: UiChildIntoElement<H>,
    {
        DialogTriggerBuild {
            child: Some(child),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

impl<H: UiHost, T> DialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DialogTrigger::new(
            self.child
                .expect("expected dialog trigger child")
                .into_child_element(cx),
        )
        .into_element(cx)
    }
}

impl<H: UiHost, T> UiHostBoundIntoElement<H> for DialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DialogTriggerBuild::into_element(self, cx)
    }
}

impl<H: UiHost, T> UiChildIntoElement<H> for DialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    fn into_child_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        DialogTriggerBuild::into_element(self, cx)
    }
}

#[doc(hidden)]
pub trait DialogCompositionTriggerArg<H: UiHost> {
    fn into_dialog_trigger(self, cx: &mut ElementContext<'_, H>) -> DialogTrigger;
}

impl<H: UiHost> DialogCompositionTriggerArg<H> for DialogTrigger {
    fn into_dialog_trigger(self, _cx: &mut ElementContext<'_, H>) -> DialogTrigger {
        self
    }
}

impl<H: UiHost, T> DialogCompositionTriggerArg<H> for DialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    fn into_dialog_trigger(self, cx: &mut ElementContext<'_, H>) -> DialogTrigger {
        DialogTrigger::new(
            self.child
                .expect("expected dialog trigger child")
                .into_child_element(cx),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DialogGlassBackdropRefinement {
    pub blur_radius_px: Px,
    pub blur_downsample: u32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
}

impl Default for DialogGlassBackdropRefinement {
    fn default() -> Self {
        Self {
            blur_radius_px: Px(14.0),
            blur_downsample: 2,
            saturation: 1.05,
            brightness: 1.0,
            contrast: 1.0,
        }
    }
}

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default)]
struct DialogOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn dialog_open_change_events(
    state: &mut DialogOpenChangeCallbackState,
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
    overlay_backdrop: DialogOverlayBackdrop,
    window_padding: Space,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for Dialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialog")
            .field("open", &"<model>")
            .field("overlay_closable", &self.overlay_closable)
            .field("overlay_color", &self.overlay_color)
            .field("overlay_backdrop", &self.overlay_backdrop)
            .field("window_padding", &self.window_padding)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .finish()
    }
}

impl Dialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            overlay_closable: true,
            overlay_color: None,
            overlay_backdrop: DialogOverlayBackdrop::Solid,
            window_padding: Space::N4,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
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

    /// Base UI-compatible alias.
    ///
    /// When `true`, outside pointer press does not dismiss the dialog.
    /// This is equivalent to `overlay_closable(false)`.
    pub fn disable_pointer_dismissal(mut self, disable: bool) -> Self {
        self.overlay_closable = !disable;
        self
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    pub fn overlay_backdrop(mut self, backdrop: DialogOverlayBackdrop) -> Self {
        self.overlay_backdrop = backdrop;
        self
    }

    pub fn overlay_glass_backdrop(mut self, enabled: bool) -> Self {
        self.overlay_backdrop = if enabled {
            DialogOverlayBackdrop::Glass(DialogGlassBackdropRefinement::default())
        } else {
            DialogOverlayBackdrop::Solid
        };
        self
    }

    pub fn overlay_glass_backdrop_refinement(
        mut self,
        refinement: DialogGlassBackdropRefinement,
    ) -> Self {
        self.overlay_backdrop = DialogOverlayBackdrop::Glass(refinement);
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
    /// shadcn/Radix/Base UI while keeping the underlying mechanism surface unchanged.
    pub fn compose<H: UiHost>(self) -> DialogComposition<H> {
        DialogComposition::new(self)
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

            let motion = OverlayController::transition_with_durations_and_cubic_bezier_duration(
                cx,
                is_open,
                overlay_motion::shadcn_overlay_open_duration(cx),
                overlay_motion::shadcn_overlay_close_duration(cx),
                overlay_motion::shadcn_overlay_ease_bezier(cx),
            );
            let (open_change, open_change_complete) = cx
                .with_state(DialogOpenChangeCallbackState::default, |state| {
                    dialog_open_change_events(state, is_open, motion.present, motion.animating)
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
                let on_open_auto_focus = self.on_open_auto_focus.clone();
                let policy = radix_dialog::DialogCloseAutoFocusGuardPolicy::for_modal(true);
                let (on_dismiss_request_for_request, on_close_auto_focus) =
                    radix_dialog::dialog_close_auto_focus_guard_hooks(
                        cx,
                        policy,
                        self.open.clone(),
                        self.on_dismiss_request.clone(),
                        self.on_close_auto_focus.clone(),
                    );

                let overlay_color = self
                    .overlay_color
                    .unwrap_or_else(|| default_overlay_color(&theme));
                let overlay_closable = self.overlay_closable;
                let window_padding_px = MetricRef::space(self.window_padding).resolve(&theme);

                let opacity = motion.progress;
                let overlay_backdrop = self.overlay_backdrop.clone();
                let portal_inherited = portal_inherited::PortalInherited::capture(cx);
                let overlay_children = portal_inherited::with_root_name_inheriting(
                    cx,
                    &overlay_root_name,
                    portal_inherited,
                    |cx| {
                        let barrier_fill: AnyElement = match overlay_backdrop {
                            DialogOverlayBackdrop::Solid => cx.container(
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
                                    background: Some(overlay_color),
                                    shadow: None,
                                    border: Edges::all(Px(0.0)),
                                    border_color: None,
                                    corner_radii: Corners::all(Px(0.0)),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            ),
                            DialogOverlayBackdrop::Glass(refinement) => {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;

                                let chrome = ChromeRefinement::default()
                                    .p(Space::N0)
                                    .radius(Px(0.0))
                                    .border_width(Px(0.0))
                                    .bg(ColorRef::Color(overlay_color));
                                let effect = GlassEffectRefinement {
                                    blur_radius_px: Some(refinement.blur_radius_px),
                                    blur_downsample: Some(refinement.blur_downsample),
                                    saturation: Some(refinement.saturation),
                                    brightness: Some(refinement.brightness),
                                    contrast: Some(refinement.contrast),
                                };

                                glass_panel(
                                    cx,
                                    GlassPanelProps {
                                        layout,
                                        chrome,
                                        effect,
                                        ..Default::default()
                                    },
                                    |_cx| Vec::<AnyElement>::new(),
                                )
                            }
                        };

                        crate::a11y_modal::begin_modal_a11y_scope(cx.app, open_id);
                        let content = with_dialog_open_provider(cx, self.open.clone(), content);
                        let content_id = content.id;
                        content_element_for_trigger.set(Some(content_id));
                        crate::a11y_modal::end_modal_a11y_scope(cx.app, open_id);

                        // Center the dialog via an input-transparent flex wrapper so we don't need
                        // last-frame bounds (which can cause a 1-frame jump on first open).
                        let outer = cx.environment_viewport_bounds(fret_ui::Invalidation::Layout);
                        let origin = Point::new(
                            Px(outer.origin.x.0 + outer.size.width.0 * 0.5),
                            Px(outer.origin.y.0 + outer.size.height.0 * 0.5),
                        );
                        let zoom = overlay_motion::shadcn_zoom_transform(origin, opacity);

                        let mut centered_layout = LayoutStyle::default();
                        centered_layout.size.width = Length::Fill;
                        centered_layout.size.height = Length::Fill;
                        let centered = cx.semantic_flex(
                            SemanticFlexProps {
                                role: SemanticsRole::Generic,
                                flex: FlexProps {
                                    layout: centered_layout,
                                    direction: fret_core::Axis::Vertical,
                                    padding: Edges::all(window_padding_px).into(),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    ..Default::default()
                                },
                            },
                            move |_cx| vec![content],
                        );
                        let dialog = overlay_motion::wrap_opacity_and_render_transform(
                            cx,
                            opacity,
                            zoom,
                            vec![centered],
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
                    },
                );

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

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over [`Dialog::into_element`] that accepts shadcn-style parts
    /// (`DialogTrigger`, `DialogPortal`, `DialogOverlay`).
    ///
    /// It also installs a default "open on activate" behavior on the trigger element when the
    /// trigger is a `Pressable` (e.g. shadcn `Button`), matching the upstream Radix trigger
    /// contract.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> DialogTrigger,
        _portal: DialogPortal,
        overlay: DialogOverlay,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let dialog = overlay.apply_to(self);
        let open_for_trigger = dialog.open.clone();
        dialog.into_element(
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

/// Recipe-level builder for composing a dialog from shadcn-style parts.
///
/// This builder stores already-authored Fret elements/parts and lowers them into the existing
/// closure-based `into_element_parts(...)` entry point at the end.
type DialogDeferredContent<H> = Box<dyn FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static>;

enum DialogCompositionContent<H: UiHost> {
    Eager(AnyElement),
    Deferred(DialogDeferredContent<H>),
}

pub struct DialogComposition<H: UiHost, TTrigger = DialogTrigger> {
    dialog: Dialog,
    trigger: Option<TTrigger>,
    portal: DialogPortal,
    overlay: DialogOverlay,
    content: Option<DialogCompositionContent<H>>,
}

impl<H: UiHost, TTrigger> std::fmt::Debug for DialogComposition<H, TTrigger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogComposition")
            .field("dialog", &self.dialog)
            .field("trigger", &self.trigger.is_some())
            .field("portal", &self.portal)
            .field("overlay", &self.overlay)
            .field("content", &self.content.is_some())
            .finish()
    }
}

impl<H: UiHost> DialogComposition<H> {
    pub fn new(dialog: Dialog) -> Self {
        Self {
            dialog,
            trigger: None,
            portal: DialogPortal::new(),
            overlay: DialogOverlay::new(),
            content: None,
        }
    }
}

impl<H: UiHost, TTrigger> DialogComposition<H, TTrigger> {
    pub fn trigger<TNextTrigger>(
        self,
        trigger: TNextTrigger,
    ) -> DialogComposition<H, TNextTrigger> {
        DialogComposition {
            dialog: self.dialog,
            trigger: Some(trigger),
            portal: self.portal,
            overlay: self.overlay,
            content: self.content,
        }
    }

    pub fn portal(mut self, portal: DialogPortal) -> Self {
        self.portal = portal;
        self
    }

    pub fn overlay(mut self, overlay: DialogOverlay) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(DialogCompositionContent::Eager(content));
        self
    }

    pub fn content_with(
        mut self,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(DialogCompositionContent::Deferred(Box::new(content)));
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TTrigger: DialogCompositionTriggerArg<H>,
    {
        let trigger = self
            .trigger
            .expect("Dialog::compose().trigger(...) must be provided before into_element()")
            .into_dialog_trigger(cx);
        let content = self
            .content
            .expect("Dialog::compose().content(...) must be provided before into_element()");

        let portal = self.portal;
        let overlay = self.overlay;

        match content {
            DialogCompositionContent::Eager(content) => self.dialog.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |_cx| content,
            ),
            DialogCompositionContent::Deferred(content) => self.dialog.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |cx| content(cx),
            ),
        }
    }
}

/// shadcn/ui `DialogContent` (v4).
#[derive(Debug)]
pub struct DialogContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    a11y_label: Option<Arc<str>>,
}

impl DialogContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            a11y_label: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
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
        let theme = Theme::global(&*cx.app).snapshot();

        let chrome = crate::ui_builder_ext::surfaces::dialog_style_chrome().merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_full()
            .max_w(Px(512.0))
            .min_w_0()
            .min_h_0()
            .merge(self.layout);

        if let Some(max_w) = layout
            .size
            .as_ref()
            .and_then(|s| s.max_width.as_ref())
            .and_then(|m| match m {
                fret_ui_kit::LengthRefinement::Px(metric) => Some(metric.resolve(&theme)),
                _ => None,
            })
        {
            crate::a11y_modal::register_modal_content_max_width(cx.app, max_w);
        }

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        let a11y_label = self.a11y_label;
        let container = shadcn_layout::container_vstack(
            cx,
            props,
            shadcn_layout::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0()),
            children,
        );

        let (labelled_by_element, described_by_element) =
            crate::a11y_modal::modal_relations_for_current_scope(cx.app);

        container.attach_semantics(SemanticsDecoration {
            role: Some(SemanticsRole::Dialog),
            label: a11y_label,
            labelled_by_element,
            described_by_element,
            ..Default::default()
        })
    }
}

/// shadcn/ui `DialogClose` (v4-aligned recipe).
///
/// Upstream shadcn's `DialogContent` renders a close affordance wired to the underlying Radix
/// primitive. Fret exposes this as an explicit building block so apps can choose to include it (or
/// replace it) while keeping the modal overlay policy decoupled from visuals.
///
/// Note: When used with absolute positioning (the default), place `DialogClose` as the *last*
/// child in `DialogContent` so it stays on top during hit testing.
#[derive(Clone)]
pub struct DialogClose {
    open: Option<Model<bool>>,
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
    /// Creates a close affordance that explicitly toggles the provided dialog open model.
    ///
    /// Prefer this constructor when you want fully explicit data flow or when the close control is
    /// authored outside the dialog content subtree.
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open: Some(open),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a close affordance that closes the dialog resolved from the current content scope.
    ///
    /// This is recipe-layer sugar for shadcn-style composition inside
    /// [`Dialog::into_element`] / [`Dialog::into_element_parts`] content closures. Explicit
    /// `DialogClose::new(open)` remains available and should be preferred when the element is built
    /// outside the dialog content subtree.
    ///
    /// Panics if no dialog content scope is active when the element is rendered.
    pub fn from_scope() -> Self {
        Self {
            open: None,
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let fg = theme
                .color_by_key("muted.foreground")
                .or_else(|| theme.color_by_key("muted-foreground"))
                .unwrap_or_else(|| theme.color_token("muted.foreground"));

            let a11y_label: Arc<str> = Arc::from("Close");
            let open = self.open.clone().unwrap_or_else(|| {
                inherited_dialog_open(cx).unwrap_or_else(|| {
                    panic!("DialogClose::from_scope() must be used while rendering Dialog content")
                })
            });

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
                    .unwrap_or_else(|| theme.color_token("ring"));
                let ring_offset_bg = theme
                    .color_by_key("ring-offset-background")
                    .unwrap_or_else(|| theme.color_token("ring-offset-background"));

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

                    vec![
                        ui::h_row(|_cx| vec![icon])
                            .justify_center()
                            .items_center()
                            .into_element(cx),
                    ]
                };

                (pressable_props, chrome_props, children)
            })
        })
    }
}

/// shadcn/ui `DialogHeader` (v4).
#[derive(Debug)]
pub struct DialogHeader {
    children: Vec<AnyElement>,
}

impl DialogHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        use fret_ui_kit::declarative::ViewportQueryHysteresis;

        // Upstream shadcn uses Tailwind `sm:` to switch `text-center` -> `text-left`.
        //
        // In normal runtime frames this should be driven by the committed viewport snapshot
        // (ADR 0232). For unit tests that construct elements without a committed viewport
        // environment, fall back to the root bounds passed to `ElementContext`.
        let sm_breakpoint = {
            let threshold = fret_ui_kit::declarative::viewport_tailwind::SM;
            let viewport_width = cx.environment_viewport_width(Invalidation::Layout);
            if viewport_width.0 <= 0.0 {
                cx.bounds.size.width.0 >= threshold.0
            } else {
                fret_ui_kit::declarative::viewport_width_at_least(
                    cx,
                    Invalidation::Layout,
                    threshold,
                    ViewportQueryHysteresis::default(),
                )
            }
        };

        fn apply_header_text_alignment(mut element: AnyElement, align: TextAlign) -> AnyElement {
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
                ElementKind::SelectableText(props) => {
                    apply_text(&mut props.layout, &mut props.align)
                }
                _ => {}
            }

            element.children = element
                .children
                .into_iter()
                .map(|child| apply_header_text_alignment(child, align))
                .collect();
            element
        }

        let align = if sm_breakpoint {
            TextAlign::Start
        } else {
            TextAlign::Center
        };

        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full().min_w_0(),
        );

        let children = self
            .children
            .into_iter()
            .map(|child| apply_header_text_alignment(child, align))
            .collect();

        shadcn_layout::container_vstack_gap(cx, props, Space::N2, children)
    }
}

/// shadcn/ui `DialogFooter` (v4).
#[derive(Debug)]
pub struct DialogFooter {
    children: Vec<AnyElement>,
}

impl DialogFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        // Upstream shadcn uses Tailwind `sm:` (viewport breakpoint), so match it via viewport
        // queries (ADR 0232).
        let sm_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            fret_ui_kit::declarative::viewport_tailwind::SM,
            fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
        );

        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full(),
        );

        let mut children = self.children;
        if sm_breakpoint {
            shadcn_layout::container_hstack(
                cx,
                props,
                shadcn_layout::HStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full())
                    .justify_end()
                    .items_center(),
                children,
            )
        } else {
            // Tailwind: `flex-col-reverse gap-2`
            children.reverse();
            shadcn_layout::container_vstack(
                cx,
                props,
                shadcn_layout::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full())
                    .items_stretch(),
                children,
            )
        }
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));

        let px = theme
            .metric_by_key("component.dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.dialog.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let title = ui::text(self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Heading)
                    .level(2),
            );
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let description = scope_description(
            ui::raw_text(self.text)
                .wrap(TextWrap::Word)
                .overflow(TextOverflow::Clip)
                .into_element(cx),
            &theme,
            "component.dialog.description",
        );
        crate::a11y_modal::register_modal_description(cx.app, description.id);
        description
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

    use crate::test_support::render_overlay_frame;
    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{
        KeyCode, Modifiers, Px, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::Effect;
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::element::PositionStyle;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
    use fret_ui_kit::ui::UiElementSinkExt as _;

    #[test]
    fn dialog_trigger_build_push_ui_accepts_late_landed_child() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let mut out = Vec::new();
            out.push_ui(cx, DialogTrigger::build(crate::Card::build(|_cx, _out| {})));

            assert_eq!(out.len(), 1);
            assert!(matches!(out[0].kind, ElementKind::Container(_)));
            assert!(out[0].inherited_foreground.is_some());
        });
    }

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

    #[test]
    fn dialog_disable_pointer_dismissal_alias_maps_overlay_closable() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let a = Dialog::new(open.clone()).disable_pointer_dismissal(true);
        assert!(!a.overlay_closable);

        let b = Dialog::new(open).disable_pointer_dismissal(false);
        assert!(b.overlay_closable);
    }

    #[test]
    fn dialog_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = DialogOpenChangeCallbackState::default();

        let (changed, completed) = dialog_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = dialog_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = dialog_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn dialog_open_change_events_complete_without_animation() {
        let mut state = DialogOpenChangeCallbackState::default();

        let _ = dialog_open_change_events(&mut state, false, false, false);
        let (changed, completed) = dialog_open_change_events(&mut state, true, true, false);

        assert_eq!(changed, Some(true));
        assert_eq!(completed, Some(true));
    }

    fn find_text_element<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        match &el.kind {
            fret_ui::element::ElementKind::Text(props) if props.text.as_ref() == needle => Some(el),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_element(child, needle)),
        }
    }

    fn find_text<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a fret_ui::element::TextProps> {
        let node = find_text_element(el, needle)?;
        match &node.kind {
            fret_ui::element::ElementKind::Text(props) => Some(props),
            _ => None,
        }
    }

    #[test]
    fn dialog_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            DialogDescription::new("Description").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected DialogDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&crate::typography_scope::description_refinement(
                &theme,
                "component.dialog.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(crate::typography_scope::muted_foreground(&theme))
        );
    }

    #[test]
    fn dialog_header_defaults_to_w_full_without_padding() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            DialogHeader::new([
                DialogTitle::new("Title").into_element(cx),
                DialogDescription::new("Description").into_element(cx),
            ])
            .into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!(
                "expected DialogHeader root to be a Container, got {:?}",
                el.kind
            );
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(
            props.padding.top,
            fret_ui::element::SpacingLength::Px(Px(0.0))
        );
        assert_eq!(
            props.padding.right,
            fret_ui::element::SpacingLength::Px(Px(0.0))
        );
        assert_eq!(
            props.padding.bottom,
            fret_ui::element::SpacingLength::Px(Px(0.0))
        );
        assert_eq!(
            props.padding.left,
            fret_ui::element::SpacingLength::Px(Px(0.0))
        );
    }

    #[test]
    fn dialog_header_centers_text_below_sm_breakpoint() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            DialogHeader::new([
                DialogTitle::new("Title").into_element(cx),
                DialogDescription::new("Description").into_element(cx),
            ])
            .into_element(cx)
        });

        for label in ["Title", "Description"] {
            let text = find_text(&el, label).expect("expected dialog header text node");
            assert_eq!(text.align, TextAlign::Center);
            assert_eq!(text.layout.size.width, Length::Fill);
            assert_eq!(text.layout.size.min_width, Some(Length::Px(Px(0.0))));
        }
    }

    #[test]
    fn dialog_header_left_aligns_text_at_sm_breakpoint() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            DialogHeader::new([
                DialogTitle::new("Title").into_element(cx),
                DialogDescription::new("Description").into_element(cx),
            ])
            .into_element(cx)
        });

        for label in ["Title", "Description"] {
            let text = find_text(&el, label).expect("expected dialog header text node");
            assert_eq!(text.align, TextAlign::Start);
            assert_eq!(text.layout.size.width, Length::Fill);
        }
    }

    fn render_dialog_frame_with_footer(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        action_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) {
        OverlayController::begin_frame(app, window);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(10.0));
                            layout.size.height = Length::Px(Px(10.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let dialog = Dialog::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let cancel_id_out = cancel_id_out.clone();
                        let cancel = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                cancel_id_out.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let action_id_out = action_id_out.clone();
                        let action = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(44.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                action_id_out.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let footer = DialogFooter::new(vec![cancel, action]).into_element(cx);
                        DialogContent::new(vec![footer]).into_element(cx)
                    },
                );

                vec![dialog]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn dialog_footer_stacks_on_base_viewport_and_rows_on_sm() {
        fn assert_footer_layout(bounds: Rect, expect_row: bool) {
            let window = AppWindowId::default();
            let mut app = App::new();
            let mut ui: UiTree<App> = UiTree::new();
            ui.set_window(window);

            let open = app.models_mut().insert(true);
            let cancel_id = Rc::new(Cell::new(None));
            let action_id = Rc::new(Cell::new(None));

            let mut services = FakeServices;

            // Viewport queries read the committed per-window environment snapshot, so render two
            // frames to allow the width to commit before asserting layout.
            for frame in 1..=2 {
                app.set_frame_id(FrameId(frame));
                render_dialog_frame_with_footer(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    open.clone(),
                    cancel_id.clone(),
                    action_id.clone(),
                );
                ui.layout_all(&mut app, &mut services, bounds, 1.0);
            }

            let cancel_bounds = fret_ui::elements::bounds_for_element(
                &mut app,
                window,
                cancel_id.get().expect("cancel element id"),
            )
            .expect("cancel bounds");
            let action_bounds = fret_ui::elements::bounds_for_element(
                &mut app,
                window,
                action_id.get().expect("action element id"),
            )
            .expect("action bounds");

            if expect_row {
                assert!(
                    (cancel_bounds.origin.y.0 - action_bounds.origin.y.0).abs() < 2.0,
                    "expected footer buttons to share a row; cancel={cancel_bounds:?} action={action_bounds:?}"
                );
                assert!(cancel_bounds.origin.x.0 < action_bounds.origin.x.0);
            } else {
                // col-reverse => action above cancel
                assert!(action_bounds.origin.y.0 < cancel_bounds.origin.y.0);
            }
        }

        // Base viewport: vertical stack (col-reverse => action above cancel).
        assert_footer_layout(
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(480.0), Px(600.0)),
            ),
            false,
        );

        // `sm:` viewport: horizontal row (cancel left of action).
        assert_footer_layout(
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            true,
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
    fn dialog_into_element_parts_trigger_opens_on_activate() {
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
            "shadcn-dialog-into-element-parts-trigger-opens",
            |cx| {
                vec![Dialog::new(open.clone()).into_element_parts(
                    cx,
                    |cx| DialogTrigger::new(crate::Button::new("Open").into_element(cx)),
                    DialogPortal::new(),
                    DialogOverlay::new(),
                    |cx| DialogContent::new([cx.text("Content")]).into_element(cx),
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
                position,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }
    #[test]
    fn dialog_composition_trigger_accepts_late_landed_child() {
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
            "shadcn-dialog-composition-trigger-accepts-late-child",
            |cx| {
                vec![
                    Dialog::new(open.clone())
                        .compose()
                        .trigger(DialogTrigger::build(
                            crate::Button::new("Open").test_id("dialog-compose-trigger-late-child"),
                        ))
                        .portal(DialogPortal::new())
                        .overlay(DialogOverlay::new())
                        .content(DialogContent::new([cx.text("Content")]).into_element(cx))
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
                position,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn dialog_compose_content_with_supports_from_scope_close() {
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
            "shadcn-dialog-compose-content-with-from-scope",
            |cx| {
                let trigger = DialogTrigger::new(crate::Button::new("Open").into_element(cx));

                vec![
                    Dialog::new(open.clone())
                        .compose()
                        .trigger(trigger)
                        .portal(DialogPortal::new())
                        .overlay(DialogOverlay::new())
                        .content_with(|cx| {
                            let close = DialogClose::from_scope().into_element(cx);
                            DialogContent::new(vec![close]).into_element(cx)
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
    #[should_panic(
        expected = "DialogClose::from_scope() must be used while rendering Dialog content"
    )]
    fn dialog_close_from_scope_panics_outside_dialog_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let _ = DialogClose::from_scope().into_element(cx);
        });
    }

    #[test]
    fn dialog_does_not_jump_on_first_open_frame_with_tall_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            OverlayController::begin_frame(app, window);

            let trigger_id = trigger_id.clone();
            let content_id = content_id.clone();
            let open_for_trigger = open.clone();
            let open_for_close = open.clone();

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
                            cx.pressable_toggle_bool(&open_for_trigger);
                            trigger_id.set(Some(id));
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let dialog = Dialog::new(open.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let tall_body = cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Px(Px(480.0));
                                        layout
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            );

                            let close = DialogClose::new(open_for_close.clone()).into_element(cx);
                            let content =
                                DialogContent::new(vec![tall_body, close]).into_element(cx);
                            content_id.set(Some(content.id));
                            content
                        },
                    );

                    vec![dialog]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            ui.layout_all(app, services, bounds, 1.0);
        };

        // Frame 1: closed.
        render_frame(&mut ui, &mut app, &mut services);

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

        // Frame 2: open.
        render_frame(&mut ui, &mut app, &mut services);
        let content_frame2 = content_id.get().expect("content element id");
        let content_node_frame2 =
            fret_ui::elements::node_for_element(&mut app, window, content_frame2)
                .expect("content node");
        let bounds_frame2 = ui
            .debug_node_bounds(content_node_frame2)
            .expect("content bounds");

        // Frame 3: open (no additional events).
        render_frame(&mut ui, &mut app, &mut services);
        let content_frame3 = content_id.get().expect("content element id");
        let content_node_frame3 =
            fret_ui::elements::node_for_element(&mut app, window, content_frame3)
                .expect("content node");
        let bounds_frame3 = ui
            .debug_node_bounds(content_node_frame3)
            .expect("content bounds");

        assert!(
            (bounds_frame2.origin.y.0 - bounds_frame3.origin.y.0).abs() <= 1.0,
            "dialog content jumped between frames: frame2={:?} frame3={:?}",
            bounds_frame2,
            bounds_frame3
        );
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

                            let close = DialogClose::from_scope().into_element(cx);
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

    fn render_dialog_frame_with_auto_focus_hooks(
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
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
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

    fn render_dialog_frame_with_open_auto_focus_redirect_target(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
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

                let redirect_focus_id_cell = redirect_focus_id_cell.clone();
                let dialog = Dialog::new(open.clone())
                    .overlay_closable(overlay_closable)
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

                            DialogContent::new(vec![initial, redirect]).into_element(cx)
                        },
                    );

                vec![dialog]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_dialog_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
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
            "dialog-underlay",
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
                            layout.position = PositionStyle::Absolute;
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

                let dialog = Dialog::new(open.clone())
                    .overlay_closable(overlay_closable)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let content =
                                DialogContent::new(vec![ui::raw_text("Content").into_element(cx)])
                                    .into_element(cx);
                            content
                        },
                    );

                vec![underlay, dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
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
    fn dialog_close_transition_keeps_modal_barrier_blocking_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices;

        // Frame 1: closed.
        render_dialog_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_activated.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_dialog_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected dialog to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> barrier must remain active.
        render_dialog_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected barrier root to remain while the dialog is closing");
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
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should remain inert while the dialog is closing"
        );

        // After the exit transition settles, the barrier must drop and the underlay becomes
        // interactive again.
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            overlay_motion::SHADCN_MOTION_DURATION_200,
        ) + 2;
        for _ in 0..settle_frames {
            render_dialog_frame_with_underlay(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
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
                modifiers: Modifiers::default(),
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
                modifiers: Modifiers::default(),
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
    fn dialog_close_transition_restores_trigger_focus_while_barrier_blocks_underlay_pointer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices;

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            open: Model<bool>,
            underlay_activated: Model<bool>,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            let _ = render_overlay_frame(
                ui,
                app,
                services,
                window,
                bounds,
                "dialog-close-transition-focus-restore",
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

                    let trigger_id_out = trigger_id_out.clone();
                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.left = Some(Px(100.0)).into();
                                layout.inset.top = Some(Px(100.0)).into();
                                layout.position = PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: PressableA11y {
                                test_id: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            cx.pressable_toggle_bool(&open);
                            Vec::new()
                        },
                    );

                    let focusable_id_out = focusable_id_out.clone();
                    let dialog = Dialog::new(open.clone()).into_element(
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
                                    a11y: PressableA11y {
                                        test_id: Some(Arc::from("dialog-focusable")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |_cx, _st, id| {
                                    focusable_id_out.set(Some(id));
                                    Vec::new()
                                },
                            );
                            DialogContent::new(vec![focusable]).into_element(cx)
                        },
                    );

                    vec![underlay, dialog]
                },
            );
        }

        // Frame 1: closed.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            focusable_id_out.clone(),
        );

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            focusable_id_out.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected dialog to install a modal barrier root"
        );

        let focusable_id = focusable_id_out.get().expect("dialog focusable id");
        let focusable_node = fret_ui::elements::node_for_element(&mut app, window, focusable_id)
            .expect("dialog focusable");
        ui.set_focus(Some(focusable_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> focus should be restored even
        // though pointer interactions remain blocked by the barrier.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            focusable_id_out.clone(),
        );

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected close transition to restore focus to the trigger"
        );

        let click = Point::new(Px(10.0), Px(10.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
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
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "expected the modal barrier to keep the underlay inert while closing"
        );
    }

    #[test]
    fn dialog_close_transition_on_close_auto_focus_can_prevent_default_and_restore_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let trigger_id_cell: Arc<std::sync::Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(std::sync::Mutex::new(None));
        let trigger_id_for_handler = trigger_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let trigger = trigger_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(trigger) = trigger {
                host.request_focus(trigger);
            }
            req.prevent_default();
        });

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            open: Model<bool>,
            underlay_activated: Model<bool>,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            trigger_id_cell: Arc<std::sync::Mutex<Option<fret_ui::elements::GlobalElementId>>>,
            focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            handler: OnCloseAutoFocus,
        ) {
            let _ = render_overlay_frame(
                ui,
                app,
                services,
                window,
                bounds,
                "dialog-close-transition-on-close-auto-focus",
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

                    let trigger_id_out = trigger_id_out.clone();
                    let trigger_id_cell = trigger_id_cell.clone();
                    let open_for_trigger = open.clone();
                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.left = Some(Px(100.0)).into();
                                layout.inset.top = Some(Px(100.0)).into();
                                layout.position = PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: PressableA11y {
                                test_id: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            *trigger_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                            cx.pressable_toggle_bool(&open_for_trigger);
                            Vec::new()
                        },
                    );

                    let focusable_id_out = focusable_id_out.clone();
                    let handler = handler.clone();
                    let dialog = Dialog::new(open.clone())
                        .on_close_auto_focus(Some(handler))
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
                                        a11y: PressableA11y {
                                            test_id: Some(Arc::from("dialog-focusable")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |_cx, _st, id| {
                                        focusable_id_out.set(Some(id));
                                        Vec::new()
                                    },
                                );
                                DialogContent::new(vec![focusable]).into_element(cx)
                            },
                        );

                    vec![underlay, dialog]
                },
            );
        }

        // Frame 1: closed.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            trigger_id_cell.clone(),
            focusable_id_out.clone(),
            handler.clone(),
        );

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            trigger_id_cell.clone(),
            focusable_id_out.clone(),
            handler.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected dialog to install a modal barrier root"
        );

        let focusable_id = focusable_id_out.get().expect("focusable id");
        let focusable_node = fret_ui::elements::node_for_element(&mut app, window, focusable_id)
            .expect("focusable node");
        ui.set_focus(Some(focusable_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing -> handler should be able to restore focus while barrier blocks pointer.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_activated.clone(),
            trigger_id_out.clone(),
            trigger_id_cell.clone(),
            focusable_id_out.clone(),
            handler.clone(),
        );

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected on_close_auto_focus to restore focus to the trigger"
        );

        let click = Point::new(Px(10.0), Px(10.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
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
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "expected the modal barrier to keep the underlay inert while closing"
        );
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
    fn dialog_escape_closes_by_default_when_handler_allows() {
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

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(
            dismiss_reason.get(),
            Some(fret_ui::action::DismissReason::Escape)
        );
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
                );

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
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_200,
        ) as usize
            + 1;
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
    fn dialog_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let close_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
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

        app.set_frame_id(FrameId(1));
        let trigger = render_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id.clone(),
            initial_focus_id.clone(),
            close_id.clone(),
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id,
            initial_focus_id.clone(),
            close_id,
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
            "expected focus containment to keep focus within the dialog layer"
        );
    }

    #[test]
    fn dialog_open_auto_focus_can_be_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
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

        app.set_frame_id(FrameId(1));
        let trigger = render_dialog_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
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
        let _ = render_dialog_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
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
    fn dialog_open_auto_focus_redirect_to_trigger_is_clamped_to_modal_layer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let close_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();

        let target_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let target_id_for_handler = target_id_cell.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = target_id_for_handler
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

        app.set_frame_id(FrameId(1));
        let trigger = render_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            content_id,
            initial_focus_id.clone(),
            close_id,
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        ui.set_focus(Some(trigger_node));
        *target_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(trigger);

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            Rc::new(Cell::new(None)),
            initial_focus_id.clone(),
            Rc::new(Cell::new(None)),
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
                .expect("initial focus node");

        let focused = ui.focus().expect("expected focus after open");
        assert_ne!(
            focused, trigger_node,
            "expected modal focus containment to prevent focusing the trigger while opening"
        );
        assert_eq!(
            ui.node_layer(focused),
            ui.node_layer(initial_focus_node),
            "expected focus containment to clamp focus within the dialog layer"
        );
    }

    #[test]
    fn dialog_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let close_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_cell: Arc<std::sync::Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(std::sync::Mutex::new(None));
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

        app.set_frame_id(FrameId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "dialog-underlay-autofocus",
            |cx| {
                let content_id = content_id.clone();
                let initial_focus_id = initial_focus_id.clone();
                let close_id = close_id.clone();
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
                        a11y: PressableA11y {
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |_cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                        Vec::new()
                    },
                );

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

                let open_for_dialog = open.clone();
                let open_for_close = open.clone();
                let dialog = Dialog::new(open_for_dialog)
                    .on_close_auto_focus(Some(handler.clone()))
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
                                    initial_focus_id.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let close = DialogClose::new(open_for_close.clone()).into_element(cx);
                            close_id.set(Some(close.id));

                            let content =
                                DialogContent::new(vec![focusable, close]).into_element(cx);
                            content_id.set(Some(content.id));
                            content
                        },
                    );

                vec![underlay, dialog]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus = initial_focus_id.get().expect("initial focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("focusable");
        ui.set_focus(Some(initial_focus_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_200,
        ) as usize
            + 2;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(2 + i as u64));
            OverlayController::begin_frame(&mut app, window);
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "dialog-underlay-autofocus-close",
                |cx| {
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
                            a11y: PressableA11y {
                                test_id: Some(Arc::from("underlay")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx, _st, id| {
                            underlay_id_out.set(Some(id));
                            *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                            Vec::new()
                        },
                    );

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

                    let dialog = Dialog::new(open.clone())
                        .on_close_auto_focus(Some(handler.clone()))
                        .into_element(
                            cx,
                            |_cx| trigger,
                            |cx| cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        );

                    vec![underlay, dialog]
                },
            );
            ui.set_root(root);
            OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
            ui.request_semantics_snapshot();
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
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_200,
        ) as usize
            + 1;
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
                n.role == fret_core::SemanticsRole::Heading
                    && n.label.as_deref() == Some("Dialog Title")
                    && n.extra.level == Some(2)
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
