use std::collections::HashMap;

use crate::{AppWindowId, Color, Edges, Event, Point, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

/// User contrast preference for accessibility.
///
/// This is based on the `prefers-contrast` media query vocabulary used on the web. Runners may
/// supply best-effort values and leave it `None` when unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastPreference {
    NoPreference,
    More,
    Less,
    Custom,
}

/// Forced colors mode (high contrast) preference.
///
/// This is based on the `forced-colors` media query vocabulary used on the web. Runners may
/// supply best-effort values and leave it `None` when unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForcedColorsMode {
    None,
    Active,
}

/// Window position in screen space, expressed in **logical pixels** (see ADR 0017).
///
/// This is intended for best-effort window placement persistence and multi-window orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowLogicalPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowAnchor {
    pub window: AppWindowId,
    pub position: Point,
}

#[derive(Debug, Default, Clone)]
pub struct WindowMetricsService {
    inner_sizes: HashMap<AppWindowId, Size>,
    logical_positions: HashMap<AppWindowId, WindowLogicalPosition>,
    scale_factors: HashMap<AppWindowId, f32>,
    focused: HashMap<AppWindowId, bool>,
    prefers_reduced_motion: HashMap<AppWindowId, Option<bool>>,
    text_scale_factor: HashMap<AppWindowId, Option<f32>>,
    prefers_reduced_transparency: HashMap<AppWindowId, Option<bool>>,
    accent_color: HashMap<AppWindowId, Option<Color>>,
    color_scheme: HashMap<AppWindowId, Option<ColorScheme>>,
    contrast_preference: HashMap<AppWindowId, Option<ContrastPreference>>,
    forced_colors_mode: HashMap<AppWindowId, Option<ForcedColorsMode>>,
    safe_area_insets: HashMap<AppWindowId, Option<Edges>>,
    occlusion_insets: HashMap<AppWindowId, Option<Edges>>,
}

impl WindowMetricsService {
    pub fn set_inner_size(&mut self, window: AppWindowId, size: Size) {
        self.inner_sizes.insert(window, size);
    }

    pub fn inner_size(&self, window: AppWindowId) -> Option<Size> {
        self.inner_sizes.get(&window).copied()
    }

    pub fn set_logical_position(&mut self, window: AppWindowId, position: WindowLogicalPosition) {
        self.logical_positions.insert(window, position);
    }

    pub fn logical_position(&self, window: AppWindowId) -> Option<WindowLogicalPosition> {
        self.logical_positions.get(&window).copied()
    }

    pub fn set_scale_factor(&mut self, window: AppWindowId, scale_factor: f32) {
        self.scale_factors.insert(window, scale_factor);
    }

    pub fn scale_factor(&self, window: AppWindowId) -> Option<f32> {
        self.scale_factors.get(&window).copied()
    }

    pub fn set_focused(&mut self, window: AppWindowId, focused: bool) {
        self.focused.insert(window, focused);
    }

    pub fn focused(&self, window: AppWindowId) -> Option<bool> {
        self.focused.get(&window).copied()
    }

    pub fn set_prefers_reduced_motion(&mut self, window: AppWindowId, prefers: Option<bool>) {
        self.prefers_reduced_motion.insert(window, prefers);
    }

    pub fn prefers_reduced_motion(&self, window: AppWindowId) -> Option<bool> {
        self.prefers_reduced_motion.get(&window).copied().flatten()
    }

    pub fn prefers_reduced_motion_is_known(&self, window: AppWindowId) -> bool {
        self.prefers_reduced_motion.contains_key(&window)
    }

    pub fn set_text_scale_factor(&mut self, window: AppWindowId, factor: Option<f32>) {
        self.text_scale_factor.insert(window, factor);
    }

    pub fn text_scale_factor(&self, window: AppWindowId) -> Option<f32> {
        self.text_scale_factor.get(&window).copied().flatten()
    }

    pub fn text_scale_factor_is_known(&self, window: AppWindowId) -> bool {
        self.text_scale_factor.contains_key(&window)
    }

    pub fn set_prefers_reduced_transparency(&mut self, window: AppWindowId, prefers: Option<bool>) {
        self.prefers_reduced_transparency.insert(window, prefers);
    }

    pub fn prefers_reduced_transparency(&self, window: AppWindowId) -> Option<bool> {
        self.prefers_reduced_transparency
            .get(&window)
            .copied()
            .flatten()
    }

    pub fn prefers_reduced_transparency_is_known(&self, window: AppWindowId) -> bool {
        self.prefers_reduced_transparency.contains_key(&window)
    }

