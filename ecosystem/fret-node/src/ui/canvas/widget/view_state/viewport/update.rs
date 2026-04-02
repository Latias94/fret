use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn update_editor_config<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphEditorConfig),
    ) {
        if let Some(store) = self.store.as_ref() {
            let _ = store.update(host, |store, _cx| store.update_editor_config(f));
        } else if let Some(editor_config) = self.editor_config_model.as_ref() {
            let _ = editor_config.update(host, |state, _cx| f(state));
        } else {
            f(&mut self.editor_config);
        }
        self.sync_view_state(host);
    }

    pub(in super::super::super) fn update_view_state<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphViewState),
    ) {
        let before = if self.callbacks.is_some() {
            if let Some(store) = self.store.as_ref() {
                store.read_ref(host, |s| s.view_state().clone()).ok()
            } else {
                self.view_state.read_ref(host, |s| s.clone()).ok()
            }
        } else {
            None
        };

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let style = self.style.clone();
        let translate_extent = self.editor_config_snapshot(host).interaction.translate_extent;
        if let Some(store) = self.store.as_ref() {
            let _ = store.update(host, |store, _cx| {
                store.update_view_state(|s| {
                    f(s);

                    let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                        s.zoom
                            .clamp(style.geometry.min_zoom, style.geometry.max_zoom)
                    } else {
                        1.0
                    };
                    s.zoom = zoom;

                    if let Some(extent) = translate_extent {
                        s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                    }
                });
            });
        } else {
            let _ = self.view_state.update(host, |s, _cx| {
                f(s);

                let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                    s.zoom
                        .clamp(style.geometry.min_zoom, style.geometry.max_zoom)
                } else {
                    1.0
                };
                s.zoom = zoom;

                if let Some(extent) = translate_extent {
                    s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                }
            });
        }
        self.sync_view_state(host);

        if let Some(before) = before {
            let after = self.view_state.read_ref(host, |s| s.clone()).ok();
            if let Some(after) = after {
                let mut changes: Vec<ViewChange> = Vec::new();
                if before.pan != after.pan || (before.zoom - after.zoom).abs() > 1.0e-6 {
                    changes.push(ViewChange::Viewport {
                        pan: after.pan,
                        zoom: after.zoom,
                    });
                }
                if before.selected_nodes != after.selected_nodes
                    || before.selected_edges != after.selected_edges
                    || before.selected_groups != after.selected_groups
                {
                    changes.push(ViewChange::Selection {
                        nodes: after.selected_nodes.clone(),
                        edges: after.selected_edges.clone(),
                        groups: after.selected_groups.clone(),
                    });
                }
                self.emit_view_callbacks(&changes);
            }
        }
    }
}
