use super::*;
use crate::widget::{CommandAvailability, CommandAvailabilityCx};
use fret_runtime::CommandScope;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::tree) struct DeclarativeCommandAvailabilityInterest {
    all: bool,
    text_edit: bool,
    selectable_text_edit: bool,
    focus_traversal: bool,
    commands: Vec<CommandId>,
}

impl DeclarativeCommandAvailabilityInterest {
    fn none() -> Self {
        Self {
            all: false,
            text_edit: false,
            selectable_text_edit: false,
            focus_traversal: false,
            commands: Vec::new(),
        }
    }

    fn all() -> Self {
        Self {
            all: true,
            ..Self::none()
        }
    }

    fn text_edit() -> Self {
        Self {
            text_edit: true,
            ..Self::none()
        }
    }

    fn selectable_text_edit() -> Self {
        Self {
            selectable_text_edit: true,
            ..Self::none()
        }
    }

    fn focus_traversal() -> Self {
        Self {
            focus_traversal: true,
            ..Self::none()
        }
    }

    fn commands(commands: Vec<CommandId>) -> Self {
        let mut commands = commands;
        commands.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        commands.dedup_by(|a, b| a.as_str() == b.as_str());
        if commands.is_empty() {
            Self::none()
        } else {
            Self {
                commands,
                ..Self::none()
            }
        }
    }

    fn matches(&self, command: &CommandId) -> bool {
        if self.all {
            return true;
        }

        let command_name = command.as_str();
        (self.text_edit && (command_name.starts_with("text.") || command_name.starts_with("edit.")))
            || (self.selectable_text_edit
                && matches!(
                    command_name,
                    "text.select_all" | "edit.select_all" | "text.copy" | "edit.copy"
                ))
            || (self.focus_traversal && matches!(command_name, "focus.next" | "focus.previous"))
            || self
                .commands
                .binary_search_by(|id| id.as_str().cmp(command_name))
                .is_ok()
    }

    fn matches_for_route(
        &self,
        command: &CommandId,
        route: CommandAvailabilityInterestRoute,
    ) -> bool {
        match route {
            CommandAvailabilityInterestRoute::DispatchPath => self.matches(command),
            CommandAvailabilityInterestRoute::NoFocusSubtreeFallback => {
                if self.all {
                    return true;
                }

                let command_name = command.as_str();
                (self.focus_traversal && matches!(command_name, "focus.next" | "focus.previous"))
                    || self
                        .commands
                        .binary_search_by(|id| id.as_str().cmp(command_name))
                        .is_ok()
            }
        }
    }

    fn union(mut self, other: Self) -> Self {
        if self.all || other.all {
            return Self::all();
        }

        self.text_edit |= other.text_edit;
        self.selectable_text_edit |= other.selectable_text_edit;
        self.focus_traversal |= other.focus_traversal;
        self.commands.extend(other.commands);
        self.commands.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        self.commands.dedup_by(|a, b| a.as_str() == b.as_str());
        self
    }
}

impl From<crate::action::CommandAvailabilityInterest> for DeclarativeCommandAvailabilityInterest {
    fn from(value: crate::action::CommandAvailabilityInterest) -> Self {
        match value {
            crate::action::CommandAvailabilityInterest::None => Self::none(),
            crate::action::CommandAvailabilityInterest::All => Self::all(),
            crate::action::CommandAvailabilityInterest::Commands(commands) => {
                Self::commands(commands)
            }
        }
    }
}

#[derive(Debug, Default)]
struct CommandAvailabilityPublicationCache {
    declarative_interest: HashMap<NodeId, DeclarativeCommandAvailabilityInterest>,
    subtree_interest: HashMap<NodeId, DeclarativeCommandAvailabilityInterest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandAvailabilityInterestRoute {
    DispatchPath,
    NoFocusSubtreeFallback,
}

fn command_is_focus_bound_text_edit(command: &CommandId) -> bool {
    let name = command.as_str();
    name.starts_with("text.") || name.starts_with("edit.")
}

impl<H: UiHost> UiTree<H> {
    pub(crate) fn layout_active(&self) -> bool {
        self.layout_call_depth > 0
    }

    fn populate_subtree_command_availability_interest_cache(
        &mut self,
        app: &mut H,
        root: NodeId,
        publication_cache: &mut CommandAvailabilityPublicationCache,
    ) {
        let frame_id = app.frame_id();
        let mut stack: Vec<(NodeId, bool)> = vec![(root, false)];
        while let Some((node, visited)) = stack.pop() {
            if visited {
                #[cfg(test)]
                super::record_command_availability_subtree_interest_probe();

                let mut interest =
                    self.declarative_node_command_availability_interest_cached(app, frame_id, node);
                if let Some(entry) = self.nodes.get(node) {
                    for &child in &entry.children {
                        if let Some(child_interest) =
                            publication_cache.subtree_interest.get(&child).cloned()
                        {
                            interest = interest.union(child_interest);
                            if interest.all {
                                break;
                            }
                        }
                    }
                }
                publication_cache.subtree_interest.insert(node, interest);
                continue;
            }

            if publication_cache.subtree_interest.contains_key(&node) {
                continue;
            }

            stack.push((node, true));
            if let Some(entry) = self.nodes.get(node) {
                for &child in entry.children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    pub(crate) fn defer_declarative_window_snapshot_commit(&mut self, root: NodeId) {
        self.pending_declarative_window_snapshot_roots
            .retain(|pending| self.nodes.contains_key(*pending));
        self.pending_declarative_window_snapshot_roots.insert(root);
    }

    pub(crate) fn clear_declarative_window_snapshot_commit(&mut self, root: NodeId) {
        self.pending_declarative_window_snapshot_roots.remove(&root);
    }

    pub(in crate::tree) fn revalidate_focus_for_dispatch_snapshot(
        &mut self,
        frame_id: fret_runtime::FrameId,
        active_focus_layers: &[NodeId],
        barrier_root: Option<NodeId>,
        reason: &'static str,
    ) {
        let dispatch_snapshot = self.cached_dispatch_snapshot_for_layer_roots(
            frame_id,
            active_focus_layers,
            barrier_root,
        );
        if self
            .focus
            .is_some_and(|node| dispatch_snapshot.pre.get(node).is_none())
        {
            self.set_focus_unchecked(None, reason);
        }
    }

    pub(in crate::tree) fn revalidate_pending_shortcut_for_current_routing_context(
        &mut self,
        app: &mut H,
        barrier_root: Option<NodeId>,
    ) {
        if self.replaying_pending_shortcut || self.pending_shortcut.keystrokes.is_empty() {
            return;
        }

        // `focus` / `barrier_root` are only proxies for the shortcut-routing context. Root
        // replacement and other retained-tree repairs can change the authoritative key-context
        // stack without changing either proxy (for example, when no node is focused). Re-check
        // the current key-context stack before continuing a multi-stroke sequence.
        let current_key_contexts = self.shortcut_key_context_stack(app, barrier_root);
        if (self.pending_shortcut.focus.is_some() && self.pending_shortcut.focus != self.focus)
            || self.pending_shortcut.barrier_root != barrier_root
            || self.pending_shortcut.key_contexts.as_slice() != current_key_contexts.as_slice()
        {
            self.clear_pending_shortcut(app);
        }
    }

    pub(in crate::tree) fn current_window_input_context(
        &self,
        app: &mut H,
        ui_has_modal: bool,
        focus_is_text_input: bool,
    ) -> InputContext {
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let mut input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal,
            window_arbitration: self
                .window
                .map(|_| self.window_input_arbitration_snapshot()),
            focus_is_text_input,
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            router_can_back: false,
            router_can_forward: false,
            dispatch_phase: InputDispatchPhase::Bubble,
        };
        if let Some(window) = self.window {
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
                input_ctx.router_can_back = availability.router_can_back;
                input_ctx.router_can_forward = availability.router_can_forward;
            }
        }
        if let Some(mode) = self.focus_text_boundary_mode_override() {
            input_ctx.text_boundary_mode = mode;
        }
        input_ctx
    }

    pub(in crate::tree) fn publish_window_input_context_snapshot(
        &self,
        app: &mut H,
        input_ctx: &InputContext,
    ) {
        let Some(window) = self.window else {
            return;
        };
        let needs_update = app
            .global::<fret_runtime::WindowInputContextService>()
            .and_then(|svc| svc.snapshot(window))
            .is_none_or(|prev| prev != input_ctx);
        if needs_update {
            app.with_global_mut(
                fret_runtime::WindowInputContextService::default,
                |svc, _app| {
                    svc.set_snapshot(window, input_ctx.clone());
                },
            );
        }
    }

    pub(in crate::tree) fn publish_window_input_context_snapshot_untracked(
        &self,
        app: &mut H,
        input_ctx: &InputContext,
        only_if_changed: bool,
    ) {
        let Some(window) = self.window else {
            return;
        };
        if only_if_changed {
            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != input_ctx);
            if !needs_update {
                return;
            }
        }
        app.with_global_mut_untracked(
            fret_runtime::WindowInputContextService::default,
            |svc, _app| {
                svc.set_snapshot(window, input_ctx.clone());
            },
        );
    }

