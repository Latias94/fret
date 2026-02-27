//! Typed helpers for building UI diagnostics scripts in Rust.
//!
//! Design goals:
//! - Keep `tools/diag-scripts/*.json` as the portable, reviewable artifact.
//! - Provide a small, type-safe API for generating scripts (e.g. via a scriptgen tool).
//! - Prefer stable selectors (`test_id`, semantics role/name) over pixel coordinates.

use crate::{
    UiActionScriptV2, UiActionStepV2, UiImeEventV1, UiIncomingOpenInjectItemV1, UiKeyModifiersV1,
    UiMouseButtonV1, UiOverlayPlacementTraceQueryV1, UiPointerKindV1, UiPredicateV1, UiSelectorV1,
    UiShortcutRoutingTraceQueryV1,
};

pub fn test_id(id: impl Into<String>) -> UiSelectorV1 {
    UiSelectorV1::TestId { id: id.into() }
}

pub fn role_and_name(role: impl Into<String>, name: impl Into<String>) -> UiSelectorV1 {
    UiSelectorV1::RoleAndName {
        role: role.into(),
        name: name.into(),
    }
}

pub fn exists(target: UiSelectorV1) -> UiPredicateV1 {
    UiPredicateV1::Exists { target }
}

pub fn not_exists(target: UiSelectorV1) -> UiPredicateV1 {
    UiPredicateV1::NotExists { target }
}

pub fn focus_is(target: UiSelectorV1) -> UiPredicateV1 {
    UiPredicateV1::FocusIs { target }
}

pub fn active_item_is(container: UiSelectorV1, item: UiSelectorV1) -> UiPredicateV1 {
    UiPredicateV1::ActiveItemIs { container, item }
}

pub fn active_item_is_none(container: UiSelectorV1) -> UiPredicateV1 {
    UiPredicateV1::ActiveItemIsNone { container }
}

pub fn selected_is(target: UiSelectorV1, selected: bool) -> UiPredicateV1 {
    UiPredicateV1::SelectedIs { target, selected }
}

pub fn text_composition_is(target: UiSelectorV1, composing: bool) -> UiPredicateV1 {
    UiPredicateV1::TextCompositionIs { target, composing }
}

pub fn ime_cursor_area_is_some(is_some: bool) -> UiPredicateV1 {
    UiPredicateV1::ImeCursorAreaIsSome { is_some }
}

pub fn ime_cursor_area_within_window(padding_px: f32, eps_px: f32) -> UiPredicateV1 {
    UiPredicateV1::ImeCursorAreaWithinWindow {
        padding_px,
        padding_insets_px: None,
        eps_px,
    }
}

pub fn ime_cursor_area_min_size(min_w_px: f32, min_h_px: f32, eps_px: f32) -> UiPredicateV1 {
    UiPredicateV1::ImeCursorAreaMinSize {
        min_w_px,
        min_h_px,
        eps_px,
    }
}

pub fn runner_accessibility_activated() -> UiPredicateV1 {
    UiPredicateV1::RunnerAccessibilityActivated
}

#[derive(Debug, Default, Clone)]
pub struct ScriptV2Builder {
    steps: Vec<UiActionStepV2>,
}

