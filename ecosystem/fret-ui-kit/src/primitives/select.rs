//! Select helpers (Radix `@radix-ui/react-select` outcomes).
//!
//! Upstream Select composes:
//! - anchored floating placement (`@radix-ui/react-popper`)
//! - portal rendering (`@radix-ui/react-portal`)
//! - focus management + outside interaction blocking (`@radix-ui/react-focus-scope`, `DismissableLayer`)
//! - aria hiding + scroll lock while open (`aria-hidden`, `react-remove-scroll`)
//! - trigger open keys + typeahead selection while closed.
//!
//! In Fret, the "blocking outside interaction" outcome is typically modeled by installing the
//! select content in a modal overlay layer (barrier-backed) while keeping the content semantics
//! as `ListBox` rather than `Dialog`.
//!
//! This module is intentionally thin: it provides Radix-named entry points for trigger a11y and
//! overlay request wiring without forcing a visual skin.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{AppWindowId, KeyCode, Modifiers};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::UiActionHost;
use fret_ui::element::{AnyElement, ElementKind, PressableA11y, PressableProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::headless::roving_focus;
use crate::headless::typeahead;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for select overlays.
pub fn select_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// Stamps Radix-like trigger semantics:
/// - `role=ComboBox`
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_select_trigger_a11y(
    mut trigger: AnyElement,
    expanded: bool,
    label: Option<Arc<str>>,
    listbox_element: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            *a11y = PressableA11y {
                role: Some(fret_core::SemanticsRole::ComboBox),
                label,
                expanded: Some(expanded),
                controls_element: listbox_element.map(|id| id.0),
                ..a11y.clone()
            };
        }
        ElementKind::Semantics(props) => {
            props.role = fret_core::SemanticsRole::ComboBox;
            props.label = label;
            props.expanded = Some(expanded);
            props.controls_element = listbox_element.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}

/// Radix Select trigger "open keys" (`OPEN_KEYS`).
pub fn is_select_open_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Space | KeyCode::Enter | KeyCode::ArrowUp | KeyCode::ArrowDown
    )
}

/// Returns `true` when the open key is expected to also produce a click/activate event on key-up.
pub fn select_open_key_suppresses_activate(key: KeyCode) -> bool {
    matches!(key, KeyCode::Space | KeyCode::Enter)
}

/// Radix-like select typeahead clear timeout (in milliseconds).
///
/// Upstream Radix resets the typeahead search 1 second after it was last updated.
pub const SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS: u64 = 1000;

/// Timer-driven typeahead query state (Radix-style).
#[derive(Debug, Default)]
pub struct TimedTypeaheadState {
    query: String,
    clear_token: Option<TimerToken>,
}

impl TimedTypeaheadState {
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    pub fn clear_and_cancel(&mut self, host: &mut dyn UiActionHost) {
        if let Some(token) = self.clear_token.take() {
            host.push_effect(Effect::CancelTimer { token });
        }
        self.query.clear();
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        if self.clear_token == Some(token) {
            self.clear_token = None;
            self.query.clear();
            return true;
        }
        false
    }

    pub fn push_key_and_arm_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        key: KeyCode,
        timeout: Duration,
    ) -> Option<char> {
        let ch = fret_core::keycode_to_ascii_lowercase(key)?;
        self.query.push(ch);
        if let Some(token) = self.clear_token.take() {
            host.push_effect(Effect::CancelTimer { token });
        }
        let token = host.next_timer_token();
        self.clear_token = Some(token);
        host.push_effect(Effect::SetTimer {
            window: Some(window),
            token,
            after: timeout,
            repeat: None,
        });
        Some(ch)
    }
}

/// Closed-state trigger policy for Radix-style select.
///
/// This models two coupled Radix outcomes:
/// - Trigger open keys open the listbox on key-down (and suppress the ensuing key-up activation).
/// - While closed, alphanumeric typeahead updates the selected value without opening.
#[derive(Debug, Default)]
pub struct SelectTriggerKeyState {
    suppress_next_activate: bool,
    typeahead: TimedTypeaheadState,
}

impl SelectTriggerKeyState {
    pub fn take_suppress_next_activate(&mut self) -> bool {
        let v = self.suppress_next_activate;
        self.suppress_next_activate = false;
        v
    }

    pub fn clear_typeahead(&mut self, host: &mut dyn UiActionHost) {
        self.typeahead.clear_and_cancel(host);
    }

    pub fn reset_typeahead_buffer(&mut self) {
        self.typeahead.query.clear();
        self.typeahead.clear_token = None;
    }

    pub fn typeahead_query(&self) -> &str {
        self.typeahead.query()
    }

