//! Window-scoped overlay manager (policy layer).
//!
//! This is a small component-layer orchestration helper that installs `UiTree` overlay roots
//! (ADR 0067) and coordinates dismissal + focus restore rules (ADR 0069).

use fret_core::{AppWindowId, NodeId, Rect, TimerToken};
use fret_runtime::{CommandId, DragKind, Effect, Model};
use fret_ui::action::DismissReason;
use fret_ui::declarative;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{ElementCx, Invalidation, UiHost, UiTree};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct DismissiblePopoverRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
    pub present: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for DismissiblePopoverRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DismissiblePopoverRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("initial_focus", &self.initial_focus)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct ModalRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: Option<GlobalElementId>,
    pub open: Model<bool>,
    pub present: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for ModalRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("initial_focus", &self.initial_focus)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct HoverOverlayRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: GlobalElementId,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for HoverOverlayRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverOverlayRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Default)]
struct WindowOverlayFrame {
    frame_id: fret_core::FrameId,
    popovers: Vec<DismissiblePopoverRequest>,
    modals: Vec<ModalRequest>,
    hover_overlays: Vec<HoverOverlayRequest>,
    tooltips: Vec<TooltipRequest>,
    toasts: Vec<ToastLayerRequest>,
}

struct ActivePopover {
    layer: UiLayerId,
    root_name: String,
    trigger: GlobalElementId,
    initial_focus: Option<GlobalElementId>,
    open: bool,
    restore_focus: Option<NodeId>,
}

struct ActiveModal {
    layer: UiLayerId,
    root_name: String,
    trigger: Option<GlobalElementId>,
    initial_focus: Option<GlobalElementId>,
    open: bool,
    restore_focus: Option<NodeId>,
}

struct ActiveTooltip {
    layer: UiLayerId,
    root_name: String,
}

struct ActiveToastLayer {
    layer: UiLayerId,
    root_name: String,
}

struct ActiveHoverOverlay {
    layer: UiLayerId,
    root_name: String,
    trigger: GlobalElementId,
}

#[derive(Default)]
struct WindowOverlays {
    windows: HashMap<AppWindowId, WindowOverlayFrame>,
    popovers: HashMap<(AppWindowId, GlobalElementId), ActivePopover>,
    modals: HashMap<(AppWindowId, GlobalElementId), ActiveModal>,
    hover_overlays: HashMap<(AppWindowId, GlobalElementId), ActiveHoverOverlay>,
    tooltips: HashMap<(AppWindowId, GlobalElementId), ActiveTooltip>,
    toast_layers: HashMap<(AppWindowId, GlobalElementId), ActiveToastLayer>,
}

pub fn begin_frame<H: UiHost>(app: &mut H, window: AppWindowId) {
    let frame_id = app.frame_id();
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.popovers.clear();
            w.modals.clear();
            w.hover_overlays.clear();
            w.tooltips.clear();
            w.toasts.clear();
        }
    });
}

pub fn request_dismissible_popover<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    request: DismissiblePopoverRequest,
) {
    request_dismissible_popover_for_window(cx.app, cx.window, request);
}

pub fn request_dismissible_popover_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.popovers.push(request);
    });
}

pub fn request_modal<H: UiHost>(cx: &mut ElementCx<'_, H>, request: ModalRequest) {
    request_modal_for_window(cx.app, cx.window, request);
}

pub fn request_modal_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ModalRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.modals.push(request);
    });
}

pub fn request_hover_overlay<H: UiHost>(cx: &mut ElementCx<'_, H>, request: HoverOverlayRequest) {
    request_hover_overlay_for_window(cx.app, cx.window, request);
}

pub fn request_hover_overlay_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: HoverOverlayRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.hover_overlays.push(request);
    });
}

#[derive(Debug, Clone)]
pub struct TooltipRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub children: Vec<AnyElement>,
}

pub fn request_tooltip<H: UiHost>(cx: &mut ElementCx<'_, H>, request: TooltipRequest) {
    request_tooltip_for_window(cx.app, cx.window, request);
}

