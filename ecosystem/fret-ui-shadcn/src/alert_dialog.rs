use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontWeight, Point, Px, SemanticsRole, TextAlign, TextOverflow, TextWrap,
};
use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::{OnCloseAutoFocus, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, LayoutStyle, Length, MainAlign,
    RenderTransformProps, SemanticFlexProps, SemanticsDecoration, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeNamedColorKey, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::alert_dialog as radix_alert_dialog;
use fret_ui_kit::primitives::portal_inherited;
use fret_ui_kit::typography::scope_description_text;
use fret_ui_kit::ui::UiChildIntoElement;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, OverlayController,
    OverlayPresence, Radius, Space, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout, ui,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

use crate::button::{Button, ButtonVariant};

fn default_overlay_color(theme: &ThemeSnapshot) -> Color {
    let mut scrim = theme.named_color(ThemeNamedColorKey::Black);
    scrim.a = 0.5;
    scrim
}

/// shadcn/ui `AlertDialogPortal` (v4).
///
/// Fret installs alert dialogs through the overlay controller, which implies a portal-like
/// boundary already. This type is a no-op marker that exists to align the shadcn part surface and
/// leave room for future portal configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlertDialogPortal;

impl AlertDialogPortal {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `AlertDialogOverlay` (v4).
///
/// Upstream exposes the overlay (scrim/backdrop) as a distinct part with styling concerns. Fret's
/// alert dialog surface currently owns the overlay knobs on [`AlertDialog`]. This type is a thin
/// adapter so part-based call sites can keep configuration in a shadcn-like location.
#[derive(Debug, Clone, Default)]
pub struct AlertDialogOverlay {
    color: Option<Color>,
}

impl AlertDialogOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    fn apply_to(self, mut dialog: AlertDialog) -> AlertDialog {
        if let Some(v) = self.color {
            dialog.overlay_color = Some(v);
        }
        dialog
    }
}

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default, Clone)]
struct AlertDialogHandleState {
    active_trigger: Option<Model<Option<GlobalElementId>>>,
    content_element: Option<Model<Option<GlobalElementId>>>,
}

#[derive(Clone)]
pub struct AlertDialogHandle {
    open: Model<bool>,
    active_trigger: Model<Option<GlobalElementId>>,
    content_element: Model<Option<GlobalElementId>>,
}

impl std::fmt::Debug for AlertDialogHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogHandle")
            .field("open", &"<model>")
            .field("active_trigger", &"<model>")
            .field("content_element", &"<model>")
            .finish()
    }
}

impl AlertDialogHandle {
    pub fn new<H: UiHost>(cx: &mut ElementContext<'_, H>, open: Model<bool>) -> Self {
        Self::new_controllable(cx, Some(open), false)
    }

    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_alert_dialog::AlertDialogRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);

        let state = cx.with_state(AlertDialogHandleState::default, |st| st.clone());
        let active_trigger = match state.active_trigger {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(None::<GlobalElementId>);
                cx.with_state(AlertDialogHandleState::default, |st| {
                    st.active_trigger = Some(model.clone())
                });
                model
            }
        };
        let content_element = match state.content_element {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(None::<GlobalElementId>);
                cx.with_state(AlertDialogHandleState::default, |st| {
                    st.content_element = Some(model.clone())
                });
                model
            }
        };

        Self {
            open,
            active_trigger,
            content_element,
        }
    }

    pub fn open_model(&self) -> Model<bool> {
        self.open.clone()
    }
}

/// Tracks the currently rendering alert dialog content scope.
///
/// This is a recipe-layer authoring helper only. It exists so leaf parts whose primary semantic
/// meaning is “close the current alert dialog” can offer an ergonomic `from_scope(...)` builder
/// without changing the lower-level mechanism contract. Callers that prefer explicit data flow can
/// keep using constructors that take an explicit `Model<bool>`.
#[derive(Default)]
struct AlertDialogScopeRegistry {
    stack: Vec<Model<bool>>,
}

fn begin_alert_dialog_scope<H: UiHost>(app: &mut H, open: Model<bool>) {
    app.with_global_mut_untracked(AlertDialogScopeRegistry::default, |reg, _app| {
        reg.stack.push(open);
    });
}

fn end_alert_dialog_scope<H: UiHost>(app: &mut H, expected: fret_runtime::ModelId) {
    app.with_global_mut_untracked(AlertDialogScopeRegistry::default, |reg, _app| {
        let popped = reg.stack.pop();
        if cfg!(debug_assertions) {
            assert_eq!(
                popped.as_ref().map(Model::id),
                Some(expected),
                "alert dialog scope stack mismatch"
            );
        } else if popped.as_ref().map(Model::id) != Some(expected) {
            reg.stack.clear();
        }
    });
}

fn alert_dialog_open_for_current_scope<H: UiHost>(app: &mut H) -> Option<Model<bool>> {
    app.with_global_mut_untracked(AlertDialogScopeRegistry::default, |reg, _app| {
        reg.stack.last().cloned()
    })
}

