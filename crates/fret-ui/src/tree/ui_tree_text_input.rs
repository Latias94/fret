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

        if let Some(window) = self.window
            && let Some(out) =
                crate::declarative::frame::with_element_record_for_node(app, window, focus, |r| {
                    match &r.instance {
                        crate::declarative::ElementInstance::TextInputRegion(props) => Some(
                            text_input_region_platform_text_input_query(props, query),
                        ),
                        _ => None,
                    }
                })
                .flatten()
        {
            return out;
        }

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
        if let Some(window) = self.window
            && let Some(is_region) =
                crate::declarative::frame::with_element_record_for_node(app, window, focus, |r| {
                    matches!(
                        &r.instance,
                        crate::declarative::ElementInstance::TextInputRegion(_)
                    )
                })
            && is_region
        {
            return false;
        }
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
        if let Some(window) = self.window
            && let Some(is_region) =
                crate::declarative::frame::with_element_record_for_node(app, window, focus, |r| {
                    matches!(
                        &r.instance,
                        crate::declarative::ElementInstance::TextInputRegion(_)
                    )
                })
            && is_region
        {
            return false;
        }
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

pub(in crate::tree) fn text_input_region_platform_text_input_snapshot(
    props: &crate::element::TextInputRegionProps,
) -> fret_runtime::WindowTextInputSnapshot {
    let value = props.a11y_value.as_deref().unwrap_or("");

    let len_utf16_usize = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value,
        value.len(),
        fret_core::utf::UtfIndexClamp::Down,
    );
    let len_utf16 = u32::try_from(len_utf16_usize).unwrap_or(u32::MAX);

    let selection_utf16 = props.a11y_text_selection.map(|(anchor, focus)| {
        let anchor_u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value,
            usize::try_from(anchor).unwrap_or(usize::MAX),
            fret_core::utf::UtfIndexClamp::Down,
        );
        let focus_u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value,
            usize::try_from(focus).unwrap_or(usize::MAX),
            fret_core::utf::UtfIndexClamp::Down,
        );
        (
            u32::try_from(anchor_u16).unwrap_or(u32::MAX),
            u32::try_from(focus_u16).unwrap_or(u32::MAX),
        )
    });

    let marked_utf16 = props.a11y_text_composition.map(|(start, end)| {
        let (s, e) = fret_core::utf::utf8_byte_range_to_utf16_range(
            value,
            usize::try_from(start).unwrap_or(usize::MAX),
            usize::try_from(end).unwrap_or(usize::MAX),
        );
        (u32::try_from(s).unwrap_or(u32::MAX), u32::try_from(e).unwrap_or(u32::MAX))
    });

    fret_runtime::WindowTextInputSnapshot {
        focus_is_text_input: true,
        is_composing: marked_utf16.is_some(),
        text_len_utf16: len_utf16,
        selection_utf16,
        marked_utf16,
        ime_cursor_area: props.ime_cursor_area,
    }
}

fn text_input_region_platform_text_input_query(
    props: &crate::element::TextInputRegionProps,
    query: &fret_runtime::PlatformTextInputQuery,
) -> fret_runtime::PlatformTextInputQueryResult {
    let value = props.a11y_value.as_deref().unwrap_or("");

    match query {
        fret_runtime::PlatformTextInputQuery::SelectedTextRange => {
            let Some((anchor, focus)) = props.a11y_text_selection else {
                return fret_runtime::PlatformTextInputQueryResult::Range(None);
            };

            let anchor_u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
                value,
                usize::try_from(anchor).unwrap_or(usize::MAX),
                fret_core::utf::UtfIndexClamp::Down,
            );
            let focus_u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
                value,
                usize::try_from(focus).unwrap_or(usize::MAX),
                fret_core::utf::UtfIndexClamp::Down,
            );
            let range = fret_runtime::Utf16Range::new(
                u32::try_from(anchor_u16).unwrap_or(u32::MAX),
                u32::try_from(focus_u16).unwrap_or(u32::MAX),
            )
            .normalized();

            fret_runtime::PlatformTextInputQueryResult::Range(Some(range))
        }
        fret_runtime::PlatformTextInputQuery::MarkedTextRange => {
            let Some((start, end)) = props.a11y_text_composition else {
                return fret_runtime::PlatformTextInputQueryResult::Range(None);
            };

            let (s, e) = fret_core::utf::utf8_byte_range_to_utf16_range(
                value,
                usize::try_from(start).unwrap_or(usize::MAX),
                usize::try_from(end).unwrap_or(usize::MAX),
            );
            let range = fret_runtime::Utf16Range::new(
                u32::try_from(s).unwrap_or(u32::MAX),
                u32::try_from(e).unwrap_or(u32::MAX),
            )
            .normalized();

            fret_runtime::PlatformTextInputQueryResult::Range(Some(range))
        }
        fret_runtime::PlatformTextInputQuery::TextForRange { range } => {
            if value.is_empty() {
                return fret_runtime::PlatformTextInputQueryResult::Text(None);
            }

            let range = range.normalized();
            let (bs, be) = fret_core::utf::utf16_range_to_utf8_byte_range(
                value,
                usize::try_from(range.start).unwrap_or(usize::MAX),
                usize::try_from(range.end).unwrap_or(usize::MAX),
            );

            let out = value.get(bs..be).map(ToString::to_string);
            fret_runtime::PlatformTextInputQueryResult::Text(out)
        }
        fret_runtime::PlatformTextInputQuery::BoundsForRange { .. } => {
            fret_runtime::PlatformTextInputQueryResult::Bounds(None)
        }
        fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint { .. } => {
            fret_runtime::PlatformTextInputQueryResult::Index(None)
        }
    }
}
