use fret_core::time::Duration;
use fret_core::{
    Color, DrawOrder, Event, MouseButton, Paint, Px, Rect, SceneOp, Size, TextConstraints,
    TextOverflow, TextWrap,
};
use fret_runtime::{CommandId, Effect};

use super::TextInput;
use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, LayoutCx, PaintCx,
    PlatformTextInputCx, Widget,
};
use crate::{Invalidation, UiHost};

impl<H: UiHost> Widget<H> for TextInput {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.queue_release_all_text_blobs();
        self.flush_pending_releases(services);
        self.text_metrics = None;
        self.prefix_metrics = None;
        self.suffix_metrics = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
    }

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
            let caret_display = crate::text_edit::ime::caret_display_index(
                caret,
                &self.preedit,
                self.preedit_cursor,
            );
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
            let caret_display = crate::text_edit::ime::caret_display_index(
                caret,
                &self.preedit,
                self.preedit_cursor,
            );
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
        let composed = if self.preedit.is_empty() {
            self.text.clone()
        } else {
            crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
                .unwrap_or_else(|| self.text.clone())
        };

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
        let composed = if self.preedit.is_empty() {
            self.text.clone()
        } else {
            crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
                .unwrap_or_else(|| self.text.clone())
        };

        let padding_left = self.chrome_style.padding.left;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) =
            cx.services
                .text()
                .prepare_str(composed.as_str(), &self.style, constraints);

        let metrics = if metrics.size.height.0 <= 0.01 {
            self.approx_text_metrics()
        } else {
            metrics
        };
        let text_height = metrics.size.height;
        let inner_height = Px(
            (self.last_bounds.size.height.0 - padding_top.0 - padding_bottom.0)
                .max(0.0)
                .max(text_height.0),
        );
        let (vertical_offset, _) = crate::text::coords::compute_text_vertical_offset_and_baseline(
            cx.services.text(),
            blob,
            inner_height,
            metrics,
            self.style.vertical_placement,
        );

        let origin = fret_core::Point::new(
            self.last_bounds.origin.x + padding_left - self.offset_x,
            Px(self.last_bounds.origin.y.0 + padding_top.0 + vertical_offset.0),
        );

        let r = range.normalized();
        let (bs, be) = fret_core::utf::utf16_range_to_utf8_byte_range(
            composed.as_str(),
            r.start as usize,
            r.end as usize,
        );

        let out = if bs == be {
            let caret = cx
                .services
                .caret_rect(blob, bs, fret_core::CaretAffinity::Downstream);
            Some(Rect::new(
                fret_core::Point::new(origin.x + caret.origin.x, origin.y + caret.origin.y),
                Size::new(Px(caret.size.width.0.max(1.0)), caret.size.height),
            ))
        } else {
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

            if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite()
            {
                None
            } else {
                Some(Rect::new(
                    fret_core::Point::new(Px(origin.x.0 + min_x), Px(origin.y.0 + min_y)),
                    Size::new(Px((max_x - min_x).max(1.0)), Px((max_y - min_y).max(1.0))),
                ))
            }
        };

        cx.services.text().release(blob);
        out
    }

    fn platform_text_input_character_index_for_point_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        point: fret_core::Point,
    ) -> Option<u32> {
        let composed = if self.preedit.is_empty() {
            self.text.clone()
        } else {
            crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
                .unwrap_or_else(|| self.text.clone())
        };

        let padding_left = self.chrome_style.padding.left;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) =
            cx.services
                .text()
                .prepare_str(composed.as_str(), &self.style, constraints);

        let metrics = if metrics.size.height.0 <= 0.01 {
            self.approx_text_metrics()
        } else {
            metrics
        };
        let text_height = metrics.size.height;
        let inner_height = Px(
            (self.last_bounds.size.height.0 - padding_top.0 - padding_bottom.0)
                .max(0.0)
                .max(text_height.0),
        );
        let (vertical_offset, _) = crate::text::coords::compute_text_vertical_offset_and_baseline(
            cx.services.text(),
            blob,
            inner_height,
            metrics,
            self.style.vertical_placement,
        );

        let origin = fret_core::Point::new(
            self.last_bounds.origin.x + padding_left - self.offset_x,
            Px(self.last_bounds.origin.y.0 + padding_top.0 + vertical_offset.0),
        );

        let local = fret_core::Point::new(point.x - origin.x, point.y - origin.y);
        let hit = cx.services.hit_test_point(blob, local);
        cx.services.text().release(blob);

        let u16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            composed.as_str(),
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
        let composed = if self.preedit.is_empty() {
            self.text.clone()
        } else {
            crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
                .unwrap_or_else(|| self.text.clone())
        };
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

        let insert = text.replace(['\r', '\n'], " ");

        let mut edit = self.edit_state();
        edit.set_selection_grapheme_clamped(start_base, end_base);
        edit.replace_selection(&insert)
    }

    fn platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
        text: &str,
        marked: Option<fret_runtime::Utf16Range>,
    ) -> bool {
        let insert = text.replace(['\r', '\n'], " ");

        let commit_composition = |this: &mut Self, insert: &str| -> bool {
            if insert.is_empty() {
                this.clear_ime_composition();
                this.mark_text_blobs_dirty();
                return true;
            }

            let (start, end) = this.ime_replace_range.unwrap_or((this.caret, this.caret));
            let mut edit = this.edit_state();
            edit.set_selection_grapheme_clamped(start, end);
            let _ = edit.replace_selection(insert);
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
        self.mark_text_blobs_dirty();
        true
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        cx.set_role(self.a11y_role);
        cx.set_focusable(self.enabled && self.focusable);
        if !self.enabled {
            cx.set_disabled(true);
        }
        cx.set_value_editable(self.enabled);
        cx.set_text_selection_supported(self.enabled);

        let (value, text_selection, text_composition) = if self.is_ime_composing()
            && let Some(value) =
                crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
        {
            let caret_display = crate::text_edit::ime::caret_display_index(
                self.caret,
                &self.preedit,
                self.preedit_cursor,
            );
            (
                value,
                Some((caret_display as u32, caret_display as u32)),
                Some((self.caret as u32, (self.caret + self.preedit.len()) as u32)),
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
                Some((selection_anchor as u32, caret as u32)),
                None,
            )
        };

        cx.set_value(value);
        if let Some((anchor, focus)) = text_selection {
            cx.set_text_selection(anchor, focus);
        } else {
            cx.clear_text_selection();
        }
        if let Some((start, end)) = text_composition {
            cx.set_text_composition(start, end);
        } else {
            cx.clear_text_composition();
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if !self.enabled {
            return;
        }
        let focused = self.is_focused(cx);

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

                let mut edit = self.edit_state();
                edit.clear_ime_composition();
                edit.set_selection_grapheme_clamped(anchor, focus);

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
            }) => match *button {
                MouseButton::Left => {
                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    self.last_sent_cursor = None;
                    self.selection_dragging = true;
                    self.last_pointer_pos = Some(*position);
                    self.ensure_selection_autoscroll_timer(cx);
                    let padding = self.chrome_style.padding.left;
                    let local_x = Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)
                        + self.offset_x.0)
                        .max(0.0));
                    let mut caret = self
                        .text_blob
                        .map(|blob| cx.services.hit_test_x(blob, local_x))
                        .unwrap_or_else(|| self.caret_from_x(local_x));

                    // While IME preedit is active, the displayed text is the base buffer with the
                    // preedit spliced at the caret (ADR 0071). Pointer hit-testing is performed
                    // against that composed view, but the widget's internal indices are tracked in
                    // base-text byte offsets. Map the display index back into base coordinates
                    // before applying selection/navigation and then cancel the inline preedit
                    // deterministically (v1 policy).
                    if !self.preedit.is_empty() {
                        caret = crate::text_edit::ime::display_to_base_index(
                            self.caret,
                            self.preedit.len(),
                            caret,
                        );
                    }

                    self.caret = caret;
                    match *click_count {
                        2 => {
                            let (anchor, caret) = crate::text_edit::utf8::select_word_range(
                                self.text.as_str(),
                                caret,
                                cx.input_ctx.text_boundary_mode,
                            );
                            self.selection_anchor = anchor;
                            self.caret = caret;
                        }
                        3 => {
                            let (anchor, caret) = crate::text_edit::utf8::select_line_range(
                                self.text.as_str(),
                                caret,
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
                    self.clear_ime_composition();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                MouseButton::Right => {
                    if *pointer_type != fret_core::PointerType::Mouse {
                        return;
                    }
                    cx.request_focus(cx.node);
                    self.last_sent_cursor = None;

                    // Avoid mutating selection/caret during IME composition; a context menu should
                    // not disrupt an in-progress preedit session.
                    if self.is_ime_composing() {
                        return;
                    }

                    let (sel_start, sel_end) = crate::text_edit::buffer::selection_range(
                        self.selection_anchor,
                        self.caret,
                    );

                    let padding = self.chrome_style.padding.left;
                    let local_x = Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)
                        + self.offset_x.0)
                        .max(0.0));
                    let caret_at_point = self
                        .text_blob
                        .map(|blob| cx.services.hit_test_x(blob, local_x))
                        .unwrap_or_else(|| self.caret_from_x(local_x));

                    // Preserve an existing selection when right-clicking inside it so "Copy" and
                    // friends remain enabled in upstream context menus.
                    if sel_start != sel_end
                        && caret_at_point >= sel_start
                        && caret_at_point <= sel_end
                    {
                        return;
                    }

                    self.caret = caret_at_point;
                    self.selection_anchor = caret_at_point;
                    self.clear_ime_composition();
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
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

                    cx.request_focus(cx.node);
                    self.last_sent_cursor = None;

                    let padding = self.chrome_style.padding.left;
                    let local_x = Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)
                        + self.offset_x.0)
                        .max(0.0));
                    let caret = self
                        .text_blob
                        .map(|blob| cx.services.hit_test_x(blob, local_x))
                        .unwrap_or_else(|| self.caret_from_x(local_x));
                    self.caret = caret;
                    self.selection_anchor = caret;
                    self.clear_ime_composition();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();

                    let token = cx.app.next_clipboard_token();
                    self.pending_primary_selection_token = Some(token);
                    cx.app
                        .push_effect(Effect::PrimarySelectionGetText { window, token });
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
                // Ensure the I-beam cursor while hovering (or dragging) inside the text field.
                if cx.captured == Some(cx.node) || self.last_bounds.contains(*position) {
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                }
                if cx.captured != Some(cx.node) || !buttons.left {
                    return;
                }

                self.last_pointer_pos = Some(*position);
                self.ensure_selection_autoscroll_timer(cx);
                let padding = self.chrome_style.padding.left;
                let local_x = Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)
                    + self.offset_x.0)
                    .max(0.0));
                self.caret = self
                    .text_blob
                    .map(|blob| cx.services.hit_test_x(blob, local_x))
                    .unwrap_or_else(|| self.caret_from_x(local_x));
                self.clear_ime_composition();
                self.selection_autoscroll_tick(cx);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if cx.captured == Some(cx.node) && *button == MouseButton::Left {
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
                    if settings.linux_primary_selection
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
                if !focused {
                    return;
                }

                if self.is_ime_composing() && !modifiers.ctrl && !modifiers.alt && !modifiers.meta {
                    // During IME composition (preedit), reserve common navigation/IME keys for the
                    // platform IME path. The runtime may still map these keys to focus traversal or
                    // global shortcuts, so we must explicitly stop propagation here (ADR 0012).
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
                        return;
                    }
                }

                if !self.is_ime_composing() {
                    match key {
                        fret_core::KeyCode::Backspace => {
                            let command = if modifiers.ctrl || modifiers.alt {
                                "text.delete_word_backward"
                            } else {
                                "text.delete_backward"
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::Delete => {
                            let command = if modifiers.ctrl || modifiers.alt {
                                "text.delete_word_forward"
                            } else {
                                "text.delete_forward"
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::ArrowLeft => {
                            let word = modifiers.ctrl || modifiers.alt;
                            let command = match (modifiers.shift, word) {
                                (true, true) => "text.select_word_left",
                                (true, false) => "text.select_left",
                                (false, true) => "text.move_word_left",
                                (false, false) => "text.move_left",
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::ArrowRight => {
                            let word = modifiers.ctrl || modifiers.alt;
                            let command = match (modifiers.shift, word) {
                                (true, true) => "text.select_word_right",
                                (true, false) => "text.select_right",
                                (false, true) => "text.move_word_right",
                                (false, false) => "text.move_right",
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::Home => {
                            let command = if modifiers.shift {
                                "text.select_home"
                            } else {
                                "text.move_home"
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::End => {
                            let command = if modifiers.shift {
                                "text.select_end"
                            } else {
                                "text.move_end"
                            };
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                command,
                                false,
                                cx.input_ctx.text_boundary_mode,
                            );
                            let delta =
                                crate::text_edit::commands::singleline_ui_delta(command, outcome);
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        _ => {}
                    }
                }
            }
            Event::TextInput(text) => {
                if !focused {
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

                if !self.is_ime_composing() {
                    let changed = self.replace_selection_changed(text.as_str());
                    let outcome = crate::text_edit::commands::Outcome {
                        handled: true,
                        invalidate_paint: false,
                        invalidate_layout: changed,
                    };
                    let delta =
                        crate::text_edit::commands::singleline_ui_delta("text.insert", outcome);
                    self.apply_singleline_ui_delta(cx, delta);
                }
            }
            Event::ClipboardText { token, text } => {
                if !focused {
                    return;
                }
                if self.is_ime_composing() {
                    return;
                }
                if self.pending_clipboard_token != Some(*token) {
                    return;
                }
                self.pending_clipboard_token = None;

                let outcome = crate::text_edit::commands::apply_clipboard_text(
                    &mut self.edit_state(),
                    crate::text_edit::commands::ClipboardTextPolicy::SingleLine,
                    text.as_str(),
                );
                if outcome.invalidate_layout {
                    self.mark_text_blobs_dirty();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            Event::ClipboardTextUnavailable { token, .. } => {
                if self.pending_clipboard_token == Some(*token) {
                    self.pending_clipboard_token = None;
                }
            }
            Event::PrimarySelectionText { token, text } => {
                if !focused {
                    return;
                }
                if self.is_ime_composing() {
                    return;
                }
                if self.pending_primary_selection_token != Some(*token) {
                    return;
                }
                self.pending_primary_selection_token = None;

                let outcome = crate::text_edit::commands::apply_clipboard_text(
                    &mut self.edit_state(),
                    crate::text_edit::commands::ClipboardTextPolicy::SingleLine,
                    text.as_str(),
                );
                if outcome.invalidate_layout {
                    self.mark_text_blobs_dirty();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            Event::PrimarySelectionTextUnavailable { token } => {
                if self.pending_primary_selection_token == Some(*token) {
                    self.pending_primary_selection_token = None;
                }
            }
            Event::Ime(ime) => {
                if !focused {
                    return;
                }
                let tick = cx.app.tick_id();
                let result = crate::text_edit::ime::apply_event(
                    ime,
                    tick,
                    false,
                    &mut self.ime_deduper,
                    &mut self.text,
                    &mut self.caret,
                    &mut self.selection_anchor,
                    &mut self.preedit,
                    &mut self.preedit_cursor,
                    &mut self.ime_replace_range,
                );
                if result != crate::text_edit::ime::ApplyResult::Noop {
                    self.mark_text_blobs_dirty();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
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

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
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

        // During IME composition the displayed text is base text with a preedit splice at the
        // caret (ADR 0071). To keep command-driven navigation/editing deterministic, cancel the
        // inline preedit before applying commands (except copy; clear cancels preedit itself).
        let is_vertical = matches!(
            cmd,
            "text.move_up" | "text.move_down" | "text.select_up" | "text.select_down"
        );
        if self.is_ime_composing() && is_vertical {
            return true;
        }
        if self.is_ime_composing() && cmd != "text.copy" && cmd != "text.clear" {
            self.clear_ime_composition();
            self.mark_text_blobs_dirty();
            cx.invalidate_self(Invalidation::Layout);
            cx.request_redraw();
        }

        match cmd {
            "text.clear" => {
                self.text.clear();
                self.clear_ime_composition();
                self.caret = 0;
                self.selection_anchor = 0;
                self.mark_text_blobs_dirty();
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
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

                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, result.outcome);
                self.apply_singleline_ui_delta(cx, delta);
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
                    let Some(window) = cx.window else {
                        return true;
                    };
                    let token = cx.app.next_clipboard_token();
                    self.pending_clipboard_token = Some(token);
                    cx.app
                        .push_effect(Effect::ClipboardGetText { window, token });
                }
                true
            }
            "text.move_up" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.move_home",
                    is_ime_composing,
                    cx.input_ctx.text_boundary_mode,
                );
                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.move_down" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.move_end",
                    is_ime_composing,
                    cx.input_ctx.text_boundary_mode,
                );
                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.select_up" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.select_home",
                    is_ime_composing,
                    cx.input_ctx.text_boundary_mode,
                );
                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.select_down" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.select_end",
                    is_ime_composing,
                    cx.input_ctx.text_boundary_mode,
                );
                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, outcome);
                self.apply_singleline_ui_delta(cx, delta);
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
                let delta = crate::text_edit::commands::singleline_ui_delta(cmd, outcome);
                if !delta.handled {
                    return false;
                }

                self.apply_singleline_ui_delta(cx, delta);
                true
            }
        }
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
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
        match cmd {
            "text.copy" | "text.cut" => {
                if !clipboard_write {
                    return CommandAvailability::Blocked;
                }
                if self.has_selection() {
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
                if !self.text.is_empty() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;

        cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);

        self.edit_state()
            .clamp_caret_and_anchor_to_grapheme_boundary();

        let theme = cx.theme().snapshot();
        self.sync_chrome_from_theme(theme.clone());
        self.sync_text_style_from_theme(theme.clone());

        let base_constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        // `TextSystem` returns zero-height metrics for empty strings (no shaped lines). Text inputs
        // want stable line metrics even when the field is empty (caret/placeholder alignment),
        // so measure a single space as a fallback.
        let measure_text = if self.text.is_empty() {
            " "
        } else {
            self.text.as_str()
        };
        let mut metrics =
            cx.services
                .text()
                .measure_str(measure_text, &self.style, base_constraints);
        if metrics.size.height.0 <= 0.01 {
            metrics = cx
                .services
                .text()
                .measure_str(" ", &self.style, base_constraints);
        }
        self.text_metrics = Some(metrics);

        let base_h = self.text_metrics.map(|m| m.size.height.0).unwrap_or(0.0);
        let chrome = &self.chrome_style;
        let border_h = chrome.border.top.0.max(0.0) + chrome.border.bottom.0.max(0.0);
        let pad_h = chrome.padding.top.0.max(0.0) + chrome.padding.bottom.0.max(0.0);
        let h = Px((base_h + pad_h + border_h).max(0.0));
        Size::new(cx.available.width, h)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.flush_pending_releases(cx.services);

        cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
        let font_stack_key = cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        if self.last_font_stack_key != Some(font_stack_key) {
            self.queue_release_all_text_blobs();
            self.flush_pending_releases(cx.services);
            self.last_font_stack_key = Some(font_stack_key);
        }

        let Some(window) = cx.window else {
            return;
        };

        let theme = cx.theme().snapshot();
        self.sync_chrome_from_theme(theme.clone());
        self.sync_text_style_from_theme(theme.clone());
        let focused = cx.focus == Some(cx.node);
        let focus_visible = focused && crate::focus_visible::is_focus_visible(cx.app, cx.window);
        if !focused && self.is_ime_composing() {
            self.clear_ime_composition();
        }
        let border_color = if focused && (focus_visible || self.chrome_style.focus_ring.is_none()) {
            self.chrome_style.border_color_focused
        } else {
            self.chrome_style.border_color
        };

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits) {
            self.queue_release_all_text_blobs();
            self.flush_pending_releases(cx.services);
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        if self.text_blob.is_none() {
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(self.text.as_str(), &self.style, constraints);
            self.text_blob = Some(blob);
            self.text_metrics = Some(metrics);
            cx.services.caret_stops(blob, &mut self.caret_stops);
        }

        let show_placeholder = self.preedit.is_empty()
            && self.text.is_empty()
            && self.placeholder.as_ref().is_some_and(|s| !s.is_empty());
        if show_placeholder && self.placeholder_blob.is_none() {
            let Some(placeholder) = self.placeholder.as_deref() else {
                debug_assert!(false, "placeholder checked above");
                return;
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(placeholder, &self.style, constraints);
            self.placeholder_blob = Some(blob);
            self.placeholder_metrics = Some(metrics);
        }

        if self.preedit.is_empty() {
            if self.prefix_blob.is_some()
                || self.suffix_blob.is_some()
                || self.preedit_blob.is_some()
            {
                self.queue_release_all_text_blobs();
                // The call above also clears `text_blob`, so re-prepare it.
                self.flush_pending_releases(cx.services);
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare_str(self.text.as_str(), &self.style, constraints);
                self.text_blob = Some(blob);
                self.text_metrics = Some(metrics);
                cx.services.caret_stops(blob, &mut self.caret_stops);
            }
        } else if self.prefix_blob.is_none()
            || self.suffix_blob.is_none()
            || self.preedit_blob.is_none()
        {
            // Preedit mode: render prefix/preedit/suffix as separate runs.
            self.queue_release_all_text_blobs();
            self.flush_pending_releases(cx.services);

            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare_str(self.text.as_str(), &self.style, constraints);
            self.text_blob = Some(blob);
            self.text_metrics = Some(metrics);
            cx.services.caret_stops(blob, &mut self.caret_stops);

            let (prefix_blob, prefix_metrics) =
                cx.services
                    .text()
                    .prepare_str(&self.text[..self.caret], &self.style, constraints);
            let (suffix_blob, suffix_metrics) =
                cx.services
                    .text()
                    .prepare_str(&self.text[self.caret..], &self.style, constraints);
            let (pre_blob, pre_metrics) =
                cx.services
                    .text()
                    .prepare_str(self.preedit.as_str(), &self.style, constraints);

            self.prefix_blob = Some(prefix_blob);
            self.prefix_metrics = Some(prefix_metrics);
            self.suffix_blob = Some(suffix_blob);
            self.suffix_metrics = Some(suffix_metrics);
            self.preedit_blob = Some(pre_blob);
            self.preedit_metrics = Some(pre_metrics);
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Paint::Solid(self.chrome_style.background),
            border: self.chrome_style.border,
            border_paint: Paint::Solid(border_color),
            corner_radii: self.chrome_style.corner_radii,
        });

        if focus_visible && let Some(mut ring) = self.chrome_style.focus_ring {
            ring.corner_radii = self.chrome_style.corner_radii;
            crate::paint::paint_focus_ring(cx.scene, DrawOrder(1), cx.bounds, ring);
        }

        let padding_left = self.chrome_style.padding.left;
        let padding_right = self.chrome_style.padding.right;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;
        let inner_width = Px((cx.bounds.size.width.0 - padding_left.0 - padding_right.0).max(0.0));
        let (placement_blob, placement_metrics) = if show_placeholder {
            (self.placeholder_blob, self.placeholder_metrics)
        } else {
            (self.text_blob, self.text_metrics)
        };
        let mut placement_metrics = placement_metrics.unwrap_or_else(|| self.approx_text_metrics());
        if placement_metrics.size.height.0 <= 0.01 {
            placement_metrics = self.approx_text_metrics();
        }
        let text_height = placement_metrics.size.height;
        let inner_height = Px((cx.bounds.size.height.0 - padding_top.0 - padding_bottom.0)
            .max(0.0)
            .max(text_height.0));

        let (vertical_offset, baseline) = if let Some(blob) = placement_blob {
            crate::text::coords::compute_text_vertical_offset_and_baseline(
                cx.services.text(),
                blob,
                inner_height,
                placement_metrics,
                self.style.vertical_placement,
            )
        } else {
            (
                crate::text::coords::compute_text_vertical_offset(inner_height, text_height),
                placement_metrics.baseline,
            )
        };

        if inner_width.0 <= 0.0 {
            self.offset_x = Px(0.0);
        } else if focused {
            let settings = cx
                .app
                .global::<fret_runtime::TextInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let margin = (settings.horizontal_autoscroll_margin_px as f32)
                .max(0.0)
                .min(inner_width.0 * 0.45);
            let caret_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, self.caret))
                .unwrap_or(Px(0.0));
            let caret_x = if self.is_ime_composing() && !self.preedit.is_empty() {
                let cursor_end =
                    crate::text_edit::ime::preedit_cursor_end(&self.preedit, self.preedit_cursor);
                let pre_w = cx
                    .services
                    .text()
                    .measure_str(&self.preedit[..cursor_end], &self.style, constraints)
                    .size
                    .width;
                caret_x + pre_w
            } else {
                caret_x
            };

            let text_end_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, self.text.len()))
                .unwrap_or(Px(0.0));
            let preedit_w = if self.is_ime_composing() && !self.preedit.is_empty() {
                cx.services
                    .text()
                    .measure_str(self.preedit.as_str(), &self.style, constraints)
                    .size
                    .width
            } else {
                Px(0.0)
            };
            let content_w = text_end_x + preedit_w;
            let max_offset = Px((content_w.0 - inner_width.0).max(0.0));

            let mut desired = self.offset_x;
            let visible_x = Px(caret_x.0 - self.offset_x.0);
            if visible_x.0 < margin {
                desired = Px(caret_x.0 - margin);
            } else if visible_x.0 > inner_width.0 - margin {
                desired = Px(caret_x.0 - (inner_width.0 - margin));
            }
            self.offset_x = Px(desired.0.clamp(0.0, max_offset.0));
        } else {
            let text_end_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, self.text.len()))
                .unwrap_or(Px(0.0));
            let preedit_w = if self.is_ime_composing() && !self.preedit.is_empty() {
                cx.services
                    .text()
                    .measure_str(self.preedit.as_str(), &self.style, constraints)
                    .size
                    .width
            } else {
                Px(0.0)
            };
            let content_w = text_end_x + preedit_w;
            let max_offset = Px((content_w.0 - inner_width.0).max(0.0));
            self.offset_x = Px(self.offset_x.0.clamp(0.0, max_offset.0));
        }

        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

        let window_focused = cx
            .app
            .global::<fret_core::WindowMetricsService>()
            .and_then(|svc| svc.focused(window))
            .unwrap_or(true);
        let selection_color = if focused || self.chrome_override {
            self.chrome_style.selection_color
        } else if !window_focused {
            theme.color_token("selection.window_inactive.background")
        } else {
            theme.color_token("selection.inactive.background")
        };
        if self.has_selection() && !self.is_ime_composing() {
            let (a, b) = self.selection_range();
            let start_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, a))
                .unwrap_or(Px(0.0));
            let end_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, b))
                .unwrap_or(Px(0.0));

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(
                    fret_core::geometry::Point::new(
                        cx.bounds.origin.x + padding_left + start_x - self.offset_x,
                        cx.bounds.origin.y + padding_top + vertical_offset,
                    ),
                    Size::new(
                        Px((end_x.0 - start_x.0).max(0.0)),
                        Px((cx.bounds.size.height.0 - padding_top.0 - padding_bottom.0).max(0.0)),
                    ),
                ),
                background: Paint::Solid(selection_color),
                border: fret_core::geometry::Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: self.chrome_style.corner_radii,
            });
        }
        let base_origin = fret_core::geometry::Point::new(
            cx.bounds.origin.x + padding_left - self.offset_x,
            cx.bounds.origin.y + padding_top + vertical_offset + baseline,
        );

        if self.preedit.is_empty() {
            if show_placeholder {
                if let Some(blob) = self.placeholder_blob {
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(0),
                        origin: base_origin,
                        text: blob,
                        paint: fret_core::scene::Paint::Solid(self.chrome_style.placeholder_color),
                        outline: None,
                        shadow: None,
                    });
                }
            } else if let Some(blob) = self.text_blob {
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: base_origin,
                    text: blob,
                    paint: fret_core::scene::Paint::Solid(self.chrome_style.text_color),
                    outline: None,
                    shadow: None,
                });
            }
        } else {
            let prefix_w = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, self.caret))
                .unwrap_or(Px(0.0));
            let pre_w = self
                .preedit_metrics
                .map(|m| m.size.width)
                .unwrap_or(Px(0.0));

            if let Some(blob) = self.prefix_blob {
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: base_origin,
                    text: blob,
                    paint: fret_core::scene::Paint::Solid(self.chrome_style.text_color),
                    outline: None,
                    shadow: None,
                });
            }
            if let Some(pre_blob) = self.preedit_blob {
                let pre_origin =
                    fret_core::geometry::Point::new(base_origin.x + prefix_w, base_origin.y);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: pre_origin,
                    text: pre_blob,
                    paint: fret_core::scene::Paint::Solid(self.chrome_style.preedit_color),
                    outline: None,
                    shadow: None,
                });
            }
            if let Some(suffix_blob) = self.suffix_blob {
                let suffix_origin = fret_core::geometry::Point::new(
                    base_origin.x + prefix_w + pre_w,
                    base_origin.y,
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: suffix_origin,
                    text: suffix_blob,
                    paint: fret_core::scene::Paint::Solid(self.chrome_style.text_color),
                    outline: None,
                    shadow: None,
                });
            }
        }

        if !focused {
            cx.scene.push(SceneOp::PopClip);
            return;
        }

        let caret_local = self.caret_rect(cx, cx.bounds, cx.scale_factor);
        let caret = Rect::new(
            fret_core::Point::new(
                cx.bounds.origin.x + caret_local.origin.x,
                cx.bounds.origin.y + caret_local.origin.y,
            ),
            caret_local.size,
        );

        // Anchor IME UI to the *visual* caret position (including preedit cursor offset).
        //
        // Note that render transforms (scrolling, anchored popovers, etc) do not affect layout
        // bounds. For platform IME positioning we must apply the accumulated transform so the OS
        // sees the same coordinates the user sees on screen.
        let ime_rect = cx.visual_rect_aabb(caret);

        if self.last_sent_cursor != Some(ime_rect) {
            self.last_sent_cursor = Some(ime_rect);
            cx.app.push_effect(Effect::ImeSetCursorArea {
                window,
                rect: ime_rect,
            });
        }

        // Draw caret as a thin quad (always visible in MVP).
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: caret,
            background: Paint::Solid(self.chrome_style.caret_color),
            border: fret_core::geometry::Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: fret_core::geometry::Corners::all(Px(1.0)),
        });

        cx.scene.push(SceneOp::PopClip);
    }
}