#[derive(Default)]
struct AlertDialogOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn alert_dialog_open_change_events(
    state: &mut AlertDialogOpenChangeCallbackState,
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

/// shadcn/ui `AlertDialog` (v4).
///
/// This is a modal overlay (barrier-backed). Unlike `Dialog`, the overlay is not closable by
/// default (Radix/shadcn behavior).
#[derive(Clone)]
pub struct AlertDialog {
    open: Model<bool>,
    handle: Option<AlertDialogHandle>,
    overlay_color: Option<Color>,
    window_padding: Space,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for AlertDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialog")
            .field("open", &"<model>")
            .field("handle", &self.handle.is_some())
            .field("overlay_color", &self.overlay_color)
            .field("window_padding", &self.window_padding)
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

impl AlertDialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            handle: None,
            overlay_color: None,
            window_padding: Space::N4,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
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

    pub fn from_handle(handle: AlertDialogHandle) -> Self {
        Self {
            open: handle.open_model(),
            handle: Some(handle),
            overlay_color: None,
            window_padding: Space::N4,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
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
    /// This is an ergonomic bridge between Fret's closure-root authoring model and the nested part
    /// mental model used by shadcn/Radix/Base UI. It intentionally stays in the recipe layer: the
    /// lower-level mechanism still routes through [`AlertDialog::into_element_parts`].
    pub fn compose<H: UiHost>(self) -> AlertDialogComposition<H> {
        AlertDialogComposition::new(self)
    }

    /// Host-bound builder-first helper that late-lands the trigger/content at the root call site.
    #[track_caller]
    pub fn build<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl UiChildIntoElement<H>,
        content: impl UiChildIntoElement<H>,
    ) -> AnyElement {
        self.into_element(
            cx,
            move |cx| trigger.into_child_element(cx),
            move |cx| content.into_child_element(cx),
        )
    }

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over [`AlertDialog::into_element`] that accepts shadcn-style parts
    /// (`AlertDialogTrigger`, `AlertDialogPortal`, `AlertDialogOverlay`).
    ///
    /// It also installs a default "open on activate" behavior on the trigger element when the
    /// trigger is a `Pressable` (e.g. shadcn `Button`), matching the upstream Radix trigger
    /// contract.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AlertDialogTrigger,
        _portal: AlertDialogPortal,
        overlay: AlertDialogOverlay,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let dialog = overlay.apply_to(self);
        let open_for_trigger = dialog.open.clone();
        let handle_for_trigger = dialog.handle.clone();
        dialog.into_element(
            cx,
            move |cx| {
                let trigger = trigger(cx);
                let trigger = if let Some(handle) = handle_for_trigger.clone() {
                    trigger.handle(handle)
                } else {
                    trigger.with_open_model(open_for_trigger.clone())
                };
                trigger.into_element(cx)
            },
            content,
        )
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
            let open_id = self.open.id();

            #[derive(Default)]
            struct AlertDialogA11yState {
                content_element: Option<fret_ui::elements::GlobalElementId>,
            }

            let trigger = trigger(cx);
            let id = trigger.id;
            let request_trigger = self
                .handle
                .as_ref()
                .and_then(|handle| {
                    cx.watch_model(&handle.active_trigger)
                        .paint()
                        .copied()
                        .unwrap_or(None)
                })
                .unwrap_or(id);
            let overlay_root_name = radix_alert_dialog::alert_dialog_root_name(id);
            let prev_content_element =
                cx.with_state(AlertDialogA11yState::default, |st| st.content_element);

            let motion = OverlayController::transition_with_durations_and_cubic_bezier_duration(
                cx,
                is_open,
                overlay_motion::shadcn_overlay_open_duration(cx),
                overlay_motion::shadcn_overlay_close_duration(cx),
                overlay_motion::shadcn_overlay_ease_bezier(cx),
            );
            let (open_change, open_change_complete) =
                cx.slot_state(AlertDialogOpenChangeCallbackState::default, |state| {
                    alert_dialog_open_change_events(
                        state,
                        is_open,
                        motion.present,
                        motion.animating,
                    )
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

            let content_element_for_trigger: std::cell::Cell<
                Option<fret_ui::elements::GlobalElementId>,
            > = std::cell::Cell::new(None);

            if overlay_presence.present {
                if is_open {
                    radix_alert_dialog::clear_cancel_for_open_model(cx, open_id);
                }

                let overlay_color = self
                    .overlay_color
                    .unwrap_or_else(|| default_overlay_color(&theme));
                let window_padding_px = MetricRef::space(self.window_padding).resolve(&theme);
                let opacity = motion.progress;

                let portal_inherited = portal_inherited::PortalInherited::capture(cx);
                let overlay_children = portal_inherited::with_root_name_inheriting(
                    cx,
                    &overlay_root_name,
                    portal_inherited,
                    |cx| {
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
                        begin_alert_dialog_scope(cx.app, self.open.clone());
                        let content = content(cx);
                        end_alert_dialog_scope(cx.app, open_id);
                        content_element_for_trigger.set(Some(content.id));
                        crate::a11y_modal::end_modal_a11y_scope(cx.app, open_id);

                        // Center like `Dialog` via an input-transparent flex wrapper so we don't need
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

                        let opacity_layout = LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        let content_layout = opacity_layout.clone();
                        let barrier_children = [barrier_fill];
                        let open_for_children = self.open.clone();

                        let content =
                            overlay_motion::wrap_opacity_and_render_transform_with_layouts(
                                cx,
                                opacity_layout,
                                opacity,
                                RenderTransformProps {
                                    layout: content_layout,
                                    transform: zoom,
                                },
                                vec![centered],
                            );
                        radix_alert_dialog::alert_dialog_modal_layer_elements(
                            cx,
                            open_for_children.clone(),
                            barrier_children,
                            content,
                        )
                    },
                );

                if let Some(content_element) = content_element_for_trigger.get() {
                    if let Some(handle) = self.handle.as_ref() {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&handle.content_element, |value| {
                                *value = Some(content_element)
                            });
                    }
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
                    request_trigger,
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

/// Recipe-level builder for composing an alert dialog from shadcn-style parts.
///
/// Unlike upstream React children composition, this builder stores already-authored Fret elements
/// and lowers them into the existing closure-based entry point at the end. That keeps the
/// mechanism surface unchanged while giving call sites a more composable authoring style.
type AlertDialogDeferredContent<H> =
    Box<dyn FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static>;

enum AlertDialogCompositionContent<H: UiHost> {
    Eager(AnyElement),
    Deferred(AlertDialogDeferredContent<H>),
}

pub struct AlertDialogComposition<H: UiHost, TTrigger = AlertDialogTrigger> {
    dialog: AlertDialog,
    trigger: Option<TTrigger>,
    portal: AlertDialogPortal,
    overlay: AlertDialogOverlay,
    content: Option<AlertDialogCompositionContent<H>>,
}

impl<H: UiHost, TTrigger> std::fmt::Debug for AlertDialogComposition<H, TTrigger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogComposition")
            .field("dialog", &self.dialog)
            .field("trigger", &self.trigger.is_some())
            .field("portal", &self.portal)
            .field("overlay", &self.overlay)
            .field("content", &self.content.is_some())
            .finish()
    }
}

impl<H: UiHost> AlertDialogComposition<H> {
    pub fn new(dialog: AlertDialog) -> Self {
        Self {
            dialog,
            trigger: None,
            portal: AlertDialogPortal::new(),
            overlay: AlertDialogOverlay::new(),
            content: None,
        }
    }
}

impl<H: UiHost, TTrigger> AlertDialogComposition<H, TTrigger> {
    pub fn trigger<TNextTrigger>(
        self,
        trigger: TNextTrigger,
    ) -> AlertDialogComposition<H, TNextTrigger> {
        AlertDialogComposition {
            dialog: self.dialog,
            trigger: Some(trigger),
            portal: self.portal,
            overlay: self.overlay,
            content: self.content,
        }
    }

    pub fn portal(mut self, portal: AlertDialogPortal) -> Self {
        self.portal = portal;
        self
    }

    pub fn overlay(mut self, overlay: AlertDialogOverlay) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(AlertDialogCompositionContent::Eager(content));
        self
    }

    pub fn content_with(
        mut self,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(AlertDialogCompositionContent::Deferred(Box::new(content)));
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TTrigger: AlertDialogCompositionTriggerArg<H>,
    {
        let trigger = match self.trigger {
            Some(trigger) => trigger.into_alert_dialog_trigger(cx),
            None if self.dialog.handle.is_some() => {
                AlertDialogTrigger::new(cx.container(ContainerProps::default(), |_cx| Vec::new()))
            }
            None => {
                panic!("AlertDialog::compose().trigger(...) must be provided before into_element()")
            }
        };
        let content = self
            .content
            .expect("AlertDialog::compose().content(...) must be provided before into_element()");

        let portal = self.portal;
        let overlay = self.overlay;

        match content {
            AlertDialogCompositionContent::Eager(content) => self.dialog.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |_cx| content,
            ),
            AlertDialogCompositionContent::Deferred(content) => self.dialog.into_element_parts(
                cx,
                move |_cx| trigger,
                portal,
                overlay,
                move |cx| content(cx),
            ),
        }
    }
}

/// shadcn/ui `AlertDialogTrigger` (v4).
#[derive(Debug)]
pub struct AlertDialogTrigger {
    child: AnyElement,
    handle: Option<AlertDialogHandle>,
    open_model: Option<Model<bool>>,
}

pub struct AlertDialogTriggerBuild<H, T> {
    child: Option<T>,
    _phantom: PhantomData<fn() -> H>,
}

impl AlertDialogTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            handle: None,
            open_model: None,
        }
    }

    pub fn handle(mut self, handle: AlertDialogHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    fn with_open_model(mut self, open_model: Model<bool>) -> Self {
        self.open_model = Some(open_model);
        self
    }

    /// Builder-first variant that late-lands the trigger child at `into_element(cx)` time.
    pub fn build<H: UiHost, T>(child: T) -> AlertDialogTriggerBuild<H, T>
    where
        T: UiChildIntoElement<H>,
    {
        AlertDialogTriggerBuild {
            child: Some(child),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut child = self.child;
        let child_id = child.id;

        if let Some(handle) = self.handle {
            let open = cx
                .watch_model(&handle.open)
                .paint()
                .copied()
                .unwrap_or(false);
            let content_element = cx
                .watch_model(&handle.content_element)
                .paint()
                .copied()
                .unwrap_or(None);
            let open_model = handle.open.clone();
            let active_trigger = handle.active_trigger.clone();
            cx.pressable_add_on_activate_for(
                child_id,
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          acx: fret_ui::action::ActionCx,
                          _reason: fret_ui::action::ActivateReason| {
                        let _ = host.models_mut().update(&open_model, |value| *value = true);
                        let _ = host
                            .models_mut()
                            .update(&active_trigger, |value| *value = Some(child_id));
                        host.request_redraw(acx.window);
                    },
                ),
            );
            child =
                radix_alert_dialog::apply_alert_dialog_trigger_a11y(child, open, content_element);
        } else if let Some(open_model) = self.open_model {
            cx.pressable_add_on_activate_for(
                child_id,
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          acx: fret_ui::action::ActionCx,
                          _reason: fret_ui::action::ActivateReason| {
                        let _ = host.models_mut().update(&open_model, |value| *value = true);
                        host.request_redraw(acx.window);
                    },
                ),
            );
        }

        child
    }
}

impl<H: UiHost, T> AlertDialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogTrigger::new(
            self.child
                .expect("expected alert dialog trigger child")
                .into_child_element(cx),
        )
        .into_element(cx)
    }
}

impl<H: UiHost, T> IntoUiElement<H> for AlertDialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogTriggerBuild::into_element(self, cx)
    }
}