    pub(in crate::tree) fn publish_window_key_context_stack_snapshot(
        &self,
        app: &mut H,
        key_contexts: Vec<Arc<str>>,
    ) {
        let Some(window) = self.window else {
            return;
        };
        let needs_update = app
            .global::<fret_runtime::WindowKeyContextStackService>()
            .and_then(|svc| svc.snapshot(window))
            .is_none_or(|prev| prev != key_contexts.as_slice());
        if needs_update {
            app.with_global_mut(
                fret_runtime::WindowKeyContextStackService::default,
                |svc, _app| {
                    svc.set_snapshot(window, key_contexts);
                },
            );
        }
    }

    pub(in crate::tree) fn publish_post_dispatch_runtime_snapshots_for_event(
        &mut self,
        app: &mut H,
        event: &Event,
    ) {
        let trace_runtime_snapshot = tracing::enabled!(tracing::Level::TRACE);
        let time_enabled = self.debug_enabled;
        let window = self.window;
        let frame_id = app.frame_id();
        let pointer_move = matches!(event, Event::Pointer(fret_core::PointerEvent::Move { .. }));

        let (focus_is_text_input, focus_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.focus_repair",
                    window = ?window,
                    frame_id = frame_id.0,
                    reason = "post_dispatch",
                    pointer_move,
                )
            },
            || {
                let focus_is_text_input = self.focus_is_text_input(app);
                self.set_ime_allowed(app, focus_is_text_input);
                focus_is_text_input
            },
        );
        if let Some(focus_elapsed) = focus_elapsed {
            self.debug_stats.window_runtime_snapshot_focus_repair_time += focus_elapsed;
        }

