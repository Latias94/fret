use fret_app::{App, CommandId};
use fret_core::{Event, KeyCode, Modifiers, NodeId, Point, Px, SemanticsRole, UiServices};

use crate::UiTree;

fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() <= epsilon
}

fn quantize_to_step(value: f64, min: f64, max: f64, step: f64) -> f64 {
    debug_assert!(min.is_finite());
    debug_assert!(max.is_finite());
    debug_assert!(min <= max);
    debug_assert!(step.is_finite());
    debug_assert!(step > 0.0);
    (min + ((value - min) / step).round() * step).clamp(min, max)
}

fn aligned_jump(jump: Option<f64>, step: f64) -> Option<f64> {
    jump.filter(|v| v.is_finite() && *v > 0.0).and_then(|jump| {
        let ratio = jump / step;
        let ratio_round = ratio.round();
        let aligned = (ratio - ratio_round).abs() <= 1e-6 && ratio_round >= 2.0;
        aligned.then_some(jump)
    })
}

#[derive(Debug, Clone, Copy)]
struct SliderNumeric {
    cur: Option<f64>,
    min: f64,
    max: f64,
    step: f64,
    jump: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
enum SliderConvergeAction {
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
}

fn slider_converge_actions(meta: SliderNumeric, desired: f64) -> Vec<SliderConvergeAction> {
    let SliderNumeric {
        cur,
        min,
        max,
        step,
        jump,
    } = meta;

    let range = (max - min).abs().max(1.0);
    let epsilon = (range * 1e-9).max(1e-9);

    let desired = quantize_to_step(desired.clamp(min, max), min, max, step);
    if approx_eq(desired, min, epsilon) {
        return vec![SliderConvergeAction::Home];
    }
    if approx_eq(desired, max, epsilon) {
        return vec![SliderConvergeAction::End];
    }

    let start = cur
        .filter(|v| v.is_finite())
        .map(|v| quantize_to_step(v, min, max, step))
        .unwrap_or_else(|| {
            if (desired - min).abs() <= (max - desired).abs() {
                min
            } else {
                max
            }
        });

    const MAX_EVENTS: usize = 1024;
    let mut out: Vec<SliderConvergeAction> = Vec::new();

    let mut simulated = start;
    if cur.is_none() {
        if approx_eq(simulated, min, epsilon) {
            out.push(SliderConvergeAction::Home);
        } else {
            out.push(SliderConvergeAction::End);
        }
    }

    let jump = aligned_jump(jump, step);

    while out.len() < MAX_EVENTS && !approx_eq(simulated, desired, epsilon) {
        let diff = desired - simulated;
        let sign = if diff >= 0.0 { 1.0 } else { -1.0 };

        if let Some(jump) = jump
            && diff.abs() + epsilon >= jump
            && out.len() + 1 < MAX_EVENTS
        {
            if sign > 0.0 {
                out.push(SliderConvergeAction::PageUp);
            } else {
                out.push(SliderConvergeAction::PageDown);
            }
            simulated = (simulated + sign * jump).clamp(min, max);
            continue;
        }

        if sign > 0.0 {
            out.push(SliderConvergeAction::ArrowUp);
        } else {
            out.push(SliderConvergeAction::ArrowDown);
        }
        simulated = (simulated + sign * step).clamp(min, max);
    }

    out
}

fn press_key(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    key: KeyCode,
) {
    ui.set_focus(Some(target));
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}

fn invoke_key_for_role(role: SemanticsRole) -> KeyCode {
    match role {
        SemanticsRole::Checkbox
        | SemanticsRole::Switch
        | SemanticsRole::RadioButton
        | SemanticsRole::MenuItemCheckbox
        | SemanticsRole::MenuItemRadio => KeyCode::Space,
        SemanticsRole::Button
        | SemanticsRole::Link
        | SemanticsRole::MenuItem
        | SemanticsRole::Tab
        | SemanticsRole::ListBoxOption
        | SemanticsRole::TreeItem
        | SemanticsRole::ComboBox => KeyCode::Enter,
        _ => KeyCode::Space,
    }
}

pub fn invoke_with_role(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    preferred_role: Option<SemanticsRole>,
) {
    let key = preferred_role
        .or_else(|| {
            ui.semantics_snapshot().and_then(|snapshot| {
                snapshot
                    .nodes
                    .iter()
                    .find(|node| node.id == target)
                    .map(|node| node.role)
            })
        })
        .map(invoke_key_for_role)
        .unwrap_or(KeyCode::Space);
    press_key(ui, app, services, target, key);
}

pub fn invoke(ui: &mut UiTree, app: &mut App, services: &mut dyn UiServices, target: NodeId) {
    invoke_with_role(ui, app, services, target, None);
}

pub fn focus(ui: &mut UiTree, app: &mut App, target: NodeId) {
    ui.set_focus(Some(target));
    ui.publish_window_runtime_snapshots(app);
}

pub fn set_value_text(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: &str,
) {
    ui.set_focus(Some(target));
    let _ = ui.dispatch_command(app, services, &CommandId::from("edit.select_all"));
    ui.dispatch_event(app, services, &Event::TextInput(value.to_string()));
}

pub fn set_value_numeric(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: f64,
) {
    if value.is_finite()
        && let Some(snapshot) = ui.semantics_snapshot()
        && let Some(node) = snapshot.nodes.iter().find(|n| n.id == target)
        && matches!(
            node.role,
            SemanticsRole::Slider | SemanticsRole::SpinButton | SemanticsRole::Splitter
        )
    {
        let Some(min) = node.extra.numeric.min.filter(|v| v.is_finite()) else {
            set_value_text(ui, app, services, target, &value.to_string());
            return;
        };
        let Some(max) = node.extra.numeric.max.filter(|v| v.is_finite()) else {
            set_value_text(ui, app, services, target, &value.to_string());
            return;
        };
        if min > max {
            set_value_text(ui, app, services, target, &value.to_string());
            return;
        }

        let Some(step) = node
            .extra
            .numeric
            .step
            .filter(|v| v.is_finite() && *v > 0.0)
        else {
            set_value_text(ui, app, services, target, &value.to_string());
            return;
        };

        let actions = slider_converge_actions(
            SliderNumeric {
                cur: node.extra.numeric.value,
                min,
                max,
                step,
                jump: node.extra.numeric.jump,
            },
            value,
        );
        for action in actions {
            let key = match action {
                SliderConvergeAction::Home => KeyCode::Home,
                SliderConvergeAction::End => KeyCode::End,
                SliderConvergeAction::PageUp => KeyCode::PageUp,
                SliderConvergeAction::PageDown => KeyCode::PageDown,
                SliderConvergeAction::ArrowUp => KeyCode::ArrowUp,
                SliderConvergeAction::ArrowDown => KeyCode::ArrowDown,
            };
            press_key(ui, app, services, target, key);
        }

        return;
    }

    // Fallback for widgets that don't support structured numeric stepping: treat the numeric value
    // as text input, primarily for text fields.
    set_value_text(ui, app, services, target, &value.to_string());
}

pub fn decrement(ui: &mut UiTree, app: &mut App, services: &mut dyn UiServices, target: NodeId) {
    press_key(ui, app, services, target, KeyCode::ArrowDown);
}

pub fn increment(ui: &mut UiTree, app: &mut App, services: &mut dyn UiServices, target: NodeId) {
    press_key(ui, app, services, target, KeyCode::ArrowUp);
}

pub fn scroll_by(ui: &mut UiTree, app: &mut App, target: NodeId, dx: f64, dy: f64) {
    let dx = dx.clamp(-1_000_000.0, 1_000_000.0) as f32;
    let dy = dy.clamp(-1_000_000.0, 1_000_000.0) as f32;
    let _ = ui.scroll_by(app, target, Point::new(Px(dx), Px(dy)));
}

pub fn set_text_selection(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    anchor: u32,
    focus: u32,
) {
    ui.set_focus(Some(target));
    ui.dispatch_event(app, services, &Event::SetTextSelection { anchor, focus });
}

pub fn replace_selected_text(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: &str,
) {
    ui.set_focus(Some(target));
    ui.dispatch_event(app, services, &Event::TextInput(value.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_converge_actions_works_without_jump() {
        let meta = SliderNumeric {
            cur: Some(50.0),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            jump: None,
        };
        let actions = slider_converge_actions(meta, 42.0);
        assert!(!actions.is_empty());
        assert!(
            actions
                .iter()
                .all(|a| matches!(a, SliderConvergeAction::ArrowDown))
        );
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn slider_converge_actions_ignores_unaligned_jump() {
        let meta = SliderNumeric {
            cur: Some(50.0),
            min: 0.0,
            max: 100.0,
            step: 6.0,
            jump: Some(10.0),
        };
        let actions = slider_converge_actions(meta, 44.0);
        assert!(!actions.is_empty());
        assert!(
            actions.iter().all(|a| !matches!(
                a,
                SliderConvergeAction::PageUp | SliderConvergeAction::PageDown
            )),
            "expected unaligned jump to avoid page actions"
        );
    }

    #[test]
    fn invoke_key_for_button_like_roles_uses_enter() {
        assert_eq!(invoke_key_for_role(SemanticsRole::Button), KeyCode::Enter);
        assert_eq!(invoke_key_for_role(SemanticsRole::MenuItem), KeyCode::Enter);
        assert_eq!(invoke_key_for_role(SemanticsRole::ComboBox), KeyCode::Enter);
    }

    #[test]
    fn invoke_key_for_toggle_like_roles_uses_space() {
        assert_eq!(invoke_key_for_role(SemanticsRole::Checkbox), KeyCode::Space);
        assert_eq!(invoke_key_for_role(SemanticsRole::Switch), KeyCode::Space);
        assert_eq!(
            invoke_key_for_role(SemanticsRole::MenuItemCheckbox),
            KeyCode::Space
        );
    }
}