#[doc(hidden)]
pub trait AlertDialogCompositionTriggerArg<H: UiHost> {
    fn into_alert_dialog_trigger(self, cx: &mut ElementContext<'_, H>) -> AlertDialogTrigger;
}

impl<H: UiHost> AlertDialogCompositionTriggerArg<H> for AlertDialogTrigger {
    fn into_alert_dialog_trigger(self, _cx: &mut ElementContext<'_, H>) -> AlertDialogTrigger {
        self
    }
}

impl<H: UiHost, T> AlertDialogCompositionTriggerArg<H> for AlertDialogTriggerBuild<H, T>
where
    T: UiChildIntoElement<H>,
{
    fn into_alert_dialog_trigger(self, cx: &mut ElementContext<'_, H>) -> AlertDialogTrigger {
        AlertDialogTrigger::new(
            self.child
                .expect("expected alert dialog trigger child")
                .into_child_element(cx),
        )
    }
}

fn collect_built_alert_dialog_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

/// shadcn/ui `AlertDialogContent` (v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertDialogContentSize {
    #[default]
    Default,
    Sm,
}

#[derive(Debug)]
pub struct AlertDialogContent {
    children: Vec<AnyElement>,
    size: AlertDialogContentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AlertDialogContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: AlertDialogContentSize::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> AlertDialogContentBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertDialogContentBuild {
            build: Some(build),
            size: AlertDialogContentSize::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            test_id: None,
            _phantom: PhantomData,
        }
    }

    pub fn size(mut self, size: AlertDialogContentSize) -> Self {
        self.size = size;
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

        let bg = theme.color_token("background");
        let border = theme.color_token("border");

        let radius = theme.metric_token("metric.radius.lg");
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
            .max_w(alert_dialog_content_default_max_width(self.size))
            .min_w_0()
            .min_h_0()
            .merge(self.layout);

        register_alert_dialog_content_max_width_hint(cx, &theme, &layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        let container = shadcn_layout::container_vstack(
            cx,
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            shadcn_layout::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0())
                .items_stretch(),
            children,
        );

        let (labelled_by_element, described_by_element) =
            crate::a11y_modal::modal_relations_for_current_scope(cx.app);

        container.attach_semantics(SemanticsDecoration {
            role: Some(SemanticsRole::AlertDialog),
            labelled_by_element,
            described_by_element,
            ..Default::default()
        })
    }
}

pub struct AlertDialogContentBuild<H, B> {
    build: Option<B>,
    size: AlertDialogContentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertDialogContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: AlertDialogContentSize) -> Self {
        self.size = size;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let layout = LayoutRefinement::default()
            .w_full()
            .max_w(alert_dialog_content_default_max_width(self.size))
            .min_w_0()
            .min_h_0()
            .merge(self.layout.clone());
        register_alert_dialog_content_max_width_hint(cx, &theme, &layout);

        let content = AlertDialogContent::new(collect_built_alert_dialog_children(
            cx,
            self.build
                .expect("expected alert dialog content build closure"),
        ))
        .size(self.size)
        .refine_style(self.chrome)
        .refine_layout(self.layout)
        .into_element(cx);
        if let Some(id) = self.test_id {
            content.test_id(id)
        } else {
            content
        }
    }
}

fn alert_dialog_content_default_max_width(size: AlertDialogContentSize) -> Px {
    match size {
        AlertDialogContentSize::Default => Px(512.0),
        AlertDialogContentSize::Sm => Px(320.0),
    }
}

fn register_alert_dialog_content_max_width_hint<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    layout: &LayoutRefinement,
) {
    if let Some(max_w) = layout
        .size
        .as_ref()
        .and_then(|s| s.max_width.as_ref())
        .and_then(|m| match m {
            fret_ui_kit::LengthRefinement::Px(metric) => Some(metric.resolve(theme)),
            _ => None,
        })
    {
        crate::a11y_modal::register_modal_content_max_width(cx.app, max_w);
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertDialogContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for AlertDialogContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for AlertDialogContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for AlertDialogContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogContentBuild::into_element(self, cx)
    }
}

/// shadcn/ui `AlertDialogHeader` (v4).
#[derive(Debug)]
pub struct AlertDialogHeader {
    media: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl AlertDialogHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            media: None,
            children,
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> AlertDialogHeaderBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertDialogHeaderBuild {
            build: Some(build),
            media: None,
            _phantom: PhantomData,
        }
    }

    pub fn media(mut self, media: AnyElement) -> Self {
        self.media = Some(media);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let content_max_w = crate::a11y_modal::modal_content_max_width_for_current_scope(cx.app);
        let content_is_sm = content_max_w.is_some_and(|w| (w.0 - 320.0).abs() < 0.5 || w.0 < 320.0);

        let sm_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            fret_ui_kit::declarative::viewport_tailwind::SM,
            fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
        );

        // Upstream: `sm:group-data-[size=default]` switches from centered to left-aligned header.
        // We approximate this by inferring `size` from the committed content max-width.
        let left_aligned = sm_breakpoint && !content_is_sm;
        let text_align = if left_aligned {
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
            .map(|child| apply_alert_dialog_header_text_alignment(child, text_align))
            .collect();
        let Some(media) = self.media else {
            return shadcn_layout::container_vstack_fill_width(
                cx,
                props,
                shadcn_layout::VStackProps::default()
                    .gap(Space::N1p5)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items(if left_aligned {
                        fret_ui_kit::Items::Stretch
                    } else {
                        fret_ui_kit::Items::Center
                    }),
                children,
            );
        };

        if left_aligned {
            let text = ui::v_flex(move |_cx| children)
                .gap(Space::N1p5)
                .items_start()
                .layout(LayoutRefinement::default().flex_1().min_w_0())
                .into_element(cx);

            return cx.container(props, move |cx| {
                vec![
                    ui::h_flex(move |_cx| vec![media, text])
                        .gap(Space::N6)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                ]
            });
        }

        let text = ui::v_flex(move |_cx| children)
            .gap(Space::N1p5)
            .items_stretch()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        cx.container(props, move |cx| {
            vec![
                ui::v_flex(move |_cx| vec![media, text])
                    .gap(Space::N1p5)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
            ]
        })
    }
}

pub struct AlertDialogHeaderBuild<H, B> {
    build: Option<B>,
    media: Option<AnyElement>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertDialogHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn media(mut self, media: AnyElement) -> Self {
        self.media = Some(media);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut header = AlertDialogHeader::new(collect_built_alert_dialog_children(
            cx,
            self.build
                .expect("expected alert dialog header build closure"),
        ));
        if let Some(media) = self.media {
            header = header.media(media);
        }
        header.into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertDialogHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for AlertDialogHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogHeaderBuild::into_element(self, cx)
    }
}

/// shadcn/ui `AlertDialogMedia` (v4).
#[derive(Debug)]
pub struct AlertDialogMedia {
    child: AnyElement,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AlertDialogMedia {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
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
        let theme = Theme::global(&*cx.app);
        let bg = theme
            .color_by_key("muted")
            .unwrap_or_else(|| theme.color_token("muted"));

        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(bg))
                .rounded(Radius::Md)
                .merge(self.chrome),
            LayoutRefinement::default()
                .w_px(Px(64.0))
                .h_px(Px(64.0))
                .flex_shrink_0()
                .merge(self.layout),
        );

        shadcn_layout::container_hstack(
            cx,
            props,
            shadcn_layout::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .justify_center()
                .items_center(),
            vec![self.child],
        )
    }
}

/// shadcn/ui `AlertDialogFooter` (v4).
#[derive(Debug)]
pub struct AlertDialogFooter {
    children: Vec<AnyElement>,
}