    pub fn set_accent_color(&mut self, window: AppWindowId, color: Option<Color>) {
        self.accent_color.insert(window, color);
    }

    pub fn accent_color(&self, window: AppWindowId) -> Option<Color> {
        self.accent_color.get(&window).copied().flatten()
    }

    pub fn accent_color_is_known(&self, window: AppWindowId) -> bool {
        self.accent_color.contains_key(&window)
    }

    pub fn set_color_scheme(&mut self, window: AppWindowId, scheme: Option<ColorScheme>) {
        self.color_scheme.insert(window, scheme);
    }

    pub fn color_scheme(&self, window: AppWindowId) -> Option<ColorScheme> {
        self.color_scheme.get(&window).copied().flatten()
    }

    pub fn color_scheme_is_known(&self, window: AppWindowId) -> bool {
        self.color_scheme.contains_key(&window)
    }

    pub fn set_contrast_preference(
        &mut self,
        window: AppWindowId,
        value: Option<ContrastPreference>,
    ) {
        self.contrast_preference.insert(window, value);
    }

    pub fn contrast_preference(&self, window: AppWindowId) -> Option<ContrastPreference> {
        self.contrast_preference.get(&window).copied().flatten()
    }

    pub fn contrast_preference_is_known(&self, window: AppWindowId) -> bool {
        self.contrast_preference.contains_key(&window)
    }

    pub fn set_forced_colors_mode(&mut self, window: AppWindowId, value: Option<ForcedColorsMode>) {
        self.forced_colors_mode.insert(window, value);
    }

    pub fn forced_colors_mode(&self, window: AppWindowId) -> Option<ForcedColorsMode> {
        self.forced_colors_mode.get(&window).copied().flatten()
    }

    pub fn forced_colors_mode_is_known(&self, window: AppWindowId) -> bool {
        self.forced_colors_mode.contains_key(&window)
    }

    pub fn set_safe_area_insets(&mut self, window: AppWindowId, insets: Option<Edges>) {
        self.safe_area_insets.insert(window, insets);
    }

    pub fn safe_area_insets(&self, window: AppWindowId) -> Option<Edges> {
        self.safe_area_insets.get(&window).copied().flatten()
    }

    pub fn safe_area_insets_is_known(&self, window: AppWindowId) -> bool {
        self.safe_area_insets.contains_key(&window)
    }

    pub fn set_occlusion_insets(&mut self, window: AppWindowId, insets: Option<Edges>) {
        self.occlusion_insets.insert(window, insets);
    }

    pub fn occlusion_insets(&self, window: AppWindowId) -> Option<Edges> {
        self.occlusion_insets.get(&window).copied().flatten()
    }

    pub fn occlusion_insets_is_known(&self, window: AppWindowId) -> bool {
        self.occlusion_insets.contains_key(&window)
    }

    pub fn inner_bounds(&self, window: AppWindowId) -> Option<Rect> {
        let size = self.inner_size(window)?;
        Some(Rect::new(Point::new(crate::Px(0.0), crate::Px(0.0)), size))
    }

    pub fn apply_event(&mut self, window: AppWindowId, event: &Event) {
        match event {
            Event::WindowResized { width, height } => {
                self.set_inner_size(window, Size::new(*width, *height));
            }
            Event::WindowMoved(position) => {
                self.set_logical_position(window, *position);
            }
            Event::WindowFocusChanged(focused) => {
                self.set_focused(window, *focused);
            }
            Event::WindowScaleFactorChanged(scale_factor) => {
                self.set_scale_factor(window, *scale_factor);
            }
            _ => {}
        }
    }

    pub fn remove(&mut self, window: AppWindowId) {
        self.inner_sizes.remove(&window);
        self.logical_positions.remove(&window);
        self.scale_factors.remove(&window);
        self.focused.remove(&window);
        self.prefers_reduced_motion.remove(&window);
        self.text_scale_factor.remove(&window);
        self.prefers_reduced_transparency.remove(&window);
        self.accent_color.remove(&window);
        self.color_scheme.remove(&window);
        self.contrast_preference.remove(&window);
        self.forced_colors_mode.remove(&window);
        self.safe_area_insets.remove(&window);
        self.occlusion_insets.remove(&window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Px;

    #[test]
    fn window_metrics_apply_event_tracks_resize_move_scale() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));

        svc.apply_event(
            window,
            &Event::WindowResized {
                width: Px(100.0),
                height: Px(200.0),
            },
        );
        assert_eq!(
            svc.inner_size(window),
            Some(Size::new(Px(100.0), Px(200.0)))
        );