    pub fn push_typeahead_key_and_arm_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        key: KeyCode,
    ) -> Option<char> {
        let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
        self.typeahead
            .push_key_and_arm_timer(host, window, key, timeout)
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        self.typeahead.on_timer(token)
    }

    pub fn handle_key_down_when_closed(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        open: &Model<bool>,
        value: &Model<Option<Arc<str>>>,
        values: &[Arc<str>],
        labels: &[Arc<str>],
        disabled: &[bool],
        key: KeyCode,
        modifiers: Modifiers,
        repeat: bool,
    ) -> bool {
        if repeat {
            return false;
        }

        let is_open = host.models_mut().get_copied(open).unwrap_or(false);
        if is_open {
            return false;
        }

        let is_modifier_key = modifiers.ctrl || modifiers.alt || modifiers.meta || modifiers.alt_gr;
        if is_modifier_key {
            return false;
        }

        if key == KeyCode::Space && !self.typeahead.query().is_empty() {
            return true;
        }

        if is_select_open_key(key) {
            if select_open_key_suppresses_activate(key) {
                self.suppress_next_activate = true;
            }
            self.typeahead.clear_and_cancel(host);
            let _ = host.models_mut().update(open, |v| *v = true);
            host.request_redraw(window);
            return true;
        }

        let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
        let Some(_ch) = self
            .typeahead
            .push_key_and_arm_timer(host, window, key, timeout)
        else {
            return false;
        };

        let current = host.models_mut().read(value, |v| v.clone()).ok().flatten();
        let current_idx = current
            .as_ref()
            .and_then(|v| values.iter().position(|it| it.as_ref() == v.as_ref()));

        if let Some(next) = typeahead::match_prefix_arc_str(
            labels,
            disabled,
            self.typeahead.query(),
            current_idx,
            true,
        ) && let Some(next_value) = values.get(next).cloned()
        {
            let _ = host.models_mut().update(value, |v| *v = Some(next_value));
            host.request_redraw(window);
        }

        true
    }
}

/// Open-state listbox policy for Radix-style select content.
///
/// This mirrors Radix outcomes inside `SelectContent`:
/// - `Escape` closes.
/// - `Home/End/ArrowUp/ArrowDown` move the active option (skipping disabled).
/// - `Enter/Space` commits the active option and closes.
/// - Typeahead search moves the active option (with repeated-search normalization).
#[derive(Debug, Default)]
pub struct SelectContentKeyState {
    active_row: Option<usize>,
    typeahead: TimedTypeaheadState,
}

impl SelectContentKeyState {
    pub fn active_row(&self) -> Option<usize> {
        self.active_row
    }

    pub fn set_active_row(&mut self, row: Option<usize>) {
        self.active_row = row;
    }

    pub fn reset_on_open(&mut self, initial_active_row: Option<usize>) {
        self.active_row = initial_active_row;
        self.typeahead.query.clear();
        self.typeahead.clear_token = None;
    }

    pub fn clear_typeahead(&mut self, host: &mut dyn UiActionHost) {
        self.typeahead.clear_and_cancel(host);
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        self.typeahead.on_timer(token)
    }

    pub fn handle_key_down_when_open(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        open: &Model<bool>,
        value: &Model<Option<Arc<str>>>,
        values_by_row: &[Option<Arc<str>>],
        labels_by_row: &[Arc<str>],
        disabled_by_row: &[bool],
        key: KeyCode,
        repeat: bool,
        loop_navigation: bool,
    ) -> bool {
        if repeat {
            return false;
        }

        let is_open = host.models_mut().get_copied(open).unwrap_or(false);
        if !is_open {
            return false;
        }

        if key == KeyCode::Space && !self.typeahead.query().is_empty() {
            return true;
        }

        let current = self
            .active_row
            .or_else(|| roving_focus::first_enabled(disabled_by_row));

        match key {
            KeyCode::Escape => {
                let _ = host.models_mut().update(open, |v| *v = false);
                host.request_redraw(window);
                true
            }
            KeyCode::Home => {
                self.active_row = roving_focus::first_enabled(disabled_by_row);
                host.request_redraw(window);
                true
            }
            KeyCode::End => {
                self.active_row = roving_focus::last_enabled(disabled_by_row);
                host.request_redraw(window);
                true
            }
            KeyCode::ArrowDown | KeyCode::ArrowUp => {
                let Some(current) = current else {
                    return true;
                };
                let forward = key == KeyCode::ArrowDown;
                self.active_row =
                    roving_focus::next_enabled(disabled_by_row, current, forward, loop_navigation)
                        .or(Some(current));
                host.request_redraw(window);
                true
            }
            KeyCode::Enter | KeyCode::Space => {
                let Some(active_row) = current else {
                    return true;
                };
                let is_disabled = disabled_by_row.get(active_row).copied().unwrap_or(true);
                if is_disabled {
                    return true;
                }
                if let Some(chosen_value) = values_by_row.get(active_row).cloned().flatten() {
                    let _ = host
                        .models_mut()
                        .update(value, |v| *v = Some(chosen_value.clone()));
                    let _ = host.models_mut().update(open, |v| *v = false);
                    host.request_redraw(window);
                }
                true
            }
            _ => {
                let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
                let Some(_ch) = self
                    .typeahead
                    .push_key_and_arm_timer(host, window, key, timeout)
                else {
                    return false;
                };

                let next = typeahead::match_prefix_arc_str(
                    labels_by_row,
                    disabled_by_row,
                    self.typeahead.query(),
                    current,
                    true,
                );
                if next != self.active_row {
                    self.active_row = next;
                    host.request_redraw(window);
                }
                true
            }
        }
    }
}