impl AlertDialogFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn build<H: UiHost, B>(build: B) -> AlertDialogFooterBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertDialogFooterBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let content_max_w = crate::a11y_modal::modal_content_max_width_for_current_scope(cx.app);
        let content_is_sm = content_max_w.is_some_and(|w| (w.0 - 320.0).abs() < 0.5 || w.0 < 320.0);

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

        let mut children = children;
        if content_is_sm {
            // Tailwind (size=sm): `grid grid-cols-2 gap-2`
            // This size-specific layout wins over viewport row stacking; otherwise desktop `sm:`
            // would incorrectly collapse back to an intrinsic-width flex row.
            let children: Vec<AnyElement> = children
                .into_iter()
                .map(|child| {
                    let child = apply_alert_dialog_footer_fill_width(child);
                    ui::v_flex(move |_cx| vec![child])
                        .items_stretch()
                        .layout(LayoutRefinement::default().w_full().min_w_0().flex_1())
                        .into_element(cx)
                })
                .collect();

            cx.container(props, move |cx| {
                vec![
                    ui::h_flex(move |_cx| children)
                        .gap(Space::N2)
                        .items_stretch()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                ]
            })
        } else if sm_breakpoint {
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

pub struct AlertDialogFooterBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertDialogFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogFooter::new(collect_built_alert_dialog_children(
            cx,
            self.build
                .expect("expected alert dialog footer build closure"),
        ))
        .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertDialogFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for AlertDialogFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDialogFooterBuild::into_element(self, cx)
    }
}

/// shadcn/ui `AlertDialogTitle` (v4).
fn patch_alert_dialog_text_style_recursive(
    el: &mut AnyElement,
    px: Px,
    line_height: Px,
    weight: FontWeight,
    color: Color,
    letter_spacing_em: f32,
) {
    fn patch_text_style(
        style: &mut Option<fret_core::TextStyle>,
        px: Px,
        line_height: Px,
        weight: FontWeight,
        letter_spacing_em: f32,
    ) {
        let mut style_value = style.take().unwrap_or_default();
        style_value.size = px;
        style_value.weight = weight;
        style_value.line_height = Some(line_height);
        style_value.line_height_em = None;
        style_value.letter_spacing_em = Some(letter_spacing_em);
        *style = Some(style_value);
    }

    match &mut el.kind {
        ElementKind::Text(props) => {
            patch_text_style(&mut props.style, px, line_height, weight, letter_spacing_em);
            props.color = Some(color);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::StyledText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight, letter_spacing_em);
            props.color = Some(color);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::SelectableText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight, letter_spacing_em);
            props.color = Some(color);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        _ => {}
    }

    for child in &mut el.children {
        patch_alert_dialog_text_style_recursive(
            child,
            px,
            line_height,
            weight,
            color,
            letter_spacing_em,
        );
    }
}

fn patch_alert_dialog_title_text_style_recursive(
    el: &mut AnyElement,
    px: Px,
    line_height: Px,
    color: Color,
) {
    patch_alert_dialog_text_style_recursive(
        el,
        px,
        line_height,
        FontWeight::SEMIBOLD,
        color,
        -0.02,
    );
}

fn apply_alert_dialog_header_text_alignment(
    mut element: AnyElement,
    align: TextAlign,
) -> AnyElement {
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
        .map(|child| apply_alert_dialog_header_text_alignment(child, align))
        .collect();
    element
}

fn apply_alert_dialog_footer_fill_width(mut element: AnyElement) -> AnyElement {
    let apply_layout = |layout: &mut LayoutStyle| {
        if matches!(layout.size.width, Length::Auto) {
            layout.size.width = Length::Fill;
        }
        if layout.size.min_width.is_none() {
            layout.size.min_width = Some(Length::Px(Px(0.0)));
        }
        if layout.flex.grow == 0.0 {
            layout.flex.grow = 1.0;
        }
        if layout.flex.shrink == 0.0 {
            layout.flex.shrink = 1.0;
        }
        if matches!(layout.flex.basis, Length::Auto) {
            layout.flex.basis = Length::Px(Px(0.0));
        }
    };

    match &mut element.kind {
        ElementKind::Container(props) => apply_layout(&mut props.layout),
        ElementKind::SemanticFlex(props) => apply_layout(&mut props.flex.layout),
        ElementKind::Pressable(props) => apply_layout(&mut props.layout),
        ElementKind::Flex(props) => apply_layout(&mut props.layout),
        ElementKind::Row(props) => apply_layout(&mut props.layout),
        ElementKind::Column(props) => apply_layout(&mut props.layout),
        ElementKind::Stack(props) => apply_layout(&mut props.layout),
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(apply_alert_dialog_footer_fill_width)
        .collect();
    element
}

#[derive(Debug)]
pub struct AlertDialogTitle {
    content: AlertDialogTitleContent,
}

#[derive(Debug)]
enum AlertDialogTitleContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertDialogTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertDialogTitleContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertDialogTitleContent::Children(children.into_iter().collect()),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));

        let px = theme
            .metric_by_key("component.alert_dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.alert_dialog.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let title = match self.content {
            AlertDialogTitleContent::Text(text) => ui::text(text)
                .text_size_px(px)
                .line_height_px(line_height)
                .font_weight(FontWeight::SEMIBOLD)
                .letter_spacing_em(-0.02)
                .text_color(ColorRef::Color(fg))
                .wrap(TextWrap::Word)
                .overflow(TextOverflow::Clip)
                .into_element(cx),
            AlertDialogTitleContent::Children(mut children) => {
                for child in &mut children {
                    patch_alert_dialog_title_text_style_recursive(child, px, line_height, fg);
                }

                match children.len() {
                    0 => ui::text("")
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::SEMIBOLD)
                        .letter_spacing_em(-0.02)
                        .text_color(ColorRef::Color(fg))
                        .wrap(TextWrap::Word)
                        .overflow(TextOverflow::Clip)
                        .into_element(cx),
                    1 => children.pop().expect("children.len() == 1"),
                    _ => ui::v_flex(move |_cx| children)
                        .gap(Space::N0)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                }
            }
        }
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Heading)
                .level(2),
        );
        crate::a11y_modal::register_modal_title(cx.app, title.id);
        title
    }
}

/// shadcn/ui `AlertDialogDescription` (v4).
#[derive(Debug)]
pub struct AlertDialogDescription {
    content: AlertDialogDescriptionContent,
}

#[derive(Debug)]
enum AlertDialogDescriptionContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertDialogDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertDialogDescriptionContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertDialogDescriptionContent::Children(children.into_iter().collect()),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let description = match self.content {
            AlertDialogDescriptionContent::Text(text) => scope_description_text(
                ui::raw_text(text)
                    .wrap(TextWrap::Word)
                    .overflow(TextOverflow::Clip)
                    .into_element(cx),
                &theme,
                "component.alert_dialog.description",
            ),
            AlertDialogDescriptionContent::Children(children) => scope_description_text(
                ui::v_flex(move |_cx| children)
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                &theme,
                "component.alert_dialog.description",
            ),
        };
        crate::a11y_modal::register_modal_description(cx.app, description.id);
        description
    }
}

/// shadcn/ui `AlertDialogAction` (v4).
///
/// This is a convenience wrapper that closes the dialog on click.
pub struct AlertDialogAction {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    open: Option<Model<bool>>,
    variant: ButtonVariant,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for AlertDialogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogAction")
            .field("label", &self.label)
            .field("a11y_label", &self.a11y_label)
            .field("children_len", &self.children.len())
            .field("open", &"<model>")
            .field("variant", &self.variant)
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl AlertDialogAction {
    /// Creates an action button that explicitly toggles the provided alert dialog open model.
    ///
    /// Prefer this constructor when you want fully explicit data flow or when the action is built
    /// outside the alert dialog content subtree.
    pub fn new(label: impl Into<Arc<str>>, open: Model<bool>) -> Self {
        Self {
            label: label.into(),
            a11y_label: None,
            children: Vec::new(),
            open: Some(open),
            variant: ButtonVariant::Default,
            disabled: false,
            test_id: None,
        }
    }

    /// Creates an action button that closes the alert dialog resolved from the current content
    /// scope.
    ///
    /// This is an authoring convenience for shadcn-style composition inside
    /// [`AlertDialog::into_element`] / [`AlertDialog::into_element_parts`] content closures. It is
    /// intentionally recipe-layer sugar: explicit `new(label, open)` remains available and should
    /// be preferred when the button is created outside the alert dialog content subtree.
    ///
    /// Panics if no alert dialog content scope is active when the element is rendered.
    pub fn from_scope(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            a11y_label: None,
            children: Vec::new(),
            open: None,
            variant: ButtonVariant::Default,
            disabled: false,
            test_id: None,
        }
    }

