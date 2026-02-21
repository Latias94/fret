use super::{PreparedKey, TextArea};
use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, LayoutCx, PaintCx,
    PlatformTextInputCx, Widget,
};
use crate::{Invalidation, UiHost};
use fret_core::time::Duration;
use fret_core::{
    CaretAffinity, Color, Corners, DrawOrder, Edges, Event, MouseButton, Paint, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::Effect;

impl<H: UiHost> Widget<H> for TextArea {
    fn is_focusable(&self) -> bool {
        self.enabled && self.focusable
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
        let caret = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.caret);
        let selection_anchor =
            crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.selection_anchor);

        let preedit_len = self.preedit.len();
        let is_composing = self.is_ime_composing();

        let (display_anchor, display_focus) = if is_composing {
            let caret_display = self.caret_display_index();
            (caret_display, caret_display)
        } else {
            (
                crate::text_edit::ime::base_to_display_index(caret, preedit_len, selection_anchor),
                crate::text_edit::ime::base_to_display_index(caret, preedit_len, caret),
            )
        };

        let anchor_u16 = crate::text_edit::ime::composed_utf16_offset_for_display_byte_offset(
            &self.text,
            caret,
            &self.preedit,
            display_anchor,
            fret_core::utf::UtfIndexClamp::Down,
        );
        let focus_u16 = crate::text_edit::ime::composed_utf16_offset_for_display_byte_offset(
            &self.text,
            caret,
            &self.preedit,
            display_focus,
            fret_core::utf::UtfIndexClamp::Down,
        );

        let marked_utf16 = is_composing.then(|| {
            crate::text_edit::ime::composed_utf16_range_for_display_byte_range(
                &self.text,
                caret,
                &self.preedit,
                caret,
                caret.saturating_add(preedit_len),
            )
        });

        Some(fret_runtime::WindowTextInputSnapshot {
            focus_is_text_input: true,
            is_composing,
            text_len_utf16: crate::text_edit::ime::composed_utf16_len(&self.text, &self.preedit),
            selection_utf16: Some((anchor_u16, focus_u16)),
            marked_utf16,
            ime_cursor_area: self.last_sent_cursor,
        })
    }

    fn platform_text_input_selected_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        let caret = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.caret);
        let selection_anchor =
            crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.selection_anchor);

        let preedit_len = self.preedit.len();
        let is_composing = self.is_ime_composing();

        let (display_anchor, display_focus) = if is_composing {
            let caret_display = self.caret_display_index();
            (caret_display, caret_display)
        } else {
            (
                crate::text_edit::ime::base_to_display_index(caret, preedit_len, selection_anchor),
                crate::text_edit::ime::base_to_display_index(caret, preedit_len, caret),
            )
        };

        let anchor_u16 = crate::text_edit::ime::composed_utf16_offset_for_display_byte_offset(
            &self.text,
            caret,
            &self.preedit,
            display_anchor,
            fret_core::utf::UtfIndexClamp::Down,
        );
        let focus_u16 = crate::text_edit::ime::composed_utf16_offset_for_display_byte_offset(
            &self.text,
            caret,
            &self.preedit,
            display_focus,
            fret_core::utf::UtfIndexClamp::Down,
        );

        Some(fret_runtime::Utf16Range::new(anchor_u16, focus_u16).normalized())
    }

    fn platform_text_input_marked_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        let caret = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.caret);
        let preedit_len = self.preedit.len();
        let is_composing = self.is_ime_composing();
        let (start, end) = is_composing.then(|| {
            crate::text_edit::ime::composed_utf16_range_for_display_byte_range(
                &self.text,
                caret,
                &self.preedit,
                caret,
                caret.saturating_add(preedit_len),
            )
        })?;
        Some(fret_runtime::Utf16Range::new(start, end).normalized())
    }

    fn platform_text_input_text_for_range_utf16(
        &self,
        range: fret_runtime::Utf16Range,
    ) -> Option<String> {
        let composed = self
            .layout_text()
            .unwrap_or_else(|| self.text().to_string());

        let r = range.normalized();
        let (bs, be) = fret_core::utf::utf16_range_to_utf8_byte_range(
            composed.as_str(),
            r.start as usize,
            r.end as usize,
        );
        Some(composed.get(bs..be)?.to_string())
    }

    fn platform_text_input_bounds_for_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
    ) -> Option<Rect> {
        self.sync_style_from_theme(cx.theme());
        self.flush_pending_releases(cx.services);

        let font_stack_key = cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);

        let inner = self.inner_bounds();
        let max_width = if self.show_scrollbar {
            Px((inner.size.width.0 - self.scrollbar_width.0).max(0.0))
        } else {
            inner.size.width
        };
        let max_width_constraint = if matches!(self.wrap, TextWrap::None) {
            None
        } else {
            Some(max_width)
        };
        let constraints = TextConstraints {
            max_width: max_width_constraint,
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let key = PreparedKey {
            max_width_bits: max_width_constraint
                .map(|w| w.0.to_bits())
                .unwrap_or(u32::MAX),
            wrap: self.wrap,
            scale_bits: cx.scale_factor.to_bits(),
            show_scrollbar: self.show_scrollbar,
            font_stack_key,
        };

        if self.text_dirty || self.blob.is_none() || self.prepared_key != Some(key) {
            self.queue_release_blob();
            self.flush_pending_releases(cx.services);
            let layout_text = match self.layout_text() {
                Some(s) => std::borrow::Cow::Owned(s),
                None => std::borrow::Cow::Borrowed(self.text.as_str()),
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(layout_text.as_ref(), &self.text_style, constraints);
            self.blob = Some(blob);
            self.metrics = Some(metrics);
            self.prepared_key = Some(key);
            self.text_dirty = false;
        }

        let blob = self.blob?;

        let composed = match self.layout_text() {
            Some(s) => std::borrow::Cow::Owned(s),
            None => std::borrow::Cow::Borrowed(self.text.as_str()),
        };

        let r = range.normalized();
        let (bs, be) = fret_core::utf::utf16_range_to_utf8_byte_range(
            composed.as_ref(),
            r.start as usize,
            r.end as usize,
        );

        let inner = self.content_bounds();
        let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
            inner.origin.x - self.offset_x,
            Px(inner.origin.y.0 - self.offset_y.0),
        ));

        if bs == be {
            let caret = cx
                .services
                .caret_rect(blob, bs, fret_core::CaretAffinity::Downstream);
            let rect = Rect::new(
                mapping.text_local_to_window_point(caret.origin),
                Size::new(Px(caret.size.width.0.max(1.0)), caret.size.height),
            );
            return Some(rect);
        }

        let mut rects: Vec<Rect> = Vec::new();
        cx.services.selection_rects(blob, (bs, be), &mut rects);

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for r in rects {
            if r.size.width.0 <= 0.0 || r.size.height.0 <= 0.0 {
                continue;
            }
            min_x = min_x.min(r.origin.x.0);
            min_y = min_y.min(r.origin.y.0);
            max_x = max_x.max(r.origin.x.0 + r.size.width.0);
            max_y = max_y.max(r.origin.y.0 + r.size.height.0);
        }

        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return None;
        }

        Some(Rect::new(
            fret_core::Point::new(
                Px(mapping.box_origin.x.0 + min_x),
                Px(mapping.box_origin.y.0 + min_y),
            ),
            Size::new(Px((max_x - min_x).max(1.0)), Px((max_y - min_y).max(1.0))),
        ))
    }

    fn platform_text_input_character_index_for_point_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        point: fret_core::Point,
    ) -> Option<u32> {
        self.sync_style_from_theme(cx.theme());
        self.flush_pending_releases(cx.services);

        let font_stack_key = cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);

        let inner = self.inner_bounds();
        let max_width = if self.show_scrollbar {
            Px((inner.size.width.0 - self.scrollbar_width.0).max(0.0))
        } else {
            inner.size.width
        };
        let max_width_constraint = if matches!(self.wrap, TextWrap::None) {
            None
        } else {
            Some(max_width)
        };
        let constraints = TextConstraints {
            max_width: max_width_constraint,
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let key = PreparedKey {
            max_width_bits: max_width_constraint
                .map(|w| w.0.to_bits())
                .unwrap_or(u32::MAX),
            wrap: self.wrap,
            scale_bits: cx.scale_factor.to_bits(),
            show_scrollbar: self.show_scrollbar,
            font_stack_key,
        };

        if self.text_dirty || self.blob.is_none() || self.prepared_key != Some(key) {
            self.queue_release_blob();
            self.flush_pending_releases(cx.services);
            let layout_text = match self.layout_text() {
                Some(s) => std::borrow::Cow::Owned(s),
                None => std::borrow::Cow::Borrowed(self.text.as_str()),
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(layout_text.as_ref(), &self.text_style, constraints);
            self.blob = Some(blob);
            self.metrics = Some(metrics);
            self.prepared_key = Some(key);
            self.text_dirty = false;
        }

        let blob = self.blob?;
        let composed = match self.layout_text() {
            Some(s) => std::borrow::Cow::Owned(s),
            None => std::borrow::Cow::Borrowed(self.text.as_str()),
        };

        let inner = self.content_bounds();
        let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
            inner.origin.x - self.offset_x,
            Px(inner.origin.y.0 - self.offset_y.0),
        ));
        let local = mapping.window_to_text_local(point);
        let hit = cx.services.hit_test_point(blob, local);

        let u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            composed.as_ref(),
            hit.index,
            fret_core::utf::UtfIndexClamp::Down,
        );
        Some(u16.min(u32::MAX as usize) as u32)
    }

    fn platform_text_input_replace_text_in_range_utf16(
        &mut self,
        _cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
        text: &str,
    ) -> bool {
        let composed = self
            .layout_text()
            .unwrap_or_else(|| self.text().to_string());
        let r = range.normalized();
        let (bs, be) = fret_core::utf::utf16_range_to_utf8_byte_range(
            composed.as_str(),
            r.start as usize,
            r.end as usize,
        );

        let preedit_len = self.preedit.len();
        let caret = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.caret);
        let (start_base, end_base) = if preedit_len == 0 {
            (bs, be)
        } else {
            (
                crate::text_edit::ime::display_to_base_index(caret, preedit_len, bs),
                crate::text_edit::ime::display_to_base_index(caret, preedit_len, be),
            )
        };

        let insert = if text.contains('\r') {
            crate::text_edit::normalize::newlines_to_lf(text)
        } else {
            text.to_string()
        };

        self.clear_preedit();
        self.edit_state()
            .set_selection_grapheme_clamped(start_base, end_base);
        self.replace_selection(&insert);
        true
    }

    fn platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
        text: &str,
        marked: Option<fret_runtime::Utf16Range>,
    ) -> bool {
        let insert = if text.contains('\r') {
            crate::text_edit::normalize::newlines_to_lf(text)
        } else {
            text.to_string()
        };

        let commit_composition = |this: &mut Self, insert: &str| -> bool {
            if insert.is_empty() {
                this.clear_preedit();
                this.text_dirty = true;
                this.ensure_caret_visible = true;
                return true;
            }

            let (start, end) = this.ime_replace_range.unwrap_or((this.caret, this.caret));
            let mut edit = this.edit_state();
            edit.set_selection_grapheme_clamped(start, end);
            let _ = edit.replace_selection(insert);

            this.affinity = fret_core::CaretAffinity::Downstream;
            this.text_dirty = true;
            this.ensure_caret_visible = true;
            true
        };

        let Some(marked) = marked else {
            if self.is_ime_composing() {
                return commit_composition(self, insert.as_str());
            }
            return self.platform_text_input_replace_text_in_range_utf16(cx, range, text);
        };

        let marked = marked.normalized();
        if marked.start == marked.end {
            if self.is_ime_composing() {
                return commit_composition(self, insert.as_str());
            }
            return self.platform_text_input_replace_text_in_range_utf16(cx, range, text);
        }

        let r = range.normalized();

        if !self.is_ime_composing() {
            // Starting composition: map the requested (UTF-16) replace range into base UTF-8 bytes.
            let (a, b) = fret_core::utf::utf16_range_to_utf8_byte_range(
                self.text.as_str(),
                r.start as usize,
                r.end as usize,
            );
            let a = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, a);
            let b = crate::text_edit::utf8::clamp_to_char_boundary(&self.text, b);
            let (start, end) = if a <= b { (a, b) } else { (b, a) };

            self.caret = start;
            self.selection_anchor = start;
            self.ime_replace_range = (start != end).then_some((start, end));
        } else {
            // When already composing, tolerate unexpected replace ranges by treating the request as
            // a whole-preedit update. This keeps IME integration robust without expanding the v1
            // contract beyond a single caret-anchored marked range.
            let _ = r;
        }

        self.preedit = insert;
        let preedit_len = self.preedit.len();
        self.preedit_cursor = Some((preedit_len, preedit_len));
        self.affinity = fret_core::CaretAffinity::Downstream;
        self.text_dirty = true;
        self.ensure_caret_visible = true;
        true
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.queue_release_blob();
        self.flush_pending_releases(services);
        self.metrics = None;
        self.prepared_key = None;
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::TextField);
        cx.set_focusable(self.enabled && self.focusable);
        if !self.enabled {
            cx.set_disabled(true);
        }
        cx.set_value_editable(self.enabled);
        cx.set_text_selection_supported(self.enabled);

        let (value, text_selection, text_composition) = if self.is_ime_composing()
            && let Some(layout_text) = self.layout_text()
        {
            let caret_display = self.caret_display_index();
            (
                layout_text,
                Some((caret_display, caret_display)),
                Some((self.caret, self.caret.saturating_add(self.preedit.len()))),
            )
        } else {
            let mut selection_anchor = self.selection_anchor;
            let mut caret = self.caret;
            crate::text_edit::utf8::clamp_selection_to_grapheme_boundaries(
                &self.text,
                &mut selection_anchor,
                &mut caret,
            );
            (
                self.text().to_string(),
                Some((selection_anchor, caret)),
                None,
            )
        };

        cx.set_value(value);
        if let Some((anchor, focus)) = text_selection {
            cx.set_text_selection(anchor as u32, focus as u32);
        } else {
            cx.clear_text_selection();
        }
        if let Some((start, end)) = text_composition {
            cx.set_text_composition(start as u32, end as u32);
        } else {
            cx.clear_text_composition();
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        if !self.enabled {
            return;
        }
        let focused = cx.focus == Some(cx.node);
        match event {
            Event::SetTextSelection { anchor, focus } => {
                if !focused {
                    return;
                }

                let mut anchor = *anchor as usize;
                let mut focus = *focus as usize;
                if self.is_ime_composing() {
                    let caret =
                        crate::text_edit::utf8::clamp_to_char_boundary(&self.text, self.caret);
                    let preedit_len = self.preedit.len();
                    anchor =
                        crate::text_edit::ime::display_to_base_index(caret, preedit_len, anchor);
                    focus = crate::text_edit::ime::display_to_base_index(caret, preedit_len, focus);
                }

                self.clear_preedit();
                self.edit_state()
                    .set_selection_grapheme_clamped(anchor, focus);
                self.ensure_caret_visible = true;

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(fret_core::PointerEvent::Wheel { delta, .. }) => {
                self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                self.clamp_offset(self.last_content_height, self.last_viewport_height);
                self.ensure_caret_visible = false;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                button,
                position,
                click_count,
                modifiers,
                pointer_type,
                ..
            }) => {
                match *button {
                    MouseButton::Left => {}
                    MouseButton::Right => {
                        if *pointer_type != fret_core::PointerType::Mouse {
                            return;
                        }

                        cx.request_focus(cx.node);
                        self.dragging_thumb = false;

                        // Avoid mutating selection/caret during IME composition; a context menu
                        // should not disrupt an in-progress preedit session.
                        if self.is_ime_composing() {
                            return;
                        }

                        let Some(blob) = self.blob else {
                            return;
                        };

                        // Only apply right-click caret/selection behavior within the text content
                        // region; keep the scrollbar area untouched.
                        if !self.content_bounds().contains(*position) {
                            return;
                        }

                        let (sel_start, sel_end) = self.selection_range();

                        let inner = self.content_bounds();
                        let mapping =
                            crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
                                inner.origin.x - self.offset_x,
                                Px(inner.origin.y.0 - self.offset_y.0),
                            ));
                        let local = mapping.window_to_text_local(*position);
                        let hit = cx.services.hit_test_point(blob, local);
                        let caret_at_point = hit.index;

                        // Preserve an existing selection when right-clicking inside it so "Copy"
                        // and friends remain enabled in upstream context menus.
                        if sel_start != sel_end
                            && caret_at_point >= sel_start
                            && caret_at_point <= sel_end
                        {
                            return;
                        }

                        self.caret = caret_at_point;
                        self.selection_anchor = caret_at_point;
                        self.affinity = hit.affinity;
                        self.ensure_caret_visible = true;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        return;
                    }
                    MouseButton::Middle => {
                        if *pointer_type != fret_core::PointerType::Mouse {
                            return;
                        }
                        let settings = cx
                            .app
                            .global::<fret_runtime::TextInteractionSettings>()
                            .copied()
                            .unwrap_or_default();
                        if !settings.linux_primary_selection
                            || !cx.input_ctx.caps.clipboard.primary_text
                        {
                            return;
                        }
                        if self.is_ime_composing() {
                            return;
                        }
                        let Some(window) = cx.window else {
                            return;
                        };

                        // Middle-click paste should use the caret under the pointer (editor-grade UX).
                        cx.request_focus(cx.node);
                        self.dragging_thumb = false;

                        let inner = self.content_bounds();
                        let mapping =
                            crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
                                inner.origin.x - self.offset_x,
                                Px(inner.origin.y.0 - self.offset_y.0),
                            ));
                        let local = mapping.window_to_text_local(*position);
                        self.set_caret_from_point(cx, local);
                        self.selection_anchor = self.caret;
                        self.clear_preedit();
                        self.ensure_caret_visible = true;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();

                        let token = cx.app.next_clipboard_token();
                        self.pending_primary_selection_token = Some(token);
                        cx.app
                            .push_effect(Effect::PrimarySelectionGetText { window, token });
                        cx.stop_propagation();
                        return;
                    }
                    _ => return,
                }

                if let Some((track, thumb)) = self.scrollbar_geometry(self.last_bounds)
                    && track.contains(*position)
                {
                    if thumb.contains(*position) {
                        self.dragging_thumb = true;
                        self.drag_pointer_start_y = position.y;
                        self.drag_offset_start_y = self.offset_y;
                        cx.capture_pointer(cx.node);
                    } else {
                        let centered = Px(position.y.0 - thumb.size.height.0 * 0.5);
                        self.set_offset_from_thumb_y(self.last_bounds, centered);
                        self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    }

                    self.ensure_caret_visible = false;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                self.dragging_thumb = false;
                self.selection_dragging = true;
                self.last_pointer_pos = Some(*position);
                self.ensure_selection_autoscroll_timer(cx);

                let had_preedit = !self.preedit.is_empty();
                let inner = self.content_bounds();
                let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
                    inner.origin.x - self.offset_x,
                    Px(inner.origin.y.0 - self.offset_y.0),
                ));
                let local = mapping.window_to_text_local(*position);
                self.set_caret_from_point(cx, local);
                match *click_count {
                    2 => {
                        let (anchor, caret) = crate::text_edit::utf8::select_word_range(
                            self.text.as_str(),
                            self.caret,
                            cx.input_ctx.text_boundary_mode,
                        );
                        self.selection_anchor = anchor;
                        self.caret = caret;
                    }
                    3 => {
                        let (anchor, caret) = crate::text_edit::utf8::select_line_range(
                            self.text.as_str(),
                            self.caret,
                        );
                        self.selection_anchor = anchor;
                        self.caret = caret;
                    }
                    _ => {
                        if !modifiers.shift {
                            self.selection_anchor = self.caret;
                        }
                    }
                }
                self.ensure_caret_visible = true;

                if had_preedit {
                    cx.invalidate_self(Invalidation::Layout);
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
                // Show the I-beam cursor when hovering the editable text region.
                if cx.captured == Some(cx.node) {
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                } else if self.last_bounds.contains(*position) {
                    if let Some((track, _thumb)) = self.scrollbar_geometry(self.last_bounds)
                        && track.contains(*position)
                    {
                        // Keep the default cursor over the scrollbar.
                    } else if self.content_bounds().contains(*position) {
                        cx.set_cursor_icon(fret_core::CursorIcon::Text);
                    }
                }

                if !buttons.left {
                    return;
                }
                if cx.captured != Some(cx.node) {
                    return;
                }

                if self.dragging_thumb {
                    let dy = position.y.0 - self.drag_pointer_start_y.0;
                    let Some((_, thumb)) = self.scrollbar_geometry(self.last_bounds) else {
                        return;
                    };

                    let max_offset = self.max_offset().0;
                    let travel = (self.last_viewport_height.0 - thumb.size.height.0).max(0.0);
                    if travel <= 0.0 || max_offset <= 0.0 {
                        return;
                    }

                    let offset_delta = dy / travel * max_offset;
                    self.offset_y = Px(self.drag_offset_start_y.0 + offset_delta);
                    self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    self.ensure_caret_visible = false;

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let had_preedit = !self.preedit.is_empty();
                self.last_pointer_pos = Some(*position);
                self.ensure_selection_autoscroll_timer(cx);
                let inner = self.content_bounds();
                let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
                    inner.origin.x - self.offset_x,
                    Px(inner.origin.y.0 - self.offset_y.0),
                ));
                let local = mapping.window_to_text_local(*position);
                self.set_caret_from_point(cx, local);
                self.ensure_caret_visible = true;
                self.selection_autoscroll_tick(cx);

                if had_preedit {
                    cx.invalidate_self(Invalidation::Layout);
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button == MouseButton::Left && cx.captured == Some(cx.node) {
                    let was_dragging_thumb = self.dragging_thumb;
                    self.dragging_thumb = false;
                    cx.release_pointer_capture();
                    self.selection_dragging = false;
                    self.last_pointer_pos = None;
                    if let Some(token) = self.selection_autoscroll_timer.take() {
                        if let Some(window) = cx.window {
                            crate::elements::clear_timer_target(cx.app, window, token);
                        }
                        cx.app.push_effect(Effect::CancelTimer { token });
                    }

                    let settings = cx
                        .app
                        .global::<fret_runtime::TextInteractionSettings>()
                        .copied()
                        .unwrap_or_default();
                    if !was_dragging_thumb
                        && settings.linux_primary_selection
                        && cx.input_ctx.caps.clipboard.primary_text
                        && !self.is_ime_composing()
                    {
                        let (start, end) = crate::text_edit::buffer::selection_range(
                            self.selection_anchor,
                            self.caret,
                        );
                        if start != end
                            && end <= self.text.len()
                            && let Some(sel) = self.text.get(start..end)
                        {
                            cx.app.push_effect(Effect::PrimarySelectionSetText {
                                text: sel.to_string(),
                            });
                        }
                    }
                }
            }
            Event::PointerCancel(_) => {
                if cx.captured == Some(cx.node) {
                    self.dragging_thumb = false;
                    cx.release_pointer_capture();
                    self.selection_dragging = false;
                    self.last_pointer_pos = None;
                    if let Some(token) = self.selection_autoscroll_timer.take() {
                        if let Some(window) = cx.window {
                            crate::elements::clear_timer_target(cx.app, window, token);
                        }
                        cx.app.push_effect(Effect::CancelTimer { token });
                    }
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }

                if self.is_ime_composing() {
                    if modifiers.ctrl || modifiers.alt || modifiers.meta {
                        return;
                    }

                    // During IME composition (preedit), reserve common navigation/IME keys for the
                    // platform IME path (ADR 0012).
                    if matches!(
                        key,
                        fret_core::KeyCode::Tab
                            | fret_core::KeyCode::Space
                            | fret_core::KeyCode::Enter
                            | fret_core::KeyCode::NumpadEnter
                            | fret_core::KeyCode::Escape
                            | fret_core::KeyCode::ArrowUp
                            | fret_core::KeyCode::ArrowDown
                            | fret_core::KeyCode::ArrowLeft
                            | fret_core::KeyCode::ArrowRight
                            | fret_core::KeyCode::Backspace
                            | fret_core::KeyCode::Delete
                            | fret_core::KeyCode::Home
                            | fret_core::KeyCode::End
                            | fret_core::KeyCode::PageUp
                            | fret_core::KeyCode::PageDown
                    ) {
                        cx.stop_propagation();
                    }
                    return;
                }

                if modifiers.ctrl || modifiers.alt || modifiers.meta {
                    return;
                }

                match key {
                    fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
                        let changed = self.edit_state().replace_selection("\n");
                        let outcome = crate::text_edit::commands::Outcome {
                            handled: true,
                            invalidate_paint: false,
                            invalidate_layout: changed,
                        };
                        let delta =
                            crate::text_edit::commands::multiline_ui_delta("text.insert", outcome);
                        self.apply_multiline_ui_delta(cx, delta);
                        cx.stop_propagation();
                    }
                    fret_core::KeyCode::Tab => {
                        let changed = self.edit_state().replace_selection("\t");
                        let outcome = crate::text_edit::commands::Outcome {
                            handled: true,
                            invalidate_paint: false,
                            invalidate_layout: changed,
                        };
                        let delta =
                            crate::text_edit::commands::multiline_ui_delta("text.insert", outcome);
                        self.apply_multiline_ui_delta(cx, delta);
                        cx.stop_propagation();
                    }
                    fret_core::KeyCode::Backspace => {
                        let command = "text.delete_backward";
                        let outcome = crate::text_edit::commands::apply_basic(
                            &mut self.edit_state(),
                            command,
                            false,
                            cx.input_ctx.text_boundary_mode,
                        );
                        let delta =
                            crate::text_edit::commands::multiline_ui_delta(command, outcome);
                        self.apply_multiline_ui_delta(cx, delta);
                        cx.stop_propagation();
                    }
                    fret_core::KeyCode::Delete => {
                        let command = "text.delete_forward";
                        let outcome = crate::text_edit::commands::apply_basic(
                            &mut self.edit_state(),
                            command,
                            false,
                            cx.input_ctx.text_boundary_mode,
                        );
                        let delta =
                            crate::text_edit::commands::multiline_ui_delta(command, outcome);
                        self.apply_multiline_ui_delta(cx, delta);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            Event::TextInput(text) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.is_ime_composing() {
                    return;
                }
                let tick = cx.app.tick_id();
                if self
                    .ime_deduper
                    .ignore_text_input_after_ime_commit(tick, text.as_str())
                {
                    return;
                }
                self.ime_deduper.record_text_input(tick, text.as_str());

                let changed = self.edit_state().replace_selection(text.as_str());
                let outcome = crate::text_edit::commands::Outcome {
                    handled: true,
                    invalidate_paint: false,
                    invalidate_layout: changed,
                };
                let delta = crate::text_edit::commands::multiline_ui_delta("text.insert", outcome);
                self.apply_multiline_ui_delta(cx, delta);
            }
            Event::ClipboardText { token, text } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.pending_clipboard_token != Some(*token) {
                    return;
                }
                self.pending_clipboard_token = None;

                let had_preedit = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_clipboard_text(
                    &mut self.edit_state(),
                    crate::text_edit::commands::ClipboardTextPolicy::Multiline,
                    text.as_str(),
                );
                let mut delta =
                    crate::text_edit::commands::multiline_ui_delta("text.clipboard_text", outcome);
                if had_preedit {
                    delta.invalidate_layout = true;
                    delta.clear_preedit = true;
                    delta.text_dirty = true;
                    delta.reset_affinity = true;
                    delta.ensure_caret_visible = true;
                }
                self.apply_multiline_ui_delta(cx, delta);
            }
            Event::ClipboardTextUnavailable { token, .. } => {
                if self.pending_clipboard_token == Some(*token) {
                    self.pending_clipboard_token = None;
                }
            }
            Event::PrimarySelectionText { token, text } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.is_ime_composing() {
                    return;
                }
                if self.pending_primary_selection_token != Some(*token) {
                    return;
                }
                self.pending_primary_selection_token = None;

                let had_preedit = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_clipboard_text(
                    &mut self.edit_state(),
                    crate::text_edit::commands::ClipboardTextPolicy::Multiline,
                    text.as_str(),
                );
                let mut delta = crate::text_edit::commands::multiline_ui_delta(
                    "text.primary_selection_text",
                    outcome,
                );
                if had_preedit {
                    delta.invalidate_layout = true;
                    delta.clear_preedit = true;
                    delta.text_dirty = true;
                    delta.reset_affinity = true;
                    delta.ensure_caret_visible = true;
                }
                self.apply_multiline_ui_delta(cx, delta);
            }
            Event::PrimarySelectionTextUnavailable { token } => {
                if self.pending_primary_selection_token == Some(*token) {
                    self.pending_primary_selection_token = None;
                }
            }
            Event::Ime(ime) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                let tick = cx.app.tick_id();
                let result = crate::text_edit::ime::apply_event(
                    ime,
                    tick,
                    true,
                    &mut self.ime_deduper,
                    &mut self.text,
                    &mut self.caret,
                    &mut self.selection_anchor,
                    &mut self.preedit,
                    &mut self.preedit_cursor,
                    &mut self.ime_replace_range,
                );
                if result != crate::text_edit::ime::ApplyResult::Noop {
                    self.apply_multiline_ui_delta(cx, Self::edit_layout_delta(false));
                }
            }
            Event::Timer { token } => {
                if self.selection_autoscroll_timer != Some(*token) {
                    return;
                }
                self.selection_autoscroll_timer = None;
                if let Some(window) = cx.window {
                    crate::elements::clear_timer_target(cx.app, window, *token);
                }
                if !self.selection_dragging {
                    return;
                }
                self.selection_autoscroll_tick(cx);
                self.ensure_selection_autoscroll_timer(cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &fret_runtime::CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        let cmd = match command.as_str() {
            "edit.copy" => "text.copy",
            "edit.cut" => "text.cut",
            "edit.paste" => "text.paste",
            "edit.select_all" => "text.select_all",
            other => other,
        };
        let is_vertical = matches!(
            cmd,
            "text.move_up" | "text.move_down" | "text.select_up" | "text.select_down"
        );
        let is_line_home_end = matches!(
            cmd,
            "text.move_home" | "text.move_end" | "text.select_home" | "text.select_end"
        );
        if cmd != "text.copy" && !is_vertical {
            self.preferred_x = None;
        }
        let had_preedit = !self.preedit.is_empty();
        if had_preedit && (is_vertical || is_line_home_end) {
            return true;
        }
        if had_preedit && cmd != "text.copy" {
            self.clear_preedit();
            cx.invalidate_self(Invalidation::Layout);
            cx.request_redraw();
        }

        let hit_test_line =
            |this: &mut Self, cx: &mut CommandCx<'_, H>, at_line_end: bool| -> bool {
                let Some(blob) = this.blob else {
                    return true;
                };

                let caret_index = this.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, this.affinity);
                let y = Px(caret_rect.origin.y.0 + caret_rect.size.height.0 * 0.5);
                let x = if at_line_end { Px(1.0e6) } else { Px(-1.0e6) };
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                this.caret = this.map_display_index_to_base(hit.index);
                this.affinity = hit.affinity;
                true
            };

        match cmd {
            "text.clear" => {
                self.text.clear();
                self.caret = 0;
                self.selection_anchor = 0;
                self.apply_multiline_ui_delta(cx, Self::edit_layout_delta(true));
                true
            }
            "text.copy" => {
                if !cx.input_ctx.caps.clipboard.text.write {
                    return true;
                }
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    cmd,
                    cx.window.is_some(),
                );
                if let Some(crate::text_edit::commands::ClipboardRequest::SetText { text }) =
                    result.request
                {
                    cx.app.push_effect(Effect::ClipboardSetText { text });
                }
                true
            }
            "text.cut" => {
                if !cx.input_ctx.caps.clipboard.text.write {
                    return true;
                }
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    cmd,
                    cx.window.is_some(),
                );
                if let Some(crate::text_edit::commands::ClipboardRequest::SetText { text }) =
                    result.request
                {
                    cx.app.push_effect(Effect::ClipboardSetText { text });
                }
                let delta = crate::text_edit::commands::multiline_ui_delta(cmd, result.outcome);
                self.apply_multiline_ui_delta(cx, delta);
                true
            }
            "text.paste" => {
                if !cx.input_ctx.caps.clipboard.text.read {
                    return true;
                }
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    cmd,
                    cx.window.is_some(),
                );
                if let Some(crate::text_edit::commands::ClipboardRequest::GetText) = result.request
                {
                    return self.request_clipboard_paste(cx);
                }
                true
            }
            "text.move_home" | "text.move_end" | "text.select_home" | "text.select_end" => {
                let is_end = matches!(cmd, "text.move_end" | "text.select_end");
                let extend = matches!(cmd, "text.select_home" | "text.select_end");

                hit_test_line(self, cx, is_end);
                if !extend {
                    self.selection_anchor = self.caret;
                }

                self.apply_multiline_ui_delta(cx, Self::nav_paint_delta());
                true
            }
            "text.move_up" | "text.move_down" | "text.select_up" | "text.select_down" => {
                let Some(blob) = self.blob else {
                    return true;
                };

                let extend = matches!(cmd, "text.select_up" | "text.select_down");
                let down = matches!(cmd, "text.move_down" | "text.select_down");

                let caret_index = self.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, self.affinity);
                let x = self.preferred_x.unwrap_or(caret_rect.origin.x);
                let y = if down {
                    Px(caret_rect.origin.y.0 + caret_rect.size.height.0 + 1.0)
                } else {
                    Px(caret_rect.origin.y.0 - 1.0)
                };

                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                self.caret = self.map_display_index_to_base(hit.index);
                if !extend {
                    self.selection_anchor = self.caret;
                }
                self.affinity = hit.affinity;
                self.preferred_x = Some(x);

                self.apply_multiline_ui_delta(cx, Self::nav_paint_delta());
                true
            }
            _ => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    cmd,
                    is_ime_composing,
                    cx.input_ctx.text_boundary_mode,
                );
                let delta = crate::text_edit::commands::multiline_ui_delta(cmd, outcome);
                if !delta.handled {
                    return false;
                }

                self.apply_multiline_ui_delta(cx, delta);
                true
            }
        }
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &fret_runtime::CommandId,
    ) -> CommandAvailability {
        if !self.enabled {
            return CommandAvailability::NotHandled;
        }
        if cx.focus != Some(cx.node) {
            return CommandAvailability::NotHandled;
        }

        let cmd = match command.as_str() {
            "edit.copy" => "text.copy",
            "edit.cut" => "text.cut",
            "edit.paste" => "text.paste",
            "edit.select_all" => "text.select_all",
            other => other,
        };
        if !cmd.starts_with("text.") {
            return CommandAvailability::NotHandled;
        }

        let clipboard_read = cx.input_ctx.caps.clipboard.text.read;
        let clipboard_write = cx.input_ctx.caps.clipboard.text.write;
        let (start, end) = self.selection_range();
        let has_selection = start != end;

        match cmd {
            "text.copy" | "text.cut" => {
                if !clipboard_write {
                    return CommandAvailability::Blocked;
                }
                if has_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "text.paste" => {
                if !clipboard_read {
                    return CommandAvailability::Blocked;
                }
                CommandAvailability::Available
            }
            "text.select_all" | "text.clear" => {
                if !self.text().is_empty() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
        self.last_bounds = cx.bounds;

        self.edit_state()
            .clamp_caret_and_anchor_to_grapheme_boundary();

        let scrollbar_w = self.scrollbar_width;

        let inner = self.inner_bounds();
        let layout_text_owned = self.layout_text();
        let layout_text = layout_text_owned.as_deref().unwrap_or(&self.text);

        let max_width = cx
            .tree
            .maybe_bucket_text_wrap_width(self.wrap, inner.size.width);
        let max_width_constraint = if matches!(self.wrap, TextWrap::None) {
            None
        } else {
            Some(max_width)
        };
        let mut constraints = TextConstraints {
            max_width: max_width_constraint,
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let mut metrics =
            cx.services
                .text()
                .measure_str(layout_text, &self.text_style, constraints);
        let show_scrollbar = metrics.size.height.0 > inner.size.height.0;
        if show_scrollbar {
            let max_width = Px((inner.size.width.0 - scrollbar_w.0).max(0.0));
            let max_width = cx.tree.maybe_bucket_text_wrap_width(self.wrap, max_width);
            if !matches!(self.wrap, TextWrap::None) {
                constraints.max_width = Some(max_width);
            }
            metrics = cx
                .services
                .text()
                .measure_str(layout_text, &self.text_style, constraints);
        }

        self.metrics = Some(metrics);
        self.show_scrollbar = show_scrollbar;

        let Some(metrics) = self.metrics else {
            return Size::new(cx.available.width, self.min_height);
        };

        self.last_content_height = metrics.size.height;
        self.last_content_width = metrics.size.width;
        self.last_viewport_height = inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);
        let inner = self.content_bounds();
        let max_offset_x = Px((self.last_content_width.0 - inner.size.width.0).max(0.0));
        self.offset_x = Px(self.offset_x.0.clamp(0.0, max_offset_x.0));

        Size::new(
            cx.available.width,
            Px((metrics.size.height.0 + self.style.padding_y.0 * 2.0).max(self.min_height.0)),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
        let font_stack_key = cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        if cx.focus != Some(cx.node) && self.is_ime_composing() {
            self.clear_preedit();
        }
        self.last_bounds = cx.bounds;
        self.flush_pending_releases(cx.services);

        let inner = self.inner_bounds();

        let max_width = if self.show_scrollbar {
            Px((inner.size.width.0 - self.scrollbar_width.0).max(0.0))
        } else {
            inner.size.width
        };
        let max_width = cx.tree.maybe_bucket_text_wrap_width(self.wrap, max_width);
        let max_width_constraint = if matches!(self.wrap, TextWrap::None) {
            None
        } else {
            Some(max_width)
        };
        let constraints = TextConstraints {
            max_width: max_width_constraint,
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let key = PreparedKey {
            max_width_bits: max_width_constraint
                .map(|w| w.0.to_bits())
                .unwrap_or(u32::MAX),
            wrap: self.wrap,
            scale_bits: cx.scale_factor.to_bits(),
            show_scrollbar: self.show_scrollbar,
            font_stack_key,
        };

        if self.text_dirty || self.blob.is_none() || self.prepared_key != Some(key) {
            self.queue_release_blob();
            self.flush_pending_releases(cx.services);
            let layout_text = match self.layout_text() {
                Some(s) => std::borrow::Cow::Owned(s),
                None => std::borrow::Cow::Borrowed(self.text.as_str()),
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(layout_text.as_ref(), &self.text_style, constraints);
            self.blob = Some(blob);
            self.metrics = Some(metrics);
            self.prepared_key = Some(key);
            self.text_dirty = false;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Paint::Solid(self.style.background),
            border: self.style.border,
            border_paint: Paint::Solid(self.style.border_color),
            corner_radii: self.style.corner_radii,
        });

        if cx.focus == Some(cx.node)
            && crate::focus_visible::is_focus_visible(cx.app, cx.window)
            && let Some(mut ring) = self.style.focus_ring
        {
            ring.corner_radii = self.style.corner_radii;
            crate::paint::paint_focus_ring(cx.scene, DrawOrder(1), cx.bounds, ring);
        }

        let Some(blob) = self.blob else {
            return;
        };
        let Some(metrics) = self.metrics else {
            return;
        };

        let padded_inner = self.inner_bounds();
        self.last_content_height = metrics.size.height;
        self.last_content_width = metrics.size.width;
        self.last_viewport_height = padded_inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);

        let inner = self.content_bounds();
        let max_offset_x = Px((self.last_content_width.0 - inner.size.width.0).max(0.0));
        self.offset_x = Px(self.offset_x.0.clamp(0.0, max_offset_x.0));
        let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
            inner.origin.x - self.offset_x,
            Px(inner.origin.y.0 - self.offset_y.0),
        ));
        cx.scene.push(SceneOp::PushClipRect { rect: inner });

        let map_base_to_display = |idx: usize| -> usize {
            crate::text_edit::ime::base_to_display_index(self.caret, self.preedit.len(), idx)
        };

        let selection_clip = Rect::new(
            fret_core::Point::new(self.offset_x, self.offset_y),
            Size::new(inner.size.width, inner.size.height),
        );
        cx.services.selection_rects_clipped(
            blob,
            (
                map_base_to_display(self.selection_anchor),
                map_base_to_display(self.caret),
            ),
            selection_clip,
            &mut self.selection_rects,
        );
        let window_focused = cx
            .window
            .and_then(|window| {
                cx.app
                    .global::<fret_core::WindowMetricsService>()
                    .and_then(|svc| svc.focused(window))
            })
            .unwrap_or(true);
        let selection_color = if cx.focus == Some(cx.node) || self.style_override {
            self.style.selection_color
        } else if !window_focused {
            cx.theme()
                .color_token("selection.window_inactive.background")
        } else {
            cx.theme().color_token("selection.inactive.background")
        };
        for r in &self.selection_rects {
            let rect = mapping.text_local_to_window_rect(*r);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect,
                background: Paint::Solid(selection_color),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if !self.preedit.is_empty() {
            let start = self.caret;
            let end = self.caret + self.preedit.len();
            cx.services.selection_rects_clipped(
                blob,
                (start, end),
                selection_clip,
                &mut self.preedit_rects,
            );
            for r in &self.preedit_rects {
                let rect = mapping.text_local_to_window_rect(*r);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: Paint::Solid(self.style.preedit_bg_color),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT),
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        } else {
            self.preedit_rects.clear();
        }

        let text_origin = mapping.baseline_origin(metrics.baseline);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin: text_origin,
            text: blob,
            paint: fret_core::scene::Paint::Solid(self.style.text_color),
            outline: None,
            shadow: None,
        });

        if cx.focus != Some(cx.node) {
            self.ensure_caret_visible = true;
        }

        if cx.focus == Some(cx.node) {
            let caret_index = self.caret_display_index();
            let affinity = if self.preedit.is_empty() {
                self.affinity
            } else {
                CaretAffinity::Downstream
            };
            let caret = cx.services.caret_rect(blob, caret_index, affinity);
            let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
            if self.ensure_caret_visible {
                let settings = cx
                    .app
                    .global::<fret_runtime::TextInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let margin_x = (settings.horizontal_autoscroll_margin_px as f32)
                    .max(0.0)
                    .min(inner.size.width.0 * 0.45);

                let caret_top = caret.origin.y.0;
                let caret_bottom = caret.origin.y.0 + caret.size.height.0;
                let viewport_top = self.offset_y.0;
                let viewport_bottom = self.offset_y.0 + inner.size.height.0;
                let mut desired_y = self.offset_y.0;
                if caret_top < viewport_top {
                    desired_y = caret_top;
                } else if caret_bottom > viewport_bottom {
                    desired_y = caret_bottom - inner.size.height.0;
                }

                let caret_x = caret.origin.x.0;
                let viewport_left = self.offset_x.0;
                let viewport_right = self.offset_x.0 + inner.size.width.0;
                let mut desired_x = self.offset_x.0;
                if caret_x < viewport_left + margin_x {
                    desired_x = caret_x - margin_x;
                } else if caret_x > viewport_right - margin_x {
                    desired_x = caret_x - (inner.size.width.0 - margin_x);
                }
                desired_x = desired_x.clamp(0.0, max_offset_x.0);

                if (desired_y - self.offset_y.0).abs() > 0.01
                    || (desired_x - self.offset_x.0).abs() > 0.01
                {
                    self.offset_y = Px(desired_y);
                    self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    self.offset_x = Px(desired_x);
                    if let Some(window) = cx.window {
                        cx.app.request_redraw(window);
                    }
                }
                self.ensure_caret_visible = false;
            }

            let caret_rect = Rect::new(
                mapping.text_local_to_window_point(caret.origin),
                Size::new(Px(hairline.0.max(1.0)), caret.size.height),
            );

            let ime_rect_layout = if self.is_ime_composing() && !self.preedit_rects.is_empty() {
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                for r in &self.preedit_rects {
                    if r.size.width.0 <= 0.0 || r.size.height.0 <= 0.0 {
                        continue;
                    }

                    let rect = mapping.text_local_to_window_rect(*r);
                    let x0 = rect.origin.x.0;
                    let y0 = rect.origin.y.0;
                    let x1 = x0 + rect.size.width.0;
                    let y1 = y0 + rect.size.height.0;

                    min_x = min_x.min(x0);
                    min_y = min_y.min(y0);
                    max_x = max_x.max(x1);
                    max_y = max_y.max(y1);
                }

                if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite()
                {
                    Rect::new(
                        fret_core::Point::new(Px(min_x), Px(min_y)),
                        Size::new(Px((max_x - min_x).max(1.0)), Px((max_y - min_y).max(1.0))),
                    )
                } else {
                    caret_rect
                }
            } else {
                caret_rect
            };

            // Anchor IME UI to the caret *bottom* (see `TextInput` for rationale).
            let ime_anchor_layout = {
                let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
                let w = Px(hairline.0.max(1.0 / 8.0));
                let h = hairline;
                let y_bottom = ime_rect_layout.origin.y.0 + ime_rect_layout.size.height.0;
                let y = Px((y_bottom - h.0).max(ime_rect_layout.origin.y.0));
                Rect::new(
                    fret_core::Point::new(ime_rect_layout.origin.x, y),
                    Size::new(w, h),
                )
            };

            let ime_rect = cx.visual_rect_aabb(ime_anchor_layout);

            if let Some(window) = cx.window
                && self.last_sent_cursor != Some(ime_rect)
            {
                self.last_sent_cursor = Some(ime_rect);
                cx.app.push_effect(Effect::ImeSetCursorArea {
                    window,
                    rect: ime_rect,
                });
            }

            if !self.preedit_rects.is_empty() {
                for r in &self.preedit_rects {
                    if r.size.width.0 <= 0.0 {
                        continue;
                    }
                    let rect = mapping.text_local_to_window_rect(*r);
                    let y = rect.origin.y.0 + rect.size.height.0 - hairline.0;
                    let underline = Rect::new(
                        fret_core::Point::new(rect.origin.x, Px(y)),
                        Size::new(Px(r.size.width.0.max(hairline.0)), hairline),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: underline,
                        background: Paint::Solid(self.style.preedit_underline_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: Paint::Solid(Color::TRANSPARENT),
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: caret_rect,
                background: Paint::Solid(self.style.caret_color),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(Px(0.0)),
            });
        } else {
            self.last_sent_cursor = None;
        }

        cx.scene.push(SceneOp::PopClip);

        if let Some((track, thumb)) = self.scrollbar_geometry(cx.bounds) {
            let (track_bg, thumb_bg, thumb_hover_bg, radius) = {
                let theme = cx.theme();
                (
                    theme.color_token("scrollbar.background"),
                    theme.color_token("scrollbar.thumb.background"),
                    theme.color_token("scrollbar.thumb.hover.background"),
                    theme.metric_token("metric.radius.sm"),
                )
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: Paint::Solid(track_bg),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(radius),
            });

            let thumb_bg = if self.dragging_thumb {
                thumb_hover_bg
            } else {
                thumb_bg
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: Paint::Solid(thumb_bg),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(radius),
            });
        }
    }
}

