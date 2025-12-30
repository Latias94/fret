use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::ElementCx;
use fret_ui::UiHost;
use fret_ui::action::RovingTypeaheadCx;

use crate::headless::typeahead::{TypeaheadBuffer, match_prefix_arc_str};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Component-layer helpers for registering runtime action hooks (ADR 0074).
///
/// These helpers keep common interaction policies out of `crates/fret-ui` while remaining easy to
/// use in declarative authoring code.
pub trait ActionHooksExt {
    fn pressable_dispatch_command(&mut self, command: CommandId);

    fn pressable_dispatch_command_opt(&mut self, command: Option<CommandId>);

    fn pressable_toggle_bool(&mut self, model: Model<bool>);

    fn pressable_set_bool(&mut self, model: Model<bool>, value: bool);

    fn pressable_set_arc_str(&mut self, model: Model<Arc<str>>, value: Arc<str>);

    fn pressable_set_option_arc_str(&mut self, model: Model<Option<Arc<str>>>, value: Arc<str>);

    fn pressable_toggle_vec_arc_str(&mut self, model: Model<Vec<Arc<str>>>, value: Arc<str>);

    fn dismissible_close_bool(&mut self, open: Model<bool>);

    fn roving_select_option_arc_str(
        &mut self,
        model: Model<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    );

    fn roving_typeahead_first_char_arc_str(&mut self, labels: Arc<[Arc<str>]>);

    fn roving_typeahead_prefix_arc_str(&mut self, labels: Arc<[Arc<str>]>, timeout_ticks: u64);
}

impl<H: UiHost> ActionHooksExt for ElementCx<'_, H> {
    fn pressable_dispatch_command(&mut self, command: CommandId) {
        self.pressable_add_on_activate(Arc::new(move |host, acx, _reason| {
            host.dispatch_command(Some(acx.window), command.clone());
        }));
    }

    fn pressable_dispatch_command_opt(&mut self, command: Option<CommandId>) {
        let Some(command) = command else {
            return;
        };
        self.pressable_dispatch_command(command);
    }

    fn pressable_toggle_bool(&mut self, model: Model<bool>) {
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let _ = host.models_mut().update(model, |v| *v = !*v);
        }));
    }

    fn pressable_set_bool(&mut self, model: Model<bool>, value: bool) {
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let _ = host.models_mut().update(model, |v| *v = value);
        }));
    }

    fn pressable_set_arc_str(&mut self, model: Model<Arc<str>>, value: Arc<str>) {
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let value = value.clone();
            let _ = host.models_mut().update(model, |v| *v = value);
        }));
    }

    fn pressable_set_option_arc_str(&mut self, model: Model<Option<Arc<str>>>, value: Arc<str>) {
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let value = Some(value.clone());
            let _ = host.models_mut().update(model, |v| *v = value);
        }));
    }

    fn pressable_toggle_vec_arc_str(&mut self, model: Model<Vec<Arc<str>>>, value: Arc<str>) {
        self.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
            let value = value.clone();
            let _ = host.models_mut().update(model, |v| {
                if let Some(pos) = v.iter().position(|it| it.as_ref() == value.as_ref()) {
                    v.remove(pos);
                } else {
                    v.push(value.clone());
                }
            });
        }));
    }

    fn dismissible_close_bool(&mut self, open: Model<bool>) {
        self.dismissible_add_on_dismiss_request(Arc::new(move |host, _cx, _reason| {
            let _ = host.models_mut().update(open, |v| *v = false);
        }));
    }

    fn roving_select_option_arc_str(
        &mut self,
        model: Model<Option<Arc<str>>>,
        values: Arc<[Arc<str>]>,
    ) {
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
                        let _ = host.models_mut().update(model, |v| *v = next);
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
        struct RovingTypeaheadFirstCharArcStrState {
            labels: Rc<RefCell<Arc<[Arc<str>]>>>,
            handler: fret_ui::action::OnRovingTypeahead,
        }

        let handler = self.with_state(
            || {
                let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> =
                    Rc::new(RefCell::new(labels.clone()));
                let labels_read = labels_cell.clone();
                let handler: fret_ui::action::OnRovingTypeahead = Arc::new(
                    move |_host: &mut dyn fret_ui::action::UiActionHost,
                          _cx,
                          it: RovingTypeaheadCx| {
                        let labels = labels_read.borrow();
                        let is_disabled =
                            |idx: usize| it.disabled.get(idx).copied().unwrap_or(false);
                        let matches = |idx: usize| -> bool {
                            if is_disabled(idx) {
                                return false;
                            }
                            let Some(label) = labels.get(idx) else {
                                return false;
                            };
                            let label = label.as_ref().trim_start();
                            let Some(first) = label.chars().next() else {
                                return false;
                            };
                            first.to_ascii_lowercase() == it.input
                        };

                        let start = it.current.map(|i| i.saturating_add(1)).unwrap_or(0);
                        if it.wrap {
                            for offset in 0..it.len {
                                let idx = (start + offset) % it.len;
                                if matches(idx) {
                                    return Some(idx);
                                }
                            }
                            None
                        } else {
                            (start..it.len).find(|&idx| matches(idx))
                        }
                    },
                );

                RovingTypeaheadFirstCharArcStrState {
                    labels: labels_cell,
                    handler,
                }
            },
            |state| {
                *state.labels.borrow_mut() = labels.clone();
                state.handler.clone()
            },
        );

        self.roving_add_on_typeahead(handler);
    }

    fn roving_typeahead_prefix_arc_str(&mut self, labels: Arc<[Arc<str>]>, timeout_ticks: u64) {
        struct RovingTypeaheadPrefixArcStrState {
            timeout_ticks: u64,
            labels: Rc<RefCell<Arc<[Arc<str>]>>>,
            handler: fret_ui::action::OnRovingTypeahead,
        }

        fn make_state(
            labels: Arc<[Arc<str>]>,
            timeout_ticks: u64,
        ) -> RovingTypeaheadPrefixArcStrState {
            let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> = Rc::new(RefCell::new(labels));
            let buffer: Rc<RefCell<TypeaheadBuffer>> =
                Rc::new(RefCell::new(TypeaheadBuffer::new(timeout_ticks)));

            let labels_read = labels_cell.clone();
            let buffer_read = buffer.clone();

            #[allow(clippy::arc_with_non_send_sync)]
            let handler: fret_ui::action::OnRovingTypeahead = Arc::new(
                move |_host: &mut dyn fret_ui::action::UiActionHost, _cx, it: RovingTypeaheadCx| {
                    let mut buf = buffer_read.borrow_mut();
                    buf.push_char(it.input, it.tick);
                    let query = buf.query(it.tick)?;

                    let labels = labels_read.borrow();
                    match_prefix_arc_str(
                        labels.as_ref(),
                        it.disabled.as_ref(),
                        query,
                        it.current,
                        it.wrap,
                    )
                },
            );

            RovingTypeaheadPrefixArcStrState {
                timeout_ticks,
                labels: labels_cell,
                handler,
            }
        }

        let handler = self.with_state(
            || make_state(labels.clone(), timeout_ticks),
            |state| {
                if state.timeout_ticks != timeout_ticks {
                    *state = make_state(labels.clone(), timeout_ticks);
                }
                *state.labels.borrow_mut() = labels.clone();
                state.handler.clone()
            },
        );

        self.roving_add_on_typeahead(handler);
    }
}
