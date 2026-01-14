//! Viewport tool arbitration helpers (Tier A embedding).
//!
//! This module provides a small, policy-heavy router for editor-style viewport tooling:
//! - gizmos,
//! - selection tools,
//! - camera navigation tools.
//!
//! It is built on top of the policy-light protocol types in `fret-viewport-tooling` (ADR 0168).

use std::cmp::Reverse;

use fret_core::{MouseButton, ViewportInputEvent, ViewportInputKind};
use fret_viewport_tooling::{
    ViewportTool, ViewportToolCx, ViewportToolId, ViewportToolInput, ViewportToolPriority,
    ViewportToolResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportToolCoordinateSpace {
    /// Use render-target pixels for cursor coordinates (recommended for 3D gizmos).
    TargetPx,
    /// Use window logical pixels for cursor coordinates (useful for HUD-ish tools).
    ScreenPx,
}

impl Default for ViewportToolCoordinateSpace {
    fn default() -> Self {
        Self::TargetPx
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportToolArbitratorConfig {
    pub primary_button: MouseButton,
    pub coordinate_space: ViewportToolCoordinateSpace,
}

impl Default for ViewportToolArbitratorConfig {
    fn default() -> Self {
        Self {
            primary_button: MouseButton::Left,
            coordinate_space: ViewportToolCoordinateSpace::TargetPx,
        }
    }
}

#[derive(Default)]
pub struct ViewportToolArbitrator {
    pub config: ViewportToolArbitratorConfig,
    tools: Vec<Box<dyn ViewportTool>>,
    hot: Option<ViewportToolId>,
    active: Option<ViewportToolId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ViewportToolRouterState {
    pub hot: Option<ViewportToolId>,
    pub active: Option<ViewportToolId>,
}

pub struct ViewportToolEntry<T> {
    pub id: ViewportToolId,
    pub priority: ViewportToolPriority,
    pub set_hot: Option<fn(&mut T, bool)>,
    pub hit_test: fn(&mut T, ViewportToolCx<'_>) -> bool,
    pub handle_event: fn(&mut T, ViewportToolCx<'_>, bool, bool) -> ViewportToolResult,
    pub cancel: Option<fn(&mut T)>,
}

pub fn route_viewport_tools<T>(
    state: &mut ViewportToolRouterState,
    config: ViewportToolArbitratorConfig,
    host: &mut T,
    event: &ViewportInputEvent,
    tools: &mut [ViewportToolEntry<T>],
) -> bool {
    if tools.is_empty() {
        state.hot = None;
        state.active = None;
        return false;
    }

    tools.sort_by_key(|t| Reverse(t.priority.0));

    let input = derive_input(config, event);
    let cx = ViewportToolCx { event, input };

    if let Some(active) = state.active {
        force_hot(state, host, tools, active);
        let handled = if let Some(entry) = tools.iter_mut().find(|t| t.id == active) {
            (entry.handle_event)(host, cx, true, true).handled
        } else {
            false
        };

        if !cx.input.dragging {
            state.active = None;
        }
        return handled;
    }

    match event.kind {
        ViewportInputKind::PointerMove { .. }
        | ViewportInputKind::PointerDown { .. }
        | ViewportInputKind::Wheel { .. } => update_hot(state, host, tools, cx),
        ViewportInputKind::PointerUp { .. } => {}
    }

    match event.kind {
        ViewportInputKind::PointerDown { .. } => dispatch_pointer_down(state, host, tools, cx),
        ViewportInputKind::PointerMove { .. } | ViewportInputKind::PointerUp { .. } => {
            dispatch_hot_only(state, host, tools, cx)
        }
        ViewportInputKind::Wheel { .. } => dispatch_wheel(state, host, tools, cx),
    }
}

fn derive_input(
    config: ViewportToolArbitratorConfig,
    event: &ViewportInputEvent,
) -> ViewportToolInput {
    match config.coordinate_space {
        ViewportToolCoordinateSpace::TargetPx => {
            ViewportToolInput::from_viewport_input_target_px(event, config.primary_button)
        }
        ViewportToolCoordinateSpace::ScreenPx => {
            ViewportToolInput::from_viewport_input_screen_px(event, config.primary_button)
        }
    }
}

fn call_set_hot<T>(host: &mut T, entry: &mut ViewportToolEntry<T>, hot: bool) {
    if let Some(f) = entry.set_hot {
        f(host, hot);
    }
}

fn force_hot<T>(
    state: &mut ViewportToolRouterState,
    host: &mut T,
    tools: &mut [ViewportToolEntry<T>],
    id: ViewportToolId,
) {
    if state.hot == Some(id) {
        return;
    }

    if let Some(old) = state.hot
        && let Some(entry) = tools.iter_mut().find(|t| t.id == old)
    {
        call_set_hot(host, entry, false);
    }

    if let Some(entry) = tools.iter_mut().find(|t| t.id == id) {
        call_set_hot(host, entry, true);
        state.hot = Some(id);
    } else {
        state.hot = None;
    }
}

fn update_hot<T>(
    state: &mut ViewportToolRouterState,
    host: &mut T,
    tools: &mut [ViewportToolEntry<T>],
    cx: ViewportToolCx<'_>,
) {
    let mut next_hot = None;
    for tool in tools.iter_mut() {
        if (tool.hit_test)(host, cx) {
            next_hot = Some(tool.id);
            break;
        }
    }

    if next_hot == state.hot {
        return;
    }

    if let Some(old) = state.hot
        && let Some(entry) = tools.iter_mut().find(|t| t.id == old)
    {
        call_set_hot(host, entry, false);
    }
    if let Some(next) = next_hot
        && let Some(entry) = tools.iter_mut().find(|t| t.id == next)
    {
        call_set_hot(host, entry, true);
        state.hot = Some(next);
    } else {
        state.hot = None;
    }
}

fn dispatch_pointer_down<T>(
    state: &mut ViewportToolRouterState,
    host: &mut T,
    tools: &mut [ViewportToolEntry<T>],
    cx: ViewportToolCx<'_>,
) -> bool {
    for tool in tools.iter_mut() {
        let id = tool.id;
        let hot = state.hot == Some(id);
        let res = (tool.handle_event)(host, cx, hot, false);
        if !res.handled {
            continue;
        }

        if res.capture {
            state.active = Some(id);
            force_hot(state, host, tools, id);
        }
        return true;
    }
    false
}

fn dispatch_hot_only<T>(
    state: &mut ViewportToolRouterState,
    host: &mut T,
    tools: &mut [ViewportToolEntry<T>],
    cx: ViewportToolCx<'_>,
) -> bool {
    let Some(hot) = state.hot else {
        return false;
    };
    let Some(entry) = tools.iter_mut().find(|t| t.id == hot) else {
        state.hot = None;
        return false;
    };
    (entry.handle_event)(host, cx, true, false).handled
}

fn dispatch_wheel<T>(
    state: &mut ViewportToolRouterState,
    host: &mut T,
    tools: &mut [ViewportToolEntry<T>],
    cx: ViewportToolCx<'_>,
) -> bool {
    if let Some(hot) = state.hot
        && let Some(entry) = tools.iter_mut().find(|t| t.id == hot)
        && (entry.handle_event)(host, cx, true, false).handled
    {
        return true;
    }

    for tool in tools.iter_mut() {
        let id = tool.id;
        if Some(id) == state.hot {
            continue;
        }
        let res = (tool.handle_event)(host, cx, false, false);
        if res.handled {
            return true;
        }
    }
    false
}

impl ViewportToolArbitrator {
    pub fn new(config: ViewportToolArbitratorConfig) -> Self {
        Self {
            config,
            tools: Vec::new(),
            hot: None,
            active: None,
        }
    }

    pub fn tools_mut(&mut self) -> &mut [Box<dyn ViewportTool>] {
        &mut self.tools
    }

    pub fn hot_tool(&self) -> Option<ViewportToolId> {
        self.hot
    }

    pub fn active_tool(&self) -> Option<ViewportToolId> {
        self.active
    }

    pub fn set_tools(&mut self, mut tools: Vec<Box<dyn ViewportTool>>) {
        tools.sort_by_key(|t| Reverse(t.priority().0));
        self.tools = tools;
        self.hot = None;
        self.active = None;
    }

    pub fn clear_tools(&mut self) {
        self.tools.clear();
        self.hot = None;
        self.active = None;
    }

    pub fn cancel_active(&mut self) {
        if let Some(active) = self.active
            && let Some(idx) = self.index_of(active)
        {
            self.tools[idx].cancel();
        }
        self.active = None;
    }

    pub fn handle_event(&mut self, event: &ViewportInputEvent) -> bool {
        if self.tools.is_empty() {
            self.hot = None;
            self.active = None;
            return false;
        }

        let input = self.derive_input(event);
        let cx = ViewportToolCx { event, input };

        if let Some(active) = self.active {
            self.force_hot(active);
            let handled = if let Some(idx) = self.index_of(active) {
                self.tools[idx].handle_event(cx, true, true).handled
            } else {
                false
            };

            if !cx.input.dragging {
                self.active = None;
            }
            return handled;
        }

        match event.kind {
            ViewportInputKind::PointerMove { .. } | ViewportInputKind::PointerDown { .. } => {
                self.update_hot(cx);
            }
            ViewportInputKind::PointerUp { .. } => {}
            ViewportInputKind::Wheel { .. } => {
                self.update_hot(cx);
            }
        }

        match event.kind {
            ViewportInputKind::PointerDown { .. } => self.dispatch_pointer_down(cx),
            ViewportInputKind::PointerMove { .. } => self.dispatch_hot_only(cx),
            ViewportInputKind::PointerUp { .. } => self.dispatch_hot_only(cx),
            ViewportInputKind::Wheel { .. } => self.dispatch_wheel(cx),
        }
    }

    fn derive_input(&self, event: &ViewportInputEvent) -> ViewportToolInput {
        match self.config.coordinate_space {
            ViewportToolCoordinateSpace::TargetPx => {
                ViewportToolInput::from_viewport_input_target_px(event, self.config.primary_button)
            }
            ViewportToolCoordinateSpace::ScreenPx => {
                ViewportToolInput::from_viewport_input_screen_px(event, self.config.primary_button)
            }
        }
    }

    fn index_of(&self, id: ViewportToolId) -> Option<usize> {
        self.tools.iter().position(|t| t.id() == id)
    }

    fn force_hot(&mut self, id: ViewportToolId) {
        if self.hot == Some(id) {
            return;
        }
        if let Some(old) = self.hot
            && let Some(idx) = self.index_of(old)
        {
            self.tools[idx].set_hot(false);
        }
        if let Some(idx) = self.index_of(id) {
            self.tools[idx].set_hot(true);
            self.hot = Some(id);
        } else {
            self.hot = None;
        }
    }

    fn update_hot(&mut self, cx: ViewportToolCx<'_>) {
        let mut next_hot = None;
        for tool in &mut self.tools {
            if tool.hit_test(cx) {
                next_hot = Some(tool.id());
                break;
            }
        }

        if next_hot == self.hot {
            return;
        }

        if let Some(old) = self.hot
            && let Some(idx) = self.index_of(old)
        {
            self.tools[idx].set_hot(false);
        }
        if let Some(next) = next_hot
            && let Some(idx) = self.index_of(next)
        {
            self.tools[idx].set_hot(true);
            self.hot = Some(next);
        } else {
            self.hot = None;
        }
    }

    fn dispatch_pointer_down(&mut self, cx: ViewportToolCx<'_>) -> bool {
        for tool in &mut self.tools {
            let id = tool.id();
            let hot = self.hot == Some(id);
            let res = tool.handle_event(cx, hot, false);
            if !res.handled {
                continue;
            }

            if res.capture {
                self.active = Some(id);
                self.force_hot(id);
            }
            return true;
        }
        false
    }

    fn dispatch_hot_only(&mut self, cx: ViewportToolCx<'_>) -> bool {
        let Some(hot) = self.hot else {
            return false;
        };
        let Some(idx) = self.index_of(hot) else {
            self.hot = None;
            return false;
        };
        self.tools[idx].handle_event(cx, true, false).handled
    }

    fn dispatch_wheel(&mut self, cx: ViewportToolCx<'_>) -> bool {
        if let Some(hot) = self.hot
            && let Some(idx) = self.index_of(hot)
            && self.tools[idx].handle_event(cx, true, false).handled
        {
            return true;
        }

        for tool in &mut self.tools {
            let id = tool.id();
            if Some(id) == self.hot {
                continue;
            }
            let res = tool.handle_event(cx, false, false);
            if res.handled {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Px, Rect, Size};
    use fret_core::{AppWindowId, Modifiers, RenderTargetId, ViewportFit, ViewportInputGeometry};
    use fret_viewport_tooling::{ViewportToolPriority, ViewportToolResult};

    fn dummy_event(kind: ViewportInputKind) -> ViewportInputEvent {
        ViewportInputEvent {
            window: AppWindowId::default(),
            target: RenderTargetId::default(),
            geometry: ViewportInputGeometry {
                content_rect_px: Rect::new(
                    fret_core::geometry::Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                draw_rect_px: Rect::new(
                    fret_core::geometry::Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                target_px_size: (1000, 500),
                fit: ViewportFit::Stretch,
                pixels_per_point: 2.0,
            },
            cursor_px: fret_core::geometry::Point::new(Px(10.0), Px(10.0)),
            uv: (0.0, 0.0),
            target_px: (0, 0),
            kind,
        }
    }

    struct TestTool {
        id: ViewportToolId,
        prio: i32,
        hit: bool,
        down_capture: bool,
        down_handled: bool,
        hot: bool,
        calls: Vec<&'static str>,
    }

    impl TestTool {
        fn new(id: u64, prio: i32) -> Self {
            Self {
                id: ViewportToolId(id),
                prio,
                hit: false,
                down_capture: false,
                down_handled: false,
                hot: false,
                calls: Vec::new(),
            }
        }
    }

    impl ViewportTool for TestTool {
        fn id(&self) -> ViewportToolId {
            self.id
        }

        fn priority(&self) -> ViewportToolPriority {
            ViewportToolPriority(self.prio)
        }

        fn set_hot(&mut self, hot: bool) {
            self.hot = hot;
            self.calls.push(if hot { "hot_on" } else { "hot_off" });
        }

        fn hit_test(&mut self, _cx: ViewportToolCx<'_>) -> bool {
            self.calls.push("hit_test");
            self.hit
        }

        fn handle_event(
            &mut self,
            cx: ViewportToolCx<'_>,
            hot: bool,
            active: bool,
        ) -> ViewportToolResult {
            match cx.event.kind {
                ViewportInputKind::PointerDown { .. } => {
                    self.calls.push(if hot { "down_hot" } else { "down_cold" });
                    if self.down_handled {
                        if self.down_capture {
                            return ViewportToolResult::handled_and_capture();
                        }
                        return ViewportToolResult::handled();
                    }
                }
                ViewportInputKind::PointerMove { .. } => {
                    self.calls.push(if hot { "move_hot" } else { "move_cold" });
                }
                ViewportInputKind::PointerUp { .. } => {
                    self.calls
                        .push(if active { "up_active" } else { "up_inactive" });
                }
                ViewportInputKind::Wheel { .. } => {
                    self.calls
                        .push(if hot { "wheel_hot" } else { "wheel_cold" });
                }
            }
            ViewportToolResult::unhandled()
        }
    }

    #[test]
    fn picks_hot_by_priority_and_clears_previous() {
        let mut a = TestTool::new(1, 10);
        a.hit = false;
        let mut b = TestTool::new(2, 0);
        b.hit = true;

        let mut arb = ViewportToolArbitrator::new(Default::default());
        arb.set_tools(vec![Box::new(a), Box::new(b)]);

        let handled = arb.handle_event(&dummy_event(ViewportInputKind::PointerMove {
            buttons: Default::default(),
            modifiers: Modifiers::default(),
        }));
        assert!(!handled);
        assert_eq!(arb.hot_tool(), Some(ViewportToolId(2)));
    }

    #[test]
    fn pointer_down_captures_and_routes_followup_to_active_only() {
        let mut a = TestTool::new(1, 10);
        a.hit = true;
        a.down_handled = true;
        a.down_capture = true;
        let mut b = TestTool::new(2, 0);
        b.hit = true;
        b.down_handled = true;
        b.down_capture = true;

        let mut arb = ViewportToolArbitrator::new(Default::default());
        arb.set_tools(vec![Box::new(a), Box::new(b)]);

        assert!(
            arb.handle_event(&dummy_event(ViewportInputKind::PointerDown {
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }))
        );
        assert_eq!(arb.active_tool(), Some(ViewportToolId(1)));

        let _ = arb.handle_event(&dummy_event(ViewportInputKind::PointerUp {
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
        }));
        assert_eq!(arb.active_tool(), None);
    }
}