pub fn request_tooltip_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: TooltipRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.tooltips.push(request);
    });
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct ToastAction {
    pub label: Arc<str>,
    pub command: CommandId,
}

#[derive(Debug, Clone)]
pub struct ToastRequest {
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub duration: Option<Duration>,
    pub variant: ToastVariant,
    pub action: Option<ToastAction>,
    pub dismissible: bool,
}

impl ToastRequest {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            duration: Some(Duration::from_secs(3)),
            variant: ToastVariant::default(),
            action: None,
            dismissible: true,
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(pub u64);

#[derive(Debug, Clone)]
struct ToastEntry {
    id: ToastId,
    title: Arc<str>,
    description: Option<Arc<str>>,
    variant: ToastVariant,
    action: Option<ToastAction>,
    dismissible: bool,
    token: Option<TimerToken>,
}

#[derive(Debug, Default)]
pub struct ToastStore {
    next_id: u64,
    by_window: HashMap<AppWindowId, Vec<ToastEntry>>,
    by_token: HashMap<TimerToken, (AppWindowId, ToastId)>,
}

impl ToastStore {
    fn toasts_for_window(&self, window: AppWindowId) -> &[ToastEntry] {
        self.by_window
            .get(&window)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn add_toast(
        &mut self,
        window: AppWindowId,
        request: ToastRequest,
        token: Option<TimerToken>,
    ) -> ToastId {
        if self.next_id == 0 {
            self.next_id = 1;
        }
        let id = ToastId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);

        if let Some(token) = token {
            self.by_token.insert(token, (window, id));
        }

        self.by_window.entry(window).or_default().push(ToastEntry {
            id,
            title: request.title,
            description: request.description,
            variant: request.variant,
            action: request.action,
            dismissible: request.dismissible,
            token,
        });

        id
    }

    fn remove_toast(&mut self, window: AppWindowId, id: ToastId) -> Option<ToastEntry> {
        let toasts = self.by_window.get_mut(&window)?;
        let idx = toasts.iter().position(|t| t.id == id)?;
        let entry = toasts.remove(idx);
        if let Some(token) = entry.token {
            self.by_token.remove(&token);
        }
        Some(entry)
    }

    fn remove_toast_by_token(&mut self, token: TimerToken) -> Option<(AppWindowId, ToastEntry)> {
        let (window, id) = self.by_token.remove(&token)?;
        let entry = self.remove_toast(window, id)?;
        Some((window, entry))
    }
}

#[derive(Clone)]
pub struct ToastLayerRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub store: Model<ToastStore>,
    pub position: ToastPosition,
}

impl std::fmt::Debug for ToastLayerRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastLayerRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("store", &"<model>")
            .field("position", &self.position)
            .finish()
    }
}

impl ToastLayerRequest {
    pub fn new(id: GlobalElementId, store: Model<ToastStore>) -> Self {
        Self {
            id,
            root_name: toast_layer_root_name(id),
            store,
            position: ToastPosition::default(),
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }
}

#[derive(Default)]
struct ToastService {
    store: Option<Model<ToastStore>>,
}

pub fn toast_store<H: UiHost>(app: &mut H) -> Model<ToastStore> {
    app.with_global_mut(ToastService::default, |svc, app| {
        *svc.store
            .get_or_insert_with(|| app.models_mut().insert(ToastStore::default()))
    })
}

pub fn request_toast_layer<H: UiHost>(cx: &mut ElementCx<'_, H>, request: ToastLayerRequest) {
    request_toast_layer_for_window(cx.app, cx.window, request);
}

pub fn request_toast_layer_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ToastLayerRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.toasts.push(request);
    });
}