impl TextArea {
    const SELECTION_AUTOSCROLL_TICK: Duration = Duration::from_millis(16);

    fn ensure_selection_autoscroll_timer<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        if self.selection_autoscroll_timer.is_some() {
            return;
        }
        if !self.selection_dragging {
            return;
        }
        if !matches!(self.wrap, TextWrap::None) {
            return;
        }
        let Some(window) = cx.window else {
            return;
        };

        let token = cx.app.next_timer_token();
        self.selection_autoscroll_timer = Some(token);
        crate::elements::record_timer_target_node(cx.app, window, token, cx.node);
        cx.app.push_effect(Effect::SetTimer {
            window: Some(window),
            token,
            after: Self::SELECTION_AUTOSCROLL_TICK,
            repeat: None,
        });
    }

    fn selection_autoscroll_tick<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        if !self.selection_dragging {
            return;
        }
        if !matches!(self.wrap, TextWrap::None) {
            return;
        }
        let Some(pos) = self.last_pointer_pos else {
            return;
        };

        let inner = self.content_bounds();
        let inner_left = inner.origin.x.0;
        let inner_right = inner.origin.x.0 + inner.size.width.0;
        let inner_width = (inner_right - inner_left).max(0.0);
        if inner_width <= 0.0 {
            return;
        }

        let settings = cx
            .app
            .global::<fret_runtime::TextInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let margin = (settings.horizontal_autoscroll_margin_px as f32)
            .max(0.0)
            .min(inner_width * 0.45);
        let max_step = settings.horizontal_autoscroll_max_step_px as f32;
        if max_step <= 0.0 {
            return;
        }

        let left_edge = inner_left + margin;
        let right_edge = inner_right - margin;
        let mut delta = 0.0_f32;
        if pos.x.0 < left_edge {
            let dist = (left_edge - pos.x.0).max(0.0);
            delta = -((dist / 4.0) + 1.0).min(max_step);
        } else if pos.x.0 > right_edge {
            let dist = (pos.x.0 - right_edge).max(0.0);
            delta = ((dist / 4.0) + 1.0).min(max_step);
        }
        if delta.abs() <= 0.01 {
            return;
        }

        let mut content_w = self.last_content_width.0;
        if content_w <= 0.0 {
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: cx.scale_factor,
            };
            let text = self.layout_text().unwrap_or_else(|| self.text.clone());
            content_w = cx
                .services
                .text()
                .measure_str(text.as_str(), &self.text_style, constraints)
                .size
                .width
                .0;
        }

        let max_offset = (content_w - inner_width).max(0.0);
        if max_offset <= 0.0 {
            return;
        }

        let next_offset = (self.offset_x.0 + delta).clamp(0.0, max_offset);
        if (next_offset - self.offset_x.0).abs() <= 0.01 {
            return;
        }
        self.offset_x = Px(next_offset);

        let had_preedit = !self.preedit.is_empty();
        let mapping = crate::text::coords::TextBoxMapping::new(fret_core::Point::new(
            inner.origin.x - self.offset_x,
            Px(inner.origin.y.0 - self.offset_y.0),
        ));
        let local = mapping.window_to_text_local(pos);
        let local = Point::new(Px(local.x.0.max(0.0)), local.y);
        self.set_caret_from_point(cx, local);
        self.ensure_caret_visible = true;

        if had_preedit {
            cx.invalidate_self(Invalidation::Layout);
        } else {
            cx.invalidate_self(Invalidation::Paint);
        }
        cx.request_redraw();
    }
}