    /// Replaces the default text label with custom button contents.
    ///
    /// The semantic label still defaults to the string passed to `new(...)` / `from_scope(...)`.
    /// Use `a11y_label(...)` when the visual children no longer describe the action clearly.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Overrides the semantic label used for accessibility and automation.
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Sets a `test_id` for deterministic automation (diagnostics/testing hook).
    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open.unwrap_or_else(|| {
            alert_dialog_open_for_current_scope(cx.app).unwrap_or_else(|| {
                panic!(
                    "AlertDialogAction::from_scope(...) must be used while rendering AlertDialog content"
                )
            })
        });
        let mut button = Button::new(self.label)
            .variant(self.variant)
            .disabled(self.disabled)
            .toggle_model(open)
            .children(self.children);
        if let Some(a11y_label) = self.a11y_label {
            button = button.a11y_label(a11y_label);
        }
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        button.into_element(cx)
    }
}

/// shadcn/ui `AlertDialogCancel` (v4).
///
/// This is a convenience wrapper that closes the dialog on click.
pub struct AlertDialogCancel {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    open: Option<Model<bool>>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for AlertDialogCancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogCancel")
            .field("label", &self.label)
            .field("a11y_label", &self.a11y_label)
            .field("children_len", &self.children.len())
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl AlertDialogCancel {
    /// Creates a cancel button that explicitly toggles the provided alert dialog open model.
    ///
    /// Prefer this constructor when you want fully explicit data flow or when the cancel button is
    /// built outside the alert dialog content subtree.
    pub fn new(label: impl Into<Arc<str>>, open: Model<bool>) -> Self {
        Self {
            label: label.into(),
            a11y_label: None,
            children: Vec::new(),
            open: Some(open),
            disabled: false,
            test_id: None,
        }
    }

    /// Creates a cancel button that closes the alert dialog resolved from the current content
    /// scope.
    ///
    /// This is an authoring convenience for shadcn-style composition inside
    /// [`AlertDialog::into_element`] / [`AlertDialog::into_element_parts`] content closures. It is
    /// intentionally recipe-layer sugar: explicit `new(label, open)` remains available and should
    /// be preferred when the button is created outside the alert dialog content subtree.
    ///
    /// Panics if no alert dialog content scope is active when the element is rendered.
    pub fn from_scope(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            a11y_label: None,
            children: Vec::new(),
            open: None,
            disabled: false,
            test_id: None,
        }
    }

    /// Replaces the default text label with custom button contents.
    ///
    /// The semantic label still defaults to the string passed to `new(...)` / `from_scope(...)`.
    /// Use `a11y_label(...)` when the visual children no longer describe the cancellation action clearly.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Overrides the semantic label used for accessibility and automation.
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Sets a `test_id` for deterministic automation (diagnostics/testing hook).
    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open.unwrap_or_else(|| {
            alert_dialog_open_for_current_scope(cx.app).unwrap_or_else(|| {
                panic!(
                    "AlertDialogCancel::from_scope(...) must be used while rendering AlertDialog content"
                )
            })
        });
        let open_id = open.id();
        let mut button = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .disabled(self.disabled)
            .toggle_model(open)
            .children(self.children);
        if let Some(a11y_label) = self.a11y_label {
            button = button.a11y_label(a11y_label);
        }
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        let element = button.into_element(cx);

        radix_alert_dialog::register_cancel_for_open_model(cx, open_id, element.id);