pub fn toast_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    store: Model<ToastStore>,
    window: AppWindowId,
    request: ToastRequest,
) -> ToastId {
    let token = request
        .duration
        .filter(|d| d.as_secs_f32() > 0.0)
        .map(|after| {
            let token = host.next_timer_token();
            host.push_effect(Effect::SetTimer {
                window: Some(window),
                token,
                after,
                repeat: None,
            });
            token
        });

    let result = host
        .models_mut()
        .update(store, |st| st.add_toast(window, request, token));

    let Ok(id) = result else {
        if let Some(token) = token {
            host.push_effect(Effect::CancelTimer { token });
        }
        return ToastId(0);
    };

    host.request_redraw(window);
    id
}

pub fn dismiss_toast_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    store: Model<ToastStore>,
    window: AppWindowId,
    id: ToastId,
) -> bool {
    let removed = host
        .models_mut()
        .update(store, |st| st.remove_toast(window, id))
        .ok();
    let Some(entry) = removed.flatten() else {
        return false;
    };

    if let Some(token) = entry.token {
        host.push_effect(Effect::CancelTimer { token });
    }
    true
}

pub fn render<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let dock_drag_affects_window = app.drag().is_some_and(|d| {
        d.kind == DragKind::DockPanel && (d.source_window == window || d.current_window == window)
    });

    let (
        modal_requests,
        popover_requests,
        hover_overlay_requests,
        tooltip_requests,
        toast_requests,
    ) = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .windows
            .get_mut(&window)
            .map(|w| {
                (
                    std::mem::take(&mut w.modals),
                    std::mem::take(&mut w.popovers),
                    std::mem::take(&mut w.hover_overlays),
                    std::mem::take(&mut w.tooltips),
                    std::mem::take(&mut w.toasts),
                )
            })
            .unwrap_or_default()
    });

    let mut seen_modals: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_popovers: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_hover_overlays: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_tooltips: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_toast_layers: HashSet<GlobalElementId> = HashSet::new();

    for req in modal_requests {
        if !req.present {
            continue;
        }
        seen_modals.insert(req.id);

        let open_now = app.models().get(req.open).copied().unwrap_or(false);

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |cx| {
                let open = req.open;
                cx.dismissible_on_dismiss_request(Arc::new(
                    move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(open, |v| *v = false);
                    },
                ));
                req.children
            },
        );

        let key = (window, req.id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let mut created = false;
            let entry = overlays.modals.entry(key).or_insert_with(|| {
                created = true;
                ActiveModal {
                    layer: ui.push_overlay_root_ex(root, true, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    initial_focus: req.initial_focus,
                    open: false,
                    restore_focus: None,
                }
            });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;

            ui.set_layer_visible(entry.layer, true);
            // For modal overlays, `present` is the authority for whether the barrier is active.
            //
            // During a close animation we may keep the modal mounted (`present=true`) while
            // `open=false`. The barrier must continue to block underlay input for the full
            // duration of the out transition.
            ui.set_layer_hit_testable(entry.layer, true);

            let opening = open_now && (!entry.open || created);
            if opening {
                should_focus_initial = true;
                entry.restore_focus = restore_focus;
            }
            entry.open = open_now;
        });

        if should_focus_initial {
            let focus = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
                overlays.modals.get(&key).and_then(|p| p.initial_focus)
            });

            if let Some(focus) = focus
                && let Some(node) = fret_ui::elements::node_for_element(app, window, focus)
            {
                ui.set_focus(Some(node));
            } else if let Some(node) =
                ui.first_focusable_descendant_including_declarative(app, window, root)
            {
                ui.set_focus(Some(node));
            }
        }
    }

    for req in popover_requests {
        if dock_drag_affects_window {
            if req.present {
                let _ = app.models_mut().update(req.open, |v| *v = false);
            }
            continue;
        }

        if !req.present {
            continue;
        }
        seen_popovers.insert(req.id);

        let open_now = app.models().get(req.open).copied().unwrap_or(false);

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |cx| {
                let open = req.open;
                cx.dismissible_on_dismiss_request(Arc::new(
                    move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(open, |v| *v = false);
                    },
                ));
                req.children
            },
        );

        let key = (window, req.id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let mut created = false;
            let entry = overlays.popovers.entry(key).or_insert_with(|| {
                created = true;
                ActivePopover {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    initial_focus: req.initial_focus,
                    open: false,
                    restore_focus: None,
                }
            });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;
            ui.set_layer_wants_pointer_down_outside_events(entry.layer, true);

            ui.set_layer_visible(entry.layer, true);
            ui.set_layer_hit_testable(entry.layer, open_now);

            let opening = open_now && (!entry.open || created);
            if opening {
                should_focus_initial = true;
                entry.restore_focus = restore_focus;
            }
            entry.open = open_now;
        });

        if should_focus_initial {
            let focus = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.get(&key).and_then(|p| p.initial_focus)
            });

            if let Some(focus) = focus
                && let Some(node) = fret_ui::elements::node_for_element(app, window, focus)
            {
                ui.set_focus(Some(node));
            } else if let Some(node) =
                ui.first_focusable_descendant_including_declarative(app, window, root)
            {
                ui.set_focus(Some(node));
            }
        }
    }

    let to_hide_popovers: Vec<(UiLayerId, GlobalElementId, Option<NodeId>)> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let mut out: Vec<(UiLayerId, GlobalElementId, Option<NodeId>)> = Vec::new();
            for ((w, id), active) in overlays.popovers.iter() {
                if *w != window || seen_popovers.contains(id) {
                    continue;
                }
                out.push((active.layer, active.trigger, active.restore_focus));
            }
            out
        });

    let to_hide_modals: Vec<(UiLayerId, Option<GlobalElementId>, Option<NodeId>)> = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .modals
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_modals.contains(id) {
                        return None;
                    }
                    Some((active.layer, active.trigger, active.restore_focus))
                })
                .collect()
        });

    for (layer, trigger, restore_focus) in to_hide_popovers {
        let focus = ui.focus();
        if focus.is_some_and(|n| ui.node_layer(n) == Some(layer)) {
            ui.set_layer_visible(layer, false);
            // Prefer resolving the trigger at restore time to avoid relying on potentially stale
            // `NodeId` snapshots across frames.
            if let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger) {
                ui.set_focus(Some(trigger_node));
            } else if let Some(node) = restore_focus
                && ui.node_layer(node).is_some()
            {
                ui.set_focus(Some(node));
            }
        } else {
            ui.set_layer_visible(layer, false);
        }
    }

    for (layer, trigger, restore_focus) in to_hide_modals {
        // Modals should restore focus deterministically on close (Radix-style): underlay focus
        // changes cannot happen while the barrier is installed, so it's safe to always restore on
        // unmount.
        ui.set_layer_visible(layer, false);

        // Prefer resolving the trigger at restore time to avoid relying on potentially stale
        // `NodeId` snapshots across frames.
        if let Some(trigger) = trigger
            && let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger)
        {
            ui.set_focus(Some(trigger_node));
        } else if let Some(node) = restore_focus
            && ui.node_layer(node).is_some()
        {
            ui.set_focus(Some(node));
        }
    }

    for req in hover_overlay_requests {
        if dock_drag_affects_window {
            continue;
        }

        seen_hover_overlays.insert(req.id);

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |_cx| req.children,
        );

        let key = (window, req.id);
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .hover_overlays
                .entry(key)
                .or_insert_with(|| ActiveHoverOverlay {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            ui.set_layer_visible(entry.layer, true);
        });
    }

    let to_hide_hover_overlays: Vec<(UiLayerId, GlobalElementId)> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .hover_overlays
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_hover_overlays.contains(id) {
                        return None;
                    }
                    Some((active.layer, active.trigger))
                })
                .collect()
        });

    for (layer, trigger) in to_hide_hover_overlays {
        let focus = ui.focus();
        if focus.is_some_and(|n| ui.node_layer(n) == Some(layer))
            && let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger)
        {
            ui.set_layer_visible(layer, false);
            ui.set_focus(Some(trigger_node));
        } else {
            ui.set_layer_visible(layer, false);
        }
    }

    for req in tooltip_requests {
        seen_tooltips.insert(req.id);

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |_cx| req.children,
        );

        let key = (window, req.id);
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .tooltips
                .entry(key)
                .or_insert_with(|| ActiveTooltip {
                    layer: ui.push_overlay_root_ex(root, false, false),
                    root_name: req.root_name.clone(),
                });
            entry.root_name = req.root_name.clone();
            ui.set_layer_visible(entry.layer, true);
        });
    }

    let to_hide_tooltips: Vec<UiLayerId> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .tooltips
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_tooltips.contains(id) {
                        return None;
                    }
                    Some(active.layer)
                })
                .collect()
        });

    for layer in to_hide_tooltips {
        ui.set_layer_visible(layer, false);
    }

    for req in toast_requests {
        seen_toast_layers.insert(req.id);

        let store = req.store;
        let position = req.position;
        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            move |cx| {
                cx.observe_model(store, Invalidation::Paint);

                let hook_store = store;
                cx.timer_on_timer_for(
                    cx.root_id(),
                    Arc::new(move |host, _cx, token| {
                        host.models_mut()
                            .update(hook_store, |st| st.remove_toast_by_token(token))
                            .ok()
                            .flatten()
                            .is_some()
                    }),
                );

                let toasts: Vec<ToastEntry> = cx
                    .app
                    .models()
                    .get(store)
                    .map(|st| st.toasts_for_window(window).to_vec())
                    .unwrap_or_default();

                if toasts.is_empty() {
                    return Vec::new();
                }

                let theme = fret_ui::Theme::global(&*cx.app).clone();
                let margin = theme.metrics.padding_md;
                let gap = theme.metrics.padding_sm;
                let toast_padding = theme.metrics.padding_sm;
                let radius = theme.metrics.radius_md;

                let mut wrapper_layout = fret_ui::element::LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    ..Default::default()
                };
                match position {
                    ToastPosition::TopLeft => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.left = Some(margin);
                    }
                    ToastPosition::TopRight => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                    ToastPosition::BottomLeft => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.left = Some(margin);
                    }
                    ToastPosition::BottomRight => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                }

                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: wrapper_layout,
                        direction: fret_core::Axis::Vertical,
                        gap,
                        padding: fret_core::Edges::all(fret_core::Px(0.0)),
                        justify: fret_ui::element::MainAlign::End,
                        align: fret_ui::element::CrossAlign::End,
                        wrap: false,
                    },
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(toasts.len());
                        for toast in toasts {
                            let store = store;
                            let toast_id = toast.id;

                            let bg = match toast.variant {
                                ToastVariant::Default => theme.colors.panel_background,
                                ToastVariant::Destructive => theme.colors.menu_background,
                            };
                            let border_color = theme.colors.panel_border;
                            let fg = theme.colors.text_primary;
                            let fg_muted = theme.colors.text_muted;

                            let close = toast.dismissible.then(|| {
                                let close_store = store;
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: false,
                                        focus_ring: None,
                                        a11y: Default::default(),
                                    },
                                    move |cx, _st| {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                let _ = dismiss_toast_action(
                                                    host,
                                                    close_store,
                                                    cx.window,
                                                    toast_id,
                                                );
                                            },
                                        ));
                                        vec![cx.text("×")]
                                    },
                                )
                            });

                            let action = toast.action.clone().map(|action| {
                                let action_store = store;
                                let cmd = action.command;
                                let label = action.label;
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: false,
                                        focus_ring: None,
                                        a11y: Default::default(),
                                    },
                                    move |cx, _st| {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                host.dispatch_command(Some(cx.window), cmd.clone());
                                            },
                                        ));
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                let _ = dismiss_toast_action(
                                                    host,
                                                    action_store,
                                                    cx.window,
                                                    toast_id,
                                                );
                                            },
                                        ));
                                        vec![cx.text(label.as_ref())]
                                    },
                                )
                            });

                            let header_row = cx.flex(
                                fret_ui::element::FlexProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: theme.metrics.padding_sm,
                                    padding: fret_core::Edges::all(fret_core::Px(0.0)),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
                                    wrap: false,
                                },
                                |cx| {
                                    let mut row: Vec<AnyElement> = Vec::new();
                                    row.push(cx.text_props(fret_ui::element::TextProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        text: toast.title.clone(),
                                        style: None,
                                        color: Some(fg),
                                        wrap: fret_core::TextWrap::None,
                                        overflow: fret_core::TextOverflow::Clip,
                                    }));
                                    if let Some(action) = action {
                                        row.push(action);
                                    }
                                    if let Some(close) = close {
                                        row.push(close);
                                    }
                                    row
                                },
                            );

                            let mut toast_children: Vec<AnyElement> = vec![header_row];
                            if let Some(desc) = toast.description.clone() {
                                toast_children.push(cx.text_props(fret_ui::element::TextProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    text: desc,
                                    style: None,
                                    color: Some(fg_muted),
                                    wrap: fret_core::TextWrap::Word,
                                    overflow: fret_core::TextOverflow::Clip,
                                }));
                            }

                            out.push(cx.container(
                                fret_ui::element::ContainerProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    padding: fret_core::Edges::all(toast_padding),
                                    background: Some(bg),
                                    shadow: None,
                                    border: fret_core::Edges::all(fret_core::Px(1.0)),
                                    border_color: Some(border_color),
                                    corner_radii: fret_core::Corners::all(radius),
                                },
                                move |_cx| toast_children,
                            ));
                        }
                        out
                    },
                )]
            },
        );

        let has_toasts = app
            .models()
            .get(store)
            .is_some_and(|st| !st.toasts_for_window(window).is_empty());

        let key = (window, req.id);
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .toast_layers
                .entry(key)
                .or_insert_with(|| ActiveToastLayer {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                });
            entry.root_name = req.root_name.clone();

            ui.set_layer_wants_timer_events(entry.layer, has_toasts);
            ui.set_layer_visible(entry.layer, has_toasts);
        });
    }

    let to_hide_toast_layers: Vec<UiLayerId> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .toast_layers
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_toast_layers.contains(id) {
                        return None;
                    }
                    Some(active.layer)
                })
                .collect()
        });

    for layer in to_hide_toast_layers {
        ui.set_layer_wants_timer_events(layer, false);
        ui.set_layer_visible(layer, false);
    }
}

