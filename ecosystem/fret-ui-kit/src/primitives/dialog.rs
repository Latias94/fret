//! Dialog helpers (Radix `@radix-ui/react-dialog` outcomes).
//!
//! Upstream Dialog composes:
//! - conditional mounting (`@radix-ui/react-presence`)
//! - portal rendering (`@radix-ui/react-portal`)
//! - dismissal + focus management (`@radix-ui/react-dismissable-layer`, `@radix-ui/react-focus-scope`)
//! - modal scrolling + aria hiding (`react-remove-scroll`, `aria-hidden`)
//!
//! In Fret, these concerns map to:
//! - presence: `crate::OverlayPresence` (driven by motion helpers in recipe layers)
//! - portal + dismissal + focus restore/initial focus: per-window overlays (`crate::OverlayController`)
//! - focus traversal scoping: modal barrier layers in `fret-ui` (ADR 0068)
//!
//! This module is intentionally thin: it provides Radix-named entry points for trigger a11y and
//! modal overlay request wiring, without forcing a visual skin.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{
    DismissReason, DismissRequestCx, OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus,
};
use fret_ui::element::{
    AnyElement, ContainerProps, Elements, InsetStyle, LayoutStyle, Length, PositionStyle,
    PressableProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::primitives::trigger_a11y;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Policy for suppressing close auto-focus based on how a dialog overlay was dismissed.
///
/// This is primarily intended to prevent "focus stealing" when a close is triggered by a
/// click-through outside interaction (e.g. non-modal dialog variants, or regressions where the
/// modal barrier fails to block underlay focus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogCloseAutoFocusGuardPolicy {
    /// Prevent close auto-focus when dismissed via an outside press.
    pub prevent_on_outside_press: bool,
    /// Prevent close auto-focus when dismissed due to focus moving outside the dismissible layer.
    pub prevent_on_focus_outside: bool,
}

impl DialogCloseAutoFocusGuardPolicy {
    /// Default policy for Radix-style dialogs.
    ///
    /// - Modal dialogs are not click-through, so outside presses generally should not suppress
    ///   focus restoration.
    /// - Focus-outside dismissals represent a real focus transfer, so restoring focus back to the
    ///   trigger is usually undesirable.
    pub fn for_modal(modal: bool) -> Self {
        Self {
            prevent_on_outside_press: !modal,
            prevent_on_focus_outside: true,
        }
    }

    /// Always prevent close auto-focus.
    pub fn prevent_always() -> Self {
        Self {
            prevent_on_outside_press: true,
            prevent_on_focus_outside: true,
        }
    }
}

