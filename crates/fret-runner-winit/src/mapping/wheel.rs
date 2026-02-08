use fret_core::{Point, Px};
use winit::dpi::LogicalPosition;
use winit::event::MouseScrollDelta;

#[derive(Debug, Clone, Copy)]
pub struct WheelConfig {
    pub line_delta_px: f32,
    pub pixel_delta_scale: f32,
}

impl Default for WheelConfig {
    fn default() -> Self {
        Self {
            line_delta_px: 16.0,
            pixel_delta_scale: 1.0,
        }
    }
}

pub fn map_wheel_delta(delta: MouseScrollDelta, scale_factor: f64, config: WheelConfig) -> Point {
    // `fret-core` wheel delta follows winit semantics: positive y means wheel up.
    match delta {
        MouseScrollDelta::LineDelta(dx, dy) => {
            Point::new(Px(dx * config.line_delta_px), Px(dy * config.line_delta_px))
        }
        MouseScrollDelta::PixelDelta(physical) => {
            let logical: LogicalPosition<f32> = physical.to_logical(scale_factor);
            Point::new(
                Px(logical.x * config.pixel_delta_scale),
                Px(logical.y * config.pixel_delta_scale),
            )
        }
    }
}