pub fn popover_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.popover.{:x}", id.0)
}

pub fn modal_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.modal.{:x}", id.0)
}

pub fn tooltip_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.tooltip.{:x}", id.0)
}

pub fn hover_overlay_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.hover-overlay.{:x}", id.0)
}

pub fn toast_layer_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.toast-layer.{:x}", id.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::action_hooks::ActionHooksExt;
    use fret_app::App;
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Point, Px, Rect, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PositionStyle, PressableProps};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
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

    fn render_base_with_trigger(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> GlobalElementId {
        begin_frame(app, window);

        let mut trigger_id: Option<GlobalElementId> = None;
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_| Vec::new())]
                    },
                )]
            });
        ui.set_root(root);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn dismissible_popover_closes_on_outside_press() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // First frame: render base to establish stable bounds for the trigger element.
        let trigger =
            render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via click.
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
        assert_eq!(app.models().get(open).copied(), Some(true));

        // Second frame: request and render a dismissible popover.
        begin_frame(&mut app, window);
        let _ = render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

        request_dismissible_popover_for_window(
            &mut app,
            window,
            DismissiblePopoverRequest {
                id: trigger,
                root_name: popover_root_name(trigger),
                trigger,
                open,
                present: true,
                initial_focus: None,
                children: vec![],
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(ui.captured(), None);

        // Pointer down outside should close (observer pass).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(250.0), Px(180.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get(open).copied(), Some(false));
    }

    #[test]
    fn dismissible_popover_does_not_close_on_inside_press() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // First frame: render base to establish stable bounds for the trigger element.
        let trigger =
            render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via click.
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
        assert_eq!(app.models().get(open).copied(), Some(true));

        // Second frame: request and render a dismissible popover with a non-pressable child so
        // the pointer-down bubbles to the root in the normal dispatch path.
        begin_frame(&mut app, window);
        let _ = render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

        let root_name = popover_root_name(trigger);
        let children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, &root_name, |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: fret_ui::element::InsetStyle {
                                top: Some(Px(40.0)),
                                left: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(Px(120.0)),
                                height: Length::Px(Px(80.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_| Vec::new(),
                )]
            });

        request_dismissible_popover_for_window(
            &mut app,
            window,
            DismissiblePopoverRequest {
                id: trigger,
                root_name,
                trigger,
                open,
                present: true,
                initial_focus: None,
                children,
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(ui.captured(), None);

        // Pointer down inside the popover content should not close it.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(50.0), Px(50.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get(open).copied(), Some(true));
    }

    #[test]
    fn modal_blocks_underlay_click_and_closes_on_escape() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_clicked = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // Base layer contains a pressable that increments underlay_clicks.
        begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(underlay_clicked);
                        vec![]
                    },
                )]
            },
        );
        ui.set_root(base);

        // Install modal layer.
        begin_frame(&mut app, window);
        let modal_children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
                vec![cx.pressable(
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
                    |_cx, _st| vec![],
                )]
            });
        request_modal_for_window(
            &mut app,
            window,
            ModalRequest {
                id: GlobalElementId(0xabc),
                root_name: modal_root_name(GlobalElementId(0xabc)),
                trigger: None,
                open,
                present: true,
                initial_focus: None,
                children: modal_children,
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Clicking underlay area should not reach base (modal barrier blocks underlay input).
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

        assert_eq!(app.models().get(underlay_clicked).copied(), Some(false));

        // Escape should close via DismissibleLayer.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get(open).copied(), Some(false));
    }

    #[test]
    fn modal_can_remain_present_while_still_blocking_underlay_during_close_animation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let overlay_clicked = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // Base layer contains a full-size pressable we expect NOT to receive the click.
        begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(underlay_clicked);
                        vec![]
                    },
                )]
            },
        );
        ui.set_root(base);

        // Install a modal layer that is still `present` but `open=false` (closing animation).
        begin_frame(&mut app, window);
        let modal_id = GlobalElementId(0xbeef);
        let modal_children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
                vec![cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: false,
                        ..Default::default()
                    },
                    |cx, _st| {
                        cx.pressable_toggle_bool(overlay_clicked);
                        vec![]
                    },
                )]
            });
        request_modal_for_window(
            &mut app,
            window,
            ModalRequest {
                id: modal_id,
                root_name: modal_root_name(modal_id),
                trigger: None,
                open,
                present: true,
                initial_focus: None,
                children: modal_children,
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

        assert_eq!(app.models().get(underlay_clicked).copied(), Some(false));
        assert_eq!(app.models().get(overlay_clicked).copied(), Some(true));
    }

    #[test]
    fn non_modal_overlay_can_remain_present_while_pointer_transparent_during_close_animation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let overlay_clicked = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // Base layer contains a full-size pressable we expect to receive the click.
        begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(underlay_clicked);
                        vec![]
                    },
                )]
            },
        );
        ui.set_root(base);

        // Install a non-modal layer that is still `present` but `open=false` (closing animation).
        begin_frame(&mut app, window);
        let trigger = GlobalElementId(0xdead);
        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
                vec![cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: false,
                        ..Default::default()
                    },
                    |cx, _st| {
                        cx.pressable_toggle_bool(overlay_clicked);
                        vec![]
                    },
                )]
            });

        request_dismissible_popover_for_window(
            &mut app,
            window,
            DismissiblePopoverRequest {
                id: trigger,
                root_name: popover_root_name(trigger),
                trigger,
                open,
                present: true,
                initial_focus: None,
                children: overlay_children,
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

        assert_eq!(app.models().get(underlay_clicked).copied(), Some(true));
        assert_eq!(app.models().get(overlay_clicked).copied(), Some(false));
    }
}
