use fret_core::{
    FontId, Px, Rect, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle,
    TextWrap,
};

use super::TextInput;
use super::cx::TextInputUiCx;
use crate::widget::{EventCx, PaintCx};
use crate::{Invalidation, TextInputStyle, UiHost};

impl TextInput {
    pub fn new() -> Self {
        Self {
            a11y_role: SemanticsRole::TextField,
            enabled: true,
            focusable: true,
            text: String::new(),
            caret: 0,
            selection_anchor: 0,
            offset_x: Px(0.0),
            selection_dragging: false,
            last_pointer_pos: None,
            selection_autoscroll_timer: None,
            preedit: String::new(),
            preedit_cursor: None,
            ime_replace_range: None,
            ime_deduper: crate::text_edit::ime::Deduper::default(),
            style: TextStyle {
                font: FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            placeholder: None,
            text_blob: None,
            text_metrics: None,
            placeholder_blob: None,
            placeholder_metrics: None,
            prefix_blob: None,
            prefix_metrics: None,
            suffix_blob: None,
            suffix_metrics: None,
            preedit_blob: None,
            preedit_metrics: None,
            caret_stops: Vec::new(),
            pending_release: Vec::new(),
            prepared_scale_factor_bits: None,
            last_font_stack_key: None,
            last_bounds: Rect::default(),
            last_sent_cursor: None,
            pending_clipboard_token: None,
            pending_primary_selection_token: None,

            chrome_style: TextInputStyle::default(),
            chrome_override: false,
            last_theme_revision: None,

            text_style_override: false,
            last_text_style_theme_revision: None,
        }
    }

    pub fn set_a11y_role(&mut self, role: SemanticsRole) {
        if self.a11y_role == role {
            return;
        }
        self.a11y_role = role;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    pub fn set_placeholder(&mut self, placeholder: Option<std::sync::Arc<str>>) {
        if self.placeholder == placeholder {
            return;
        }
        self.placeholder = placeholder;
        if let Some(blob) = self.placeholder_blob.take() {
            self.pending_release.push(blob);
        }
        self.placeholder_metrics = None;
        self.last_sent_cursor = None;
    }

    pub fn set_chrome_style(&mut self, style: TextInputStyle) {
        self.chrome_style = style;
        self.chrome_override = true;
        self.last_theme_revision = None;
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        if self.style == style {
            return;
        }
        self.queue_release_all_text_blobs();
        self.style = style;
        self.text_style_override = true;
        self.last_text_style_theme_revision = None;
        self.last_sent_cursor = None;
    }

    pub(super) fn sync_chrome_from_theme(&mut self, theme: crate::ThemeSnapshot) {
        if self.chrome_override {
            return;
        }
        if self.last_theme_revision == Some(theme.revision) {
            return;
        }
        self.last_theme_revision = Some(theme.revision);
        self.chrome_style = TextInputStyle::from_theme(theme);
    }

    pub(super) fn sync_text_style_from_theme(&mut self, theme: crate::ThemeSnapshot) {
        if self.text_style_override {
            return;
        }
        if self.last_text_style_theme_revision == Some(theme.revision) {
            return;
        }
        self.last_text_style_theme_revision = Some(theme.revision);

        let next_size = theme.metric_token("font.size");

        let mut changed = false;
        if self.style.size != next_size {
            self.style.size = next_size;
            changed = true;
        }

        let (base_size, base_line_height) = match self.style.font {
            FontId::Monospace => (
                theme.metric_token("mono_font.size"),
                theme.metric_token("mono_font.line_height"),
            ),
            _ => (
                theme.metric_token("font.size"),
                theme.metric_token("font.line_height"),
            ),
        };
        let base_size_px = base_size.0;
        let base_line_height_px = base_line_height.0;
        let ratio = if base_size_px.is_finite()
            && base_line_height_px.is_finite()
            && base_size_px > 0.0
            && base_line_height_px > 0.0
        {
            base_line_height_px / base_size_px
        } else {
            1.25
        };
        let size_px = self.style.size.0.max(0.0);
        let next_line_height = Px((size_px * ratio).max(size_px));

        if self.style.line_height != Some(next_line_height) {
            self.style.line_height = Some(next_line_height);
            changed = true;
        }

        if changed {
            self.queue_release_all_text_blobs();
            self.prepared_scale_factor_bits = None;
            self.last_sent_cursor = None;
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self.offset_x = Px(0.0);
        self.selection_dragging = false;
        self.last_pointer_pos = None;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.queue_release_all_text_blobs();
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self.offset_x = Px(0.0);
        self.selection_dragging = false;
        self.last_pointer_pos = None;
        self.clear_ime_composition();
        self.ime_deduper = crate::text_edit::ime::Deduper::default();
        self.text_blob = None;
        self.text_metrics = None;
        self.prefix_blob = None;
        self.prefix_metrics = None;
        self.suffix_blob = None;
        self.suffix_metrics = None;
        self.preedit_blob = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
        self.last_sent_cursor = None;
    }

    pub(super) fn is_ime_composing(&self) -> bool {
        crate::text_edit::ime::is_composing(&self.preedit, self.preedit_cursor)
    }

    pub(super) fn preedit_cursor_end(&self) -> usize {
        crate::text_edit::ime::preedit_cursor_end(&self.preedit, self.preedit_cursor)
    }

    pub(super) fn clear_ime_composition(&mut self) {
        crate::text_edit::ime::clear_state(
            &mut self.preedit,
            &mut self.preedit_cursor,
            &mut self.ime_replace_range,
        );
    }

    pub(super) fn queue_release_all_text_blobs(&mut self) {
        for blob in [
            self.text_blob.take(),
            self.placeholder_blob.take(),
            self.prefix_blob.take(),
            self.suffix_blob.take(),
            self.preedit_blob.take(),
        ]
        .into_iter()
        .flatten()
        {
            self.pending_release.push(blob);
        }
        self.text_metrics = None;
        self.placeholder_metrics = None;
        self.prefix_metrics = None;
        self.suffix_metrics = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
        self.prepared_scale_factor_bits = None;
    }

    pub(super) fn flush_pending_releases(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.pending_release.drain(..) {
            services.text().release(blob);
        }
    }

    pub(super) fn approx_text_metrics(&self) -> TextMetrics {
        let line_height = self
            .style
            .line_height
            .unwrap_or(Px(self.style.size.0 * 1.2))
            .max(Px(1.0));
        let baseline = Px((line_height.0 * 0.8).max(0.0));
        TextMetrics {
            size: Size::new(Px(0.0), line_height),
            baseline,
        }
    }
}

impl TextInput {
    pub(super) fn is_focused<H: UiHost>(&self, cx: &EventCx<'_, H>) -> bool {
        cx.focus == Some(cx.node)
    }

    pub(super) fn edit_state(&mut self) -> crate::text_edit::state::TextEditState<'_> {
        crate::text_edit::state::TextEditState::new(
            &mut self.text,
            &mut self.caret,
            &mut self.selection_anchor,
            &mut self.preedit,
            &mut self.preedit_cursor,
            &mut self.ime_replace_range,
        )
    }

    pub(super) fn mark_text_blobs_dirty(&mut self) {
        self.queue_release_all_text_blobs();
        self.last_sent_cursor = None;
    }

    pub(super) fn apply_singleline_ui_delta(
        &mut self,
        cx: &mut impl TextInputUiCx,
        delta: crate::text_edit::commands::SingleLineUiDelta,
    ) {
        if delta.invalidate_layout {
            if delta.release_text_blobs {
                self.mark_text_blobs_dirty();
            }
            cx.invalidate_self(Invalidation::Layout);
        } else if delta.invalidate_paint {
            cx.invalidate_self(Invalidation::Paint);
        }
        if delta.request_redraw {
            cx.request_redraw();
        }
    }

    pub(super) fn selection_range(&self) -> (usize, usize) {
        crate::text_edit::buffer::selection_range(self.selection_anchor, self.caret)
    }

    pub(super) fn has_selection(&self) -> bool {
        crate::text_edit::buffer::has_selection(self.selection_anchor, self.caret)
    }

    pub(super) fn replace_selection(&mut self, insert: &str) {
        if self.edit_state().replace_selection(insert) {
            self.mark_text_blobs_dirty();
        }
    }

    pub(super) fn replace_selection_changed(&mut self, insert: &str) -> bool {
        let changed = self.edit_state().replace_selection(insert);
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn delete_selection_if_any(&mut self) -> bool {
        let changed = self.edit_state().delete_selection_if_any();
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn delete_backward_char(&mut self) -> bool {
        let changed = self.edit_state().delete_backward_char();
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn delete_forward_char(&mut self) -> bool {
        let changed = self.edit_state().delete_forward_char();
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn delete_word_backward(&mut self) -> bool {
        let changed = self.edit_state().delete_word_backward();
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn delete_word_forward(&mut self) -> bool {
        let changed = self.edit_state().delete_word_forward();
        if changed {
            self.mark_text_blobs_dirty();
        }
        changed
    }

    pub(super) fn caret_rect<H: UiHost>(
        &self,
        cx: &mut PaintCx<'_, H>,
        bounds: Rect,
        scale_factor: f32,
    ) -> Rect {
        let padding_left = self.chrome_style.padding.left;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;

        let caret_x = self
            .text_blob
            .map(|blob| cx.services.caret_x(blob, self.caret))
            .unwrap_or(Px(0.0));

        let mut x = padding_left + caret_x - self.offset_x;

        if self.is_ime_composing() && !self.preedit.is_empty() {
            let cursor =
                crate::text_edit::ime::preedit_cursor_end(&self.preedit, self.preedit_cursor);
            let constraints = TextConstraints {
                max_width: Some(bounds.size.width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: cx.scale_factor,
            };
            let pre_metrics =
                cx.services
                    .text()
                    .measure_str(&self.preedit[..cursor], &self.style, constraints);
            x = x + pre_metrics.size.width;
        }

        let mut metrics = self
            .text_metrics
            .unwrap_or_else(|| self.approx_text_metrics());
        if metrics.size.height.0 <= 0.01 {
            metrics = self.approx_text_metrics();
        }

        let inner_h = Px((bounds.size.height.0 - padding_top.0 - padding_bottom.0)
            .max(0.0)
            .max(metrics.size.height.0));

        let (vertical_offset, baseline) = if let Some(blob) = self.text_blob {
            crate::text::coords::compute_text_vertical_offset_and_baseline(
                cx.services.text(),
                blob,
                inner_h,
                metrics,
                self.style.vertical_placement,
            )
        } else {
            (
                crate::text::coords::compute_text_vertical_offset(inner_h, metrics.size.height),
                metrics.baseline,
            )
        };

        let (caret_top, caret_height) = if let Some(blob) = self.text_blob {
            crate::text::coords::compute_first_line_box_top_and_height(
                cx.services.text(),
                blob,
                baseline,
                metrics.size.height.max(Px(16.0)),
            )
        } else {
            (Px(0.0), metrics.size.height.max(Px(16.0)))
        };

        let hairline = Px((1.0 / scale_factor.max(1.0)).max(1.0 / 8.0));
        Rect::new(
            fret_core::geometry::Point::new(x, padding_top + vertical_offset + caret_top),
            Size::new(Px(hairline.0.max(1.0)), caret_height),
        )
    }

    pub(super) fn caret_from_x(&self, x: Px) -> usize {
        if self.caret_stops.is_empty() {
            return 0;
        }
        let mut best = self.caret_stops[0].0;
        let mut best_dist = (self.caret_stops[0].1.0 - x.0).abs();
        for (idx, px) in &self.caret_stops {
            let dist = (px.0 - x.0).abs();
            if dist < best_dist {
                best = *idx;
                best_dist = dist;
            }
        }
        best
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