impl TextInput {
    const SELECTION_AUTOSCROLL_TICK: Duration = Duration::from_millis(16);

    fn ensure_selection_autoscroll_timer<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        if self.selection_autoscroll_timer.is_some() {
            return;
        }
        if !self.selection_dragging {
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
        let Some(pos) = self.last_pointer_pos else {
            return;
        };

        let padding_left = self.chrome_style.padding.left;
        let padding_right = self.chrome_style.padding.right;
        let inner_left = self.last_bounds.origin.x.0 + padding_left.0;
        let inner_right =
            self.last_bounds.origin.x.0 + self.last_bounds.size.width.0 - padding_right.0;
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

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let content_w = cx
            .services
            .text()
            .measure_str(self.text.as_str(), &self.style, constraints)
            .size
            .width
            .0;
        let max_offset = (content_w - inner_width).max(0.0);
        if max_offset <= 0.0 {
            return;
        }

        let next_offset = (self.offset_x.0 + delta).clamp(0.0, max_offset);
        if (next_offset - self.offset_x.0).abs() <= 0.01 {
            return;
        }
        self.offset_x = Px(next_offset);

        let local_x = Px((pos.x.0 - (self.last_bounds.origin.x.0 + padding_left.0)
            + self.offset_x.0)
            .max(0.0));
        self.caret = self
            .text_blob
            .map(|blob| cx.services.hit_test_x(blob, local_x))
            .unwrap_or_else(|| self.caret_from_x(local_x));
        self.clear_ime_composition();

        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
    }
}