/// Wrap `on_dismiss_request` to preserve default close behavior and install a close auto-focus
/// guard that persists across frames.
///
/// Notes:
/// - The returned dismiss handler applies Radix-like defaults: it closes the overlay unless the
///   request is prevented.
/// - The returned close hook runs the caller hook (if any) and then applies the guard policy
///   unless the caller prevented default.
pub fn dialog_close_auto_focus_guard_hooks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    policy: DialogCloseAutoFocusGuardPolicy,
    open: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
) -> (Option<OnDismissRequest>, Option<OnCloseAutoFocus>) {
    #[derive(Default)]
    struct DialogCloseAutoFocusGuardState {
        dismiss_reason: Option<Model<Option<DismissReason>>>,
    }

    let should_install = policy.prevent_on_outside_press
        || policy.prevent_on_focus_outside
        || on_dismiss_request.is_some();
    let should_install_close = policy.prevent_on_outside_press
        || policy.prevent_on_focus_outside
        || on_close_auto_focus.is_some();

    if !should_install && !should_install_close {
        return (on_dismiss_request, on_close_auto_focus);
    }

    let dismiss_reason = cx
        .with_state(DialogCloseAutoFocusGuardState::default, |st| {
            st.dismiss_reason.clone()
        })
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(None);
            cx.with_state(DialogCloseAutoFocusGuardState::default, |st| {
                st.dismiss_reason = Some(model.clone());
            });
            model
        });

    // Clear stale reasons when the overlay is open again (new session).
    let open_now = cx.app.models().get_copied(&open).unwrap_or(false);
    if open_now {
        let _ = cx.app.models_mut().update(&dismiss_reason, |v| *v = None);
    }

    let dismiss_handler: Option<OnDismissRequest> = should_install.then(|| {
        let user_dismiss_request = on_dismiss_request.clone();
        let open_for_default_close = open.clone();
        let dismiss_reason_for_hook = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |host, cx, req| {
            if let Some(user) = user_dismiss_request.as_ref() {
                user(host, cx, req);
            }

            if !req.default_prevented() {
                let should_store = match req.reason {
                    DismissReason::OutsidePress { .. } => policy.prevent_on_outside_press,
                    DismissReason::FocusOutside => policy.prevent_on_focus_outside,
                    _ => false,
                };
                let _ = host.models_mut().update(&dismiss_reason_for_hook, |v| {
                    *v = should_store.then_some(req.reason);
                });
                let _ = host
                    .models_mut()
                    .update(&open_for_default_close, |v| *v = false);
            } else {
                let _ = host
                    .models_mut()
                    .update(&dismiss_reason_for_hook, |v| *v = None);
            }
        });
        handler
    });

    let on_close_auto_focus: Option<OnCloseAutoFocus> = should_install_close.then(|| {
        let dismiss_reason_for_close = dismiss_reason.clone();
        let user = on_close_auto_focus.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, cx, req| {
            if let Some(user) = user.as_ref() {
                user(host, cx, req);
            }

            let reason = host
                .models_mut()
                .read(&dismiss_reason_for_close, |v| *v)
                .ok()
                .flatten();
            let _ = host
                .models_mut()
                .update(&dismiss_reason_for_close, |v| *v = None);

            if req.default_prevented() {
                return;
            }

            let should_prevent = match reason {
                Some(DismissReason::OutsidePress { .. }) => policy.prevent_on_outside_press,
                Some(DismissReason::FocusOutside) => policy.prevent_on_focus_outside,
                _ => false,
            };
            if should_prevent {
                req.prevent_default();
            }
        });
        handler
    });

    (dismiss_handler.or(on_dismiss_request), on_close_auto_focus)
}

#[derive(Clone)]
pub struct DialogOptions {
    pub dismiss_on_overlay_press: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
}

impl std::fmt::Debug for DialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogOptions")
            .field("dismiss_on_overlay_press", &self.dismiss_on_overlay_press)
            .field("initial_focus", &self.initial_focus)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl Default for DialogOptions {
    fn default() -> Self {
        Self {
            dismiss_on_overlay_press: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }
}

impl DialogOptions {
    pub fn dismiss_on_overlay_press(mut self, dismiss_on_overlay_press: bool) -> Self {
        self.dismiss_on_overlay_press = dismiss_on_overlay_press;
        self
    }

    pub fn initial_focus(mut self, initial_focus: Option<GlobalElementId>) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }
}

/// Stable per-overlay root naming convention for dialog-like modal overlays.
pub fn dialog_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// This is a convenience helper for authoring Radix-shaped dialog roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn dialog_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// A Radix-shaped `Dialog` root configuration surface.
///
/// Upstream supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In Fret this
/// maps to either:
/// - a caller-provided `Model<bool>` (controlled), or
/// - an internal `Model<bool>` stored in element state (uncontrolled).
#[derive(Debug, Clone, Default)]
pub struct DialogRoot {
    open: Option<Model<bool>>,
    default_open: bool,
    options: DialogOptions,
}

