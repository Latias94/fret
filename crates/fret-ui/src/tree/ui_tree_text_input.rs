use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn platform_text_input_query(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        query: &fret_runtime::PlatformTextInputQuery,
    ) -> fret_runtime::PlatformTextInputQueryResult {
        let focus_is_text_input = self.focus_is_text_input(app);
        if !focus_is_text_input {
            return match query {
                fret_runtime::PlatformTextInputQuery::SelectedTextRange
                | fret_runtime::PlatformTextInputQuery::MarkedTextRange => {
                    fret_runtime::PlatformTextInputQueryResult::Range(None)
                }
                fret_runtime::PlatformTextInputQuery::TextForRange { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Text(None)
                }
                fret_runtime::PlatformTextInputQuery::BoundsForRange { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Bounds(None)
                }
                fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Index(None)
                }
            };
        }

        let Some(focus) = self.focus else {
            return match query {
                fret_runtime::PlatformTextInputQuery::SelectedTextRange
                | fret_runtime::PlatformTextInputQuery::MarkedTextRange => {
                    fret_runtime::PlatformTextInputQueryResult::Range(None)
                }
                fret_runtime::PlatformTextInputQuery::TextForRange { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Text(None)
                }
                fret_runtime::PlatformTextInputQuery::BoundsForRange { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Bounds(None)
                }
                fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint { .. } => {
                    fret_runtime::PlatformTextInputQueryResult::Index(None)
                }
            };
        };

        let bounds = self.nodes.get(focus).map(|n| n.bounds).unwrap_or_default();

        match query {
            fret_runtime::PlatformTextInputQuery::SelectedTextRange => {
                let range = self
                    .nodes
                    .get(focus)
                    .and_then(|n| n.widget.as_ref())
                    .and_then(|w| w.platform_text_input_selected_range_utf16());
                fret_runtime::PlatformTextInputQueryResult::Range(range)
            }
            fret_runtime::PlatformTextInputQuery::MarkedTextRange => {
                let range = self
                    .nodes
                    .get(focus)
                    .and_then(|n| n.widget.as_ref())
                    .and_then(|w| w.platform_text_input_marked_range_utf16());
                fret_runtime::PlatformTextInputQueryResult::Range(range)
            }
            fret_runtime::PlatformTextInputQuery::TextForRange { range } => {
                let text = self
                    .nodes
                    .get(focus)
                    .and_then(|n| n.widget.as_ref())
                    .and_then(|w| w.platform_text_input_text_for_range_utf16(*range));
                fret_runtime::PlatformTextInputQueryResult::Text(text)
            }
            fret_runtime::PlatformTextInputQuery::BoundsForRange { range } => {
                let out = self.with_widget_mut(focus, |w, _tree| {
                    let mut cx = PlatformTextInputCx {
                        app,
                        services,
                        window: _tree.window,
                        node: focus,
                        bounds,
                        scale_factor,
                    };
                    w.platform_text_input_bounds_for_range_utf16(&mut cx, *range)
                });
                fret_runtime::PlatformTextInputQueryResult::Bounds(out)
            }
            fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint { point } => {
                let out = self.with_widget_mut(focus, |w, _tree| {
                    let mut cx = PlatformTextInputCx {
                        app,
                        services,
                        window: _tree.window,
                        node: focus,
                        bounds,
                        scale_factor,
                    };
                    w.platform_text_input_character_index_for_point_utf16(&mut cx, *point)
                });
                fret_runtime::PlatformTextInputQueryResult::Index(out)
            }
        }
    }

    pub fn platform_text_input_replace_text_in_range_utf16(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        range: fret_runtime::Utf16Range,
        text: &str,
    ) -> bool {
        if !self.focus_is_text_input(app) {
            return false;
        }
        let Some(focus) = self.focus else {
            return false;
        };
        let bounds = self.nodes.get(focus).map(|n| n.bounds).unwrap_or_default();

        let changed = self.with_widget_mut(focus, |w, _tree| {
            let mut cx = PlatformTextInputCx {
                app,
                services,
                window: _tree.window,
                node: focus,
                bounds,
                scale_factor,
            };
            w.platform_text_input_replace_text_in_range_utf16(&mut cx, range, text)
        });
        if changed {
            self.invalidate(focus, Invalidation::Layout);
            self.request_redraw_coalesced(app);
        }
        changed
    }

    pub fn platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        range: fret_runtime::Utf16Range,
        text: &str,
        marked: Option<fret_runtime::Utf16Range>,
    ) -> bool {
        if !self.focus_is_text_input(app) {
            return false;
        }
        let Some(focus) = self.focus else {
            return false;
        };
        let bounds = self.nodes.get(focus).map(|n| n.bounds).unwrap_or_default();

        let changed = self.with_widget_mut(focus, |w, _tree| {
            let mut cx = PlatformTextInputCx {
                app,
                services,
                window: _tree.window,
                node: focus,
                bounds,
                scale_factor,
            };
            w.platform_text_input_replace_and_mark_text_in_range_utf16(&mut cx, range, text, marked)
        });
        if changed {
            self.invalidate(focus, Invalidation::Layout);
            self.request_redraw_coalesced(app);
        }
        changed
    }

    pub(in crate::tree) fn set_ime_allowed(&mut self, app: &mut H, enabled: bool) {
        if self.ime_allowed == enabled {
            return;
        }
        self.ime_allowed = enabled;
        let Some(window) = self.window else {
            return;
        };
        app.push_effect(Effect::ImeAllow { window, enabled });
    }
}
