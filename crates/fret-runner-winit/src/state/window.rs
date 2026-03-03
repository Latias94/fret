use fret_core::{CursorIcon, Rect};
use winit::window::Window;

use crate::mapping::map_cursor_icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImeSurroundingTextUpdate {
    pub text: String,
    pub cursor: usize,
    pub anchor: usize,
}

#[derive(Debug, Default, Clone)]
pub struct WinitWindowState {
    ime_allowed: bool,
    ime_cursor_area: Option<Rect>,
    ime_cursor_area_dispatched_px: Option<ImeCursorAreaPx>,
    last_prepared_scale_factor: Option<f64>,
    cursor_icon: CursorIcon,
    ime_surrounding_text: Option<ImeSurroundingTextUpdate>,
    pending: WinitWindowPendingState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ImeCursorAreaPx {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn normalize_scale_factor(scale_factor: f64) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn quantize_ime_cursor_area_px(rect: Rect, scale_factor: f64) -> ImeCursorAreaPx {
    let scale = normalize_scale_factor(scale_factor);

    ImeCursorAreaPx {
        x: (rect.origin.x.0 as f64 * scale).round() as i32,
        y: (rect.origin.y.0 as f64 * scale).round() as i32,
        width: (rect.size.width.0 as f64 * scale).round().max(1.0) as i32,
        height: (rect.size.height.0 as f64 * scale).round().max(1.0) as i32,
    }
}

#[derive(Debug, Default, Clone)]
struct WinitWindowPendingState {
    ime_allowed: Option<bool>,
    ime_cursor_area: Option<Rect>,
    cursor_icon: Option<CursorIcon>,
    ime_surrounding_text: Option<ImeSurroundingTextUpdate>,
}

impl WinitWindowState {
    fn should_dispatch_ime_cursor_area(&mut self, rect: Rect, scale_factor: f64) -> bool {
        let quantized = quantize_ime_cursor_area_px(rect, scale_factor);
        if self.ime_cursor_area_dispatched_px == Some(quantized) {
            return false;
        }
        self.ime_cursor_area_dispatched_px = Some(quantized);
        true
    }

    fn reset_ime_cursor_area_dispatch(&mut self) {
        self.ime_cursor_area_dispatched_px = None;
    }

    fn begin_prepare_frame(&mut self, scale_factor: f64) -> f64 {
        let scale_factor = normalize_scale_factor(scale_factor);
        let scale_changed = self.last_prepared_scale_factor != Some(scale_factor);
        self.last_prepared_scale_factor = Some(scale_factor);

        if scale_changed {
            self.reset_ime_cursor_area_dispatch();
            if self.ime_allowed
                && self.pending.ime_cursor_area.is_none()
                && let Some(rect) = self.ime_cursor_area
            {
                self.pending.ime_cursor_area = Some(rect);
            }
        }

        scale_factor
    }

    pub fn set_ime_allowed(&mut self, enabled: bool) -> bool {
        if self.ime_allowed == enabled {
            return false;
        }
        self.ime_allowed = enabled;
        self.pending.ime_allowed = Some(enabled);
        true
    }

    pub fn set_ime_cursor_area(&mut self, rect: Rect) -> bool {
        if self.ime_cursor_area == Some(rect) {
            return false;
        }
        self.ime_cursor_area = Some(rect);
        self.pending.ime_cursor_area = Some(rect);
        true
    }

    pub fn set_ime_surrounding_text(
        &mut self,
        surrounding: Option<ImeSurroundingTextUpdate>,
    ) -> bool {
        if self.ime_surrounding_text == surrounding {
            return false;
        }
        self.ime_surrounding_text = surrounding.clone();
        self.pending.ime_surrounding_text = surrounding;
        true
    }

    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.ime_cursor_area
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) -> bool {
        if self.cursor_icon == icon {
            return false;
        }
        self.cursor_icon = icon;
        self.pending.cursor_icon = Some(icon);
        true
    }

    pub fn prepare_frame(&mut self, window: &dyn Window) {
        let scale_factor = self.begin_prepare_frame(window.scale_factor());

        if let Some(enabled) = self.pending.ime_allowed.take() {
            if enabled {
                let rect = self.ime_cursor_area.unwrap_or_else(|| Rect {
                    origin: fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                    size: fret_core::Size::new(fret_core::Px(1.0), fret_core::Px(1.0)),
                });

                #[cfg(windows)]
                let (cursor_area_x, cursor_area_y, cursor_area_w, cursor_area_h) =
                    crate::windows_ime::winit_cursor_area_from_rect(rect);

                #[cfg(not(windows))]
                let (cursor_area_x, cursor_area_y, cursor_area_w, cursor_area_h) = {
                    (
                        rect.origin.x.0,
                        rect.origin.y.0,
                        rect.size.width.0.max(1.0),
                        rect.size.height.0.max(1.0),
                    )
                };

                let request_data = winit::window::ImeRequestData::default().with_cursor_area(
                    winit::dpi::LogicalPosition::new(cursor_area_x, cursor_area_y).into(),
                    winit::dpi::LogicalSize::new(cursor_area_w, cursor_area_h).into(),
                );

                let mut request_data = request_data;
                let surrounding = self
                    .ime_surrounding_text
                    .clone()
                    .and_then(|u| {
                        winit::window::ImeSurroundingText::new(u.text, u.cursor, u.anchor).ok()
                    })
                    .unwrap_or_else(|| {
                        winit::window::ImeSurroundingText::new(String::new(), 0, 0)
                            .expect("empty surrounding text is always valid")
                    });
                request_data = request_data.with_surrounding_text(surrounding);

                let caps = winit::window::ImeCapabilities::new()
                    .with_cursor_area()
                    .with_surrounding_text();
                if let Some(enable) = winit::window::ImeEnableRequest::new(caps, request_data) {
                    let _ = window.request_ime_update(winit::window::ImeRequest::Enable(enable));
                    self.ime_cursor_area_dispatched_px =
                        Some(quantize_ime_cursor_area_px(rect, scale_factor));
                }
                // The enable request carries the initial cursor area + surrounding text. Avoid
                // immediately re-sending them via `ImeRequest::Update` in the same frame.
                self.pending.ime_cursor_area = None;
                self.pending.ime_surrounding_text = None;
            } else {
                let _ = window.request_ime_update(winit::window::ImeRequest::Disable);
                self.reset_ime_cursor_area_dispatch();
                self.ime_surrounding_text = None;
                self.pending.ime_surrounding_text = None;
            }
        }

        let pending_cursor_area = self.pending.ime_cursor_area.take();
        let pending_surrounding = self.pending.ime_surrounding_text.take();
        let mut needs_winit_update = false;
        let mut request_data = winit::window::ImeRequestData::default();
        if let Some(rect) = pending_cursor_area
            && self.ime_allowed
            && self.should_dispatch_ime_cursor_area(rect, scale_factor)
        {
            #[cfg(windows)]
            {
                crate::windows_ime::set_ime_cursor_area(window, rect);
            }

            #[cfg(not(windows))]
            {
                request_data = request_data.with_cursor_area(
                    winit::dpi::LogicalPosition::new(rect.origin.x.0, rect.origin.y.0).into(),
                    winit::dpi::LogicalSize::new(
                        rect.size.width.0.max(1.0),
                        rect.size.height.0.max(1.0),
                    )
                    .into(),
                );
                needs_winit_update = true;
            }
        }

        if let Some(update) = pending_surrounding
            && self.ime_allowed
            && let Ok(surrounding) =
                winit::window::ImeSurroundingText::new(update.text, update.cursor, update.anchor)
        {
            request_data = request_data.with_surrounding_text(surrounding);
            needs_winit_update = true;
        }

        if needs_winit_update {
            let _ = window.request_ime_update(winit::window::ImeRequest::Update(request_data));
        }

        if let Some(icon) = self.pending.cursor_icon.take() {
            window.set_cursor(winit::cursor::Cursor::Icon(map_cursor_icon(icon)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Size};

    #[test]
    fn ime_cursor_area_quantization_clamps_min_size() {
        let rect = Rect {
            origin: Point::new(Px(10.25), Px(20.25)),
            size: Size::new(Px(0.0), Px(0.4)),
        };
        let quantized = quantize_ime_cursor_area_px(rect, 1.5);
        assert_eq!(
            quantized,
            ImeCursorAreaPx {
                x: 15,
                y: 30,
                width: 1,
                height: 1,
            }
        );
    }

    #[test]
    fn ime_cursor_area_dispatch_is_deduplicated_by_quantized_px() {
        let mut state = WinitWindowState::default();
        let a = Rect {
            origin: Point::new(Px(10.10), Px(20.10)),
            size: Size::new(Px(1.0), Px(1.0)),
        };
        let b = Rect {
            origin: Point::new(Px(10.20), Px(20.20)),
            size: Size::new(Px(1.0), Px(1.0)),
        };

        assert!(state.should_dispatch_ime_cursor_area(a, 2.0));
        assert!(!state.should_dispatch_ime_cursor_area(b, 2.0));

        let c = Rect {
            origin: Point::new(Px(10.80), Px(20.80)),
            size: Size::new(Px(1.0), Px(1.0)),
        };
        assert!(state.should_dispatch_ime_cursor_area(c, 2.0));
    }

    #[test]
    fn ime_cursor_area_dispatch_reset_allows_same_rect_again() {
        let mut state = WinitWindowState::default();
        let rect = Rect {
            origin: Point::new(Px(3.0), Px(4.0)),
            size: Size::new(Px(1.0), Px(1.0)),
        };

        assert!(state.should_dispatch_ime_cursor_area(rect, 1.0));
        assert!(!state.should_dispatch_ime_cursor_area(rect, 1.0));
        state.reset_ime_cursor_area_dispatch();
        assert!(state.should_dispatch_ime_cursor_area(rect, 1.0));
    }

    #[test]
    fn begin_prepare_frame_requeues_cursor_area_after_scale_change() {
        let mut state = WinitWindowState::default();
        let rect = Rect {
            origin: Point::new(Px(3.0), Px(4.0)),
            size: Size::new(Px(1.0), Px(1.0)),
        };

        assert!(state.set_ime_allowed(true));
        assert!(state.set_ime_cursor_area(rect));

        let first_scale = state.begin_prepare_frame(1.0);
        assert_eq!(first_scale, 1.0);
        state.pending.ime_cursor_area = None;
        assert!(state.should_dispatch_ime_cursor_area(rect, first_scale));
        assert!(!state.should_dispatch_ime_cursor_area(rect, first_scale));

        let second_scale = state.begin_prepare_frame(1.25);
        assert_eq!(second_scale, 1.25);
        assert_eq!(state.pending.ime_cursor_area, Some(rect));
        assert!(state.should_dispatch_ime_cursor_area(rect, second_scale));
    }

    #[test]
    fn begin_prepare_frame_skips_requeue_when_ime_disabled() {
        let mut state = WinitWindowState::default();
        let rect = Rect {
            origin: Point::new(Px(8.0), Px(9.0)),
            size: Size::new(Px(1.0), Px(1.0)),
        };

        assert!(state.set_ime_cursor_area(rect));
        state.pending.ime_cursor_area = None;

        let _ = state.begin_prepare_frame(1.0);
        assert_eq!(state.pending.ime_cursor_area, None);

        let _ = state.begin_prepare_frame(1.5);
        assert_eq!(state.pending.ime_cursor_area, None);
    }
}
