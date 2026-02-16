use super::super::*;

impl<H: UiHost> UiTree<H> {
    pub(crate) fn debug_paint_widget_exclusive_resume(&mut self) {
        if !self.debug_enabled {
            return;
        }
        if self.debug_paint_widget_exclusive_started.is_some() {
            return;
        }
        self.debug_paint_widget_exclusive_started = Some(Instant::now());
    }

    pub(crate) fn debug_paint_widget_exclusive_pause(&mut self) -> bool {
        if !self.debug_enabled {
            return false;
        }
        let Some(started) = self.debug_paint_widget_exclusive_started.take() else {
            return false;
        };
        self.debug_stats.paint_widget_time = self
            .debug_stats
            .paint_widget_time
            .saturating_add(started.elapsed());
        true
    }

    pub(crate) fn debug_record_paint_host_widget_observed_models(
        &mut self,
        elapsed: Duration,
        items: usize,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.paint_host_widget_observed_models_time = self
            .debug_stats
            .paint_host_widget_observed_models_time
            .saturating_add(elapsed);
        self.debug_stats.paint_host_widget_observed_models_items = self
            .debug_stats
            .paint_host_widget_observed_models_items
            .saturating_add(items.min(u32::MAX as usize) as u32);
    }

    pub(crate) fn debug_record_paint_host_widget_observed_globals(
        &mut self,
        elapsed: Duration,
        items: usize,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.paint_host_widget_observed_globals_time = self
            .debug_stats
            .paint_host_widget_observed_globals_time
            .saturating_add(elapsed);
        self.debug_stats.paint_host_widget_observed_globals_items = self
            .debug_stats
            .paint_host_widget_observed_globals_items
            .saturating_add(items.min(u32::MAX as usize) as u32);
    }

    pub(crate) fn debug_record_paint_host_widget_instance_lookup(&mut self, elapsed: Duration) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.paint_host_widget_instance_lookup_time = self
            .debug_stats
            .paint_host_widget_instance_lookup_time
            .saturating_add(elapsed);
        self.debug_stats.paint_host_widget_instance_lookup_calls = self
            .debug_stats
            .paint_host_widget_instance_lookup_calls
            .saturating_add(1);
    }

    pub(crate) fn debug_record_paint_text_prepare(&mut self, elapsed: Duration) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.paint_text_prepare_time = self
            .debug_stats
            .paint_text_prepare_time
            .saturating_add(elapsed);
        self.debug_stats.paint_text_prepare_calls =
            self.debug_stats.paint_text_prepare_calls.saturating_add(1);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn debug_record_paint_text_prepare_hotspot(
        &mut self,
        node: NodeId,
        element: Option<GlobalElementId>,
        element_kind: &'static str,
        text_len: u32,
        constraints: TextConstraints,
        reasons_mask: u16,
        prepare_time: Duration,
    ) {
        if !self.debug_enabled {
            return;
        }
        const MAX_PREPARE_HOTSPOTS: usize = 16;
        let record = UiDebugPaintTextPrepareHotspot {
            node,
            element,
            element_kind,
            text_len,
            constraints,
            reasons_mask,
            prepare_time,
        };
        let idx = self
            .debug_paint_text_prepare_hotspots
            .iter()
            .position(|h| h.prepare_time < record.prepare_time)
            .unwrap_or(self.debug_paint_text_prepare_hotspots.len());
        self.debug_paint_text_prepare_hotspots.insert(idx, record);
        if self.debug_paint_text_prepare_hotspots.len() > MAX_PREPARE_HOTSPOTS {
            self.debug_paint_text_prepare_hotspots
                .truncate(MAX_PREPARE_HOTSPOTS);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn debug_record_paint_text_prepare_reasons(
        &mut self,
        blob_missing: bool,
        scale_changed: bool,
        text_changed: bool,
        rich_changed: bool,
        style_changed: bool,
        wrap_changed: bool,
        overflow_changed: bool,
        width_changed: bool,
        font_stack_changed: bool,
    ) {
        if !self.debug_enabled {
            return;
        }
        if blob_missing {
            self.debug_stats.paint_text_prepare_reason_blob_missing = self
                .debug_stats
                .paint_text_prepare_reason_blob_missing
                .saturating_add(1);
        }
        if scale_changed {
            self.debug_stats.paint_text_prepare_reason_scale_changed = self
                .debug_stats
                .paint_text_prepare_reason_scale_changed
                .saturating_add(1);
        }
        if text_changed {
            self.debug_stats.paint_text_prepare_reason_text_changed = self
                .debug_stats
                .paint_text_prepare_reason_text_changed
                .saturating_add(1);
        }
        if rich_changed {
            self.debug_stats.paint_text_prepare_reason_rich_changed = self
                .debug_stats
                .paint_text_prepare_reason_rich_changed
                .saturating_add(1);
        }
        if style_changed {
            self.debug_stats.paint_text_prepare_reason_style_changed = self
                .debug_stats
                .paint_text_prepare_reason_style_changed
                .saturating_add(1);
        }
        if wrap_changed {
            self.debug_stats.paint_text_prepare_reason_wrap_changed = self
                .debug_stats
                .paint_text_prepare_reason_wrap_changed
                .saturating_add(1);
        }
        if overflow_changed {
            self.debug_stats.paint_text_prepare_reason_overflow_changed = self
                .debug_stats
                .paint_text_prepare_reason_overflow_changed
                .saturating_add(1);
        }
        if width_changed {
            self.debug_stats.paint_text_prepare_reason_width_changed = self
                .debug_stats
                .paint_text_prepare_reason_width_changed
                .saturating_add(1);
        }
        if font_stack_changed {
            self.debug_stats
                .paint_text_prepare_reason_font_stack_changed = self
                .debug_stats
                .paint_text_prepare_reason_font_stack_changed
                .saturating_add(1);
        }
    }
}
