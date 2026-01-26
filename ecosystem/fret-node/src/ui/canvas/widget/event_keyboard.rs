use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_key_down<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
        modifiers: fret_core::Modifiers,
    ) {
        if cx.input_ctx.focus_is_text_input {
            return;
        }

        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);

        if key == fret_core::KeyCode::Escape {
            if searcher::handle_searcher_escape(self, cx)
                || context_menu::handle_context_menu_escape(self, cx)
            {
                return;
            }
            cancel::handle_escape_cancel(self, cx);
            return;
        }

        if searcher::handle_searcher_key_down(self, cx, key, modifiers)
            || context_menu::handle_context_menu_key_down(self, cx, key)
        {
            return;
        }

        if modifiers.ctrl || modifiers.meta {
            if !snapshot.interaction.disable_keyboard_a11y && key == fret_core::KeyCode::Tab {
                let cmd = if modifiers.shift {
                    CMD_NODE_GRAPH_FOCUS_PREV_EDGE
                } else {
                    CMD_NODE_GRAPH_FOCUS_NEXT_EDGE
                };
                cx.dispatch_command(CommandId::from(cmd));
                cx.stop_propagation();
                return;
            }

            match key {
                fret_core::KeyCode::KeyA => {
                    cx.dispatch_command(CommandId::from("edit.select_all"));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyZ => {
                    let cmd = if modifiers.shift {
                        CMD_NODE_GRAPH_REDO
                    } else {
                        CMD_NODE_GRAPH_UNDO
                    };
                    cx.dispatch_command(CommandId::from(cmd));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyY => {
                    cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_REDO));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyC => {
                    cx.dispatch_command(CommandId::from("edit.copy"));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyX => {
                    cx.dispatch_command(CommandId::from("edit.cut"));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyV => {
                    cx.dispatch_command(CommandId::from("edit.paste"));
                    cx.stop_propagation();
                    return;
                }
                fret_core::KeyCode::KeyD => {
                    cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DUPLICATE));
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        if !snapshot.interaction.disable_keyboard_a11y
            && key == fret_core::KeyCode::Tab
            && !modifiers.ctrl
            && !modifiers.meta
            && !modifiers.alt
            && !modifiers.alt_gr
        {
            if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
                return;
            }

            let cmd = if modifiers.shift {
                CMD_NODE_GRAPH_FOCUS_PREV
            } else {
                CMD_NODE_GRAPH_FOCUS_NEXT
            };
            cx.dispatch_command(CommandId::from(cmd));
            cx.stop_propagation();
            return;
        }

        if !modifiers.ctrl && !modifiers.meta && !modifiers.alt && !modifiers.alt_gr {
            if snapshot.interaction.space_to_pan
                && self.interaction.searcher.is_none()
                && self.interaction.context_menu.is_none()
            {
                if let Some(crate::io::NodeGraphKeyCode(key_code)) =
                    snapshot.interaction.pan_activation_key_code
                {
                    if key == key_code && !self.interaction.pan_activation_key_held {
                        self.interaction.pan_activation_key_held = true;
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        cx.stop_propagation();
                        return;
                    }
                }
            }
        }

        if matches!(
            key,
            fret_core::KeyCode::ArrowLeft
                | fret_core::KeyCode::ArrowRight
                | fret_core::KeyCode::ArrowUp
                | fret_core::KeyCode::ArrowDown
        ) && !modifiers.ctrl
            && !modifiers.meta
            && !modifiers.alt
            && !modifiers.alt_gr
        {
            if snapshot.interaction.disable_keyboard_a11y {
                return;
            }

            if snapshot.selected_nodes.is_empty() && snapshot.selected_groups.is_empty() {
                return;
            }

            let cmd = match (key, modifiers.shift) {
                (fret_core::KeyCode::ArrowLeft, false) => CMD_NODE_GRAPH_NUDGE_LEFT,
                (fret_core::KeyCode::ArrowRight, false) => CMD_NODE_GRAPH_NUDGE_RIGHT,
                (fret_core::KeyCode::ArrowUp, false) => CMD_NODE_GRAPH_NUDGE_UP,
                (fret_core::KeyCode::ArrowDown, false) => CMD_NODE_GRAPH_NUDGE_DOWN,
                (fret_core::KeyCode::ArrowLeft, true) => CMD_NODE_GRAPH_NUDGE_LEFT_FAST,
                (fret_core::KeyCode::ArrowRight, true) => CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
                (fret_core::KeyCode::ArrowUp, true) => CMD_NODE_GRAPH_NUDGE_UP_FAST,
                (fret_core::KeyCode::ArrowDown, true) => CMD_NODE_GRAPH_NUDGE_DOWN_FAST,
                _ => return,
            };
            cx.dispatch_command(CommandId::from(cmd));
            cx.stop_propagation();
            return;
        }

        if !snapshot.interaction.delete_key.matches(key) {
            return;
        }

        cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION));
        cx.stop_propagation();
    }

    pub(super) fn handle_key_up<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
    ) {
        let Some(crate::io::NodeGraphKeyCode(key_code)) =
            snapshot.interaction.pan_activation_key_code
        else {
            return;
        };
        if key == key_code && self.interaction.pan_activation_key_held {
            self.interaction.pan_activation_key_held = false;
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
    }
}
