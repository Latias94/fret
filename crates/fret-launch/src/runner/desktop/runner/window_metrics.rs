use fret_core::{AppWindowId, ColorScheme, Edges, WindowMetricsService};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn apply_window_metrics_insets_request(
        &mut self,
        window: AppWindowId,
        safe_area_insets: Option<Option<Edges>>,
        occlusion_insets: Option<Option<Edges>>,
    ) {
        if safe_area_insets.is_some() || occlusion_insets.is_some() {
            let entry = self.diag_window_insets_overrides.entry(window).or_default();
            if let Some(value) = safe_area_insets {
                entry.safe_area_insets = Some(value);
            }
            if let Some(value) = occlusion_insets {
                entry.occlusion_insets = Some(value);
            }
        }

        let mut changed = false;
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                if let Some(value) = safe_area_insets {
                    let current = svc.safe_area_insets(window);
                    let current_known = svc.safe_area_insets_is_known(window);
                    let needs_set = if value.is_none() {
                        !current_known || current.is_some()
                    } else {
                        !current_known || current != value
                    };
                    if needs_set {
                        svc.set_safe_area_insets(window, value);
                        changed = true;
                    }
                }
                if let Some(value) = occlusion_insets {
                    let current = svc.occlusion_insets(window);
                    let current_known = svc.occlusion_insets_is_known(window);
                    let needs_set = if value.is_none() {
                        !current_known || current.is_some()
                    } else {
                        !current_known || current != value
                    };
                    if needs_set {
                        svc.set_occlusion_insets(window, value);
                        changed = true;
                    }
                }
            });
        self.request_window_metrics_redraw_if_changed(window, changed);
    }

    pub(super) fn apply_window_metrics_preferences_request(
        &mut self,
        window: AppWindowId,
        color_scheme: Option<Option<ColorScheme>>,
        prefers_reduced_motion: Option<Option<bool>>,
        text_scale_factor: Option<Option<f32>>,
    ) {
        let override_entry = self
            .diag_window_preference_overrides
            .entry(window)
            .or_default();
        if color_scheme.is_some() {
            override_entry.color_scheme = color_scheme;
        }
        if prefers_reduced_motion.is_some() {
            override_entry.prefers_reduced_motion = prefers_reduced_motion;
        }
        if text_scale_factor.is_some() {
            override_entry.text_scale_factor = text_scale_factor;
        }

        let mut changed = false;
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                if let Some(value) = color_scheme {
                    let current = svc.color_scheme(window);
                    let current_known = svc.color_scheme_is_known(window);
                    let needs_set = if value.is_none() {
                        !current_known || current.is_some()
                    } else {
                        !current_known || current != value
                    };
                    if needs_set {
                        svc.set_color_scheme(window, value);
                        changed = true;
                    }
                }
                if let Some(value) = prefers_reduced_motion {
                    let current = svc.prefers_reduced_motion(window);
                    let current_known = svc.prefers_reduced_motion_is_known(window);
                    let needs_set = if value.is_none() {
                        !current_known || current.is_some()
                    } else {
                        !current_known || current != value
                    };
                    if needs_set {
                        svc.set_prefers_reduced_motion(window, value);
                        changed = true;
                    }
                }
                if let Some(value) = text_scale_factor {
                    let current = svc.text_scale_factor(window);
                    let current_known = svc.text_scale_factor_is_known(window);
                    let needs_set = if value.is_none() {
                        !current_known || current.is_some()
                    } else {
                        !current_known || current != value
                    };
                    if needs_set {
                        svc.set_text_scale_factor(window, value);
                        changed = true;
                    }
                }
            });
        self.request_window_metrics_redraw_if_changed(window, changed);
    }

    fn request_window_metrics_redraw_if_changed(&mut self, window: AppWindowId, changed: bool) {
        if changed && let Some(state) = self.windows.get(window) {
            state.window.request_redraw();
            self.raf_windows.request(window);
        }
    }
}