        let (_active_layers, barrier_root) = self.active_input_layers();
        if pointer_move {
            let (_, input_context_elapsed) = fret_perf::measure_span(
                time_enabled,
                trace_runtime_snapshot,
                || {
                    tracing::trace_span!(
                        "fret.ui.window_runtime_snapshot.input_context",
                        window = ?window,
                        frame_id = frame_id.0,
                        reason = "post_dispatch_pointer_move",
                    )
                },
                || {
                    let input_ctx = self.current_window_input_context(
                        app,
                        barrier_root.is_some(),
                        focus_is_text_input,
                    );
                    self.publish_window_input_context_snapshot_untracked(app, &input_ctx, false);
                },
            );
            if let Some(input_context_elapsed) = input_context_elapsed {
                self.debug_stats.window_runtime_snapshot_input_context_time +=
                    input_context_elapsed;
            }
        } else {
            self.publish_window_runtime_snapshots(app);
        }
    }

    /// Publishes authoritative window-level runtime snapshots for the tree's current retained
    /// state.
    ///
    /// Raw `UiTree` mutation APIs (`set_root`, `set_focus`, overlay/layer mutation, subtree
    /// removal, and similar helpers) only update retained tree state. Cross-surface consumers that
    /// read `WindowInputContextService`, `WindowKeyContextStackService`,
    /// `PendingShortcutOverlayState`, or
    /// `WindowCommandActionAvailabilityService` become authoritative only after this publish step
    /// or another full snapshot commit boundary such as declarative rebuild or non-pointer input
    /// dispatch. Paint-only boundaries refresh `WindowInputContextService`, but they do not
    /// republish the full key-context / command-availability snapshot set.
    ///
    /// Layout-time raw focus/layer mutations are the one exception: they automatically schedule a
    /// post-layout refine so final layout boundaries can republish authoritative snapshots without
    /// forcing policy code to publish from inside `layout()`.
    ///
    /// Call this after imperative tree mutations when later same-frame consumers must observe the
    /// new authoritative window state immediately.
    pub fn publish_window_runtime_snapshots(&mut self, app: &mut H) {
        let trace_runtime_snapshot = tracing::enabled!(tracing::Level::TRACE);
        let time_enabled = self.debug_enabled;
        let window = self.window;
        let frame_id = app.frame_id();

        let (input_barrier_root, focus_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.focus_repair",
                    window = ?window,
                    frame_id = frame_id.0,
                    reason = "full_publish",
                )
            },
            || {
                self.pending_declarative_window_snapshot_roots
                    .retain(|pending| self.nodes.contains_key(*pending));
                self.resolve_pending_focus_target_if_needed(app);
                let focused_element_before_revalidate = self.focus.and_then(|focused| {
                    self.node_element(focused).or_else(|| {
                        self.window.and_then(|window| {
                            crate::elements::with_window_state(app, window, |state| {
                                state.element_for_node(focused)
                            })
                        })
                    })
                });
                let (_active_input_layers, input_barrier_root) = self.active_input_layers();
                let (active_focus_layers, focus_barrier_root) = self.active_focus_layers();
                let barrier_root = focus_barrier_root.or(input_barrier_root);

                let focus_before_revalidate = self.focus;
                self.revalidate_focus_for_dispatch_snapshot(
                    app.frame_id(),
                    active_focus_layers.as_slice(),
                    barrier_root,
                    "commands: focus missing from dispatch snapshot",
                );
                if focus_before_revalidate.is_some()
                    && self.focus.is_none()
                    && let Some(window) = self.window
                    && let Some(element) = focused_element_before_revalidate
                    && crate::elements::element_identity_is_live_in_current_frame(
                        app, window, element,
                    )
                {
                    // Declarative overlay/content roots can attach before final layout makes them part of
                    // the authoritative dispatch snapshot. Preserve the element identity as a deferred
                    // target so the final-layout snapshot refine can recover focus instead of dropping it
                    // for the rest of the frame.
                    self.pending_focus_target = Some(element);
                    self.request_post_layout_window_runtime_snapshot_refine();
                }

                self.revalidate_pending_shortcut_for_current_routing_context(app, barrier_root);
                input_barrier_root
            },
        );
        if let Some(focus_elapsed) = focus_elapsed {
            self.debug_stats.window_runtime_snapshot_focus_repair_time += focus_elapsed;
        }

        let (input_ctx, input_context_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.input_context",
                    window = ?window,
                    frame_id = frame_id.0,
                    reason = "full_publish",
                )
            },
            || {
                let focus_is_text_input = self.focus_is_text_input(app);
                let input_ctx = self.current_window_input_context(
                    app,
                    input_barrier_root.is_some(),
                    focus_is_text_input,
                );

                self.publish_window_input_context_snapshot(app, &input_ctx);
                input_ctx
            },
        );
        if let Some(input_context_elapsed) = input_context_elapsed {
            self.debug_stats.window_runtime_snapshot_input_context_time += input_context_elapsed;
        }

        let (_, command_availability_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.command_availability",
                    window = ?window,
                    frame_id = frame_id.0,
                )
            },
            || {
                self.publish_window_command_action_availability_snapshot_for_current_demand(
                    app, &input_ctx,
                )
            },
        );
        if let Some(command_availability_elapsed) = command_availability_elapsed {
            self.debug_stats
                .window_runtime_snapshot_command_availability_time += command_availability_elapsed;
        }

        let (_, shortcut_overlay_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.shortcut_overlay",
                    window = ?window,
                    frame_id = frame_id.0,
                )
            },
            || self.refresh_pending_shortcut_overlay_state_if_needed(app, &input_ctx),
        );
        if let Some(shortcut_overlay_elapsed) = shortcut_overlay_elapsed {
            self.debug_stats
                .window_runtime_snapshot_shortcut_overlay_time += shortcut_overlay_elapsed;
        }
    }

    fn publish_window_command_action_availability_snapshot_for_current_demand(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
    ) {
        let demand = self.window.and_then(|window| {
            crate::elements::with_window_state(app, window, |state| {
                state.command_action_availability_demand().cloned()
            })
        });

        match demand {
            Some(
                crate::elements::WindowCommandActionAvailabilityDemand::AllRegisteredWidgetCommands,
            )
            | None => {
                self.publish_window_command_action_availability_snapshot(app, input_ctx);
            }
            Some(
                crate::elements::WindowCommandActionAvailabilityDemand::FilteredWidgetCommands(
                    commands,
                ),
            ) if commands.is_empty() => {
                self.publish_window_command_action_availability_snapshot_filtered(
                    app, input_ctx, commands,
                );
            }
            Some(
                crate::elements::WindowCommandActionAvailabilityDemand::FilteredWidgetCommands(
                    commands,
                ),
            ) => {
                self.publish_window_command_action_availability_snapshot_filtered(
                    app, input_ctx, commands,
                );
            }
        }
    }

    pub(crate) fn request_post_layout_window_runtime_snapshot_refine(&mut self) {
        self.pending_post_layout_window_runtime_snapshot_refine = true;
    }

    pub(crate) fn request_post_layout_window_runtime_snapshot_refine_if_layout_active(&mut self) {
        if self.layout_call_depth > 0 {
            self.request_post_layout_window_runtime_snapshot_refine();
        }
    }

    /// Finalize a declarative rebuild that mounted a detached root and only later attached it to
    /// the retained tree.
    ///
    /// `render_layer_interaction_root_with_hooks(...)` can rebuild an overlay/portal root before the
    /// caller attaches that root to a layer or parent. In that case the helper defers the window
    /// snapshot commit until the root is actually attached. Call this after `push_overlay_root`,
    /// `set_children`, or another attach operation that makes the returned root authoritative for
    /// same-frame window-level consumers.
    ///
    /// This is intentionally narrower than `publish_window_runtime_snapshots(...)`: raw imperative
    /// tree mutation still requires an explicit commit, while declarative detached-root authoring
    /// can finish its pending commit once attachment is complete.
    pub fn commit_pending_declarative_window_runtime_snapshots(
        &mut self,
        app: &mut H,
        root: NodeId,
    ) -> bool {
        self.pending_declarative_window_snapshot_roots
            .retain(|pending| self.nodes.contains_key(*pending));
        if !self
            .pending_declarative_window_snapshot_roots
            .contains(&root)
        {
            return false;
        }

        let attached = self.node_is_attached_to_layer_tree(root);
        if !attached {
            return false;
        }

        self.pending_declarative_window_snapshot_roots.remove(&root);
        self.publish_window_runtime_snapshots(app);
        true
    }

    fn focus_menu_bar_command_availability(&self, app: &mut H) -> CommandAvailability {
        let Some(window) = self.window else {
            return CommandAvailability::NotHandled;
        };
        let present = app
            .global::<fret_runtime::WindowMenuBarFocusService>()
            .is_some_and(|svc| svc.present(window));
        if present {
            CommandAvailability::Available
        } else {
            CommandAvailability::NotHandled
        }
    }

    #[stacksafe::stacksafe]
    pub fn is_command_available(&mut self, app: &mut H, command: &CommandId) -> bool {
        self.command_availability(app, command) == CommandAvailability::Available
    }

    /// GPUI naming parity: "is this action available along the dispatch path?"
    ///
    /// Note: Fret models "actions" as `CommandId` today (especially for widget-scoped commands).
    #[stacksafe::stacksafe]
    pub fn is_action_available(&mut self, app: &mut H, command: &CommandId) -> bool {
        self.is_command_available(app, command)
    }

    /// GPUI naming parity for availability queries.
    #[stacksafe::stacksafe]
    pub fn action_availability(&mut self, app: &mut H, command: &CommandId) -> CommandAvailability {
        self.command_availability(app, command)
    }

    #[stacksafe::stacksafe]
    pub fn command_availability(
        &mut self,
        app: &mut H,
        command: &CommandId,
    ) -> CommandAvailability {
        if command.as_str() == "focus.menu_bar" {
            return self.focus_menu_bar_command_availability(app);
        }

        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return CommandAvailability::NotHandled;
        };

        let (_active_input_layers, input_barrier_root) = self.active_input_layers();
        let (active_focus_layers, focus_barrier_root) = self.active_focus_layers();
        let barrier_root = focus_barrier_root.or(input_barrier_root);
        let dispatch_snapshot = self.cached_dispatch_snapshot_for_layer_roots(
            app.frame_id(),
            active_focus_layers.as_slice(),
            barrier_root,
        );
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let mut input_ctx: InputContext = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: input_barrier_root.is_some(),
            window_arbitration: None,
            focus_is_text_input: self.focus_is_text_input(app),
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            router_can_back: false,
            router_can_forward: false,
            dispatch_phase: InputDispatchPhase::Bubble,
        };
        if let Some(window) = self.window {
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
                input_ctx.router_can_back = availability.router_can_back;
                input_ctx.router_can_forward = availability.router_can_forward;
            }
            input_ctx.window_arbitration = Some(self.window_input_arbitration_snapshot());
        }

        if self
            .focus
            .is_some_and(|n| dispatch_snapshot.pre.get(n).is_none())
        {
            self.set_focus_unchecked(None, "commands: focus missing from dispatch snapshot");
        }

        let default_root = barrier_root.unwrap_or(base_root);
        let start = self.focus.unwrap_or(default_root);
        let (mut availability, _) = self.command_availability_from_node(
            app,
            &input_ctx,
            &dispatch_snapshot,
            start,
            command,
            None,
        );
        // When focus lives in a non-default layer (e.g. a non-modal overlay), we still want
        // widget-scoped command availability to fall back to the default root so global shortcuts
        // and menus remain usable.
        if availability == CommandAvailability::NotHandled
            && start != default_root
            && !dispatch_snapshot.is_descendant(default_root, start)
        {
            availability = self
                .command_availability_from_node(
                    app,
                    &input_ctx,
                    &dispatch_snapshot,
                    default_root,
                    command,
                    None,
                )
                .0;
        }

        if availability == CommandAvailability::NotHandled
            && matches!(command.as_str(), "focus.next" | "focus.previous")
        {
            return self.focus_traversal_command_availability(
                app,
                app.frame_id(),
                &dispatch_snapshot,
                barrier_root,
            );
        }

        if availability == CommandAvailability::NotHandled && barrier_root.is_none() {
            availability = self
                .command_availability_in_subtree(app, &input_ctx, base_root, command, None)
                .0;
        }

        availability
    }

    fn focus_traversal_command_availability(
        &mut self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        barrier_root: Option<NodeId>,
    ) -> CommandAvailability {
        self.focus_traversal_command_availability_for_snapshot(
            app,
            frame_id,
            dispatch_snapshot,
            barrier_root,
        )
        .0
    }

    fn focus_traversal_command_availability_for_snapshot(
        &mut self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
    ) -> (CommandAvailability, bool) {
        let (has_focusable, needs_layout_refine) = self.focus_traversal_has_candidate_for_snapshot(
            app,
            frame_id,
            dispatch_snapshot,
            scope_root,
        );

        (
            if has_focusable {
                CommandAvailability::Available
            } else {
                CommandAvailability::NotHandled
            },
            needs_layout_refine && has_focusable,
        )
    }

    fn timed_focus_traversal_command_availability_for_snapshot(
        &mut self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
        command: &CommandId,
        window: AppWindowId,
    ) -> (CommandAvailability, bool) {
        let start_node = scope_root.or(dispatch_snapshot.barrier_root).or_else(|| {
            self.base_layer
                .and_then(|id| self.layers.get(id).map(|l| l.root))
        });
        let start_time = if self.debug_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let (availability, needs_layout_refine) = self
            .focus_traversal_command_availability_for_snapshot(
                app,
                frame_id,
                dispatch_snapshot,
                scope_root,
            );
        if let (Some(start_time), Some(start_node)) = (start_time, start_node) {
            self.debug_record_command_availability_hotspot(
                app,
                window,
                command,
                "focus_traversal_snapshot",
                start_node,
                None,
                availability,
                start_time.elapsed(),
            );
        }
        (availability, needs_layout_refine)
    }

    fn focus_traversal_availability_cache_key_for_snapshot(
        &self,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
    ) -> WindowFocusTraversalAvailabilityCacheKey {
        let resolved_scope_root = scope_root.or(dispatch_snapshot.barrier_root).or_else(|| {
            self.base_layer
                .and_then(|id| self.layers.get(id).map(|l| l.root))
        });
        let layout_ready = match resolved_scope_root {
            Some(scope_root) => {
                self.last_layout_frame_id == Some(frame_id)
                    && !self.node_subtree_layout_dirty(scope_root)
            }
            None => true,
        };

        WindowFocusTraversalAvailabilityCacheKey {
            frame_id,
            dispatch_snapshot_generation: self.dispatch_snapshot_products.generation(),
            dispatch_snapshot_topology_epoch: dispatch_snapshot.topology_epoch,
            window: dispatch_snapshot.window,
            active_layer_roots: dispatch_snapshot.active_layer_roots.clone(),
            barrier_root: dispatch_snapshot.barrier_root,
            scope_root,
            resolved_scope_root,
            command_availability_revision: self
                .command_routing_snapshots
                .command_availability_revision(),
            layout_ready,
            inspection_active: self.inspection_active,
        }
    }

    fn cached_timed_focus_traversal_command_availability_for_snapshot(
        &mut self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
        command: &CommandId,
        window: AppWindowId,
    ) -> (CommandAvailability, bool) {
        let key = self.focus_traversal_availability_cache_key_for_snapshot(
            frame_id,
            dispatch_snapshot,
            scope_root,
        );
        if let Some(entry) = self
            .command_routing_snapshots
            .focus_traversal_availability()
            && entry.key == key
        {
            return (entry.availability, entry.needs_layout_refine);
        }

        let (availability, needs_layout_refine) = self
            .timed_focus_traversal_command_availability_for_snapshot(
                app,
                frame_id,
                dispatch_snapshot,
                scope_root,
                command,
                window,
            );
        self.command_routing_snapshots
            .set_focus_traversal_availability(WindowFocusTraversalAvailabilityCacheEntry {
                key,
                availability,
                needs_layout_refine,
            });
        (availability, needs_layout_refine)
    }

    fn focus_traversal_candidates_for_snapshot(
        &mut self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
    ) -> (Vec<NodeId>, bool) {
        let scope_root = scope_root.or(dispatch_snapshot.barrier_root).or_else(|| {
            self.base_layer
                .and_then(|id| self.layers.get(id).map(|l| l.root))
        });
        let Some(scope_root) = scope_root else {
            return (Vec::new(), false);
        };

        let mut focusables: Vec<NodeId> = Vec::new();
        let needs_layout_refine = self.last_layout_frame_id != Some(frame_id)
            || self.node_subtree_layout_dirty(scope_root);
        if needs_layout_refine {
            for &root in &dispatch_snapshot.active_layer_roots {
                self.collect_focusables_structural(app, root, dispatch_snapshot, &mut focusables);
            }
        } else {
            let scope_bounds = self
                .nodes
                .get(scope_root)
                .map(|n| n.bounds)
                .unwrap_or_default();
            for &root in &dispatch_snapshot.active_layer_roots {
                self.collect_focusables(root, dispatch_snapshot, scope_bounds, &mut focusables);
            }
        }

        (focusables, needs_layout_refine)
    }

    fn focus_traversal_has_candidate_for_snapshot(
        &self,
        app: &mut H,
        frame_id: fret_runtime::FrameId,
        dispatch_snapshot: &UiDispatchSnapshot,
        scope_root: Option<NodeId>,
    ) -> (bool, bool) {
        let scope_root = scope_root.or(dispatch_snapshot.barrier_root).or_else(|| {
            self.base_layer
                .and_then(|id| self.layers.get(id).map(|l| l.root))
        });
        let Some(scope_root) = scope_root else {
            return (false, false);
        };

        let needs_layout_refine = self.last_layout_frame_id != Some(frame_id)
            || self.node_subtree_layout_dirty(scope_root);
        if needs_layout_refine {
            let has_focusable = dispatch_snapshot
                .active_layer_roots
                .iter()
                .any(|&root| self.has_focusable_structural(app, root, dispatch_snapshot));
            return (has_focusable, true);
        }

        let scope_bounds = self
            .nodes
            .get(scope_root)
            .map(|n| n.bounds)
            .unwrap_or_default();
        let has_focusable = dispatch_snapshot
            .active_layer_roots
            .iter()
            .any(|&root| self.has_focusable(root, dispatch_snapshot, scope_bounds));
        (has_focusable, false)
    }

    fn collect_focusables_structural(
        &self,
        app: &mut H,
        node: NodeId,
        dispatch_snapshot: &UiDispatchSnapshot,
        out: &mut Vec<NodeId>,
    ) {
        if dispatch_snapshot.pre.get(node).is_none() {
            return;
        }

        let Some(n) = self.nodes.get(node) else {
            return;
        };

        let (is_focusable, traverse_children) =
            self.structural_focus_traversal_state_for_node(app, node);
        if is_focusable {
            out.push(node);
        }

        if traverse_children {
            for &child in &n.children {
                self.collect_focusables_structural(app, child, dispatch_snapshot, out);
            }
        }
    }

    fn has_focusable_structural(
        &self,
        app: &mut H,
        node: NodeId,
        dispatch_snapshot: &UiDispatchSnapshot,
    ) -> bool {
        if dispatch_snapshot.pre.get(node).is_none() {
            return false;
        }

        let Some(n) = self.nodes.get(node) else {
            return false;
        };

        let (is_focusable, traverse_children) =
            self.structural_focus_traversal_state_for_node(app, node);
        if is_focusable {
            return true;
        }

        traverse_children
            && n.children
                .iter()
                .any(|&child| self.has_focusable_structural(app, child, dispatch_snapshot))
    }

    fn structural_focus_traversal_state_for_node(&self, app: &mut H, node: NodeId) -> (bool, bool) {
        if let Some(window) = self.window
            && let Some((is_focusable, traverse_children)) =
                crate::declarative::frame::with_element_record_for_node(
                    app,
                    window,
                    node,
                    |record| match &record.instance {
                        crate::declarative::frame::ElementInstance::TextInput(_)
                        | crate::declarative::frame::ElementInstance::TextArea(_)
                        | crate::declarative::frame::ElementInstance::TextInputRegion(_) => {
                            (true, true)
                        }
                        crate::declarative::frame::ElementInstance::SelectableText(_) => {
                            (true, true)
                        }
                        crate::declarative::frame::ElementInstance::Pressable(props) => {
                            (props.enabled && props.focusable, props.enabled)
                        }
                        crate::declarative::frame::ElementInstance::Semantics(props) => {
                            (props.focusable && !props.disabled && !props.hidden, true)
                        }
                        crate::declarative::frame::ElementInstance::InteractivityGate(props) => {
                            (false, props.present && props.interactive)
                        }
                        crate::declarative::frame::ElementInstance::FocusTraversalGate(props) => {
                            (false, props.traverse)
                        }
                        crate::declarative::frame::ElementInstance::Spinner(_) => (false, false),
                        _ => (false, true),
                    },
                )
        {
            return (is_focusable, traverse_children);
        }

        let Some(n) = self.nodes.get(node) else {
            return (false, true);
        };
        let prepaint =
            (!self.inspection_active && !n.invalidation.hit_test && !n.invalidation.layout)
                .then_some(n.prepaint_hit_test)
                .flatten();
        (
            prepaint
                .as_ref()
                .map(|p| p.is_focusable)
                .unwrap_or_else(|| n.widget.as_ref().is_some_and(|w| w.is_focusable())),
            prepaint
                .as_ref()
                .map(|p| p.focus_traversal_children)
                .unwrap_or_else(|| {
                    n.widget
                        .as_ref()
                        .map(|w| w.focus_traversal_children())
                        .unwrap_or(true)
                }),
        )
    }

    #[stacksafe::stacksafe]
    fn command_availability_from_node(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        dispatch_snapshot: &UiDispatchSnapshot,
        start: NodeId,
        command: &CommandId,
        mut publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        let mut node_id = start;
        loop {
            let Some(parent) = Self::dispatch_snapshot_parent_for_node(
                dispatch_snapshot,
                node_id,
                "command availability bubble",
            ) else {
                break;
            };
            let availability = self.command_availability_at_node(
                app,
                input_ctx,
                node_id,
                command,
                publication_cache.as_deref_mut(),
            );
            match availability {
                CommandAvailability::Available | CommandAvailability::Blocked => {
                    return (availability, Some(node_id));
                }
                CommandAvailability::NotHandled => {}
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        (CommandAvailability::NotHandled, None)
    }

    fn dispatch_snapshot_parent_for_node(
        snapshot: &UiDispatchSnapshot,
        node: NodeId,
        context: &'static str,
    ) -> Option<Option<NodeId>> {
        if snapshot.pre.get(node).is_none() {
            debug_assert!(
                false,
                "{context}: node missing from dispatch snapshot (node={node:?}, frame_id={:?}, window={:?})",
                snapshot.frame_id, snapshot.window
            );
            return None;
        }
        Some(snapshot.parent.get(node).copied().flatten())
    }

    fn dispatch_command_source_node_in_snapshot(
        snapshot: &UiDispatchSnapshot,
        source_node: Option<NodeId>,
    ) -> Option<NodeId> {
        source_node.filter(|&node| snapshot.pre.get(node).is_some())
    }

    fn command_availability_at_node(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        node_id: NodeId,
        command: &CommandId,
        mut publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> CommandAvailability {
        self.command_availability_at_node_with_interest_route(
            app,
            input_ctx,
            node_id,
            command,
            CommandAvailabilityInterestRoute::DispatchPath,
            publication_cache.as_deref_mut(),
        )
    }

    fn command_availability_at_node_with_interest_route(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        node_id: NodeId,
        command: &CommandId,
        route: CommandAvailabilityInterestRoute,
        mut publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> CommandAvailability {
        let may_handle = if let Some(cache) = publication_cache.as_mut() {
            self.declarative_node_may_handle_command_availability(
                app,
                node_id,
                command,
                route,
                Some(&mut **cache),
            )
        } else {
            self.declarative_node_may_handle_command_availability(
                app, node_id, command, route, None,
            )
        };
        if !may_handle {
            return CommandAvailability::NotHandled;
        }

        #[cfg(test)]
        super::record_command_availability_widget_probe();

        let availability = self.with_widget_mut(node_id, |widget, tree| {
            let window = tree.window;
            let focus = tree.focus;
            let mut cx = CommandAvailabilityCx {
                app,
                tree: &*tree,
                node: node_id,
                window,
                input_ctx: input_ctx.clone(),
                focus,
            };
            widget.command_availability(&mut cx, command)
        });

        availability
    }

    fn declarative_node_may_handle_command_availability(
        &mut self,
        app: &mut H,
        node: NodeId,
        command: &CommandId,
        route: CommandAvailabilityInterestRoute,
        publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> bool {
        let frame_id = app.frame_id();
        if let Some(cache) = publication_cache {
            let interest = if let Some(interest) = cache.declarative_interest.get(&node).cloned() {
                interest.clone()
            } else {
                let interest =
                    self.declarative_node_command_availability_interest_cached(app, frame_id, node);
                cache.declarative_interest.insert(node, interest.clone());
                interest
            };
            return interest.matches_for_route(command, route);
        }

        self.declarative_node_command_availability_interest_cached(app, frame_id, node)
            .matches_for_route(command, route)
    }

    fn declarative_node_command_availability_interest_cached(
        &mut self,
        app: &mut H,
        frame_id: FrameId,
        node: NodeId,
    ) -> DeclarativeCommandAvailabilityInterest {
        let key = WindowCommandAvailabilityInterestCacheKey {
            frame_id,
            command_availability_revision: self
                .command_routing_snapshots
                .command_availability_revision(),
            window: self.window,
        };

        if self
            .command_routing_snapshots
            .command_availability_interest()
            .is_none_or(|cache| cache.key != key)
        {
            self.command_routing_snapshots
                .reset_command_availability_interest(WindowCommandAvailabilityInterestCache {
                    key,
                    by_node: HashMap::new(),
                });
        }

        if let Some(cache) = self
            .command_routing_snapshots
            .command_availability_interest()
            && let Some(interest) = cache.by_node.get(&node)
        {
            return interest.clone();
        }

        let interest = self.declarative_node_command_availability_interest(app, node);
        if let Some(cache) = self
            .command_routing_snapshots
            .command_availability_interest_mut()
        {
            cache.by_node.insert(node, interest.clone());
        }
        interest
    }

    fn declarative_node_command_availability_interest(
        &self,
        app: &mut H,
        node: NodeId,
    ) -> DeclarativeCommandAvailabilityInterest {
        #[cfg(test)]
        super::record_command_availability_interest_probe();

        let Some(window) = self.window else {
            return DeclarativeCommandAvailabilityInterest::all();
        };
        let Some(element) = self.nodes.get(node).and_then(|n| n.element) else {
            return DeclarativeCommandAvailabilityInterest::all();
        };

        let built_in_interest =
            crate::declarative::frame::with_element_record_for_node(app, window, node, |record| {
                match &record.instance {
                    crate::declarative::frame::ElementInstance::ManagedSurface(_) => {
                        DeclarativeCommandAvailabilityInterest::all()
                    }
                    crate::declarative::frame::ElementInstance::SelectableText(_) => {
                        DeclarativeCommandAvailabilityInterest::selectable_text_edit()
                    }
                    crate::declarative::frame::ElementInstance::TextInput(_)
                    | crate::declarative::frame::ElementInstance::TextArea(_) => {
                        DeclarativeCommandAvailabilityInterest::text_edit()
                    }
                    crate::declarative::frame::ElementInstance::FocusScope(props)
                        if props.trap_focus =>
                    {
                        DeclarativeCommandAvailabilityInterest::focus_traversal()
                    }
                    _ => DeclarativeCommandAvailabilityInterest::none(),
                }
            })
            .unwrap_or_else(DeclarativeCommandAvailabilityInterest::all);
        if built_in_interest.all {
            return built_in_interest;
        }

        let action_route_interest = crate::elements::try_with_element_state(
            app,
            window,
            element,
            |hooks: &mut crate::action::ActionRouteHooks| hooks.command_availability_interest(),
        )
        .map(DeclarativeCommandAvailabilityInterest::from)
        .unwrap_or_else(DeclarativeCommandAvailabilityInterest::none);
        let legacy_interest = if crate::elements::try_with_element_state(
            app,
            window,
            element,
            |hooks: &mut crate::action::CommandAvailabilityActionHooks| {
                hooks.on_command_availability.is_some()
            },
        )
        .unwrap_or(false)
        {
            DeclarativeCommandAvailabilityInterest::all()
        } else {
            DeclarativeCommandAvailabilityInterest::none()
        };

        if legacy_interest.all {
            return legacy_interest;
        }

        built_in_interest
            .union(action_route_interest)
            .union(legacy_interest)
    }

    fn timed_command_availability_from_node(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        dispatch_snapshot: &UiDispatchSnapshot,
        start: NodeId,
        command: &CommandId,
        route: &'static str,
        window: AppWindowId,
        publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        let start_time = if self.debug_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let (availability, resolved_node) = self.command_availability_from_node(
            app,
            input_ctx,
            dispatch_snapshot,
            start,
            command,
            publication_cache,
        );
        if let Some(start_time) = start_time {
            self.debug_record_command_availability_hotspot(
                app,
                window,
                command,
                route,
                start,
                resolved_node,
                availability,
                start_time.elapsed(),
            );
        }
        (availability, resolved_node)
    }

    fn command_availability_in_subtree(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        root: NodeId,
        command: &CommandId,
        publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        self.command_availability_in_subtree_with_interest_route(
            app,
            input_ctx,
            root,
            command,
            CommandAvailabilityInterestRoute::DispatchPath,
            publication_cache,
        )
    }

    fn command_availability_in_subtree_with_interest_route(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        root: NodeId,
        command: &CommandId,
        route: CommandAvailabilityInterestRoute,
        mut publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        let no_focus_interest = if route == CommandAvailabilityInterestRoute::NoFocusSubtreeFallback
        {
            if let Some(cache) = publication_cache.as_deref_mut() {
                self.populate_subtree_command_availability_interest_cache(app, root, cache);
                cache.subtree_interest.get(&root).cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(interest) = &no_focus_interest
            && !interest.matches_for_route(command, route)
        {
            return (CommandAvailability::NotHandled, None);
        }

        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if let Some(entry) = self.nodes.get(node) {
                for &child in entry.children.iter().rev() {
                    let child_matches = if no_focus_interest.is_some() {
                        publication_cache
                            .as_deref()
                            .and_then(|cache| cache.subtree_interest.get(&child))
                            .is_some_and(|interest| interest.matches_for_route(command, route))
                    } else {
                        true
                    };
                    if child_matches {
                        stack.push(child);
                    }
                }
            }

            let availability = self.command_availability_at_node_with_interest_route(
                app,
                input_ctx,
                node,
                command,
                route,
                publication_cache.as_deref_mut(),
            );
            match availability {
                CommandAvailability::Available => {
                    return (availability, Some(node));
                }
                CommandAvailability::Blocked => return (availability, None),
                CommandAvailability::NotHandled => {}
            }
        }

        (CommandAvailability::NotHandled, None)
    }

    fn timed_no_focus_command_availability_in_subtree(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        root: NodeId,
        command: &CommandId,
        window: AppWindowId,
        publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        let start_time = if self.debug_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let (availability, resolved_node) = self
            .command_availability_in_subtree_with_interest_route(
                app,
                input_ctx,
                root,
                command,
                CommandAvailabilityInterestRoute::NoFocusSubtreeFallback,
                publication_cache,
            );
        if let Some(start_time) = start_time {
            self.debug_record_command_availability_hotspot(
                app,
                window,
                command,
                "subtree_no_focus_fallback",
                root,
                resolved_node,
                availability,
                start_time.elapsed(),
            );
        }
        (availability, resolved_node)
    }

    fn command_availability_in_action_route_fallback_roots(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        dispatch_snapshot: &UiDispatchSnapshot,
        command: &CommandId,
    ) -> (CommandAvailability, Option<NodeId>) {
        let Some(window) = self.window else {
            return (CommandAvailability::NotHandled, None);
        };

        let roots = crate::elements::action_route_fallback_roots(app, window);
        let (availability, resolved_node, _) = self
            .command_availability_in_action_route_fallback_root_elements(
                app,
                input_ctx,
                dispatch_snapshot,
                command,
                window,
                roots,
                None,
            );
        (availability, resolved_node)
    }

    fn command_availability_in_action_route_fallback_root_elements(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        dispatch_snapshot: &UiDispatchSnapshot,
        command: &CommandId,
        window: AppWindowId,
        roots: impl IntoIterator<Item = GlobalElementId>,
        mut publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>, Option<NodeId>) {
        let mut first_resolved_root = None;
        for element in roots {
            let Some(node) =
                self.resolve_live_attached_node_for_element(app, Some(window), element)
            else {
                continue;
            };
            first_resolved_root.get_or_insert(node);
            let (availability, resolved_node) = self.command_availability_from_node(
                app,
                input_ctx,
                dispatch_snapshot,
                node,
                command,
                publication_cache.as_deref_mut(),
            );
            match availability {
                CommandAvailability::Available => {
                    return (
                        availability,
                        resolved_node.or(Some(node)),
                        first_resolved_root,
                    );
                }
                CommandAvailability::Blocked => return (availability, None, first_resolved_root),
                CommandAvailability::NotHandled => {}
            }
        }

        (CommandAvailability::NotHandled, None, first_resolved_root)
    }

    fn timed_command_availability_in_action_route_fallback_roots(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        command: &CommandId,
        route: &'static str,
        window: AppWindowId,
        dispatch_snapshot: &UiDispatchSnapshot,
        publication_cache: Option<&mut CommandAvailabilityPublicationCache>,
    ) -> (CommandAvailability, Option<NodeId>) {
        let start_time = if self.debug_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let (availability, resolved_node, start_node) = if let Some(active_window) = self.window {
            let roots = crate::elements::action_route_fallback_roots(app, active_window);
            self.command_availability_in_action_route_fallback_root_elements(
                app,
                input_ctx,
                dispatch_snapshot,
                command,
                active_window,
                roots,
                publication_cache,
            )
        } else {
            (CommandAvailability::NotHandled, None, None)
        };
        let start_node = start_node.or_else(|| {
            self.base_layer
                .and_then(|id| self.layers.get(id).map(|l| l.root))
        });
        if let (Some(start_time), Some(start_node)) = (start_time, start_node) {
            self.debug_record_command_availability_hotspot(
                app,
                window,
                command,
                route,
                start_node,
                resolved_node,
                availability,
                start_time.elapsed(),
            );
        }
        (availability, resolved_node)
    }

    /// Publish a per-window action availability snapshot for widget-scoped commands.
    ///
    /// This is a data-only integration seam for runner/platform and UI-kit layers (menus, command
    /// palette, shortcut help). Most apps should prefer publishing a filtered snapshot (e.g. only
    /// menu/palette command sets) at the app-driver layer.
    ///
    /// Notes:
    /// - This retained-runtime helper publishes a conservative baseline: for each widget-scoped
    ///   command in the registry, `NotHandled` is treated as "unavailable" (`false`) so
    ///   cross-surface gating behaves consistently.
    /// - Explicit action-route fallback roots participate after the focused/default route. This
    ///   keeps app/view-level typed action handlers available to menus, palettes, and overlays
    ///   without scanning arbitrary unfocused widget subtrees.
    /// - The no-focus, no-barrier case is allowed to use the same subtree route fallback as
    ///   dispatch. This keeps first-open command palette/menu discovery in sync with commands that
    ///   are registered on retained action roots before a focus target exists.
    pub fn publish_window_command_action_availability_snapshot(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
    ) {
        self.publish_window_command_action_availability_snapshot_for_command_set(
            app,
            input_ctx,
            WindowCommandActionAvailabilityCommandSetSignature::AllRegisteredWidgetCommands,
        );
    }

    /// Publish a per-window action availability snapshot for a caller-owned command set.
    ///
    /// This is intended for app/driver layers that know the exact command family consumed by a
    /// surface. Missing commands remain "unknown" to `WindowCommandActionAvailabilityService`, not
    /// disabled, so this should not replace the conservative full-window publisher unless the
    /// consumer owns the filtered set.
    pub fn publish_window_command_action_availability_snapshot_filtered(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        commands: impl IntoIterator<Item = CommandId>,
    ) {
        let mut commands = commands.into_iter().collect::<Vec<_>>();
        commands.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        commands.dedup_by(|a, b| a.as_str() == b.as_str());
        self.publish_window_command_action_availability_snapshot_for_command_set(
            app,
            input_ctx,
            WindowCommandActionAvailabilityCommandSetSignature::FilteredWidgetCommands(commands),
        );
    }

    fn publish_window_command_action_availability_snapshot_for_command_set(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        command_set: WindowCommandActionAvailabilityCommandSetSignature,
    ) {
        let Some(window) = self.window else {
            self.command_routing_snapshots
                .clear_action_availability_signature();
            return;
        };

        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            self.command_routing_snapshots
                .clear_action_availability_signature();
            return;
        };
        let (_active_input_layers, input_barrier_root) = self.active_input_layers();
        let (active_focus_layers, focus_barrier_root) = self.active_focus_layers();
        let barrier_root = focus_barrier_root.or(input_barrier_root);
        let frame_id = app.frame_id();
        let dispatch_snapshot = self.cached_dispatch_snapshot_for_layer_roots(
            frame_id,
            active_focus_layers.as_slice(),
            barrier_root,
        );
        self.revalidate_focus_for_dispatch_snapshot(
            frame_id,
            active_focus_layers.as_slice(),
            barrier_root,
            "commands: focus missing from dispatch snapshot",
        );

        let default_root = barrier_root.unwrap_or(base_root);
        let focus = self.focus;
        let focus_in_default_root =
            focus.is_some_and(|n| dispatch_snapshot.is_descendant(default_root, n));
        let start = focus.unwrap_or(default_root);
        let next_key_contexts = self.shortcut_key_context_stack(app, barrier_root);
        let menu_bar_present = app
            .global::<fret_runtime::WindowMenuBarFocusService>()
            .is_some_and(|svc| svc.present(window));
        let command_registry_revision = app.commands().revision();
        let mut pending_declarative_roots = self
            .pending_declarative_window_snapshot_roots
            .iter()
            .copied()
            .collect::<Vec<_>>();
        pending_declarative_roots.sort_by_key(|node| node.data().as_ffi());
        let snapshot_signature = WindowCommandActionAvailabilitySnapshotSignature {
            window: Some(window),
            base_root: Some(base_root),
            active_focus_layers: active_focus_layers.clone(),
            barrier_root,
            focus,
            pending: WindowRuntimeSnapshotPendingSignature {
                declarative_roots: pending_declarative_roots,
                post_layout_refine_frame: self
                    .pending_post_layout_window_runtime_snapshot_refine
                    .then_some(frame_id),
            },
            commands: command_set.clone(),
            command_availability_revision: self
                .command_routing_snapshots
                .command_availability_revision(),
            input_ctx: WindowCommandActionAvailabilityInputSignature::from(input_ctx),
            key_contexts: next_key_contexts.clone(),
            command_registry_revision,
            menu_bar_present,
        };
        if self
            .command_routing_snapshots
            .action_availability_signature()
            .is_some_and(|prev| prev == &snapshot_signature)
        {
            return;
        }
        self.publish_window_key_context_stack_snapshot(app, next_key_contexts);
        let trace_runtime_snapshot = tracing::enabled!(tracing::Level::TRACE);
        let time_enabled = self.debug_enabled;

        let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
        let (widget_commands, collect_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.command_registry_collect",
                    window = ?window,
                    frame_id = frame_id.0,
                    command_registry_revision,
                )
            },
            || match &command_set {
                WindowCommandActionAvailabilityCommandSetSignature::AllRegisteredWidgetCommands => {
                    app.commands()
                        .iter()
                        .filter_map(|(id, meta)| {
                            (meta.scope == CommandScope::Widget).then_some(id.clone())
                        })
                        .collect::<Vec<_>>()
                }
                WindowCommandActionAvailabilityCommandSetSignature::FilteredWidgetCommands(
                    commands,
                ) => commands
                    .iter()
                    .filter(|id| {
                        app.commands()
                            .get((*id).clone())
                            .is_some_and(|meta| meta.scope == CommandScope::Widget)
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            },
        );
        if let Some(collect_elapsed) = collect_elapsed {
            self.debug_stats
                .window_runtime_snapshot_command_registry_collect_time += collect_elapsed;
        }
        let widget_command_count = widget_commands.len().min(u32::MAX as usize) as u32;
        if self.debug_enabled {
            self.debug_stats
                .window_runtime_snapshot_widget_command_count = widget_command_count;
        }
        let mut publication_cache = CommandAvailabilityPublicationCache::default();

        let (_, eval_elapsed) = fret_perf::measure_span(
            time_enabled,
            trace_runtime_snapshot,
            || {
                tracing::trace_span!(
                    "fret.ui.window_runtime_snapshot.command_availability_eval",
                    window = ?window,
                    frame_id = frame_id.0,
                    widget_command_count,
                )
            },
            || {
                for id in widget_commands {
                    if id.as_str() == "focus.menu_bar" {
                        let present = app
                            .global::<fret_runtime::WindowMenuBarFocusService>()
                            .is_some_and(|svc| svc.present(window));
                        snapshot.insert(id, present);
                        continue;
                    }

                    let (mut availability, _) = self.timed_command_availability_from_node(
                        app,
                        input_ctx,
                        &dispatch_snapshot,
                        start,
                        &id,
                        "focused_or_default",
                        window,
                        Some(&mut publication_cache),
                    );
                    if availability == CommandAvailability::NotHandled
                        && focus.is_some()
                        && !focus_in_default_root
                        && start != default_root
                    {
                        availability = self
                            .timed_command_availability_from_node(
                                app,
                                input_ctx,
                                &dispatch_snapshot,
                                default_root,
                                &id,
                                "default_root_fallback",
                                window,
                                Some(&mut publication_cache),
                            )
                            .0;
                    }
                    if availability == CommandAvailability::NotHandled
                        && matches!(id.as_str(), "focus.next" | "focus.previous")
                    {
                        let (focus_traversal_availability, needs_layout_refine) = self
                            .cached_timed_focus_traversal_command_availability_for_snapshot(
                                app,
                                frame_id,
                                &dispatch_snapshot,
                                barrier_root,
                                &id,
                                window,
                            );
                        availability = focus_traversal_availability;
                        if needs_layout_refine {
                            self.pending_post_layout_window_runtime_snapshot_refine = true;
                        }
                    }
                    if availability == CommandAvailability::NotHandled && barrier_root.is_none() {
                        availability = self
                            .timed_command_availability_in_action_route_fallback_roots(
                                app,
                                input_ctx,
                                &id,
                                "action_route_fallback_roots",
                                window,
                                &dispatch_snapshot,
                                Some(&mut publication_cache),
                            )
                            .0;
                    }
                    // Cross-surface action availability is dispatch-path availability. Whole-subtree
                    // fallback is intentionally excluded once focus exists: it can mark actions available
                    // from unfocused widgets and turns snapshot publication into commands * nodes * depth
                    // work. Explicit action-route fallback roots above cover view/app-level typed action
                    // handlers without weakening that contract. Before any focus target exists, match the
                    // actual dispatch fallback so first-open discovery surfaces do not disable app-level
                    // action roots.
                    if availability == CommandAvailability::NotHandled
                        && focus.is_none()
                        && barrier_root.is_none()
                        && !command_is_focus_bound_text_edit(&id)
                    {
                        availability = self
                            .timed_no_focus_command_availability_in_subtree(
                                app,
                                input_ctx,
                                base_root,
                                &id,
                                window,
                                Some(&mut publication_cache),
                            )
                            .0;
                    }

                    match availability {
                        CommandAvailability::Available => {
                            snapshot.insert(id, true);
                        }
                        CommandAvailability::Blocked => {
                            snapshot.insert(id, false);
                        }
                        CommandAvailability::NotHandled => {
                            // For widget-scoped commands, “not handled anywhere on the dispatch path”
                            // means “not available” (disabled) for cross-surface gating (menus, palettes,
                            // shortcuts).
                            snapshot.insert(id, false);
                        }
                    }
                }
            },
        );
        if let Some(eval_elapsed) = eval_elapsed {
            self.debug_stats
                .window_runtime_snapshot_command_availability_eval_time += eval_elapsed;
        }

        let needs_update = app
            .global::<fret_runtime::WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_none_or(|prev| prev != &snapshot);
        if needs_update {
            app.with_global_mut(
                fret_runtime::WindowCommandActionAvailabilityService::default,
                |svc, _app| {
                    svc.set_snapshot(window, snapshot);
                },
            );
        }
        self.command_routing_snapshots
            .set_action_availability_signature(snapshot_signature);
    }

    pub(in crate::tree) fn refine_pending_window_runtime_snapshots_after_layout(
        &mut self,
        app: &mut H,
    ) {
        self.pending_declarative_window_snapshot_roots
            .retain(|pending| self.nodes.contains_key(*pending));
        let attached_pending = self
            .pending_declarative_window_snapshot_roots
            .iter()
            .copied()
            .filter(|&root| self.node_is_attached_to_layer_tree(root))
            .collect::<Vec<_>>();
        let had_attached_pending = !attached_pending.is_empty();
        for root in attached_pending {
            self.pending_declarative_window_snapshot_roots.remove(&root);
        }

        let had_pending_refine =
            std::mem::take(&mut self.pending_post_layout_window_runtime_snapshot_refine);
        if !had_pending_refine && !had_attached_pending {
            return;
        }
        if had_pending_refine || had_attached_pending {
            self.command_routing_snapshots
                .clear_action_availability_signature();
        }
        self.publish_window_runtime_snapshots(app);
    }

    #[stacksafe::stacksafe]
    pub fn dispatch_command(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        command: &CommandId,
    ) -> bool {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return false;
        };

        let (_active_input_layers, input_barrier_root) = self.active_input_layers();
        let (active_focus_layers, focus_barrier_root) = self.active_focus_layers();
        let barrier_root = focus_barrier_root.or(input_barrier_root);
        let dispatch_snapshot = self.cached_dispatch_snapshot_for_layer_roots(
            app.frame_id(),
            active_focus_layers.as_slice(),
            barrier_root,
        );
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let mut input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: input_barrier_root.is_some(),
            window_arbitration: None,
            focus_is_text_input: self.focus_is_text_input(app),
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            router_can_back: false,
            router_can_forward: false,
            dispatch_phase: InputDispatchPhase::Bubble,
        };
        if let Some(window) = self.window {
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
                input_ctx.router_can_back = availability.router_can_back;
                input_ctx.router_can_forward = availability.router_can_forward;
            }

            let window_arbitration = self.window_input_arbitration_snapshot();
            input_ctx.window_arbitration = Some(window_arbitration);

            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != &input_ctx);
            if needs_update {
                app.with_global_mut(
                    fret_runtime::WindowInputContextService::default,
                    |svc, _app| {
                        svc.set_snapshot(window, input_ctx.clone());
                    },
                );
            }
        }
        let is_focus_traversal_command =
            matches!(command.as_str(), "focus.next" | "focus.previous");

        if self
            .focus
            .is_some_and(|n| dispatch_snapshot.pre.get(n).is_none())
        {
            self.set_focus_unchecked(None, "commands: focus missing from dispatch snapshot");
        }
        self.revalidate_pending_shortcut_for_current_routing_context(app, barrier_root);

        let default_root = barrier_root.unwrap_or(base_root);
        let focus = self.focus;

        let source = if let Some(window) = self.window {
            app.with_global_mut(
                fret_runtime::WindowPendingCommandDispatchSourceService::default,
                |svc, app| {
                    svc.consume(window, app.tick_id(), command)
                        .unwrap_or_else(fret_runtime::CommandDispatchSourceV1::programmatic)
                },
            )
        } else {
            fret_runtime::CommandDispatchSourceV1::programmatic()
        };

        let source_node = source.element.and_then(|element| {
            self.resolve_live_attached_node_for_element(
                app,
                self.window,
                crate::GlobalElementId(element),
            )
        });
        let source_node =
            Self::dispatch_command_source_node_in_snapshot(&dispatch_snapshot, source_node);

        let start = source_node.or(focus).unwrap_or(default_root);
        let start_in_default_root =
            start == default_root || dispatch_snapshot.is_descendant(default_root, start);
        let base_dispatch_roots = [base_root];
        let base_dispatch_snapshot = self.cached_dispatch_snapshot_for_layer_roots(
            app.frame_id(),
            &base_dispatch_roots,
            None,
        );
        let action_route_fallback = {
            let (availability, route_node) = self
                .command_availability_in_action_route_fallback_roots(
                    app,
                    &input_ctx,
                    &base_dispatch_snapshot,
                    command,
                );
            (availability == CommandAvailability::Available)
                .then_some(route_node)
                .flatten()
        };
        let descendant_fallback_route = if barrier_root.is_none() && action_route_fallback.is_none()
        {
            let (availability, route_node) =
                self.command_availability_in_subtree(app, &input_ctx, base_root, command, None);
            (availability == CommandAvailability::Available)
                .then_some(route_node)
                .flatten()
        } else {
            None
        };

        let mut bubble_from = |dispatch_snapshot: &UiDispatchSnapshot,
                               start: NodeId|
         -> (bool, bool, bool, Option<NodeId>) {
            let mut node_id = start;
            let mut handled = false;
            let mut needs_redraw = false;
            let mut stopped = false;
            let mut handled_by_node: Option<NodeId> = None;

            loop {
                let (
                    did_handle,
                    invalidations,
                    requested_focus,
                    notify_requested,
                    notify_requested_location,
                    stop_bubbling,
                ) = self.with_widget_mut(node_id, |widget, tree| {
                    let window = tree.window;
                    let focus = tree.focus;
                    let mut cx = CommandCx {
                        app,
                        services: &mut *services,
                        tree,
                        node: node_id,
                        window,
                        input_ctx: input_ctx.clone(),
                        focus,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        notify_requested: false,
                        notify_requested_location: None,
                        stop_propagation: false,
                    };
                    let did_handle = widget.command(&mut cx, command);
                    (
                        did_handle,
                        cx.invalidations,
                        cx.requested_focus,
                        cx.notify_requested,
                        cx.notify_requested_location,
                        cx.stop_propagation,
                    )
                });

                if did_handle {
                    handled = true;
                    handled_by_node = handled_by_node.or(Some(node_id));
                }

                if !invalidations.is_empty() || requested_focus.is_some() || notify_requested {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if notify_requested {
                    self.debug_record_notify_request(
                        app.frame_id(),
                        node_id,
                        notify_requested_location,
                    );
                    self.mark_invalidation_with_source(
                        node_id,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Notify,
                    );
                    needs_redraw = true;
                }

                if let Some(focus) = requested_focus {
                    let (active_roots, barrier_root) = self.active_input_layers();
                    let snapshot = self.cached_dispatch_snapshot_for_layer_roots(
                        app.frame_id(),
                        active_roots.as_slice(),
                        barrier_root,
                    );
                    if self.focus_request_is_allowed(
                        app,
                        self.window,
                        &active_roots,
                        focus,
                        Some(&snapshot),
                    ) {
                        if let Some(prev) = self.focus {
                            self.mark_invalidation(prev, Invalidation::Paint);
                        }
                        self.focus = Some(focus);
                        self.mark_invalidation(focus, Invalidation::Paint);
                    }
                }

                if did_handle {
                    break;
                }
                if stop_bubbling {
                    stopped = true;
                    break;
                }

                let Some(parent) = Self::dispatch_snapshot_parent_for_node(
                    &dispatch_snapshot,
                    node_id,
                    "command dispatch bubble",
                ) else {
                    break;
                };
                node_id = match parent {
                    Some(parent) => parent,
                    None => break,
                };
            }

            (handled, needs_redraw, stopped, handled_by_node)
        };

        let (mut handled, mut needs_redraw, mut stopped, mut handled_by_node) =
            bubble_from(&dispatch_snapshot, start);
        let mut used_default_root_fallback = false;
        if !handled && !stopped && start != default_root && !start_in_default_root {
            used_default_root_fallback = true;
            let (handled2, needs_redraw2, stopped2, handled_by_node2) =
                bubble_from(&dispatch_snapshot, default_root);
            handled = handled || handled2;
            needs_redraw = needs_redraw || needs_redraw2;
            stopped = stopped || stopped2;
            handled_by_node = handled_by_node.or(handled_by_node2);
        }

        if !handled
            && !stopped
            && let Some(route_node) = action_route_fallback
        {
            used_default_root_fallback = true;
            let (handled2, needs_redraw2, stopped2, handled_by_node2) =
                bubble_from(&base_dispatch_snapshot, route_node);
            handled = handled || handled2;
            needs_redraw = needs_redraw || needs_redraw2;
            stopped = stopped || stopped2;
            handled_by_node = handled_by_node.or(handled_by_node2);
        }

        if !handled
            && !stopped
            && barrier_root.is_none()
            && let Some(route_node) = descendant_fallback_route
        {
            used_default_root_fallback = true;
            let (handled2, needs_redraw2, stopped2, handled_by_node2) =
                bubble_from(&base_dispatch_snapshot, route_node);
            handled = handled || handled2;
            needs_redraw = needs_redraw || needs_redraw2;
            stopped = stopped || stopped2;
            handled_by_node = handled_by_node.or(handled_by_node2);
        }

        if !handled && !stopped && is_focus_traversal_command {
            handled = self.dispatch_focus_traversal(
                app,
                command,
                active_focus_layers.as_slice(),
                barrier_root,
            );
            needs_redraw = true;
        }

        if needs_redraw {
            self.request_redraw_coalesced(app);
        }

        // Publish a post-dispatch snapshot so runner-level integration surfaces (e.g. OS menubars)
        // see the latest focus/modal state without waiting for the next paint pass.
        if let Some(window) = self.window {
            let (_active_layers, input_barrier_root) = self.active_input_layers();
            let (_active_focus_layers, focus_barrier_root) = self.active_focus_layers();
            let barrier_root = focus_barrier_root.or(input_barrier_root);
            self.revalidate_pending_shortcut_for_current_routing_context(app, barrier_root);
            let caps = app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let mut input_ctx = InputContext {
                platform: Platform::current(),
                caps,
                ui_has_modal: input_barrier_root.is_some(),
                window_arbitration: None,
                focus_is_text_input: self.focus_is_text_input(app),
                text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
                edit_can_undo: true,
                edit_can_redo: true,
                router_can_back: false,
                router_can_forward: false,
                dispatch_phase: InputDispatchPhase::Bubble,
            };
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
                input_ctx.router_can_back = availability.router_can_back;
                input_ctx.router_can_forward = availability.router_can_forward;
            }

            let window_arbitration = self.window_input_arbitration_snapshot();
            input_ctx.window_arbitration = Some(window_arbitration);

            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != &input_ctx);
            if needs_update {
                app.with_global_mut(
                    fret_runtime::WindowInputContextService::default,
                    |svc, _app| {
                        svc.set_snapshot(window, input_ctx.clone());
                    },
                );
            }

            self.publish_window_command_action_availability_snapshot(app, &input_ctx);
            self.refresh_pending_shortcut_overlay_state_if_needed(app, &input_ctx);
        }

        if let Some(window) = self.window {
            let handled_by_element = handled_by_node
                .and_then(|node| self.node_element(node))
                .map(|id| id.0);
            let started_from_focus = focus.is_some();

            app.with_global_mut(
                fret_runtime::WindowCommandDispatchDiagnosticsStore::default,
                |store, app| {
                    let handled_by_scope = if handled {
                        Some(fret_runtime::CommandScope::Widget)
                    } else {
                        None
                    };
                    store.record(fret_runtime::CommandDispatchDecisionV1 {
                        seq: 0,
                        frame_id: app.frame_id(),
                        tick_id: app.tick_id(),
                        window,
                        command: command.clone(),
                        source,
                        handled,
                        handled_by_element,
                        handled_by_scope,
                        handled_by_driver: false,
                        stopped,
                        started_from_focus,
                        used_default_root_fallback,
                    });
                },
            );
        }

        handled
    }

    fn dispatch_focus_traversal(
        &mut self,
        app: &mut H,
        command: &CommandId,
        active_focus_layers: &[NodeId],
        scope_root: Option<NodeId>,
    ) -> bool {
        let direction = match command.as_str() {
            "focus.next" => Some(true),
            "focus.previous" => Some(false),
            _ => None,
        };
        let Some(forward) = direction else {
            return false;
        };

        self.focus_traverse_in_roots(app, active_focus_layers, forward, scope_root)
    }

    /// Focus traversal mechanism used by both the runtime default and component-owned focus scopes.
    ///
    /// Notes:
    /// - `roots` are treated as the authoritative traversal roots for this dispatch path.
    /// - `scope_root` gates authoritative geometry clipping when layout is current.
    /// - This is intentionally conservative until we formalize a scroll-into-view contract (ADR 0068).
    pub fn focus_traverse_in_roots(
        &mut self,
        app: &mut H,
        roots: &[NodeId],
        forward: bool,
        scope_root: Option<NodeId>,
    ) -> bool {
        let dispatch_snapshot =
            self.cached_dispatch_snapshot_for_layer_roots(app.frame_id(), roots, scope_root);
        let (focusables, _) = self.focus_traversal_candidates_for_snapshot(
            app,
            app.frame_id(),
            &dispatch_snapshot,
            scope_root,
        );
        if focusables.is_empty() {
            return true;
        }

        let next = match self
            .focus
            .and_then(|f| focusables.iter().position(|n| *n == f))
        {
            Some(idx) => {
                if forward {
                    focusables[(idx + 1) % focusables.len()]
                } else {
                    focusables[(idx + focusables.len() - 1) % focusables.len()]
                }
            }
            None => {
                if forward {
                    focusables[0]
                } else {
                    focusables[focusables.len() - 1]
                }
            }
        };

        if self.focus != Some(next) {
            if let Some(prev) = self.focus {
                self.mark_invalidation(prev, Invalidation::Paint);
            }
            self.focus = Some(next);
            self.mark_invalidation(next, Invalidation::Paint);
            self.scroll_node_into_view(app, next);
        }
        self.request_redraw_coalesced(app);
        true
    }
    pub fn scroll_node_into_view(&mut self, app: &mut H, target: NodeId) -> bool {
        let Some(target_bounds) = self.nodes.get(target).map(|n| n.bounds) else {
            return false;
        };

        // Only scroll *ancestors* of the target into view.
        //
        // If the target itself is scrollable, attempting to scroll it “into view” via itself can
        // incorrectly mutate its offset (e.g. resetting a virtual list to top when it receives
        // focus).
        let mut node = self.parent_in_layer_forest_via_children(target);
        let mut any_scrolled = false;
        let mut descendant_bounds = target_bounds;
        while let Some(id) = node {
            let parent = self.parent_in_layer_forest_via_children(id);
            node = parent;

            let Some(bounds) = self.nodes.get(id).map(|n| n.bounds) else {
                continue;
            };

            let Some(widget) = self.nodes.get(id).and_then(|n| n.widget.as_ref()) else {
                continue;
            };
            if !widget.can_scroll_descendant_into_view() {
                continue;
            }

            let result = self.with_widget_mut(id, |widget, tree| {
                let mut cx = crate::widget::ScrollIntoViewCx {
                    app,
                    node: id,
                    window: tree.window,
                    bounds,
                };
                widget.scroll_descendant_into_view(&mut cx, descendant_bounds)
            });

            if let crate::widget::ScrollIntoViewResult::Handled {
                did_scroll,
                propagated_bounds,
            } = result
            {
                if did_scroll {
                    any_scrolled = true;
                    self.mark_invalidation(id, Invalidation::HitTest);
                    if self.focus == Some(target)
                        && self
                            .nodes
                            .get(target)
                            .and_then(|n| n.widget.as_ref())
                            .is_some_and(|w| w.is_text_input())
                    {
                        self.mark_invalidation(target, Invalidation::Paint);
                    }
                    self.request_redraw_coalesced(app);
                }
                // Once an ancestor handles the request, outer ancestors should align that
                // ancestor's effective viewport rather than the original deep target bounds.
                descendant_bounds = propagated_bounds.unwrap_or(bounds);
                continue;
            }
        }

        any_scrolled
    }

    pub fn scroll_by(&mut self, app: &mut H, target: NodeId, delta: Point) -> bool {
        let Some(bounds) = self.nodes.get(target).map(|n| n.bounds) else {
            return false;
        };

        let result = self.with_widget_mut(target, |widget, tree| {
            let mut cx = crate::widget::ScrollByCx {
                app,
                node: target,
                window: tree.window,
                bounds,
            };
            widget.scroll_by(&mut cx, delta)
        });

        match result {
            crate::widget::ScrollByResult::NotHandled => false,
            crate::widget::ScrollByResult::Handled { did_scroll } => {
                if did_scroll {
                    self.mark_invalidation(target, Invalidation::HitTestOnly);
                    self.request_redraw_coalesced(app);
                }
                did_scroll
            }
        }
    }
}