        svc.apply_event(
            window,
            &Event::WindowMoved(WindowLogicalPosition { x: 10, y: 20 }),
        );
        assert_eq!(
            svc.logical_position(window),
            Some(WindowLogicalPosition { x: 10, y: 20 })
        );

        svc.apply_event(window, &Event::WindowScaleFactorChanged(2.0));
        assert_eq!(svc.scale_factor(window), Some(2.0));

        svc.apply_event(window, &Event::WindowFocusChanged(true));
        assert_eq!(svc.focused(window), Some(true));
    }

    #[test]
    fn window_metrics_remove_clears_all_fields() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(2));

        svc.set_inner_size(window, Size::new(Px(1.0), Px(2.0)));
        svc.set_logical_position(window, WindowLogicalPosition { x: 1, y: 2 });
        svc.set_scale_factor(window, 1.5);
        svc.set_focused(window, true);
        svc.set_prefers_reduced_motion(window, Some(true));
        svc.set_text_scale_factor(window, Some(1.25));
        svc.set_prefers_reduced_transparency(window, Some(true));
        svc.set_accent_color(
            window,
            Some(crate::Color {
                r: 1.0,
                g: 0.5,
                b: 0.25,
                a: 1.0,
            }),
        );
        svc.set_color_scheme(window, Some(ColorScheme::Dark));
        svc.set_contrast_preference(window, Some(ContrastPreference::More));
        svc.set_forced_colors_mode(window, Some(ForcedColorsMode::Active));
        svc.set_safe_area_insets(window, Some(Edges::all(Px(1.0))));
        svc.set_occlusion_insets(window, Some(Edges::all(Px(2.0))));
        svc.remove(window);

        assert_eq!(svc.inner_size(window), None);
        assert_eq!(svc.logical_position(window), None);
        assert_eq!(svc.scale_factor(window), None);
        assert_eq!(svc.focused(window), None);
        assert_eq!(svc.prefers_reduced_motion(window), None);
        assert_eq!(svc.text_scale_factor(window), None);
        assert_eq!(svc.prefers_reduced_transparency(window), None);
        assert_eq!(svc.accent_color(window), None);
        assert_eq!(svc.color_scheme(window), None);
        assert_eq!(svc.contrast_preference(window), None);
        assert_eq!(svc.forced_colors_mode(window), None);
        assert_eq!(svc.safe_area_insets(window), None);
        assert_eq!(svc.occlusion_insets(window), None);
    }

    #[test]
    fn window_metrics_insets_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(3));

        svc.set_safe_area_insets(window, None);
        svc.set_occlusion_insets(window, None);

        assert_eq!(svc.safe_area_insets(window), None);
        assert_eq!(svc.occlusion_insets(window), None);
        assert!(svc.safe_area_insets_is_known(window));
        assert!(svc.occlusion_insets_is_known(window));
    }

    #[test]
    fn window_metrics_prefers_reduced_motion_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(4));

        svc.set_prefers_reduced_motion(window, None);

        assert_eq!(svc.prefers_reduced_motion(window), None);
        assert!(svc.prefers_reduced_motion_is_known(window));
    }

    #[test]
    fn window_metrics_text_scale_factor_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(41));

        svc.set_text_scale_factor(window, None);

        assert_eq!(svc.text_scale_factor(window), None);
        assert!(svc.text_scale_factor_is_known(window));
    }

    #[test]
    fn window_metrics_prefers_reduced_transparency_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(42));

        svc.set_prefers_reduced_transparency(window, None);

        assert_eq!(svc.prefers_reduced_transparency(window), None);
        assert!(svc.prefers_reduced_transparency_is_known(window));
    }

    #[test]
    fn window_metrics_accent_color_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(43));

        svc.set_accent_color(window, None);

        assert_eq!(svc.accent_color(window), None);
        assert!(svc.accent_color_is_known(window));
    }

    #[test]
    fn window_metrics_color_scheme_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(5));

        svc.set_color_scheme(window, None);

        assert_eq!(svc.color_scheme(window), None);
        assert!(svc.color_scheme_is_known(window));
    }

    #[test]
    fn window_metrics_contrast_preference_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(6));

        svc.set_contrast_preference(window, None);

        assert_eq!(svc.contrast_preference(window), None);
        assert!(svc.contrast_preference_is_known(window));
    }

    #[test]
    fn window_metrics_forced_colors_mode_can_be_explicitly_set_to_none() {
        let mut svc = WindowMetricsService::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(7));

        svc.set_forced_colors_mode(window, None);

        assert_eq!(svc.forced_colors_mode(window), None);
        assert!(svc.forced_colors_mode_is_known(window));
    }
}
