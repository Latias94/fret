//! Gradient preview canvas paint owner.

use std::sync::{Arc, Mutex};

use fret_core::scene::{ColorSpace, GradientStop, LinearGradient, MAX_STOPS, Paint, TileMode};
use fret_core::{Color, Corners, Edges, Point, Px, Rect};
use fret_ui::canvas::OnCanvasPaint;

use super::{GradientPreviewState, PreviewStop};

pub(super) struct GradientPreviewPaintInput {
    pub(super) angle_deg: f64,
    pub(super) stops: Vec<PreviewStop>,
    pub(super) active_stop: Option<fret_ui::ItemKey>,
    pub(super) preview_state: Arc<Mutex<GradientPreviewState>>,
}

pub(super) fn gradient_preview_paint(input: GradientPreviewPaintInput) -> OnCanvasPaint {
    let GradientPreviewPaintInput {
        angle_deg,
        stops,
        active_stop,
        preview_state,
    } = input;

    Arc::new(move |p| {
        let bounds = p.bounds();
        let rect = Rect {
            origin: bounds.origin,
            size: bounds.size,
        };

        let muted = p.theme().color_token("muted");
        let border = p.theme().color_token("border");
        let accent = p.theme().color_token("accent");

        let angle = (angle_deg as f32).to_radians();
        let dx = angle.cos();
        let dy = angle.sin();

        let len = (rect.size.width.0.powi(2) + rect.size.height.0.powi(2))
            .sqrt()
            .max(1.0);
        let half = len * 0.5;
        let cx0 = rect.origin.x.0 + rect.size.width.0 * 0.5;
        let cy0 = rect.origin.y.0 + rect.size.height.0 * 0.5;
        let start = Point::new(Px(cx0 - dx * half), Px(cy0 - dy * half));
        let end = Point::new(Px(cx0 + dx * half), Px(cy0 + dy * half));

        let mut fixed = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
        let mut n: u8 = 0;
        for (i, s) in stops.iter().take(MAX_STOPS).enumerate() {
            fixed[i] = GradientStop::new(s.position.clamp(0.0, 1.0), s.color);
            n = (i as u8) + 1;
        }
        if n == 0 {
            fixed[0] = GradientStop::new(0.0, muted);
            fixed[1] = GradientStop::new(1.0, muted);
            n = 2;
        }

        let gradient = LinearGradient {
            start,
            end,
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count: n,
            stops: fixed,
        };

        p.scene().push(fret_core::SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect,
            background: Paint::LinearGradient(gradient).into(),
            border: Edges::all(Px(1.0)),
            border_paint: Paint::Solid(border).into(),
            corner_radii: Corners::all(Px(6.0)),
        });

        let w = rect.size.width.0.max(1.0);
        let h = rect.size.height.0.max(1.0);

        let marker_d = (h * 0.55).min(12.0).max(6.0);
        let marker_y = rect.origin.y.0 + h - marker_d * 0.5 - 1.0;
        let marker_radius = Px(marker_d * 0.5);

        let active = preview_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active_stop
            .or(active_stop);

        for s in stops.iter() {
            let x = rect.origin.x.0 + w * s.position.clamp(0.0, 1.0);
            let marker_rect = Rect {
                origin: Point::new(Px(x - marker_d * 0.5), Px(marker_y - marker_d * 0.5)),
                size: fret_core::Size::new(Px(marker_d), Px(marker_d)),
            };

            let outline = if Some(s.id) == active {
                Paint::Solid(accent)
            } else {
                Paint::Solid(border)
            };
            let stroke_w = if Some(s.id) == active {
                Px(2.0)
            } else {
                Px(1.0)
            };

            p.scene().push(fret_core::SceneOp::Quad {
                order: fret_core::DrawOrder(1),
                rect: marker_rect,
                background: Paint::Solid(s.color).into(),
                border: Edges::all(stroke_w),
                border_paint: outline.into(),
                corner_radii: Corners::all(marker_radius),
            });
        }
    })
}
