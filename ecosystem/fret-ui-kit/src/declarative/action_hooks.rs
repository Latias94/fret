use fret_runtime::{CommandId, Model, WeakModel};
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt;

use crate::command::ElementCommandGatingExt as _;
use crate::primitives::roving_focus_group;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Component-layer helpers for registering runtime action hooks (ADR 0074).
///
/// These helpers keep common interaction policies out of `crates/fret-ui` while remaining easy to
/// use in declarative authoring code.
///
/// Note: command-dispatch helpers are intentionally gated (`*_if_enabled`) so "disabled" UI stays
/// consistent across surfaces (menus, command palette, shortcuts, OS menus).
pub trait ActionHooksExt {
    fn pressable_dispatch_command_if_enabled(&mut self, command: CommandId);

    fn pressable_dispatch_command_if_enabled_opt(&mut self, command: Option<CommandId>);

    fn pressable_update_model<T, F>(&mut self, model: &Model<T>, update: F)
    where
        T: 'static,
        F: Fn(&mut T) + 'static;

    fn pressable_update_weak_model<T, F>(&mut self, model: &WeakModel<T>, update: F)
    where
        T: 'static,
        F: Fn(&mut T) + 'static;

    fn pressable_set_model<T>(&mut self, model: &Model<T>, value: T)
    where
        T: Clone + 'static;

    fn pressable_set_weak_model<T>(&mut self, model: &WeakModel<T>, value: T)
    where
        T: Clone + 'static;

    fn pressable_toggle_bool(&mut self, model: &Model<bool>);

    fn pressable_toggle_bool_weak(&mut self, model: &WeakModel<bool>);

    fn pressable_set_bool(&mut self, model: &Model<bool>, value: bool);

    fn pressable_set_bool_weak(&mut self, model: &WeakModel<bool>, value: bool);

    fn pressable_set_arc_str(&mut self, model: &Model<Arc<str>>, value: Arc<str>);

    fn pressable_set_arc_str_weak(&mut self, model: &WeakModel<Arc<str>>, value: Arc<str>);

    fn pressable_set_option_arc_str(&mut self, model: &Model<Option<Arc<str>>>, value: Arc<str>);

    fn pressable_set_option_arc_str_weak(
        &mut self,
        model: &WeakModel<Option<Arc<str>>>,
        value: Arc<str>,
    );

    fn pressable_toggle_vec_arc_str(&mut self, model: &Model<Vec<Arc<str>>>, value: Arc<str>);

    fn pressable_toggle_vec_arc_str_weak(
        &mut self,
        model: &WeakModel<Vec<Arc<str>>>,
        value: Arc<str>,
    );

    fn dismissible_close_bool(&mut self, open: &Model<bool>);

    fn dismissible_close_bool_weak(&mut self, open: &WeakModel<bool>);

    fn roving_select_option_arc_str(
        &mut self,
        model: &Model<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    );

    fn roving_select_option_arc_str_weak(
        &mut self,
        model: &WeakModel<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    );

    fn roving_typeahead_first_char_arc_str(&mut self, labels: Arc<[Arc<str>]>);

    fn roving_typeahead_prefix_arc_str(&mut self, labels: Arc<[Arc<str>]>, timeout_ticks: u64);

    /// Install an APG-aligned default keyboard navigation policy for `RovingFlex`.
    ///
    /// This keeps navigation policy out of `crates/fret-ui` while keeping declarative call sites
    /// small and consistent.
    fn roving_nav_apg(&mut self);
}