impl ScriptV2Builder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn push(mut self, step: UiActionStepV2) -> Self {
        self.steps.push(step);
        self
    }

    pub fn extend(mut self, steps: impl IntoIterator<Item = UiActionStepV2>) -> Self {
        self.steps.extend(steps);
        self
    }

    pub fn reset_diagnostics(self) -> Self {
        self.push(UiActionStepV2::ResetDiagnostics)
    }

    pub fn click(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::Click {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
        })
    }

    pub fn click_touch(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::Click {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Touch),
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
        })
    }

    pub fn click_pen(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::Click {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Pen),
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
        })
    }

    pub fn tap(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::Tap {
            window: None,
            pointer_kind: None,
            target,
            modifiers: None,
        })
    }

    pub fn tap_pen(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::Tap {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Pen),
            target,
            modifiers: None,
        })
    }

    pub fn click_with_modifiers(self, target: UiSelectorV1, modifiers: UiKeyModifiersV1) -> Self {
        self.push(UiActionStepV2::Click {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: Some(modifiers),
        })
    }

    pub fn pointer_down(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::PointerDown {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            modifiers: None,
        })
    }

    pub fn pointer_down_touch(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::PointerDown {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Touch),
            target,
            button: UiMouseButtonV1::Left,
            modifiers: None,
        })
    }

    pub fn pointer_down_pen(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::PointerDown {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Pen),
            target,
            button: UiMouseButtonV1::Left,
            modifiers: None,
        })
    }

    pub fn pointer_down_with_modifiers(
        self,
        target: UiSelectorV1,
        modifiers: UiKeyModifiersV1,
    ) -> Self {
        self.push(UiActionStepV2::PointerDown {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            modifiers: Some(modifiers),
        })
    }

    pub fn pointer_move(self, delta_x: f32, delta_y: f32) -> Self {
        self.push(UiActionStepV2::PointerMove {
            window: None,
            pointer_kind: None,
            delta_x,
            delta_y,
            steps: 8,
        })
    }

    pub fn pointer_up(self) -> Self {
        self.push(UiActionStepV2::PointerUp {
            window: None,
            pointer_kind: None,
            button: None,
        })
    }

    pub fn click_stable(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::ClickStable {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
            stable_frames: 2,
            max_move_px: 1.0,
            timeout_frames: 180,
        })
    }

    pub fn click_stable_touch(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::ClickStable {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Touch),
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
            stable_frames: 2,
            max_move_px: 1.0,
            timeout_frames: 180,
        })
    }

    pub fn click_selectable_text_span_stable(
        self,
        target: UiSelectorV1,
        tag: impl Into<String>,
    ) -> Self {
        self.push(UiActionStepV2::ClickSelectableTextSpanStable {
            window: None,
            pointer_kind: None,
            target,
            tag: tag.into(),
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
            stable_frames: 2,
            max_move_px: 1.0,
            timeout_frames: 180,
        })
    }

    pub fn click_stable_with_modifiers(
        self,
        target: UiSelectorV1,
        modifiers: UiKeyModifiersV1,
    ) -> Self {
        self.push(UiActionStepV2::ClickStable {
            window: None,
            pointer_kind: None,
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: Some(modifiers),
            stable_frames: 2,
            max_move_px: 1.0,
            timeout_frames: 180,
        })
    }

    pub fn wait_bounds_stable(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::WaitBoundsStable {
            window: None,
            target,
            stable_frames: 2,
            max_move_px: 1.0,
            timeout_frames: 180,
        })
    }

    pub fn press_key(self, key: impl Into<String>) -> Self {
        self.push(UiActionStepV2::PressKey {
            key: key.into(),
            modifiers: UiKeyModifiersV1::default(),
            repeat: false,
        })
    }

    pub fn press_shortcut(self, shortcut: impl Into<String>) -> Self {
        self.push(UiActionStepV2::PressShortcut {
            shortcut: shortcut.into(),
            repeat: false,
        })
    }

    pub fn type_text(self, text: impl Into<String>) -> Self {
        self.push(UiActionStepV2::TypeText { text: text.into() })
    }

    pub fn ime_enabled(self) -> Self {
        self.push(UiActionStepV2::Ime {
            event: UiImeEventV1::Enabled,
        })
    }

    pub fn ime_disabled(self) -> Self {
        self.push(UiActionStepV2::Ime {
            event: UiImeEventV1::Disabled,
        })
    }

    pub fn ime_preedit(self, text: impl Into<String>, cursor_bytes: Option<(u32, u32)>) -> Self {
        self.push(UiActionStepV2::Ime {
            event: UiImeEventV1::Preedit {
                text: text.into(),
                cursor_bytes,
            },
        })
    }

    pub fn ime_commit(self, text: impl Into<String>) -> Self {
        self.push(UiActionStepV2::Ime {
            event: UiImeEventV1::Commit { text: text.into() },
        })
    }

    pub fn ime_delete_surrounding(self, before_bytes: u32, after_bytes: u32) -> Self {
        self.push(UiActionStepV2::Ime {
            event: UiImeEventV1::DeleteSurrounding {
                before_bytes,
                after_bytes,
            },
        })
    }

    pub fn type_text_into(self, target: UiSelectorV1, text: impl Into<String>) -> Self {
        self.push(UiActionStepV2::TypeTextInto {
            window: None,
            pointer_kind: None,
            target,
            text: text.into(),
            clear_before_type: false,
            timeout_frames: 180,
        })
    }

    pub fn type_text_into_touch(self, target: UiSelectorV1, text: impl Into<String>) -> Self {
        self.push(UiActionStepV2::TypeTextInto {
            window: None,
            pointer_kind: Some(UiPointerKindV1::Touch),
            target,
            text: text.into(),
            clear_before_type: false,
            timeout_frames: 180,
        })
    }

    pub fn wait_frames(self, n: u32) -> Self {
        self.push(UiActionStepV2::WaitFrames { n })
    }

    pub fn wait_until(self, predicate: UiPredicateV1, timeout_frames: u32) -> Self {
        self.push(UiActionStepV2::WaitUntil {
            window: None,
            predicate,
            timeout_frames,
        })
    }

    pub fn wait_shortcut_routing_trace(
        self,
        query: UiShortcutRoutingTraceQueryV1,
        timeout_frames: u32,
    ) -> Self {
        self.push(UiActionStepV2::WaitShortcutRoutingTrace {
            query,
            timeout_frames,
        })
    }

    pub fn wait_overlay_placement_trace(
        self,
        query: UiOverlayPlacementTraceQueryV1,
        timeout_frames: u32,
    ) -> Self {
        self.push(UiActionStepV2::WaitOverlayPlacementTrace {
            query,
            timeout_frames,
        })
    }

    pub fn wait_exists(self, target: UiSelectorV1, timeout_frames: u32) -> Self {
        self.wait_until(exists(target), timeout_frames)
    }

    pub fn wait_not_exists(self, target: UiSelectorV1, timeout_frames: u32) -> Self {
        self.wait_until(not_exists(target), timeout_frames)
    }

    pub fn assert(self, predicate: UiPredicateV1) -> Self {
        self.push(UiActionStepV2::Assert {
            window: None,
            predicate,
        })
    }

    pub fn assert_exists(self, target: UiSelectorV1) -> Self {
        self.assert(exists(target))
    }

    pub fn assert_not_exists(self, target: UiSelectorV1) -> Self {
        self.assert(not_exists(target))
    }

    pub fn assert_focus_is(self, target: UiSelectorV1) -> Self {
        self.assert(focus_is(target))
    }

    pub fn capture_bundle(self, label: impl Into<Option<String>>) -> Self {
        self.push(UiActionStepV2::CaptureBundle {
            label: label.into(),
            max_snapshots: None,
        })
    }

    pub fn capture_bundle_with_max_snapshots(
        self,
        label: impl Into<Option<String>>,
        max_snapshots: u32,
    ) -> Self {
        self.push(UiActionStepV2::CaptureBundle {
            label: label.into(),
            max_snapshots: Some(max_snapshots),
        })
    }

    pub fn capture_screenshot(self, label: impl Into<Option<String>>) -> Self {
        self.push(UiActionStepV2::CaptureScreenshot {
            label: label.into(),
            timeout_frames: 300,
        })
    }

    pub fn set_clipboard_force_unavailable(self, enabled: bool) -> Self {
        self.push(UiActionStepV2::SetClipboardForceUnavailable { enabled })
    }

    pub fn inject_incoming_open(self, items: Vec<UiIncomingOpenInjectItemV1>) -> Self {
        self.push(UiActionStepV2::InjectIncomingOpen { items })
    }

    pub fn build(self) -> UiActionScriptV2 {
        UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: self.steps,
        }
    }
}