impl DialogRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn dismiss_on_overlay_press(mut self, dismiss_on_overlay_press: bool) -> Self {
        self.options = self
            .options
            .dismiss_on_overlay_press(dismiss_on_overlay_press);
        self
    }

    pub fn initial_focus(mut self, initial_focus: Option<GlobalElementId>) -> Self {
        self.options = self.options.initial_focus(initial_focus);
        self
    }

    pub fn options(&self) -> DialogOptions {
        self.options.clone()
    }

    pub fn modal_request_with_dismiss_handler<H: UiHost, I>(
        &self,
        cx: &mut ElementContext<'_, H>,
        id: GlobalElementId,
        trigger: GlobalElementId,
        presence: OverlayPresence,
        on_dismiss_request: Option<OnDismissRequest>,
        children: I,
    ) -> OverlayRequest
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let children: Vec<AnyElement> = children.into_iter().collect();
        modal_dialog_request_with_options_and_dismiss_handler(
            id,
            trigger,
            self.open_model(cx),
            presence,
            self.options.clone(),
            on_dismiss_request,
            children,
        )
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        dialog_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    /// Reads the current open value from the derived open model.
    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }

    pub fn modal_request<H: UiHost, I>(
        &self,
        cx: &mut ElementContext<'_, H>,
        id: GlobalElementId,
        trigger: GlobalElementId,
        presence: OverlayPresence,
        children: I,
    ) -> OverlayRequest
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let children: Vec<AnyElement> = children.into_iter().collect();
        modal_dialog_request_with_options(
            id,
            trigger,
            self.open_model(cx),
            presence,
            self.options.clone(),
            children,
        )
    }
}

/// Stamps Radix-like trigger semantics:
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_dialog_trigger_a11y(
    trigger: AnyElement,
    expanded: bool,
    content_element: Option<GlobalElementId>,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls_expanded(trigger, Some(expanded), content_element)
}

/// Builds an overlay request for a Radix-style modal dialog.
pub fn modal_dialog_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    modal_dialog_request_with_options(
        id,
        trigger,
        open,
        presence,
        DialogOptions::default(),
        children.into_iter().collect::<Vec<_>>(),
    )
}

/// Builds an overlay request for a Radix-style modal dialog, with explicit options.
pub fn modal_dialog_request_with_options(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    options: DialogOptions,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let children: Vec<AnyElement> = children.into_iter().collect();
    let mut request = OverlayRequest::modal(id, Some(trigger), open, presence, children);
    request.root_name = Some(dialog_root_name(id));
    request.initial_focus = options.initial_focus;
    request.on_open_auto_focus = options.on_open_auto_focus.clone();
    request.on_close_auto_focus = options.on_close_auto_focus.clone();
    request
}

/// Builds an overlay request for a Radix-style modal dialog, with a custom dismiss handler.
///
/// This mirrors the Radix `DismissableLayer` contract: callers may "prevent default" by not
/// closing the `open` model in the handler.
pub fn modal_dialog_request_with_options_and_dismiss_handler(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    options: DialogOptions,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let mut request =
        modal_dialog_request_with_options(id, trigger, open, presence, options, children);
    request.dismissible_on_dismiss_request = on_dismiss_request;
    request
}

/// Standard full-window modal barrier layout (absolute inset 0, fill).
pub fn modal_barrier_layout() -> LayoutStyle {
    LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            top: Some(fret_core::Px(0.0)),
            right: Some(fret_core::Px(0.0)),
            bottom: Some(fret_core::Px(0.0)),
            left: Some(fret_core::Px(0.0)),
        },
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Builds a modal overlay barrier element that can optionally dismiss the given `open` model when
/// pressed.
///
/// The barrier is intentionally skin-agnostic: pass any background/visual elements as `children`.
pub fn modal_barrier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    modal_barrier_with_dismiss_handler(cx, open, dismiss_on_press, None, children)
}

/// Builds a modal overlay barrier element that can optionally route dismissals through a custom
/// dismiss handler.
///
/// When `on_dismiss_request` is provided and `dismiss_on_press` is enabled, barrier presses invoke
/// the handler with `DismissReason::OutsidePress` and do not close `open` automatically.
pub fn modal_barrier_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    let layout = modal_barrier_layout();
    let children: Vec<AnyElement> = children.into_iter().collect();

    if dismiss_on_press {
        cx.pressable(
            PressableProps {
                layout,
                enabled: true,
                focusable: false,
                ..Default::default()
            },
            move |cx, _st| {
                if let Some(on_dismiss_request) = on_dismiss_request.clone() {
                    let open_for_dismiss = open.clone();
                    cx.pressable_add_on_pointer_up(Arc::new(move |host, action_cx, up| {
                        let mut req = DismissRequestCx::new(DismissReason::OutsidePress {
                            pointer: Some(fret_ui::action::OutsidePressCx {
                                pointer_id: up.pointer_id,
                                pointer_type: up.pointer_type,
                                button: up.button,
                                modifiers: up.modifiers,
                                click_count: up.click_count,
                            }),
                        });
                        on_dismiss_request(host, action_cx, &mut req);
                        if !req.default_prevented() {
                            let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                        }
                        fret_ui::action::PressablePointerUpResult::SkipActivate
                    }));
                } else {
                    cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        fret_ui::action::PressablePointerUpResult::SkipActivate
                    }));
                }

                children
            },
        )
    } else {
        cx.container(
            ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

/// Convenience helper to assemble modal overlay children in a Radix-like order: barrier then
/// content.
pub fn modal_dialog_layer_elements<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    options: DialogOptions,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        modal_barrier(cx, open, options.dismiss_on_overlay_press, barrier_children),
        content,
    ])
}