/// Builds an overlay request for a Radix-style select content overlay.
///
/// This uses a modal overlay layer to approximate Radix Select's outside interaction blocking.
pub fn modal_select_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::modal(id, Some(trigger), open, presence, children);
    request.root_name = Some(select_root_name(id));
    request
}

/// Requests a select overlay for the current window.
pub fn request_select<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, Point, Px, Rect, Size};
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui::element::{LayoutStyle, PressableProps};
    use std::time::Duration;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn apply_select_trigger_a11y_sets_role_expanded_and_controls() {
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

            let listbox = GlobalElementId(0xbeef);
            let trigger =
                apply_select_trigger_a11y(trigger, true, Some(Arc::from("Select")), Some(listbox));

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.role, Some(fret_core::SemanticsRole::ComboBox));
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(listbox.0));
            assert_eq!(a11y.label.as_deref(), Some("Select"));
        });
    }

    #[test]
    fn modal_select_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);

        let req = modal_select_request(
            id,
            trigger,
            open,
            OverlayPresence::instant(true),
            Vec::new(),
        );
        let expected = select_root_name(id);
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn select_open_keys_match_radix_defaults() {
        assert!(is_select_open_key(KeyCode::Enter));
        assert!(is_select_open_key(KeyCode::Space));
        assert!(is_select_open_key(KeyCode::ArrowDown));
        assert!(is_select_open_key(KeyCode::ArrowUp));
        assert!(!is_select_open_key(KeyCode::Escape));

        assert!(select_open_key_suppresses_activate(KeyCode::Enter));
        assert!(select_open_key_suppresses_activate(KeyCode::Space));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowDown));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowUp));
    }

    #[test]
    fn trigger_typeahead_updates_value_without_opening() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values: Vec<Arc<str>> = vec![Arc::from("alpha"), Arc::from("beta")];
        let labels: Vec<Arc<str>> = vec![Arc::from("Alpha"), Arc::from("Beta")];
        let disabled = vec![false, false];

        let mut state = SelectTriggerKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_closed(
            &mut host,
            window,
            &open,
            &value,
            &values,
            &labels,
            &disabled,
            KeyCode::KeyB,
            Modifiers::default(),
            false,
        ));

        assert!(!app.models().get_copied(&open).unwrap_or(false));
        assert_eq!(
            app.models().get_cloned(&value).flatten().as_deref(),
            Some("beta")
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::SetTimer { after, .. }
                    if *after == Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS)
            )),
            "expected a typeahead clear timer"
        );
    }

    #[test]
    fn trigger_open_key_opens_and_suppresses_activate() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values: Vec<Arc<str>> = vec![Arc::from("alpha")];
        let labels: Vec<Arc<str>> = vec![Arc::from("Alpha")];
        let disabled = vec![false];

        let mut state = SelectTriggerKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_closed(
            &mut host,
            window,
            &open,
            &value,
            &values,
            &labels,
            &disabled,
            KeyCode::Enter,
            Modifiers::default(),
            false,
        ));

        assert!(app.models().get_copied(&open).unwrap_or(false));
        assert!(state.take_suppress_next_activate());
    }

    #[test]
    fn content_arrow_navigation_updates_active_row() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values_by_row: Vec<Option<Arc<str>>> = vec![
            Some(Arc::from("alpha")),
            Some(Arc::from("beta")),
            Some(Arc::from("gamma")),
        ];
        let labels_by_row: Vec<Arc<str>> =
            vec![Arc::from("Alpha"), Arc::from("Beta"), Arc::from("Gamma")];
        let disabled_by_row = vec![false, true, false];

        let mut state = SelectContentKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };

        assert!(state.handle_key_down_when_open(
            &mut host,
            window,
            &open,
            &value,
            &values_by_row,
            &labels_by_row,
            &disabled_by_row,
            KeyCode::ArrowDown,
            false,
            true,
        ));
        // Skips disabled row 1, so we land on row 2.
        assert_eq!(state.active_row(), Some(2));
    }

    #[test]
    fn content_enter_commits_value_and_closes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values_by_row: Vec<Option<Arc<str>>> = vec![Some(Arc::from("beta"))];
        let labels_by_row: Vec<Arc<str>> = vec![Arc::from("Beta")];
        let disabled_by_row = vec![false];

        let mut state = SelectContentKeyState::default();
        state.set_active_row(Some(0));

        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_open(
            &mut host,
            window,
            &open,
            &value,
            &values_by_row,
            &labels_by_row,
            &disabled_by_row,
            KeyCode::Enter,
            false,
            true,
        ));

        assert!(!app.models().get_copied(&open).unwrap_or(false));
        assert_eq!(
            app.models().get_cloned(&value).flatten().as_deref(),
            Some("beta")
        );
    }
}