        element
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
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::element::PressableProps;
    use fret_ui::elements::bounds_for_element;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    fn find_first_styled_text(el: &AnyElement) -> Option<&fret_ui::element::StyledTextProps> {
        if let ElementKind::StyledText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_styled_text)
    }

    fn find_first_selectable_text(
        el: &AnyElement,
    ) -> Option<&fret_ui::element::SelectableTextProps> {
        if let ElementKind::SelectableText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_selectable_text)
    }

    fn contains_plain_text(el: &AnyElement, needle: &str) -> bool {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => true,
            _ => el
                .children
                .iter()
                .any(|child| contains_plain_text(child, needle)),
        }
    }

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

    #[test]
    fn alert_dialog_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = AlertDialogOpenChangeCallbackState::default();

        let (changed, completed) = alert_dialog_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = alert_dialog_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = alert_dialog_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn alert_dialog_open_change_events_complete_without_animation() {
        let mut state = AlertDialogOpenChangeCallbackState::default();

        let _ = alert_dialog_open_change_events(&mut state, false, false, false);
        let (changed, completed) = alert_dialog_open_change_events(&mut state, true, true, false);

        assert_eq!(changed, Some(true));
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn alert_dialog_title_children_patch_rich_text_with_title_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = fret_core::AttributedText::new(
                Arc::<str>::from("Delete project and revoke shared access?"),
                Arc::<[fret_core::TextSpan]>::from([fret_core::TextSpan::new(
                    "Delete project and revoke shared access?".len(),
                )]),
            );

            AlertDialogTitle::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let ElementKind::StyledText(props) = &element.kind else {
            panic!(
                "expected AlertDialogTitle children root to stay a StyledText, got {:?}",
                element.kind
            );
        };

        let style = props
            .style
            .as_ref()
            .expect("expected AlertDialogTitle children to receive explicit title text style");
        let theme = Theme::global(&app).snapshot();
        let expected_px = theme
            .metric_by_key("component.alert_dialog.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.alert_dialog.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));
        let expected_fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));

        assert_eq!(style.size, expected_px);
        assert_eq!(style.weight, fret_core::FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(style.letter_spacing_em, Some(-0.02));
        assert_eq!(props.color, Some(expected_fg));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_dialog_description_children_scope_rich_text_with_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = fret_core::AttributedText::new(
                Arc::<str>::from("Export an audit archive before deleting the project."),
                Arc::<[fret_core::TextSpan]>::from([fret_core::TextSpan::new(
                    "Export an audit archive before deleting the project.".len(),
                )]),
            );

            AlertDialogDescription::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let props = find_first_styled_text(&element)
            .expect("expected AlertDialogDescription children to keep the rich text node");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert_dialog.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_dialog_description_children_preserve_interactive_spans_under_description_scope() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = fret_core::AttributedText::new(
                Arc::<str>::from("Review the retention policy"),
                Arc::<[fret_core::TextSpan]>::from([fret_core::TextSpan::new(
                    "Review the retention policy".len(),
                )]),
            );

            let mut props = fret_ui::element::SelectableTextProps::new(rich);
            props.interactive_spans =
                Arc::from([fret_ui::element::SelectableTextInteractiveSpan {
                    range: 0.."Review the retention policy".len(),
                    tag: Arc::<str>::from("retention-policy"),
                }]);

            AlertDialogDescription::new_children([cx.selectable_text_props(props)]).into_element(cx)
        });

        let props = find_first_selectable_text(&element)
            .expect("expected AlertDialogDescription children to keep selectable text nodes");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(props.interactive_spans.len(), 1);
        assert_eq!(props.interactive_spans[0].tag.as_ref(), "retention-policy");
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert_dialog.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_dialog_action_children_override_visual_label_but_keep_semantic_label() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );
        let open = app.models_mut().insert(false);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let visual = ui::h_row(|cx| {
                vec![
                    crate::icon::icon(cx, fret_icons::IconId::new_static("lucide.trash-2")),
                    ui::text("Delete now").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            AlertDialogAction::new("Delete project", open.clone())
                .children([visual])
                .test_id("alert-dialog-action-custom")
                .into_element(cx)
        });

        let ElementKind::Pressable(props) = &element.kind else {
            panic!("expected AlertDialogAction to render a Pressable root");
        };

        assert_eq!(
            props.a11y.test_id.as_deref(),
            Some("alert-dialog-action-custom")
        );
        assert_eq!(props.a11y.label.as_deref(), Some("Delete project"));
        assert!(contains_plain_text(&element, "Delete now"));
        assert!(!contains_plain_text(&element, "Delete project"));
    }

    #[test]
    fn alert_dialog_cancel_children_can_override_a11y_label() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );
        let open = app.models_mut().insert(false);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let visual = ui::h_row(|cx| {
                vec![
                    crate::icon::icon(cx, fret_icons::IconId::new_static("lucide.arrow-left")),
                    ui::text("Back to safety").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            AlertDialogCancel::new("Cancel", open.clone())
                .children([visual])
                .a11y_label("Cancel deletion")
                .test_id("alert-dialog-cancel-custom")
                .into_element(cx)
        });

        let ElementKind::Pressable(props) = &element.kind else {
            panic!("expected AlertDialogCancel to render a Pressable root");
        };

        assert_eq!(
            props.a11y.test_id.as_deref(),
            Some("alert-dialog-cancel-custom")
        );
        assert_eq!(props.a11y.label.as_deref(), Some("Cancel deletion"));
        assert!(contains_plain_text(&element, "Back to safety"));
        assert!(!contains_plain_text(&element, "Cancel"));
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
    fn alert_dialog_into_element_parts_trigger_opens_on_activate() {
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
            "shadcn-alert-dialog-into-element-parts-trigger-opens",
            |cx| {
                vec![AlertDialog::new(open.clone()).into_element_parts(
                    cx,
                    |cx| AlertDialogTrigger::new(crate::Button::new("Open").into_element(cx)),
                    AlertDialogPortal::new(),
                    AlertDialogOverlay::new(),
                    |cx| AlertDialogContent::new([cx.text("Content")]).into_element(cx),
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
    fn alert_dialog_compose_content_with_supports_from_scope_buttons() {
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
            "shadcn-alert-dialog-compose-content-with-from-scope",
            |cx| {
                let trigger = AlertDialogTrigger::new(crate::Button::new("Open").into_element(cx));

                vec![
                    AlertDialog::new(open.clone())
                        .compose()
                        .trigger(trigger)
                        .portal(AlertDialogPortal::new())
                        .overlay(AlertDialogOverlay::new())
                        .content_with(|cx| {
                            let footer = AlertDialogFooter::new(vec![
                                AlertDialogCancel::from_scope("Cancel").into_element(cx),
                                AlertDialogAction::from_scope("Continue").into_element(cx),
                            ])
                            .into_element(cx);

                            AlertDialogContent::new(vec![footer]).into_element(cx)
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

    fn render_alert_dialog_frame_with_footer(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        content_size: AlertDialogContentSize,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        action_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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
                let open_for_action = open.clone();

                let alert = AlertDialog::new(open_for_dialog).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
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

                        let action = cx.pressable_with_id(
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
                                cx.pressable_set_bool(&open_for_action, false);
                                action_id_out.set(Some(id));
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let footer = AlertDialogFooter::new(vec![cancel, action]).into_element(cx);
                        AlertDialogContent::new(vec![footer])
                            .size(content_size)
                            .into_element(cx)
                    },
                );

                vec![alert]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_alert_dialog_frame_with_real_content(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        content_size: AlertDialogContentSize,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        description_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        action_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) {
        OverlayController::begin_frame(app, window);

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
                    |_cx, _st, _id| Vec::new(),
                );

                let open_for_dialog = open.clone();
                let open_for_cancel = open.clone();
                let open_for_action = open.clone();
                let content_id_out = content_id_out.clone();
                let description_id_out = description_id_out.clone();
                let cancel_id_out = cancel_id_out.clone();
                let action_id_out = action_id_out.clone();

                let alert = AlertDialog::new(open_for_dialog).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let content = AlertDialogContent::build(move |cx, children| {
                            let title = AlertDialogTitle::new("Delete the production project?")
                                .into_element(cx);
                            let description = AlertDialogDescription::new(
                                "This dialog is mounted separately from its triggers. Closing restores focus to whichever detached trigger opened it most recently, and the content should wrap within the alert dialog panel instead of measuring against the window width.",
                            )
                            .into_element(cx);
                            description_id_out.set(Some(description.id));
                            children.push(
                                AlertDialogHeader::new(vec![title, description]).into_element(cx),
                            );

                            let cancel = AlertDialogCancel::new("Cancel", open_for_cancel.clone())
                                .into_element(cx);
                            cancel_id_out.set(Some(cancel.id));

                            let action = AlertDialogAction::new("Delete", open_for_action.clone())
                                .variant(ButtonVariant::Destructive)
                                .into_element(cx);
                            action_id_out.set(Some(action.id));

                            children.push(
                                AlertDialogFooter::new(vec![cancel, action]).into_element(cx),
                            );
                        })
                        .size(content_size)
                        .into_element(cx);
                        content_id_out.set(Some(content.id));
                        content
                    },
                );

                vec![alert]
            },
        );

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_alert_dialog_frame_with_media_content(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        content_size: AlertDialogContentSize,
        title_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        description_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        action_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) {
        OverlayController::begin_frame(app, window);

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "alert-dialog-media-test",
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
                    |_cx, _st, _id| Vec::new(),
                );

                let open_for_dialog = open.clone();
                let open_for_cancel = open.clone();
                let open_for_action = open.clone();
                let title_id_out = title_id_out.clone();
                let description_id_out = description_id_out.clone();
                let content_id_out = content_id_out.clone();
                let cancel_id_out = cancel_id_out.clone();
                let action_id_out = action_id_out.clone();

                let alert = AlertDialog::new(open_for_dialog).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let content = AlertDialogContent::build(move |cx, children| {
                            let title = AlertDialogTitle::new("Delete chat?").into_element(cx);
                            title_id_out.set(Some(title.id));

                            let description = AlertDialogDescription::new(
                                "This will permanently delete this chat conversation. Review settings if you need to clear related memories.",
                            )
                            .into_element(cx);
                            description_id_out.set(Some(description.id));

                            let media = AlertDialogMedia::new(ui::text("!").into_element(cx))
                                .into_element(cx);

                            children.push(
                                AlertDialogHeader::new(vec![title, description])
                                    .media(media)
                                    .into_element(cx),
                            );

                            let cancel = AlertDialogCancel::new("Cancel", open_for_cancel.clone())
                                .into_element(cx);
                            cancel_id_out.set(Some(cancel.id));

                            let action = AlertDialogAction::new("Delete", open_for_action.clone())
                                .variant(ButtonVariant::Destructive)
                                .into_element(cx);
                            action_id_out.set(Some(action.id));

                            children.push(
                                AlertDialogFooter::new(vec![cancel, action]).into_element(cx),
                            );
                        })
                        .size(content_size)
                        .into_element(cx);
                        content_id_out.set(Some(content.id));
                        content
                    },
                );

                vec![alert]
            },
        );

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn alert_dialog_footer_stacks_on_base_viewport_and_rows_on_sm() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));

        let mut services = FakeServices;

        // Base viewport: vertical stack (col-reverse => action above cancel).
        let base_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(600.0)),
        );
        // Viewport queries read the committed per-window environment snapshot, so render two
        // frames to allow the width to commit before asserting layout.
        for frame in 1..=2 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_footer(
                &mut ui,
                &mut app,
                &mut services,
                window,
                base_bounds,
                open.clone(),
                AlertDialogContentSize::Default,
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, base_bounds, 1.0);
        }

        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");
        assert!(action_bounds.origin.y.0 < cancel_bounds.origin.y.0);

        // `sm:` viewport: horizontal row (cancel left of action).
        let sm_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        for frame in 3..=4 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_footer(
                &mut ui,
                &mut app,
                &mut services,
                window,
                sm_bounds,
                open.clone(),
                AlertDialogContentSize::Default,
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, sm_bounds, 1.0);
        }

        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");
        assert!((cancel_bounds.origin.y.0 - action_bounds.origin.y.0).abs() < 0.5);
        assert!(cancel_bounds.origin.x.0 < action_bounds.origin.x.0);
    }

    #[test]
    fn alert_dialog_small_footer_keeps_two_equal_columns_on_sm_viewport() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id = Rc::new(Cell::new(None));
        let description_id = Rc::new(Cell::new(None));
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for frame in 1..=2 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_real_content(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                AlertDialogContentSize::Sm,
                content_id.clone(),
                description_id.clone(),
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");

        assert!(
            (cancel_bounds.origin.y.0 - action_bounds.origin.y.0).abs() < 0.5,
            "expected small footer actions to share one row, got cancel={cancel_bounds:?} action={action_bounds:?}"
        );
        assert!(
            (cancel_bounds.size.width.0 - action_bounds.size.width.0).abs() < 1.0,
            "expected small footer actions to keep equal-width columns, got cancel={cancel_bounds:?} action={action_bounds:?}"
        );
        assert!(cancel_bounds.origin.x.0 < action_bounds.origin.x.0);
    }

    #[test]
    fn alert_dialog_real_content_stays_within_panel_bounds_on_sm_viewport() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id = Rc::new(Cell::new(None));
        let description_id = Rc::new(Cell::new(None));
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for frame in 1..=2 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_real_content(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                AlertDialogContentSize::Default,
                content_id.clone(),
                description_id.clone(),
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let content_bounds = bounds_for_element(
            &mut app,
            window,
            content_id.get().expect("content element id"),
        )
        .expect("content bounds");
        let description_bounds = bounds_for_element(
            &mut app,
            window,
            description_id.get().expect("description element id"),
        )
        .expect("description bounds");
        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");

        let content_left = content_bounds.origin.x.0 - 0.5;
        let content_right = content_bounds.origin.x.0 + content_bounds.size.width.0 + 0.5;

        assert!(
            content_bounds.size.width.0 <= 512.5,
            "expected alert dialog content width to stay near shadcn's sm:max-w-lg, got {content_bounds:?}"
        );
        assert!(
            description_bounds.origin.x.0 >= content_left
                && description_bounds.origin.x.0 + description_bounds.size.width.0 <= content_right,
            "expected description to stay inside alert dialog content; content={content_bounds:?} description={description_bounds:?}"
        );
        assert!(
            cancel_bounds.origin.x.0 >= content_left
                && cancel_bounds.origin.x.0 + cancel_bounds.size.width.0 <= content_right,
            "expected cancel button to stay inside alert dialog content; content={content_bounds:?} cancel={cancel_bounds:?}"
        );
        assert!(
            action_bounds.origin.x.0 >= content_left
                && action_bounds.origin.x.0 + action_bounds.size.width.0 <= content_right,
            "expected action button to stay inside alert dialog content; content={content_bounds:?} action={action_bounds:?}"
        );
    }

    #[test]
    fn alert_dialog_panel_keeps_text_and_footer_within_compact_height() {
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
                "alert-dialog-compact-panel",
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

                    let open_for_children = open.clone();
                    let alert = AlertDialog::new(open.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            let header = AlertDialogHeader::new(vec![
                                AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                                AlertDialogDescription::new(
                                    "This action cannot be undone. This will permanently delete your account from our servers.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx);
                            let footer = AlertDialogFooter::new(vec![
                                AlertDialogCancel::new("Cancel", open_for_children.clone())
                                    .into_element(cx),
                                AlertDialogAction::new("Continue", open_for_children.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx);

                            AlertDialogContent::new(vec![header, footer]).into_element(cx)
                        },
                    );

                    vec![alert]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            trigger_id.expect("trigger id")
        };

        let _trigger = render_frame(&mut ui, &mut app, &mut services, 1);
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
                n.role == fret_core::SemanticsRole::Heading
                    && n.label.as_deref() == Some("Are you absolutely sure?")
            })
            .expect("title semantics node");
        let description = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Text
                    && n.label.as_deref()
                        == Some(
                            "This action cannot be undone. This will permanently delete your account from our servers.",
                        )
            })
            .expect("description semantics node");
        let cancel = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("Cancel")
            })
            .expect("cancel button semantics node");
        let action = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("Continue")
            })
            .expect("action button semantics node");

        let panel_left = alert_dialog.bounds.origin.x.0 - 0.5;
        let panel_right = alert_dialog.bounds.origin.x.0 + alert_dialog.bounds.size.width.0 + 0.5;
        let panel_top = alert_dialog.bounds.origin.y.0 - 0.5;
        let panel_bottom = alert_dialog.bounds.origin.y.0 + alert_dialog.bounds.size.height.0 + 0.5;

        assert!(
            alert_dialog.bounds.size.width.0 <= 512.5,
            "expected alert dialog width to stay near shadcn's sm:max-w-lg, got {:?}",
            alert_dialog.bounds
        );
        assert!(
            alert_dialog.bounds.size.height.0 <= 260.0,
            "expected alert dialog height to remain compact for header+footer content, got {:?}",
            alert_dialog.bounds
        );
        for (label, node) in [
            ("title", title),
            ("description", description),
            ("cancel", cancel),
            ("action", action),
        ] {
            assert!(
                node.bounds.origin.x.0 >= panel_left
                    && node.bounds.origin.x.0 + node.bounds.size.width.0 <= panel_right
                    && node.bounds.origin.y.0 >= panel_top
                    && node.bounds.origin.y.0 + node.bounds.size.height.0 <= panel_bottom,
                "expected {label} to stay inside alert dialog panel; panel={:?} node={:?}",
                alert_dialog.bounds,
                node.bounds
            );
        }
    }

    #[test]
    fn alert_dialog_media_panel_stays_compact_within_default_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let title_id = Rc::new(Cell::new(None));
        let description_id = Rc::new(Cell::new(None));
        let content_id = Rc::new(Cell::new(None));
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for frame in 1..=2 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_media_content(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                AlertDialogContentSize::Default,
                title_id.clone(),
                description_id.clone(),
                content_id.clone(),
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let content_bounds = bounds_for_element(
            &mut app,
            window,
            content_id.get().expect("content element id"),
        )
        .expect("content bounds");
        let title_bounds =
            bounds_for_element(&mut app, window, title_id.get().expect("title element id"))
                .expect("title bounds");
        let description_bounds = bounds_for_element(
            &mut app,
            window,
            description_id.get().expect("description element id"),
        )
        .expect("description bounds");
        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");

        let content_left = content_bounds.origin.x.0 - 0.5;
        let content_right = content_bounds.origin.x.0 + content_bounds.size.width.0 + 0.5;
        let content_bottom = content_bounds.origin.y.0 + content_bounds.size.height.0 + 0.5;

        assert!(
            content_bounds.size.width.0 <= 512.5,
            "expected media alert dialog content width to stay near the default max width, got {content_bounds:?}"
        );
        assert!(
            content_bounds.size.height.0 <= 260.0,
            "expected media alert dialog height to stay compact, got {content_bounds:?}"
        );
        for (label, bounds) in [
            ("title", title_bounds),
            ("description", description_bounds),
            ("cancel", cancel_bounds),
            ("action", action_bounds),
        ] {
            assert!(
                bounds.origin.x.0 >= content_left
                    && bounds.origin.x.0 + bounds.size.width.0 <= content_right
                    && bounds.origin.y.0 + bounds.size.height.0 <= content_bottom,
                "expected {label} to stay inside media alert dialog content; content={content_bounds:?} node={bounds:?}"
            );
        }
    }

    #[test]
    fn alert_dialog_small_media_panel_keeps_text_and_footer_within_compact_height() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let title_id = Rc::new(Cell::new(None));
        let description_id = Rc::new(Cell::new(None));
        let content_id = Rc::new(Cell::new(None));
        let cancel_id = Rc::new(Cell::new(None));
        let action_id = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for frame in 1..=2 {
            app.set_frame_id(FrameId(frame));
            render_alert_dialog_frame_with_media_content(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                AlertDialogContentSize::Sm,
                title_id.clone(),
                description_id.clone(),
                content_id.clone(),
                cancel_id.clone(),
                action_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let content_bounds = bounds_for_element(
            &mut app,
            window,
            content_id.get().expect("content element id"),
        )
        .expect("content bounds");
        let title_bounds =
            bounds_for_element(&mut app, window, title_id.get().expect("title element id"))
                .expect("title bounds");
        let description_bounds = bounds_for_element(
            &mut app,
            window,
            description_id.get().expect("description element id"),
        )
        .expect("description bounds");
        let cancel_bounds = bounds_for_element(
            &mut app,
            window,
            cancel_id.get().expect("cancel element id"),
        )
        .expect("cancel bounds");
        let action_bounds = bounds_for_element(
            &mut app,
            window,
            action_id.get().expect("action element id"),
        )
        .expect("action bounds");

        let content_left = content_bounds.origin.x.0 - 0.5;
        let content_right = content_bounds.origin.x.0 + content_bounds.size.width.0 + 0.5;
        let content_bottom = content_bounds.origin.y.0 + content_bounds.size.height.0 + 0.5;

        assert!(
            content_bounds.size.width.0 <= 320.5,
            "expected small media alert dialog content width to stay near the small max width, got {content_bounds:?}"
        );
        assert!(
            content_bounds.size.height.0 <= 320.0,
            "expected small media alert dialog height to stay compact, got {content_bounds:?}"
        );
        for (label, bounds) in [
            ("title", title_bounds),
            ("description", description_bounds),
            ("cancel", cancel_bounds),
            ("action", action_bounds),
        ] {
            assert!(
                bounds.origin.x.0 >= content_left
                    && bounds.origin.x.0 + bounds.size.width.0 <= content_right
                    && bounds.origin.y.0 + bounds.size.height.0 <= content_bottom,
                "expected {label} to stay inside small media alert dialog content; content={content_bounds:?} node={bounds:?}"
            );
        }
        assert!(
            (cancel_bounds.origin.y.0 - action_bounds.origin.y.0).abs() < 0.5,
            "expected small media footer buttons to share one row, got cancel={cancel_bounds:?} action={action_bounds:?}"
        );
        assert!(
            (cancel_bounds.size.width.0 - action_bounds.size.width.0).abs() < 1.0,
            "expected small media footer buttons to keep equal widths, got cancel={cancel_bounds:?} action={action_bounds:?}"
        );
    }

    fn render_alert_dialog_frame_with_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>>,
        cancel_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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

                let alert = AlertDialog::new(open.clone())
                    .on_open_auto_focus(on_open_auto_focus.clone())
                    .on_close_auto_focus(on_close_auto_focus.clone())
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
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
                                    cancel_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );
                            AlertDialogContent::new(vec![cancel]).into_element(cx)
                        },
                    );

                vec![underlay, alert]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_alert_dialog_frame_with_open_auto_focus_redirect_target(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
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
                let alert = AlertDialog::new(open.clone())
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

                            AlertDialogContent::new(vec![initial, redirect]).into_element(cx)
                        },
                    );

                vec![underlay, alert]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_alert_dialog_frame_with_underlay(
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
            "alert-dialog-underlay",
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

                let open_for_cancel = open.clone();
                let alert = AlertDialog::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let cancel = cx.pressable(
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
                            move |cx, _st| {
                                cx.pressable_set_bool(&open_for_cancel, false);
                                Vec::new()
                            },
                        );
                        AlertDialogContent::new(vec![cancel]).into_element(cx)
                    },
                );

                vec![underlay, alert]
            },
        );

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
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
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_100,
        ) + 1;
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
    fn alert_dialog_close_transition_keeps_modal_barrier_blocking_underlay() {
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
        render_alert_dialog_frame_with_underlay(
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
        render_alert_dialog_frame_with_underlay(
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
            "expected alert dialog to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> barrier must remain active.
        render_alert_dialog_frame_with_underlay(
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
            .expect("expected barrier root to remain while the alert dialog is closing");
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
            "underlay should remain inert while the alert dialog is closing"
        );

        // After the exit transition settles, the barrier must drop and the underlay becomes
        // interactive again.
        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_200,
        ) + 2;
        for _ in 0..settle_frames {
            render_alert_dialog_frame_with_underlay(
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
    fn alert_dialog_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let cancel_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

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
        let trigger = render_alert_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id.clone(),
            None,
            cancel_id.clone(),
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_alert_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id,
            None,
            cancel_id.clone(),
            Some(handler),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let cancel = cancel_id.get().expect("cancel element");
        let cancel_node =
            fret_ui::elements::node_for_element(&mut app, window, cancel).expect("cancel");
        assert_ne!(
            ui.focus(),
            Some(cancel_node),
            "expected preventDefault to suppress focusing the first focusable"
        );
        let focused = ui.focus().expect("expected focus to be set");
        assert_eq!(
            ui.node_layer(focused),
            ui.node_layer(cancel_node),
            "expected focus containment to keep focus within the alert dialog layer"
        );
    }

    #[test]
    fn alert_dialog_open_auto_focus_can_be_redirected() {
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

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_alert_dialog_frame_with_open_auto_focus_redirect_target(
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
        let _ = render_alert_dialog_frame_with_open_auto_focus_redirect_target(
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
    fn alert_dialog_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer() {
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

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_alert_dialog_frame_with_open_auto_focus_redirect_target(
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
        let _ = render_alert_dialog_frame_with_open_auto_focus_redirect_target(
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
            "expected focus containment to clamp focus within the alert dialog layer"
        );
    }

    #[test]
    fn alert_dialog_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let cancel_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
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

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let _trigger = render_alert_dialog_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_id_out.clone(),
            Some(underlay_id_cell.clone()),
            cancel_id.clone(),
            None,
            Some(handler.clone()),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let cancel = cancel_id.get().expect("cancel element");
        let cancel_node =
            fret_ui::elements::node_for_element(&mut app, window, cancel).expect("cancel");
        ui.set_focus(Some(cancel_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_200,
        ) as usize
            + 2;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(2 + i as u64));
            let _ = render_alert_dialog_frame_with_auto_focus_hooks(
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

        let underlay = underlay_id_out.get().expect("underlay element");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay");
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
                    let cancel_id_out = cancel_id.clone();

                    let alert = AlertDialog::new(open_for_dialog).into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let action = AlertDialogAction::from_scope("Delete").into_element(cx);
                            let cancel = AlertDialogCancel::from_scope("Cancel").into_element(cx);
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

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_100,
        ) + 1;
        for frame in 3..=(2 + settle_frames) {
            let _ = render_frame(&mut ui, &mut app, &mut services, frame);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    #[should_panic(
        expected = "AlertDialogCancel::from_scope(...) must be used while rendering AlertDialog content"
    )]
    fn alert_dialog_cancel_from_scope_panics_outside_alert_dialog_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let _ = AlertDialogCancel::from_scope("Cancel").into_element(cx);
        });
    }

    #[test]
    #[should_panic(
        expected = "AlertDialogAction::from_scope(...) must be used while rendering AlertDialog content"
    )]
    fn alert_dialog_action_from_scope_panics_outside_alert_dialog_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let _ = AlertDialogAction::from_scope("Continue").into_element(cx);
        });
    }

    #[test]
    fn alert_dialog_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertDialogDescription::new("Description").into_element(cx)
        });

        let fret_ui::element::ElementKind::Text(props) = &element.kind else {
            panic!("expected AlertDialogDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert_dialog.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn alert_dialog_handle_detached_trigger_restores_focus_to_last_activated_trigger() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let cancel_id = Rc::new(Cell::new(None::<fret_ui::elements::GlobalElementId>));
        let handle_open = Rc::new(std::cell::RefCell::new(None::<Model<bool>>));

        let render_frame = |ui: &mut UiTree<App>,
                            app: &mut App,
                            services: &mut dyn fret_core::UiServices,
                            frame: u64| {
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);

            let mut detached_trigger_id: Option<fret_ui::elements::GlobalElementId> = None;
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "alert-dialog-detached-trigger-handle",
                |cx| {
                    let handle = AlertDialogHandle::new_controllable(cx, None, false);
                    *handle_open.borrow_mut() = Some(handle.open_model());

                    let detached_trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(140.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            detached_trigger_id = Some(id);
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let detached_trigger = AlertDialogTrigger::new(detached_trigger)
                        .handle(handle.clone())
                        .into_element(cx);

                    let cancel_id_out = cancel_id.clone();
                    let dialog = AlertDialog::from_handle(handle)
                        .compose()
                        .content_with(move |cx| {
                            let action = AlertDialogAction::from_scope("Delete").into_element(cx);
                            let cancel = AlertDialogCancel::from_scope("Cancel").into_element(cx);
                            cancel_id_out.set(Some(cancel.id));
                            AlertDialogContent::new(vec![action, cancel]).into_element(cx)
                        })
                        .into_element(cx);

                    vec![detached_trigger, dialog]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            detached_trigger_id.expect("detached trigger id")
        };

        let detached_trigger = render_frame(&mut ui, &mut app, &mut services, 1);
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

        let open_model = handle_open
            .borrow()
            .as_ref()
            .cloned()
            .expect("handle open model");
        assert_eq!(app.models().get_copied(&open_model), Some(true));

        let _ = render_frame(&mut ui, &mut app, &mut services, 2);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let cancel = cancel_id.get().expect("cancel id");
        let cancel_node =
            fret_ui::elements::node_for_element(&mut app, window, cancel).expect("cancel node");
        assert_eq!(ui.focus(), Some(cancel_node));

        let detached_trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, detached_trigger)
                .expect("detached trigger node");
        let _ = app.models_mut().update(&open_model, |value| *value = false);

        let settle_frames = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            crate::overlay_motion::SHADCN_MOTION_DURATION_100,
        ) + 1;
        for frame in 3..=(2 + settle_frames) {
            let _ = render_frame(&mut ui, &mut app, &mut services, frame);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }
        assert_eq!(ui.focus(), Some(detached_trigger_node));
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
                n.role == fret_core::SemanticsRole::Heading
                    && n.label.as_deref() == Some("AlertDialog Title")
                    && n.extra.level == Some(2)
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
