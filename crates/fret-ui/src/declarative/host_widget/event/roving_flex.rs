use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_roving_flex<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::RovingFlexProps,
    event: &Event,
) {
    if !props.roving.enabled {
        return;
    }

    fn is_roving_item_instance(instance: &crate::declarative::frame::ElementInstance) -> bool {
        matches!(
            instance,
            crate::declarative::frame::ElementInstance::Pressable(_)
                | crate::declarative::frame::ElementInstance::TextInput(_)
                | crate::declarative::frame::ElementInstance::TextArea(_)
        )
    }

    fn is_roving_group_wrapper_instance(
        instance: &crate::declarative::frame::ElementInstance,
    ) -> bool {
        match instance {
            crate::declarative::frame::ElementInstance::Semantics(props) => matches!(
                props.role,
                fret_core::SemanticsRole::Group | fret_core::SemanticsRole::RadioGroup
            ),
            crate::declarative::frame::ElementInstance::SemanticFlex(props) => matches!(
                props.role,
                fret_core::SemanticsRole::Group | fret_core::SemanticsRole::RadioGroup
            ),
            _ => false,
        }
    }

    fn collect_roving_items_in_subtree<H: UiHost>(
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        out: &mut Vec<NodeId>,
    ) {
        let children =
            crate::declarative::mount::children_for_node_in_window_frame(app, window, node);
        for child in children {
            let Some(record) =
                crate::declarative::frame::element_record_for_node(app, window, child)
            else {
                continue;
            };

            if is_roving_item_instance(&record.instance) {
                out.push(child);
                continue;
            }

            // Within a group wrapper subtree, we want to pick up all focusable items in visual
            // order, even if they are nested under layout containers (e.g. Flex).
            collect_roving_items_in_subtree(app, window, child, out);
        }
    }

    fn node_contains_in_window_frame<H: UiHost>(
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
        needle: NodeId,
    ) -> bool {
        if root == needle {
            return true;
        }

        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let children =
                crate::declarative::mount::children_for_node_in_window_frame(app, window, node);
            for child in children {
                if child == needle {
                    return true;
                }
                stack.push(child);
            }
        }
        false
    }

    struct RovingHookHost<'a, H: UiHost> {
        app: &'a mut H,
        window: AppWindowId,
        element: crate::GlobalElementId,
        requested_focus: &'a mut Option<NodeId>,
        notify_requested: &'a mut bool,
        notify_requested_location: &'a mut Option<crate::widget::UiSourceLocation>,
    }

    impl<H: UiHost> action::UiActionHost for RovingHookHost<'_, H> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            match effect {
                Effect::SetTimer {
                    window: Some(window),
                    token,
                    ..
                } if window == self.window => {
                    crate::elements::record_timer_target(
                        &mut *self.app,
                        window,
                        token,
                        self.element,
                    );
                }
                Effect::CancelTimer { token } => {
                    crate::elements::clear_timer_target(&mut *self.app, self.window, token);
                }
                _ => {}
            }
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            self.app.next_share_sheet_token()
        }

        #[track_caller]
        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
            if self.notify_requested_location.is_none() {
                let caller = std::panic::Location::caller();
                *self.notify_requested_location = Some(crate::widget::UiSourceLocation {
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                });
            }
        }
    }

    impl<H: UiHost> action::UiFocusActionHost for RovingHookHost<'_, H> {
        fn request_focus(&mut self, target: crate::GlobalElementId) {
            let Some(node) =
                crate::elements::with_window_state(&mut *self.app, self.window, |window_state| {
                    window_state.node_entry(target).map(|e| e.node)
                })
            else {
                return;
            };
            *self.requested_focus = Some(node);
        }
    }

    let Event::KeyDown {
        key,
        modifiers,
        repeat,
    } = event
    else {
        return;
    };
    if *repeat {
        return;
    }

    let key_hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::RovingActionHooks::default,
        |hooks| hooks.on_key_down.clone(),
    );
    if let Some(h) = key_hook {
        let mut host = RovingHookHost {
            app: &mut *cx.app,
            window,
            element: this.element,
            requested_focus: &mut cx.requested_focus,
            notify_requested: &mut cx.notify_requested,
            notify_requested_location: &mut cx.notify_requested_location,
        };
        let handled = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            action::KeyDownCx {
                key: *key,
                modifiers: *modifiers,
                repeat: *repeat,
            },
        );
        if handled {
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }
    }

    let len = cx.children.len();
    if len == 0 {
        return;
    }

    // By default, roving items are expected to be direct children of the roving container.
    //
    // Some composite widgets (e.g. menu `Group`/`RadioGroup`) introduce structural wrappers for
    // semantics while keeping the same visual/interaction model. For those wrappers, we collect
    // roving items from their descendant subtree.
    let mut roving_items: Vec<NodeId> = Vec::new();
    for &child in cx.children {
        let Some(record) =
            crate::declarative::frame::element_record_for_node(cx.app, window, child)
        else {
            continue;
        };
        if is_roving_item_instance(&record.instance) {
            roving_items.push(child);
            continue;
        }
        if is_roving_group_wrapper_instance(&record.instance) {
            collect_roving_items_in_subtree(cx.app, window, child, &mut roving_items);
        }
    }
    if roving_items.is_empty() {
        return;
    }

    let current = cx.focus.and_then(|focus| {
        roving_items.iter().position(|n| *n == focus).or_else(|| {
            roving_items
                .iter()
                .position(|&root| node_contains_in_window_frame(cx.app, window, root, focus))
        })
    });

    let navigate_hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::RovingActionHooks::default,
        |hooks| hooks.on_navigate.clone(),
    );

    let mut target: Option<usize> = None;
    let mut handled = false;

    if let Some(h) = navigate_hook {
        let mut host = RovingHookHost {
            app: &mut *cx.app,
            window,
            element: this.element,
            requested_focus: &mut cx.requested_focus,
            notify_requested: &mut cx.notify_requested,
            notify_requested_location: &mut cx.notify_requested_location,
        };
        let result = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            crate::action::RovingNavigateCx {
                key: *key,
                modifiers: *modifiers,
                repeat: *repeat,
                axis: props.flex.direction,
                current,
                len: roving_items.len(),
                disabled: props.roving.disabled.clone(),
                wrap: props.roving.wrap,
            },
        );

        if let crate::action::RovingNavigateResult::Handled {
            target: next_target,
        } = result
        {
            handled = true;
            target = next_target;
        }
    }

    if !handled
        && target.is_none()
        && let Some(ch) = fret_core::keycode_to_ascii_lowercase(*key)
    {
        let hook = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            this.element,
            crate::action::RovingActionHooks::default,
            |hooks| hooks.on_typeahead.clone(),
        );

        if let Some(h) = hook {
            let tick = cx.app.tick_id().0;
            let mut host = RovingHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                requested_focus: &mut cx.requested_focus,
                notify_requested: &mut cx.notify_requested,
                notify_requested_location: &mut cx.notify_requested_location,
            };
            target = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                crate::action::RovingTypeaheadCx {
                    input: ch,
                    current,
                    len: roving_items.len(),
                    disabled: props.roving.disabled.clone(),
                    wrap: props.roving.wrap,
                    tick,
                },
            );
        }
    }

    if handled && target.is_none() {
        cx.stop_propagation();
        return;
    }

    let Some(target) = target else {
        return;
    };
    if current.is_some_and(|current| target == current) {
        if handled {
            cx.stop_propagation();
        }
        return;
    }

    if target >= roving_items.len() {
        return;
    }
    cx.request_focus(roving_items[target]);

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::RovingActionHooks::default,
        |hooks| hooks.on_active_change.clone(),
    );

    if let Some(h) = hook {
        let mut host = RovingHookHost {
            app: &mut *cx.app,
            window,
            element: this.element,
            requested_focus: &mut cx.requested_focus,
            notify_requested: &mut cx.notify_requested,
            notify_requested_location: &mut cx.notify_requested_location,
        };
        h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            target,
        );
    }

    cx.request_redraw();
    cx.stop_propagation();
}
