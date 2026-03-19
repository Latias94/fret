//! Field state primitives (shadcn/Base UI aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/ui/apps/v4/registry/bases/base/ui/field.tsx` (`data-invalid`, `data-disabled`)
//!
//! In shadcn/ui v4, `Field` is a styling/grouping wrapper that carries state via data attributes
//! (e.g. `data-invalid`, `data-disabled`). Downstream parts like `FieldLabel` and `FieldTitle`
//! respond to those attributes via CSS selectors.
//!
//! Fret does not have DOM/CSS inheritance, so we model the same outcome via an element-scope
//! provider that components can query during `into_element` construction.

use crate::primitives::control_registry::ControlId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FieldState {
    pub invalid: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldControlAssociation {
    pub control_id: ControlId,
}

pub fn inherited_field_state<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<FieldState> {
    cx.provided::<FieldState>().copied()
}

pub fn use_field_state_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
    local: Option<FieldState>,
) -> FieldState {
    local.or(inherited_field_state(cx)).unwrap_or_default()
}

pub fn inherited_field_control_id<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<ControlId> {
    cx.provided::<FieldControlAssociation>()
        .map(|assoc| assoc.control_id.clone())
}

pub fn use_field_control_id_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
    local: Option<ControlId>,
) -> Option<ControlId> {
    local.or_else(|| inherited_field_control_id(cx))
}

#[track_caller]
pub fn with_field_state_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    state: FieldState,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(state, f)
}

#[track_caller]
pub fn with_field_control_association_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    control_id: Option<ControlId>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    if let Some(control_id) = control_id {
        cx.provide(FieldControlAssociation { control_id }, f)
    } else {
        f(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)))
    }

    #[test]
    fn field_state_provider_inherits_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            assert_eq!(inherited_field_state(cx), None);
            assert_eq!(use_field_state_in_scope(cx, None), FieldState::default());

            with_field_state_provider(
                cx,
                FieldState {
                    invalid: true,
                    disabled: false,
                },
                |cx| {
                    assert_eq!(
                        use_field_state_in_scope(cx, None),
                        FieldState {
                            invalid: true,
                            disabled: false,
                        }
                    );
                    cx.scope(|cx| {
                        assert_eq!(
                            use_field_state_in_scope(cx, None),
                            FieldState {
                                invalid: true,
                                disabled: false,
                            }
                        );
                    });
                },
            );

            assert_eq!(use_field_state_in_scope(cx, None), FieldState::default());
        });
    }

    #[test]
    fn field_control_association_provider_inherits_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let control_id = ControlId::from("field.control");

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            assert_eq!(inherited_field_control_id(cx), None);
            assert_eq!(use_field_control_id_in_scope(cx, None), None);

            with_field_control_association_provider(cx, Some(control_id.clone()), |cx| {
                assert_eq!(inherited_field_control_id(cx), Some(control_id.clone()));
                assert_eq!(
                    use_field_control_id_in_scope(cx, None),
                    Some(control_id.clone())
                );
                cx.scope(|cx| {
                    assert_eq!(
                        use_field_control_id_in_scope(cx, None),
                        Some(control_id.clone())
                    );
                });
            });

            assert_eq!(use_field_control_id_in_scope(cx, None), None);
        });
    }
}