/// Convenience helper to assemble modal overlay children in a Radix-like order (barrier then
/// content), while routing barrier presses through an optional dismiss handler.
pub fn modal_dialog_layer_elements_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    options: DialogOptions,
    on_dismiss_request: Option<OnDismissRequest>,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        modal_barrier_with_dismiss_handler(
            cx,
            open,
            options.dismiss_on_overlay_press,
            on_dismiss_request,
            barrier_children,
        ),
        content,
    ])
}

/// Requests a Radix-style modal dialog overlay for the current window.
pub fn request_modal_dialog<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_ui::action::DismissReason;
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::AppWindowId;
    use fret_core::Event;
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect, Size};
    use fret_core::{TextBlobId, TextConstraints, TextInput, TextMetrics, TextService};
    use fret_ui::UiTree;
    use fret_ui::element::{ContainerProps, ElementKind, LayoutStyle, Length, PressableProps};
    use fret_ui::elements::GlobalElementId;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
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
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn dialog_root_open_model_uses_controlled_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = DialogRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn dialog_root_options_builder_updates_options() {
        let root = DialogRoot::new()
            .dismiss_on_overlay_press(false)
            .initial_focus(Some(GlobalElementId(0xbeef)));
        let options = root.options();
        assert!(!options.dismiss_on_overlay_press);
        assert_eq!(options.initial_focus, Some(GlobalElementId(0xbeef)));
    }

    #[test]
    fn modal_dialog_request_with_options_and_dismiss_handler_sets_dismiss_handler() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let handler: OnDismissRequest = Arc::new(|_host, _cx, _req: &mut DismissRequestCx| {});
        let req = modal_dialog_request_with_options_and_dismiss_handler(
            GlobalElementId(0x123),
            GlobalElementId(0x123),
            open,
            OverlayPresence::instant(true),
            DialogOptions::default(),
            Some(handler),
            Vec::new(),
        );

        assert!(req.dismissible_on_dismiss_request.is_some());
    }

    #[test]
    fn apply_dialog_trigger_a11y_sets_controls_and_expanded() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let content = GlobalElementId(0xdead);
            let trigger = apply_dialog_trigger_a11y(trigger, true, Some(content));

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(content.0));
        });
    }

    #[test]
    fn modal_dialog_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);

        let req = modal_dialog_request(
            id,
            trigger,
            open,
            OverlayPresence::instant(true),
            Vec::new(),
        );
        let expected = dialog_root_name(id);
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn modal_dialog_request_with_options_sets_initial_focus() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);
        let initial_focus = GlobalElementId(0xbeef);

        let opts = DialogOptions::default().initial_focus(Some(initial_focus));
        let req = modal_dialog_request_with_options(
            id,
            trigger,
            open,
            OverlayPresence::instant(true),
            opts,
            Vec::new(),
        );
        assert_eq!(req.initial_focus, Some(initial_focus));
    }

    #[test]
    fn modal_dialog_installs_barrier_root_for_semantics_snapshot() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let b = bounds();

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "base",
            |_cx| Vec::new(),
        );
        ui.set_root(base);

        let open = app.models_mut().insert(true);
        let modal_id = GlobalElementId(0xabc);

        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, b, "modal", |cx| {
                let content = cx.container(ContainerProps::default(), |_cx| Vec::new());
                modal_dialog_layer_elements(
                    cx,
                    open.clone(),
                    DialogOptions::default(),
                    Vec::new(),
                    content,
                )
            });

        let req = modal_dialog_request(
            modal_id,
            modal_id,
            open,
            OverlayPresence::instant(true),
            overlay_children,
        );
        OverlayController::request_for_window(&mut app, window, req);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap.barrier_root.expect("barrier_root");
        assert!(
            snap.roots
                .iter()
                .any(|r| r.root == barrier_root && r.blocks_underlay_input),
            "expected barrier root to block underlay input"
        );
    }

    #[test]
    fn modal_barrier_can_dismiss_on_press() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let b = bounds();

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "base",
            |_cx| Vec::new(),
        );
        ui.set_root(base);

        let open = app.models_mut().insert(true);
        let modal_id = GlobalElementId(0xabc);

        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, b, "modal", |cx| {
                vec![modal_barrier(cx, open.clone(), true, Vec::new())]
            });

        let req = modal_dialog_request(
            modal_id,
            modal_id,
            open.clone(),
            OverlayPresence::instant(true),
            overlay_children,
        );
        OverlayController::request_for_window(&mut app, window, req);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let modal_root = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.blocks_underlay_input && l.visible)
            .expect("modal layer root")
            .root;
        let barrier = ui.children(modal_root)[0];
        let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");
        assert!(
            barrier_bounds.origin.x.0 <= 10.0
                && barrier_bounds.origin.y.0 <= 10.0
                && barrier_bounds.origin.x.0 + barrier_bounds.size.width.0 >= 10.0
                && barrier_bounds.origin.y.0 + barrier_bounds.size.height.0 >= 10.0,
            "expected modal barrier to cover (10, 10), got {barrier_bounds:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn modal_barrier_can_route_dismissals_through_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let b = bounds();

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "base",
            |_cx| Vec::new(),
        );
        ui.set_root(base);

        let open = app.models_mut().insert(true);
        let modal_id = GlobalElementId(0xabc);

        let reason_cell: Arc<std::sync::Mutex<Option<DismissReason>>> =
            Arc::new(std::sync::Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            *reason_cell_for_handler.lock().expect("reason lock") = Some(req.reason);
            req.prevent_default();
        });

        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, b, "modal", |cx| {
                vec![modal_barrier_with_dismiss_handler(
                    cx,
                    open.clone(),
                    true,
                    Some(handler.clone()),
                    Vec::new(),
                )]
            });

        let req = modal_dialog_request(
            modal_id,
            modal_id,
            open.clone(),
            OverlayPresence::instant(true),
            overlay_children,
        );
        OverlayController::request_for_window(&mut app, window, req);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let modal_root = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.blocks_underlay_input && l.visible)
            .expect("modal layer root")
            .root;
        let barrier = ui.children(modal_root)[0];
        let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");
        assert!(
            barrier_bounds.origin.x.0 <= 10.0
                && barrier_bounds.origin.y.0 <= 10.0
                && barrier_bounds.origin.x.0 + barrier_bounds.size.width.0 >= 10.0
                && barrier_bounds.origin.y.0 + barrier_bounds.size.height.0 >= 10.0,
            "expected modal barrier to cover (10, 10), got {barrier_bounds:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
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
    fn modal_dialog_focuses_first_focusable_descendant_by_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let b = bounds();

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "base",
            |_cx| Vec::new(),
        );
        ui.set_root(base);

        let open = app.models_mut().insert(true);
        let modal_id = GlobalElementId(0xabc);

        let mut focusable_element: Option<GlobalElementId> = None;
        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, b, "modal", |cx| {
                let content = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        focusable_element = Some(id);
                        Vec::new()
                    },
                );

                modal_dialog_layer_elements(
                    cx,
                    open.clone(),
                    DialogOptions::default(),
                    Vec::new(),
                    content,
                )
            });
        let focusable_element = focusable_element.expect("focusable element id");

        let req = modal_dialog_request(
            modal_id,
            modal_id,
            open,
            OverlayPresence::instant(true),
            overlay_children,
        );
        OverlayController::request_for_window(&mut app, window, req);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let focused = ui.focus();
        let expected = fret_ui::elements::node_for_element(&mut app, window, focusable_element);
        assert_eq!(focused, expected);
    }
}
