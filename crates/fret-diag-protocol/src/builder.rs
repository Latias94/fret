//! Typed helpers for building UI diagnostics scripts in Rust.
//!
//! Design goals:
//! - Keep `tools/diag-scripts/*.json` as the portable, reviewable artifact.
//! - Provide a small, type-safe API for generating scripts (e.g. via a scriptgen tool).
//! - Prefer stable selectors (`test_id`, semantics role/name) over pixel coordinates.

use crate::{
    UiActionScriptV2, UiActionStepV2, UiKeyModifiersV1, UiMouseButtonV1, UiPredicateV1,
    UiSelectorV1,
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
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
        })
    }

    pub fn click_with_modifiers(self, target: UiSelectorV1, modifiers: UiKeyModifiersV1) -> Self {
        self.push(UiActionStepV2::Click {
            window: None,
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: Some(modifiers),
        })
    }

    pub fn click_stable(self, target: UiSelectorV1) -> Self {
        self.push(UiActionStepV2::ClickStable {
            target,
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
            target,
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: Some(modifiers),
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

    pub fn type_text_into(self, target: UiSelectorV1, text: impl Into<String>) -> Self {
        self.push(UiActionStepV2::TypeTextInto {
            target,
            text: text.into(),
            timeout_frames: 180,
        })
    }

    pub fn wait_frames(self, n: u32) -> Self {
        self.push(UiActionStepV2::WaitFrames { n })
    }

    pub fn wait_until(self, predicate: UiPredicateV1, timeout_frames: u32) -> Self {
        self.push(UiActionStepV2::WaitUntil {
            predicate,
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
        self.push(UiActionStepV2::Assert { predicate })
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
        })
    }

    pub fn capture_screenshot(self, label: impl Into<Option<String>>) -> Self {
        self.push(UiActionStepV2::CaptureScreenshot {
            label: label.into(),
            timeout_frames: 300,
        })
    }

    pub fn build(self) -> UiActionScriptV2 {
        UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: self.steps,
        }
    }
}
