use fret_core::{
    Color, DrawOrder, Event, MouseButton, Px, Rect, SceneOp, Size, TextConstraints, TextOverflow,
    TextWrap,
};
use fret_runtime::{CommandId, Effect};

use super::TextInput;
use crate::widget::{CommandCx, EventCx, LayoutCx, PaintCx, Widget};
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
            (
                self.text().to_string(),
                Some((self.selection_anchor as u32, self.caret as u32)),
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
                let mut edit = self.edit_state();
                edit.clear_ime_composition();
                edit.set_selection_char_clamped(*anchor as usize, *focus as usize);

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                button, position, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                self.last_sent_cursor = None;
                let padding = self.chrome_style.padding.left;
                let local_x =
                    Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)).max(0.0));
                self.caret = self
                    .text_blob
                    .map(|blob| cx.services.hit_test_x(blob, local_x))
                    .unwrap_or_else(|| self.caret_from_x(local_x));
                self.selection_anchor = self.caret;
                self.clear_ime_composition();
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
            }
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
                let padding = self.chrome_style.padding.left;
                let local_x =
                    Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)).max(0.0));
                self.caret = self
                    .text_blob
                    .map(|blob| cx.services.hit_test_x(blob, local_x))
                    .unwrap_or_else(|| self.caret_from_x(local_x));
                self.clear_ime_composition();
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if cx.captured == Some(cx.node) && *button == MouseButton::Left {
                    cx.release_pointer_capture();
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
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                "text.delete_backward",
                                false,
                            );
                            let delta = crate::text_edit::commands::singleline_ui_delta(
                                "text.delete_backward",
                                outcome,
                            );
                            self.apply_singleline_ui_delta(cx, delta);
                        }
                        fret_core::KeyCode::Delete => {
                            let outcome = crate::text_edit::commands::apply_basic(
                                &mut self.edit_state(),
                                "text.delete_forward",
                                false,
                            );
                            let delta = crate::text_edit::commands::singleline_ui_delta(
                                "text.delete_forward",
                                outcome,
                            );
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
            Event::ClipboardTextUnavailable { token } => {
                if self.pending_clipboard_token == Some(*token) {
                    self.pending_clipboard_token = None;
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
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        match command.as_str() {
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
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    command.as_str(),
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
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    command.as_str(),
                    cx.window.is_some(),
                );
                if let Some(crate::text_edit::commands::ClipboardRequest::SetText { text }) =
                    result.request
                {
                    cx.app.push_effect(Effect::ClipboardSetText { text });
                }

                let delta = crate::text_edit::commands::singleline_ui_delta(
                    command.as_str(),
                    result.outcome,
                );
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.paste" => {
                let result = crate::text_edit::commands::apply_clipboard(
                    &mut self.edit_state(),
                    command.as_str(),
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
                );
                let delta =
                    crate::text_edit::commands::singleline_ui_delta(command.as_str(), outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.move_down" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.move_end",
                    is_ime_composing,
                );
                let delta =
                    crate::text_edit::commands::singleline_ui_delta(command.as_str(), outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.select_up" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.select_home",
                    is_ime_composing,
                );
                let delta =
                    crate::text_edit::commands::singleline_ui_delta(command.as_str(), outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            "text.select_down" => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    "text.select_end",
                    is_ime_composing,
                );
                let delta =
                    crate::text_edit::commands::singleline_ui_delta(command.as_str(), outcome);
                self.apply_singleline_ui_delta(cx, delta);
                true
            }
            _ => {
                let is_ime_composing = self.is_ime_composing();
                let outcome = crate::text_edit::commands::apply_basic(
                    &mut self.edit_state(),
                    command.as_str(),
                    is_ime_composing,
                );
                let delta =
                    crate::text_edit::commands::singleline_ui_delta(command.as_str(), outcome);
                if !delta.handled {
                    return false;
                }

                self.apply_singleline_ui_delta(cx, delta);
                true
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;

        cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);

        self.edit_state().clamp_caret_and_anchor_to_char_boundary();

        let theme = cx.theme().snapshot();
        self.sync_chrome_from_theme(theme);
        self.sync_text_style_from_theme(theme);

        let base_constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics =
            cx.services
                .text()
                .measure_str(self.text.as_str(), &self.style, base_constraints);
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
        self.sync_chrome_from_theme(theme);
        self.sync_text_style_from_theme(theme);
        let focused = cx.focus == Some(cx.node);
        if !focused && self.is_ime_composing() {
            self.clear_ime_composition();
        }
        let border_color = if focused && self.chrome_style.focus_ring.is_some() {
            self.chrome_style.border_color
        } else if focused {
            self.chrome_style.border_color_focused
        } else {
            self.chrome_style.border_color
        };

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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
            let placeholder = self.placeholder.as_ref().expect("checked above").as_ref();
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
            background: self.chrome_style.background,
            border: self.chrome_style.border,
            border_color,
            corner_radii: self.chrome_style.corner_radii,
        });

        if focused
            && crate::focus_visible::is_focus_visible(cx.app, cx.window)
            && let Some(mut ring) = self.chrome_style.focus_ring
        {
            ring.corner_radii = self.chrome_style.corner_radii;
            crate::paint::paint_focus_ring(cx.scene, DrawOrder(1), cx.bounds, ring);
        }

        let padding_left = self.chrome_style.padding.left;
        let _padding_right = self.chrome_style.padding.right;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;
        let text_height = if show_placeholder {
            self.placeholder_metrics
                .map(|m| m.size.height)
                .unwrap_or(Px(16.0))
        } else {
            self.text_metrics.map(|m| m.size.height).unwrap_or(Px(16.0))
        };
        let inner_height = Px((cx.bounds.size.height.0 - padding_top.0 - padding_bottom.0)
            .max(0.0)
            .max(text_height.0));
        let vertical_offset = Px(((inner_height.0 - text_height.0).max(0.0)) / 2.0);

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
                        cx.bounds.origin.x + padding_left + start_x,
                        cx.bounds.origin.y + padding_top + vertical_offset,
                    ),
                    Size::new(
                        Px((end_x.0 - start_x.0).max(0.0)),
                        Px((cx.bounds.size.height.0 - padding_top.0 - padding_bottom.0).max(0.0)),
                    ),
                ),
                background: self.chrome_style.selection_color,
                border: fret_core::geometry::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: self.chrome_style.corner_radii,
            });
        }
        let baseline = if show_placeholder {
            self.placeholder_metrics.map(|m| m.baseline)
        } else {
            self.text_metrics.map(|m| m.baseline)
        }
        .unwrap_or(Px(10.0));
        let base_origin = fret_core::geometry::Point::new(
            cx.bounds.origin.x + padding_left,
            cx.bounds.origin.y + padding_top + vertical_offset + baseline,
        );

        if self.preedit.is_empty() {
            if show_placeholder {
                if let Some(blob) = self.placeholder_blob {
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(0),
                        origin: base_origin,
                        text: blob,
                        color: self.chrome_style.placeholder_color,
                    });
                }
            } else if let Some(blob) = self.text_blob {
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: base_origin,
                    text: blob,
                    color: self.chrome_style.text_color,
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
                    color: self.chrome_style.text_color,
                });
            }
            if let Some(pre_blob) = self.preedit_blob {
                let pre_origin =
                    fret_core::geometry::Point::new(base_origin.x + prefix_w, base_origin.y);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: pre_origin,
                    text: pre_blob,
                    color: self.chrome_style.preedit_color,
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
                    color: self.chrome_style.text_color,
                });
            }
        }

        if !focused {
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

        // Anchor IME UI to the *current* caret position (including preedit cursor offset).
        // This keeps the IME candidate/composition UI tracking the cursor within the preedit text.
        let ime_rect = caret;

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
            background: self.chrome_style.caret_color,
            border: fret_core::geometry::Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::geometry::Corners::all(Px(1.0)),
        });
    }
}