impl<H: UiHost> ActionHooksExt for ElementContext<'_, H> {
    fn pressable_dispatch_command_if_enabled(&mut self, command: CommandId) {
        if !self.command_is_enabled(&command) {
            return;
        }
        self.pressable_add_on_activate(Arc::new(move |host, acx, _reason| {
            host.dispatch_command(Some(acx.window), command.clone());
        }));
    }

    fn pressable_dispatch_command_if_enabled_opt(&mut self, command: Option<CommandId>) {
        let Some(command) = command else {
            return;
        };
        self.pressable_dispatch_command_if_enabled(command);
    }

    fn pressable_update_model<T, F>(&mut self, model: &Model<T>, update: F)
    where
        T: 'static,
        F: Fn(&mut T) + 'static,
    {
        let model = model.clone();
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let _ = host.models_mut().update(&model, |v| update(v));
        }));
    }

    fn pressable_update_weak_model<T, F>(&mut self, model: &WeakModel<T>, update: F)
    where
        T: 'static,
        F: Fn(&mut T) + 'static,
    {
        let model = model.clone();
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let _ = host.update_weak_model(&model, |v| update(v));
        }));
    }

    fn pressable_set_model<T>(&mut self, model: &Model<T>, value: T)
    where
        T: Clone + 'static,
    {
        self.pressable_update_model(model, move |v| *v = value.clone());
    }

    fn pressable_set_weak_model<T>(&mut self, model: &WeakModel<T>, value: T)
    where
        T: Clone + 'static,
    {
        self.pressable_update_weak_model(model, move |v| *v = value.clone());
    }

    fn pressable_toggle_bool(&mut self, model: &Model<bool>) {
        self.pressable_update_model(model, |v| *v = !*v);
    }

    fn pressable_toggle_bool_weak(&mut self, model: &WeakModel<bool>) {
        self.pressable_update_weak_model(model, |v| *v = !*v);
    }

    fn pressable_set_bool(&mut self, model: &Model<bool>, value: bool) {
        self.pressable_set_model(model, value);
    }

    fn pressable_set_bool_weak(&mut self, model: &WeakModel<bool>, value: bool) {
        self.pressable_set_weak_model(model, value);
    }

    fn pressable_set_arc_str(&mut self, model: &Model<Arc<str>>, value: Arc<str>) {
        self.pressable_set_model(model, value);
    }

    fn pressable_set_arc_str_weak(&mut self, model: &WeakModel<Arc<str>>, value: Arc<str>) {
        self.pressable_set_weak_model(model, value);
    }

    fn pressable_set_option_arc_str(&mut self, model: &Model<Option<Arc<str>>>, value: Arc<str>) {
        self.pressable_set_model(model, Some(value));
    }

    fn pressable_set_option_arc_str_weak(
        &mut self,
        model: &WeakModel<Option<Arc<str>>>,
        value: Arc<str>,
    ) {
        self.pressable_set_weak_model(model, Some(value));
    }

    fn pressable_toggle_vec_arc_str(&mut self, model: &Model<Vec<Arc<str>>>, value: Arc<str>) {
        let model = model.clone();
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let value = value.clone();
            let _ = host.models_mut().update(&model, |v| {
                if let Some(pos) = v.iter().position(|it| it.as_ref() == value.as_ref()) {
                    v.remove(pos);
                } else {
                    v.push(value.clone());
                }
            });
        }));
    }

    fn pressable_toggle_vec_arc_str_weak(
        &mut self,
        model: &WeakModel<Vec<Arc<str>>>,
        value: Arc<str>,
    ) {
        let model = model.clone();
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let value = value.clone();
            let _ = host.update_weak_model(&model, |v| {
                if let Some(pos) = v.iter().position(|it| it.as_ref() == value.as_ref()) {
                    v.remove(pos);
                } else {
                    v.push(value.clone());
                }
            });
        }));
    }

    fn dismissible_close_bool(&mut self, open: &Model<bool>) {
        let open = open.clone();
        self.dismissible_add_on_dismiss_request(Arc::new(move |host, _cx, _req| {
            let _ = host.models_mut().update(&open, |v| *v = false);
        }));
    }

    fn dismissible_close_bool_weak(&mut self, open: &WeakModel<bool>) {
        let open = open.clone();
        self.dismissible_add_on_dismiss_request(Arc::new(move |host, _cx, _req| {
            let _ = host.update_weak_model(&open, |v| *v = false);
        }));
    }

    fn roving_select_option_arc_str(
        &mut self,
        model: &Model<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    ) {
        let model = model.clone();
        struct RovingSelectOptionArcStrState {
            values: Rc<RefCell<Arc<[Arc<str>]>>>,
            handler: fret_ui::action::OnRovingActiveChange,
        }

        let handler = self.with_state(
            || {
                let values_cell: Rc<RefCell<Arc<[Arc<str>]>>> =
                    Rc::new(RefCell::new(values.clone()));
                let values_read = values_cell.clone();
                let handler: fret_ui::action::OnRovingActiveChange = Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost, _cx, idx| {
                        let values = values_read.borrow();
                        let Some(value) = values.get(idx).cloned() else {
                            return;
                        };
                        let next = Some(value);
                        let _ = host.models_mut().update(&model, |v| *v = next);
                    },
                );

                RovingSelectOptionArcStrState {
                    values: values_cell,
                    handler,
                }
            },
            |state| {
                *state.values.borrow_mut() = values.clone();
                state.handler.clone()
            },
        );

        self.roving_add_on_active_change(handler);
    }

    fn roving_select_option_arc_str_weak(
        &mut self,
        model: &WeakModel<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    ) {
        let model = model.clone();
        struct RovingSelectOptionArcStrState {
            values: Rc<RefCell<Arc<[Arc<str>]>>>,
            handler: fret_ui::action::OnRovingActiveChange,
        }

        let handler = self.with_state(
            || {
                let values_cell: Rc<RefCell<Arc<[Arc<str>]>>> =
                    Rc::new(RefCell::new(values.clone()));
                let values_read = values_cell.clone();
                let handler: fret_ui::action::OnRovingActiveChange = Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost, _cx, idx| {
                        let values = values_read.borrow();
                        let Some(value) = values.get(idx).cloned() else {
                            return;
                        };
                        let next = Some(value);
                        let _ = host.update_weak_model(&model, |v| *v = next);
                    },
                );

                RovingSelectOptionArcStrState {
                    values: values_cell,
                    handler,
                }
            },
            |state| {
                *state.values.borrow_mut() = values.clone();
                state.handler.clone()
            },
        );

        self.roving_add_on_active_change(handler);
    }

    fn roving_typeahead_first_char_arc_str(&mut self, labels: Arc<[Arc<str>]>) {
        roving_focus_group::typeahead_first_char_arc_str(self, labels);
    }

    fn roving_typeahead_prefix_arc_str(&mut self, labels: Arc<[Arc<str>]>, timeout_ticks: u64) {
        roving_focus_group::typeahead_prefix_arc_str(self, labels, timeout_ticks);
    }

    fn roving_nav_apg(&mut self) {
        roving_focus_group::nav_apg(self);
    }
}
