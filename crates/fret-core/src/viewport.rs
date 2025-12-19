use crate::geometry::{Point, Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportFit {
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMapping {
    pub content_rect: Rect,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMapped {
    pub draw_rect: Rect,
}

impl ViewportMapping {
    pub fn map(self) -> ViewportMapped {
        let (tw, th) = self.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let cw = self.content_rect.size.width.0.max(0.0);
        let ch = self.content_rect.size.height.0.max(0.0);
        if cw <= 0.0 || ch <= 0.0 {
            return ViewportMapped {
                draw_rect: Rect::new(self.content_rect.origin, Size::new(Px(0.0), Px(0.0))),
            };
        }

        match self.fit {
            ViewportFit::Stretch => ViewportMapped {
                draw_rect: self.content_rect,
            },
            ViewportFit::Contain | ViewportFit::Cover => {
                let sx = cw / tw;
                let sy = ch / th;
                let s = match self.fit {
                    ViewportFit::Contain => sx.min(sy),
                    ViewportFit::Cover => sx.max(sy),
                    ViewportFit::Stretch => unreachable!(),
                };

                let dw = tw * s;
                let dh = th * s;
                let x = self.content_rect.origin.x.0 + (cw - dw) * 0.5;
                let y = self.content_rect.origin.y.0 + (ch - dh) * 0.5;

                ViewportMapped {
                    draw_rect: Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(dw), Px(dh))),
                }
            }
        }
    }

    pub fn window_point_to_uv(self, p: Point) -> Option<(f32, f32)> {
        let mapped = self.map();
        if !mapped.draw_rect.contains(p) {
            return None;
        }

        let x = (p.x.0 - mapped.draw_rect.origin.x.0) / mapped.draw_rect.size.width.0.max(1.0);
        let y = (p.y.0 - mapped.draw_rect.origin.y.0) / mapped.draw_rect.size.height.0.max(1.0);
        Some((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)))
    }

    pub fn window_point_to_uv_clamped(self, p: Point) -> (f32, f32) {
        let mapped = self.map();
        let x = (p.x.0 - mapped.draw_rect.origin.x.0) / mapped.draw_rect.size.width.0.max(1.0);
        let y = (p.y.0 - mapped.draw_rect.origin.y.0) / mapped.draw_rect.size.height.0.max(1.0);
        (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
    }

    pub fn window_point_to_target_px(self, p: Point) -> Option<(u32, u32)> {
        let (u, v) = self.window_point_to_uv(p)?;
        let (tw, th) = self.target_px_size;
        let x = (u * tw as f32)
            .floor()
            .clamp(0.0, (tw.saturating_sub(1)) as f32) as u32;
        let y = (v * th as f32)
            .floor()
            .clamp(0.0, (th.saturating_sub(1)) as f32) as u32;
        Some((x, y))
    }

    pub fn window_point_to_target_px_clamped(self, p: Point) -> (u32, u32) {
        let (u, v) = self.window_point_to_uv_clamped(p);
        let (tw, th) = self.target_px_size;
        let x = (u * tw as f32)
            .floor()
            .clamp(0.0, (tw.saturating_sub(1)) as f32) as u32;
        let y = (v * th as f32)
            .floor()
            .clamp(0.0, (th.saturating_sub(1)) as f32) as u32;
        (x, y)
    }
}
