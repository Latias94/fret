use std::collections::BTreeMap;

use delinea::engine::EngineError;
use delinea::engine::model::{ChartPatch, ModelError, PatchMode};
use delinea::marks::{MarkKind, MarkPayloadRef};
use delinea::text::{TextMeasurer, TextMetrics};
use delinea::{ChartEngine, WorkBudget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, PathCommand, PathConstraints, PathStyle, Px, Rect, SceneOp,
    StrokeStyle,
};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{LayoutCx, PaintCx, Widget};

use crate::retained::style::ChartStyle;

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl TextMeasurer for NullTextMeasurer {
    fn measure(
        &mut self,
        _text: delinea::ids::StringId,
        _style: delinea::text::TextStyleId,
    ) -> TextMetrics {
        TextMetrics::default()
    }
}

#[derive(Debug, Default)]
struct CachedPath {
    path: fret_core::PathId,
}

pub struct ChartCanvas {
    engine: ChartEngine,
    style: ChartStyle,
    last_bounds: Rect,
    last_marks_rev: delinea::ids::Revision,
    last_scale_factor_bits: u32,
    cached_paths: BTreeMap<delinea::ids::MarkId, CachedPath>,
}

impl ChartCanvas {
    pub fn new(spec: delinea::ChartSpec) -> Result<Self, ModelError> {
        Ok(Self {
            engine: ChartEngine::new(spec)?,
            style: ChartStyle::default(),
            last_bounds: Rect::default(),
            last_marks_rev: delinea::ids::Revision::default(),
            last_scale_factor_bits: 0,
            cached_paths: BTreeMap::default(),
        })
    }

    pub fn engine(&self) -> &ChartEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut ChartEngine {
        &mut self.engine
    }

    pub fn set_style(&mut self, style: ChartStyle) {
        self.style = style;
    }

    fn sync_viewport(&mut self, bounds: Rect) {
        if self.engine.model().viewport == Some(bounds) {
            return;
        }
        let _ = self.engine.apply_patch(
            ChartPatch {
                viewport: Some(Some(bounds)),
                ..ChartPatch::default()
            },
            PatchMode::Merge,
        );
    }

    fn rebuild_paths_if_needed<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let marks_rev = self.engine.output().marks.revision;
        let scale_factor_bits = cx.scale_factor.to_bits();

        if marks_rev == self.last_marks_rev && scale_factor_bits == self.last_scale_factor_bits {
            return;
        }
        self.last_marks_rev = marks_rev;
        self.last_scale_factor_bits = scale_factor_bits;

        for cached in self.cached_paths.values() {
            cx.services.path().release(cached.path);
        }
        self.cached_paths.clear();

        let marks = &self.engine.output().marks;
        let origin = self.last_bounds.origin;

        for node in &marks.nodes {
            if node.kind != MarkKind::Polyline {
                continue;
            }

            let MarkPayloadRef::Polyline(poly) = &node.payload else {
                continue;
            };

            let start = poly.points.start;
            let end = poly.points.end;
            if end <= start || end > marks.arena.points.len() {
                continue;
            }

            let mut commands: Vec<PathCommand> =
                Vec::with_capacity((end - start).saturating_add(1));
            for (i, p) in marks.arena.points[start..end].iter().enumerate() {
                let local = fret_core::Point::new(Px(p.x.0 - origin.x.0), Px(p.y.0 - origin.y.0));
                if i == 0 {
                    commands.push(PathCommand::MoveTo(local));
                } else {
                    commands.push(PathCommand::LineTo(local));
                }
            }

            if commands.len() < 2 {
                continue;
            }

            let stroke_width = poly
                .stroke
                .as_ref()
                .map(|(_, s)| s.width)
                .unwrap_or(self.style.stroke_width);

            let (path, _metrics) = cx.services.path().prepare(
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                PathConstraints {
                    scale_factor: cx.scale_factor,
                },
            );

            let mark_id = node.id;
            self.cached_paths.insert(mark_id, CachedPath { path });
        }
    }
}

impl<H: UiHost> Widget<H> for ChartCanvas {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        self.last_bounds = cx.bounds;
        self.sync_viewport(cx.bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        self.sync_viewport(cx.bounds);

        // P0: run the engine synchronously for now.
        let mut measurer = NullTextMeasurer::default();
        let _ = self
            .engine
            .step(&mut measurer, WorkBudget::new(u32::MAX, 0, u32::MAX))
            .map_err(|_e: EngineError| ());

        self.rebuild_paths_if_needed(cx);

        if let Some(background) = self.style.background {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(self.style.draw_order.0.saturating_sub(1)),
                rect: self.last_bounds,
                background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        cx.scene.push(SceneOp::PushClipRect {
            rect: self.last_bounds,
        });

        for cached in self.cached_paths.values() {
            cx.scene.push(SceneOp::Path {
                order: self.style.draw_order,
                origin: self.last_bounds.origin,
                path: cached.path,
                color: self.style.stroke_color,
            });
        }

        cx.scene.push(SceneOp::PopClip);
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for cached in self.cached_paths.values() {
            services.path().release(cached.path);
        }
        self.cached_paths.clear();
    }
}
