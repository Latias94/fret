use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn dismiss_topmost_overlay_on_escape(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        base_root: NodeId,
        barrier_root: Option<NodeId>,
    ) -> bool {
        struct EscapeDismissHookHost<'a, H: crate::UiHost> {
            app: &'a mut H,
            window: AppWindowId,
            element: crate::GlobalElementId,
        }

        impl<H: crate::UiHost> crate::action::UiActionHost for EscapeDismissHookHost<'_, H> {
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
        }

        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers.into_iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if layer.root == base_root {
                continue;
            }

            let Some(root_element) = self.nodes.get(layer.root).and_then(|n| n.element) else {
                continue;
            };
            let hook = crate::elements::with_element_state(
                app,
                window,
                root_element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_dismiss_request.clone(),
            );
            let Some(hook) = hook else {
                if barrier_root == Some(layer.root) {
                    break;
                }
                continue;
            };

            let mut host = EscapeDismissHookHost {
                app,
                window,
                element: root_element,
            };
            let mut req =
                crate::action::DismissRequestCx::new(crate::action::DismissReason::Escape);
            hook(
                &mut host,
                crate::action::ActionCx {
                    window,
                    target: root_element,
                },
                &mut req,
            );
            return true;
        }

        false
    }
}
