use fret_core::time::Instant;
use std::collections::BTreeMap;
use std::ops::Range;
use std::time::Duration;

use delinea::FilterMode;
use delinea::engine::EngineError;
use delinea::engine::model::{ChartPatch, ModelError, PatchMode};
use delinea::engine::window::{DataWindow, WindowSpanAnchor};
use delinea::marks::{MarkKind, MarkPayloadRef};
use delinea::text::{TextMeasurer, TextMetrics};
use delinea::{Action, BrushSelection2D, ChartEngine, WorkBudget};
use fret_canvas::cache::{PathCache, SceneOpCache};
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};
use fret_canvas::scale::effective_scale_factor;
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, FontWeight, KeyCode, Modifiers, MouseButton,
    PathCommand, PathConstraints, PathStyle, Point, PointerEvent, PointerType, Px, Rect, SceneOp,
    Size, StrokeStyle, TextBlobId, TextConstraints, TextOverflow, TextStyle, TextWrap, Transform2D,
};
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::UiHost;
use fret_ui::retained_bridge::{EventCx, Invalidation, LayoutCx, PaintCx, PrepaintCx, Widget};
use slotmap::Key;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::input_map::{ChartInputMap, ModifierKey, ModifiersMask};
use crate::retained::style::ChartStyle;
use crate::retained::text_cache::{KeyBuilder, TextCacheGroup};
use crate::retained::tooltip::{DefaultTooltipFormatter, TooltipFormatter};
use crate::retained::{ChartCanvasOutput, ChartCanvasOutputSnapshot};

fn mark_path_cache_key(mark_id: delinea::ids::MarkId, variant: u8) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    mark_id.hash(&mut hasher);
    variant.hash(&mut hasher);
    hasher.finish()
}

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

#[derive(Debug)]
struct CachedPath {
    fill_alpha: Option<f32>,
    order: u32,
    source_series: Option<delinea::SeriesId>,
}

#[derive(Debug, Clone, Copy)]
struct CachedRect {
    rect: Rect,
    order: u32,
    source_series: Option<delinea::SeriesId>,
    fill: Option<delinea::PaintId>,
    opacity_mul: f32,
    stroke_width: Option<Px>,
}

#[derive(Debug, Clone, Copy)]
struct CachedPoint {
    point: Point,
    order: u32,
    source_series: Option<delinea::SeriesId>,
    fill: Option<delinea::PaintId>,
    opacity_mul: f32,
    radius_mul: f32,
    stroke_width: Option<Px>,
}

#[derive(Debug, Clone, Copy)]
struct PanDrag {
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    pan_x: bool,
    pan_y: bool,
    start_pos: Point,
    start_x: DataWindow,
    start_y: DataWindow,
}

#[derive(Debug, Clone, Copy)]
struct BoxZoomDrag {
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    button: MouseButton,
    required_mods: ModifiersMask,
    start_pos: Point,
    current_pos: Point,
    start_x: DataWindow,
    start_y: DataWindow,
}

#[derive(Debug, Clone, Copy)]
enum SliderDragKind {
    Pan,
    HandleMin,
    HandleMax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliderAxisKind {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
struct DataZoomSliderDrag {
    axis_kind: SliderAxisKind,
    axis: delinea::AxisId,
    kind: SliderDragKind,
    track: Rect,
    extent: DataWindow,
    start_pos: Point,
    start_window: DataWindow,
}

#[derive(Debug, Clone, Copy)]
struct VisualMapDrag {
    visual_map: delinea::VisualMapId,
    kind: SliderDragKind,
    track: Rect,
    domain: DataWindow,
    start_window: DataWindow,
    start_value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisRegion {
    Plot,
    XAxis(delinea::AxisId),
    YAxis(delinea::AxisId),
}

#[derive(Debug, Clone, Copy)]
struct AxisBandLayout {
    axis: delinea::AxisId,
    position: delinea::AxisPosition,
    rect: Rect,
}

#[derive(Debug, Default, Clone)]
struct ChartLayout {
    bounds: Rect,
    plot: Rect,
    x_axes: Vec<AxisBandLayout>,
    y_axes: Vec<AxisBandLayout>,
    visual_map: Option<Rect>,
}

pub struct ChartCanvas {
    engine: ChartCanvasEngine,
    grid_override: Option<delinea::GridId>,
    style: ChartStyle,
    style_source: ChartStyleSource,
    last_theme_revision: u64,
    force_uncached_paint: bool,
    last_sampling_window_key: u64,
    text_cache_prune: ChartTextCachePruneTuning,
    tooltip_formatter: Box<dyn TooltipFormatter>,
    input_map: ChartInputMap,
    last_bounds: Rect,
    last_layout: ChartLayout,
    last_pointer_pos: Option<Point>,
    active_x_axis: Option<delinea::AxisId>,
    active_y_axis: Option<delinea::AxisId>,
    last_marks_rev: delinea::ids::Revision,
    last_scale_factor_bits: u32,
    path_cache: PathCache,
    cached_paths: BTreeMap<delinea::ids::MarkId, CachedPath>,
    cached_rects: Vec<CachedRect>,
    cached_points: Vec<CachedPoint>,
    cached_rect_scene_ops: SceneOpCache<u64>,
    cached_point_scene_ops: SceneOpCache<u64>,
    series_rank_by_id: BTreeMap<delinea::SeriesId, usize>,
    axis_text: TextCacheGroup,
    tooltip_text: TextCacheGroup,
    legend_text: TextCacheGroup,
    legend_item_rects: Vec<(delinea::SeriesId, Rect)>,
    legend_selector_rects: Vec<(LegendSelectorAction, Rect)>,
    legend_panel_rect: Option<Rect>,
    legend_hover: Option<delinea::SeriesId>,
    legend_anchor: Option<delinea::SeriesId>,
    legend_selector_hover: Option<LegendSelectorAction>,
    legend_scroll_y: Px,
    legend_content_height: Px,
    legend_view_height: Px,
    pan_drag: Option<PanDrag>,
    box_zoom_drag: Option<BoxZoomDrag>,
    brush_drag: Option<BoxZoomDrag>,
    slider_drag: Option<DataZoomSliderDrag>,
    visual_map_drag: Option<VisualMapDrag>,
    visual_map_piece_anchor: Option<(delinea::VisualMapId, u32)>,
    axis_extent_cache: BTreeMap<delinea::AxisId, AxisExtentCacheEntry>,
    linked_brush_model: Option<Model<Option<BrushSelection2D>>>,
    output_model: Option<Model<ChartCanvasOutput>>,
    output: ChartCanvasOutput,
}

type SharedChartEngine = Rc<RefCell<ChartEngine>>;

enum ChartCanvasEngine {
    Owned(ChartEngine),
    Shared(SharedChartEngine),
}

enum ChartEngineReadGuard<'a> {
    Owned(&'a ChartEngine),
    Shared(Ref<'a, ChartEngine>),
}

impl core::ops::Deref for ChartEngineReadGuard<'_> {
    type Target = ChartEngine;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(engine) => engine,
            Self::Shared(engine) => engine,
        }
    }
}

enum ChartEngineWriteGuard<'a> {
    Owned(&'a mut ChartEngine),
    Shared(RefMut<'a, ChartEngine>),
}

impl core::ops::Deref for ChartEngineWriteGuard<'_> {
    type Target = ChartEngine;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(engine) => engine,
            Self::Shared(engine) => engine,
        }
    }
}

impl core::ops::DerefMut for ChartEngineWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(engine) => engine,
            Self::Shared(engine) => engine,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChartTextCachePruneTuning {
    pub max_age_frames: u64,
    pub max_entries: usize,
}

impl Default for ChartTextCachePruneTuning {
    fn default() -> Self {
        Self {
            max_age_frames: 1_200,
            max_entries: 4_096,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartStyleSource {
    Theme,
    Fixed,
}

#[derive(Debug, Clone, Copy)]
struct AxisExtentCacheEntry {
    spec_rev: delinea::ids::Revision,
    visual_rev: delinea::ids::Revision,
    data_sig: u64,
    window: DataWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegendSelectorAction {
    All,
    None,
    Invert,
}

impl ChartCanvas {
    pub fn new(spec: delinea::ChartSpec) -> Result<Self, ModelError> {
        let mut spec = spec;
        spec.axis_pointer.get_or_insert_with(Default::default);
        Ok(Self::new_with_engine(ChartCanvasEngine::Owned(
            ChartEngine::new(spec)?,
        )))
    }

    pub fn new_grid_view(engine: SharedChartEngine, grid: delinea::GridId) -> Self {
        let mut out = Self::new_with_engine(ChartCanvasEngine::Shared(engine));
        out.grid_override = Some(grid);
        out
    }

    fn new_with_engine(engine: ChartCanvasEngine) -> Self {
        Self {
            engine,
            grid_override: None,
            style: ChartStyle::default(),
            style_source: ChartStyleSource::Theme,
            last_theme_revision: 0,
            force_uncached_paint: true,
            last_sampling_window_key: 0,
            text_cache_prune: ChartTextCachePruneTuning::default(),
            tooltip_formatter: Box::new(DefaultTooltipFormatter::default()),
            input_map: ChartInputMap::default(),
            last_bounds: Rect::default(),
            last_layout: ChartLayout::default(),
            last_pointer_pos: None,
            active_x_axis: None,
            active_y_axis: None,
            last_marks_rev: delinea::ids::Revision::default(),
            last_scale_factor_bits: 0,
            path_cache: PathCache::default(),
            cached_paths: BTreeMap::default(),
            cached_rects: Vec::default(),
            cached_points: Vec::default(),
            cached_rect_scene_ops: SceneOpCache::default(),
            cached_point_scene_ops: SceneOpCache::default(),
            series_rank_by_id: BTreeMap::default(),
            axis_text: TextCacheGroup::default(),
            tooltip_text: TextCacheGroup::default(),
            legend_text: TextCacheGroup::default(),
            legend_item_rects: Vec::default(),
            legend_selector_rects: Vec::default(),
            legend_panel_rect: None,
            legend_hover: None,
            legend_anchor: None,
            legend_selector_hover: None,
            legend_scroll_y: Px(0.0),
            legend_content_height: Px(0.0),
            legend_view_height: Px(0.0),
            pan_drag: None,
            box_zoom_drag: None,
            brush_drag: None,
            slider_drag: None,
            visual_map_drag: None,
            visual_map_piece_anchor: None,
            axis_extent_cache: BTreeMap::default(),
            linked_brush_model: None,
            output_model: None,
            output: ChartCanvasOutput::default(),
        }
    }

    fn sampling_window_key(&self, plot: Rect, scale_factor: f32) -> u64 {
        self.with_engine(|engine| {
            let output = engine.output();
            let mut key = KeyBuilder::new();

            key.mix_f32_bits(plot.size.width.0);
            key.mix_f32_bits(plot.size.height.0);
            key.mix_f32_bits(scale_factor);

            key.mix_u64(output.axis_windows.len() as u64);
            for (axis, window) in &output.axis_windows {
                key.mix_u64(axis.0 as u64);
                key.mix_f64_bits(window.min);
                key.mix_f64_bits(window.max);
            }

            key.finish()
        })
    }

    pub fn set_text_cache_prune_tuning(&mut self, tuning: ChartTextCachePruneTuning) {
        self.text_cache_prune = tuning;
    }

    fn with_engine<R>(&self, f: impl FnOnce(&ChartEngine) -> R) -> R {
        let engine = self.engine_read();
        f(&engine)
    }

    fn with_engine_mut<R>(&mut self, f: impl FnOnce(&mut ChartEngine) -> R) -> R {
        let mut engine = self.engine_write();
        f(&mut engine)
    }

    fn engine_read(&self) -> ChartEngineReadGuard<'_> {
        match &self.engine {
            ChartCanvasEngine::Owned(engine) => ChartEngineReadGuard::Owned(engine),
            ChartCanvasEngine::Shared(engine) => ChartEngineReadGuard::Shared(engine.borrow()),
        }
    }

    fn engine_write(&mut self) -> ChartEngineWriteGuard<'_> {
        match &mut self.engine {
            ChartCanvasEngine::Owned(engine) => ChartEngineWriteGuard::Owned(engine),
            ChartCanvasEngine::Shared(engine) => ChartEngineWriteGuard::Shared(engine.borrow_mut()),
        }
    }

    pub fn engine(&self) -> &ChartEngine {
        match &self.engine {
            ChartCanvasEngine::Owned(engine) => engine,
            ChartCanvasEngine::Shared(_) => {
                panic!("ChartCanvas::engine is not available for shared-engine grid views")
            }
        }
    }

    pub fn engine_mut(&mut self) -> &mut ChartEngine {
        match &mut self.engine {
            ChartCanvasEngine::Owned(engine) => engine,
            ChartCanvasEngine::Shared(_) => {
                panic!("ChartCanvas::engine_mut is not available for shared-engine grid views")
            }
        }
    }

    pub fn set_style(&mut self, style: ChartStyle) {
        self.style = style;
        self.style_source = ChartStyleSource::Fixed;
    }

    pub fn set_style_source(&mut self, source: ChartStyleSource) {
        self.style_source = source;
    }

    pub fn set_tooltip_formatter(&mut self, formatter: Box<dyn TooltipFormatter>) {
        self.tooltip_formatter = formatter;
    }

    pub fn set_input_map(&mut self, map: ChartInputMap) {
        self.input_map = map;
    }

    pub fn linked_brush(mut self, brush: Model<Option<BrushSelection2D>>) -> Self {
        self.linked_brush_model = Some(brush);
        self
    }

    pub fn output_model(mut self, output: Model<ChartCanvasOutput>) -> Self {
        self.output_model = Some(output);
        self
    }

    fn sync_linked_brush<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(model) = &self.linked_brush_model else {
            return;
        };
        cx.observe_model(model, Invalidation::Paint);

        let Ok(selection) = model.read(cx.app, |_, s| *s) else {
            return;
        };
        let current = self.with_engine(|engine| engine.state().brush_selection_2d);
        if selection == current {
            return;
        }

        match selection {
            Some(sel) => {
                self.with_engine_mut(|engine| {
                    engine.apply_action(Action::SetBrushSelection2D {
                        x_axis: sel.x_axis,
                        y_axis: sel.y_axis,
                        x: sel.x,
                        y: sel.y,
                    });
                });
            }
            None => {
                self.with_engine_mut(|engine| {
                    engine.apply_action(Action::ClearBrushSelection);
                });
            }
        }
    }

    fn publish_output<H: UiHost>(&mut self, app: &mut H) {
        let link_events = self.with_engine_mut(|engine| engine.drain_link_events());
        let snapshot = ChartCanvasOutputSnapshot {
            brush_selection_2d: self.with_engine(|engine| engine.state().brush_selection_2d),
            brush_x_row_ranges_by_series: self
                .with_engine(|engine| engine.output().brush_x_row_ranges_by_series.clone()),
            link_events,
        };

        if self.output.snapshot == snapshot {
            return;
        }

        self.output.revision = self.output.revision.wrapping_add(1);
        self.output.snapshot = snapshot;

        if let Some(model) = &self.output_model {
            let next = self.output.clone();
            let _ = model.update(app, |s, _cx| {
                *s = next;
            });
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) -> bool {
        if self.style_source != ChartStyleSource::Theme {
            return false;
        }

        let rev = theme.revision();
        if self.last_theme_revision == rev {
            return false;
        }

        self.last_theme_revision = rev;
        self.style = ChartStyle::from_theme(theme);
        true
    }

    fn compute_layout(&self, bounds: Rect) -> ChartLayout {
        self.with_engine(|engine| {
            let model = engine.model();

            let mut inner = bounds;
            inner.origin.x.0 += self.style.padding.left.0;
            inner.origin.y.0 += self.style.padding.top.0;
            inner.size.width.0 =
                (inner.size.width.0 - self.style.padding.left.0 - self.style.padding.right.0)
                    .max(0.0);
            inner.size.height.0 =
                (inner.size.height.0 - self.style.padding.top.0 - self.style.padding.bottom.0)
                    .max(0.0);

            let axis_band_x = self.style.axis_band_x.0.max(0.0);
            let axis_band_y = self.style.axis_band_y.0.max(0.0);

            let active_grid = self.grid_override.or_else(|| {
                let primary = model.series_in_order().find(|s| s.visible)?;
                model.axes.get(&primary.x_axis).map(|a| a.grid)
            });

            let has_visual_map = active_grid.is_some_and(|grid| {
                model.series_in_order().any(|s| {
                    s.visible
                        && model.axes.get(&s.x_axis).is_some_and(|a| a.grid == grid)
                        && model.visual_map_by_series.contains_key(&s.id)
                })
            });
            let visual_map_band_x = if has_visual_map {
                self.style.visual_map_band_x.0.max(0.0)
            } else {
                0.0
            };

            let mut x_top: Vec<delinea::AxisId> = Vec::new();
            let mut x_bottom: Vec<delinea::AxisId> = Vec::new();
            let mut y_left: Vec<delinea::AxisId> = Vec::new();
            let mut y_right: Vec<delinea::AxisId> = Vec::new();

            if let Some(grid) = active_grid {
                for (axis_id, axis) in &model.axes {
                    if axis.grid != grid {
                        continue;
                    }

                    match (axis.kind, axis.position) {
                        (delinea::AxisKind::X, delinea::AxisPosition::Top) => x_top.push(*axis_id),
                        (delinea::AxisKind::X, delinea::AxisPosition::Bottom) => {
                            x_bottom.push(*axis_id)
                        }
                        (delinea::AxisKind::Y, delinea::AxisPosition::Left) => {
                            y_left.push(*axis_id)
                        }
                        (delinea::AxisKind::Y, delinea::AxisPosition::Right) => {
                            y_right.push(*axis_id)
                        }
                        _ => {}
                    }
                }
            }

            let left_total = axis_band_x * (y_left.len() as f32);
            let right_total = axis_band_x * (y_right.len() as f32);
            let top_total = axis_band_y * (x_top.len() as f32);
            let bottom_total = axis_band_y * (x_bottom.len() as f32);

            let plot_w =
                (inner.size.width.0 - left_total - right_total - visual_map_band_x).max(0.0);
            let plot_h = (inner.size.height.0 - top_total - bottom_total).max(0.0);

            let plot = Rect::new(
                Point::new(
                    Px(inner.origin.x.0 + left_total),
                    Px(inner.origin.y.0 + top_total),
                ),
                Size::new(Px(plot_w), Px(plot_h)),
            );

            let mut x_axes: Vec<AxisBandLayout> = Vec::with_capacity(x_top.len() + x_bottom.len());
            for (i, axis) in x_top.iter().copied().enumerate() {
                let rect = Rect::new(
                    Point::new(
                        plot.origin.x,
                        Px(plot.origin.y.0 - axis_band_y * (i as f32 + 1.0)),
                    ),
                    Size::new(plot.size.width, Px(axis_band_y)),
                );
                x_axes.push(AxisBandLayout {
                    axis,
                    position: delinea::AxisPosition::Top,
                    rect,
                });
            }
            for (i, axis) in x_bottom.iter().copied().enumerate() {
                let rect = Rect::new(
                    Point::new(
                        plot.origin.x,
                        Px(plot.origin.y.0 + plot.size.height.0 + axis_band_y * (i as f32)),
                    ),
                    Size::new(plot.size.width, Px(axis_band_y)),
                );
                x_axes.push(AxisBandLayout {
                    axis,
                    position: delinea::AxisPosition::Bottom,
                    rect,
                });
            }

            let mut y_axes: Vec<AxisBandLayout> = Vec::with_capacity(y_left.len() + y_right.len());
            for (i, axis) in y_left.iter().copied().enumerate() {
                let rect = Rect::new(
                    Point::new(
                        Px(plot.origin.x.0 - axis_band_x * (i as f32 + 1.0)),
                        plot.origin.y,
                    ),
                    Size::new(Px(axis_band_x), plot.size.height),
                );
                y_axes.push(AxisBandLayout {
                    axis,
                    position: delinea::AxisPosition::Left,
                    rect,
                });
            }
            for (i, axis) in y_right.iter().copied().enumerate() {
                let rect = Rect::new(
                    Point::new(
                        Px(plot.origin.x.0 + plot.size.width.0 + axis_band_x * (i as f32)),
                        plot.origin.y,
                    ),
                    Size::new(Px(axis_band_x), plot.size.height),
                );
                y_axes.push(AxisBandLayout {
                    axis,
                    position: delinea::AxisPosition::Right,
                    rect,
                });
            }

            let visual_map = (visual_map_band_x > 0.0).then(|| {
                let x0 = plot.origin.x.0 + plot.size.width.0 + axis_band_x * (y_right.len() as f32);
                Rect::new(
                    Point::new(Px(x0), plot.origin.y),
                    Size::new(Px(visual_map_band_x), plot.size.height),
                )
            });

            ChartLayout {
                bounds,
                plot,
                x_axes,
                y_axes,
                visual_map,
            }
        })
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;
        ui.create_node_retained(canvas)
    }

    fn sync_viewport(&mut self, viewport: Rect) {
        if let Some(grid) = self.grid_override {
            let already = self.with_engine(|engine| {
                engine.model().plot_viewports_by_grid.get(&grid).copied() == Some(viewport)
            });
            if already {
                return;
            }

            let mut patch = ChartPatch::default();
            patch.plot_viewports_by_grid.insert(grid, Some(viewport));
            let _ = self.with_engine_mut(|engine| engine.apply_patch(patch, PatchMode::Merge));
            return;
        }

        let already = self.with_engine(|engine| engine.model().viewport == Some(viewport));
        if already {
            return;
        }
        let _ = self.with_engine_mut(|engine| {
            engine.apply_patch(
                ChartPatch {
                    viewport: Some(Some(viewport)),
                    ..ChartPatch::default()
                },
                PatchMode::Merge,
            )
        });
    }

    fn primary_axes(&self) -> Option<(delinea::AxisId, delinea::AxisId)> {
        self.with_engine(|engine| {
            let model = engine.model();
            let primary = model.series_in_order().find(|s| {
                s.visible
                    && self.grid_override.is_none_or(|grid| {
                        model.axes.get(&s.x_axis).is_some_and(|a| a.grid == grid)
                    })
            })?;
            Some((primary.x_axis, primary.y_axis))
        })
    }

    fn update_active_axes_for_position(&mut self, layout: &ChartLayout, position: Point) {
        match Self::axis_region(layout, position) {
            AxisRegion::XAxis(axis) => {
                self.active_x_axis = Some(axis);
            }
            AxisRegion::YAxis(axis) => {
                self.active_y_axis = Some(axis);
            }
            AxisRegion::Plot => {}
        }
    }

    fn x_axis_is_present_in_layout(layout: &ChartLayout, axis: delinea::AxisId) -> bool {
        layout.x_axes.iter().any(|a| a.axis == axis)
    }

    fn y_axis_is_present_in_layout(layout: &ChartLayout, axis: delinea::AxisId) -> bool {
        layout.y_axes.iter().any(|a| a.axis == axis)
    }

    fn active_axes(&self, layout: &ChartLayout) -> Option<(delinea::AxisId, delinea::AxisId)> {
        let (primary_x, primary_y) = self.primary_axes()?;

        let x_axis = self
            .active_x_axis
            .filter(|a| Self::x_axis_is_present_in_layout(layout, *a))
            .unwrap_or(primary_x);
        let y_axis = self
            .active_y_axis
            .filter(|a| Self::y_axis_is_present_in_layout(layout, *a))
            .unwrap_or(primary_y);

        Some((x_axis, y_axis))
    }

    fn axis_range(&self, axis: delinea::AxisId) -> delinea::AxisRange {
        self.with_engine(|engine| {
            engine
                .model()
                .axes
                .get(&axis)
                .map(|a| a.range)
                .unwrap_or_default()
        })
    }

    fn axis_is_fixed(&self, axis: delinea::AxisId) -> Option<DataWindow> {
        match self.axis_range(axis) {
            delinea::AxisRange::Fixed { min, max } => {
                let mut w = DataWindow { min, max };
                w.clamp_non_degenerate();
                Some(w)
            }
            _ => None,
        }
    }

    fn axis_constraints(&self, axis: delinea::AxisId) -> (Option<f64>, Option<f64>) {
        match self.axis_range(axis) {
            delinea::AxisRange::Auto => (None, None),
            delinea::AxisRange::LockMin { min } => (Some(min), None),
            delinea::AxisRange::LockMax { max } => (None, Some(max)),
            delinea::AxisRange::Fixed { min, max } => (Some(min), Some(max)),
        }
    }

    fn current_window_x(&mut self, axis: delinea::AxisId) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        let zoom_window = self.with_engine(|engine| {
            engine
                .state()
                .data_zoom_x
                .get(&axis)
                .copied()
                .and_then(|z| z.window)
        });
        if let Some(window) = zoom_window {
            return window;
        }

        let mut window = self.compute_axis_extent(axis, true);
        let (locked_min, locked_max) = self.axis_constraints(axis);
        window = window.apply_constraints(locked_min, locked_max);
        window
    }

    fn current_window_y(&mut self, axis: delinea::AxisId) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        let window = self.with_engine(|engine| engine.state().data_window_y.get(&axis).copied());
        if let Some(window) = window {
            return window;
        }

        let mut window = self.compute_axis_extent(axis, false);
        let (locked_min, locked_max) = self.axis_constraints(axis);
        window = window.apply_constraints(locked_min, locked_max);
        window
    }

    fn compute_axis_extent(&mut self, axis: delinea::AxisId, is_x: bool) -> DataWindow {
        self.with_engine_mut(|engine| {
            if let Some(window) = engine.output().axis_windows.get(&axis).copied() {
                return window;
            }

            let model = engine.model();
            if let Some(axis_model) = model.axes.get(&axis) {
                if let delinea::AxisScale::Category(scale) = &axis_model.scale
                    && !scale.categories.is_empty()
                {
                    return DataWindow {
                        min: -0.5,
                        max: scale.categories.len() as f64 - 0.5,
                    };
                }
            }

            let mut series_cols: Vec<(delinea::DatasetId, usize)> = Vec::new();
            for series in model.series.values() {
                let axis_id = if is_x { series.x_axis } else { series.y_axis };
                if axis_id != axis {
                    continue;
                }

                let Some(dataset) = model.datasets.get(&series.dataset) else {
                    continue;
                };

                if is_x {
                    let Some(col) = dataset.fields.get(&series.encode.x).copied() else {
                        continue;
                    };
                    series_cols.push((series.dataset, col));
                    continue;
                }

                if let Some(col) = dataset.fields.get(&series.encode.y).copied() {
                    series_cols.push((series.dataset, col));
                }
                if series.kind == delinea::SeriesKind::Band
                    && let Some(y2) = series.encode.y2
                    && let Some(col) = dataset.fields.get(&y2).copied()
                {
                    series_cols.push((series.dataset, col));
                }
            }

            let store = engine.datasets_mut();
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for (dataset_id, col) in series_cols {
                let Some(table) = store.dataset_mut(dataset_id) else {
                    continue;
                };
                let Some(values) = table.column_f64(col) else {
                    continue;
                };

                for &v in values {
                    if !v.is_finite() {
                        continue;
                    }
                    min = min.min(v);
                    max = max.max(v);
                }
            }

            let mut out = if min.is_finite() && max.is_finite() && max > min {
                DataWindow { min, max }
            } else {
                DataWindow { min: 0.0, max: 1.0 }
            };
            out.clamp_non_degenerate();
            out
        })
    }

    fn set_data_window_x(&mut self, axis: delinea::AxisId, window: Option<DataWindow>) {
        self.with_engine_mut(|engine| engine.apply_action(Action::SetDataWindowX { axis, window }));
    }

    fn set_data_window_x_filter_mode(&mut self, axis: delinea::AxisId, mode: Option<FilterMode>) {
        self.with_engine_mut(|engine| {
            engine.apply_action(Action::SetDataWindowXFilterMode { axis, mode });
        });
    }

    fn toggle_data_window_x_filter_mode(&mut self, axis: delinea::AxisId) {
        let current = self.with_engine(|engine| {
            engine
                .state()
                .data_zoom_x
                .get(&axis)
                .copied()
                .unwrap_or_default()
                .filter_mode
        });

        match current {
            FilterMode::Filter => self.set_data_window_x_filter_mode(axis, Some(FilterMode::None)),
            FilterMode::WeakFilter => {
                self.set_data_window_x_filter_mode(axis, Some(FilterMode::None))
            }
            FilterMode::Empty => self.set_data_window_x_filter_mode(axis, Some(FilterMode::None)),
            FilterMode::None => self.set_data_window_x_filter_mode(axis, None),
        }
    }

    fn set_data_window_y(&mut self, axis: delinea::AxisId, window: Option<DataWindow>) {
        self.with_engine_mut(|engine| engine.apply_action(Action::SetDataWindowY { axis, window }));
    }

    fn view_window_2d_action_from_zoom(
        x_axis: delinea::AxisId,
        y_axis: delinea::AxisId,
        base_x: DataWindow,
        base_y: DataWindow,
        x: Option<DataWindow>,
        y: Option<DataWindow>,
    ) -> Action {
        Action::SetViewWindow2DFromZoom {
            x_axis,
            y_axis,
            base_x,
            base_y,
            x,
            y,
        }
    }

    fn axis_pointer_hover_point(layout: &ChartLayout, position: Point) -> Point {
        let plot = layout.plot;
        if plot.contains(position) {
            return position;
        }

        let plot_left = plot.origin.x.0;
        let plot_top = plot.origin.y.0;
        let plot_right = plot.origin.x.0 + plot.size.width.0;
        let plot_bottom = plot.origin.y.0 + plot.size.height.0;

        let x_in_plot = position.x.0.clamp(plot_left, plot_right);
        let y_in_plot = position.y.0.clamp(plot_top, plot_bottom);

        if layout.x_axes.iter().any(|a| a.rect.contains(position)) {
            let y = (plot_bottom - 1.0).max(plot_top);
            return Point::new(Px(x_in_plot), Px(y));
        }

        if let Some(y_axis) = layout.y_axes.iter().find(|a| a.rect.contains(position)) {
            let x = match y_axis.position {
                delinea::AxisPosition::Right => (plot_right - 1.0).max(plot_left),
                _ => (plot_left + 1.0).min(plot_right),
            };
            return Point::new(Px(x), Px(y_in_plot));
        }

        position
    }

    fn refresh_hover_for_axis_pointer(&mut self, layout: &ChartLayout, position: Point) {
        let axis_pointer_enabled = self.with_engine(|engine| {
            engine
                .model()
                .axis_pointer
                .as_ref()
                .is_some_and(|p| p.enabled)
        });
        if !axis_pointer_enabled {
            return;
        }

        let region = Self::axis_region(layout, position);
        let in_plot = layout.plot.contains(position);
        let in_axis = matches!(region, AxisRegion::XAxis(_) | AxisRegion::YAxis(_));
        if in_plot || in_axis {
            let point = Self::axis_pointer_hover_point(layout, position);
            self.with_engine_mut(|engine| engine.apply_action(Action::HoverAt { point }));
        }
    }

    fn clear_brush(&mut self) {
        self.brush_drag = None;
        self.with_engine_mut(|engine| engine.apply_action(Action::ClearBrushSelection));
    }

    fn clear_slider_drag(&mut self) {
        self.slider_drag = None;
    }

    fn selection_windows_for_drag(
        &self,
        plot: Rect,
        start_x: DataWindow,
        start_y: DataWindow,
        start_pos: Point,
        end_pos: Point,
        modifiers: Modifiers,
        required_mods: ModifiersMask,
    ) -> Option<(DataWindow, DataWindow)> {
        let width = plot.size.width.0;
        let height = plot.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        let start_local = Point::new(
            Px(start_pos.x.0 - plot.origin.x.0),
            Px(start_pos.y.0 - plot.origin.y.0),
        );
        let end_local = Point::new(
            Px(end_pos.x.0 - plot.origin.x.0),
            Px(end_pos.y.0 - plot.origin.y.0),
        );

        let (start_local, end_local) = Self::apply_box_select_modifiers(
            plot.size,
            start_local,
            end_local,
            modifiers,
            self.input_map.box_zoom_expand_x,
            self.input_map.box_zoom_expand_y,
            required_mods,
        );

        let w = (start_local.x.0 - end_local.x.0).abs();
        let h = (start_local.y.0 - end_local.y.0).abs();
        if w < 4.0 || h < 4.0 {
            return None;
        }

        let x0 = start_local.x.0.min(end_local.x.0).clamp(0.0, width);
        let x1 = start_local.x.0.max(end_local.x.0).clamp(0.0, width);
        let x_min = delinea::engine::axis::data_at_px(start_x, x0, 0.0, width);
        let x_max = delinea::engine::axis::data_at_px(start_x, x1, 0.0, width);
        let mut x = DataWindow {
            min: x_min,
            max: x_max,
        };
        x.clamp_non_degenerate();

        let y0 = start_local.y.0.min(end_local.y.0).clamp(0.0, height);
        let y1 = start_local.y.0.max(end_local.y.0).clamp(0.0, height);
        let y0_from_bottom = height - y1;
        let y1_from_bottom = height - y0;
        let y_min = delinea::engine::axis::data_at_px(start_y, y0_from_bottom, 0.0, height);
        let y_max = delinea::engine::axis::data_at_px(start_y, y1_from_bottom, 0.0, height);
        let mut y = DataWindow {
            min: y_min,
            max: y_max,
        };
        y.clamp_non_degenerate();

        Some((x, y))
    }

    fn px_at_data(window: DataWindow, value: f64, origin_px: f32, span_px: f32) -> f32 {
        let mut window = window;
        window.clamp_non_degenerate();
        let span = window.span();
        if !span.is_finite() || span <= 0.0 {
            return origin_px;
        }
        if !span_px.is_finite() || span_px <= 0.0 {
            return origin_px;
        }
        let t = ((value - window.min) / span).clamp(0.0, 1.0) as f32;
        origin_px + t * span_px
    }

    fn brush_rect_px(&mut self, brush: BrushSelection2D) -> Option<Rect> {
        let plot = self.last_layout.plot;
        let width = plot.size.width.0;
        let height = plot.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        let x_window = self.current_window_x(brush.x_axis);
        let y_window = self.current_window_y(brush.y_axis);

        let (xmin, xmax) = if brush.x.min <= brush.x.max {
            (brush.x.min, brush.x.max)
        } else {
            (brush.x.max, brush.x.min)
        };
        let (ymin, ymax) = if brush.y.min <= brush.y.max {
            (brush.y.min, brush.y.max)
        } else {
            (brush.y.max, brush.y.min)
        };

        let x0 = Self::px_at_data(x_window, xmin, 0.0, width);
        let x1 = Self::px_at_data(x_window, xmax, 0.0, width);

        let y0_from_bottom = Self::px_at_data(y_window, ymin, 0.0, height);
        let y1_from_bottom = Self::px_at_data(y_window, ymax, 0.0, height);
        let y0 = height - y1_from_bottom;
        let y1 = height - y0_from_bottom;

        let p0 = Point::new(Px(plot.origin.x.0 + x0), Px(plot.origin.y.0 + y0));
        let p1 = Point::new(Px(plot.origin.x.0 + x1), Px(plot.origin.y.0 + y1));
        Some(rect_from_points_clamped(plot, p0, p1))
    }

    fn compute_axis_extent_from_data(&mut self, axis: delinea::AxisId, is_x: bool) -> DataWindow {
        let (spec_rev, visual_rev) = self.with_engine(|engine| {
            let model = engine.model();
            (model.revs.spec, model.revs.visual)
        });

        let data_sig = self.data_signature();
        if let Some(entry) = self.axis_extent_cache.get(&axis).copied()
            && entry.spec_rev == spec_rev
            && entry.visual_rev == visual_rev
            && entry.data_sig == data_sig
        {
            return entry.window;
        }

        let series_cols = self.with_engine(|engine| {
            let model = engine.model();
            if let Some(axis_model) = model.axes.get(&axis) {
                if let delinea::AxisScale::Category(scale) = &axis_model.scale
                    && !scale.categories.is_empty()
                {
                    return Err(DataWindow {
                        min: -0.5,
                        max: scale.categories.len() as f64 - 0.5,
                    });
                }
            }

            let mut series_cols: Vec<(delinea::DatasetId, usize)> = Vec::new();
            for series_id in &model.series_order {
                let Some(series) = model.series.get(series_id) else {
                    continue;
                };
                if !series.visible {
                    continue;
                }

                let axis_id = if is_x { series.x_axis } else { series.y_axis };
                if axis_id != axis {
                    continue;
                }

                let Some(dataset) = model.datasets.get(&series.dataset) else {
                    continue;
                };
                let field = if is_x {
                    series.encode.x
                } else {
                    series.encode.y
                };
                let Some(col) = dataset.fields.get(&field).copied() else {
                    continue;
                };
                series_cols.push((series.dataset, col));
            }

            Ok(series_cols)
        });

        let series_cols = match series_cols {
            Ok(cols) => cols,
            Err(window) => return window,
        };

        let (min, max) = self.with_engine_mut(|engine| {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;

            let datasets = engine.datasets_mut();
            for (dataset_id, col) in &series_cols {
                let Some(table) = datasets.dataset_mut(*dataset_id) else {
                    continue;
                };
                let Some(values) = table.column_f64(*col) else {
                    continue;
                };

                for &v in values {
                    if !v.is_finite() {
                        continue;
                    }
                    min = min.min(v);
                    max = max.max(v);
                }
            }

            (min, max)
        });

        let mut out = if min.is_finite() && max.is_finite() && max > min {
            DataWindow { min, max }
        } else {
            DataWindow { min: 0.0, max: 1.0 }
        };

        let (locked_min, locked_max) = self.axis_constraints(axis);
        out = out.apply_constraints(locked_min, locked_max);
        out.clamp_non_degenerate();

        self.axis_extent_cache.insert(
            axis,
            AxisExtentCacheEntry {
                spec_rev,
                visual_rev,
                data_sig,
                window: out,
            },
        );
        out
    }

    fn data_signature(&mut self) -> u64 {
        use std::hash::{Hash, Hasher};

        self.with_engine_mut(|engine| {
            let dataset_ids: Vec<delinea::DatasetId> =
                engine.model().datasets.keys().copied().collect();

            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            let datasets = engine.datasets_mut();
            for dataset_id in dataset_ids {
                dataset_id.0.hash(&mut hasher);
                if let Some(table) = datasets.dataset_mut(dataset_id) {
                    table.revision.0.hash(&mut hasher);
                    table.row_count.hash(&mut hasher);
                }
            }
            hasher.finish()
        })
    }

    fn x_slider_track_for_axis(&self, axis: delinea::AxisId) -> Option<Rect> {
        let plot = self.last_layout.plot;
        if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
            return None;
        }

        let band = self
            .last_layout
            .x_axes
            .iter()
            .find(|b| b.axis == axis && b.position == delinea::AxisPosition::Bottom)?;

        let h = 9.0f32;
        let pad = 4.0f32;
        let y = band.rect.origin.y.0 + band.rect.size.height.0 - h - pad;
        let track = Rect::new(
            Point::new(plot.origin.x, Px(y)),
            Size::new(plot.size.width, Px(h)),
        );

        Some(track)
    }

    fn current_window_x_for_slider(
        &mut self,
        axis: delinea::AxisId,
        extent: DataWindow,
    ) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        let zoom_window = self.with_engine(|engine| {
            engine
                .state()
                .data_zoom_x
                .get(&axis)
                .copied()
                .and_then(|z| z.window)
        });
        if let Some(window) = zoom_window {
            return window;
        }

        extent
    }

    fn slider_norm(extent: DataWindow, v: f64) -> f32 {
        let span = extent.span();
        if !span.is_finite() || span <= 0.0 {
            return 0.0;
        }
        (((v - extent.min) / span) as f32).clamp(0.0, 1.0)
    }

    fn slider_value_at(track: Rect, extent: DataWindow, px_x: f32) -> f64 {
        delinea::engine::axis::data_at_px(extent, px_x, track.origin.x.0, track.size.width.0)
    }

    fn slider_window_after_delta(
        extent: DataWindow,
        start_window: DataWindow,
        delta_value: f64,
        kind: SliderDragKind,
    ) -> DataWindow {
        let extent_span = extent.span();
        if !extent_span.is_finite() || extent_span <= 0.0 {
            return start_window;
        }

        let mut min = start_window.min;
        let mut max = start_window.max;

        if !delta_value.is_finite() || !min.is_finite() || !max.is_finite() {
            return start_window;
        }

        match kind {
            SliderDragKind::Pan => {
                min += delta_value;
                max += delta_value;
            }
            SliderDragKind::HandleMin => {
                min += delta_value;
            }
            SliderDragKind::HandleMax => {
                max += delta_value;
            }
        }

        let eps = (extent_span.abs() * 1e-12).max(1e-9).max(f64::MIN_POSITIVE);

        match kind {
            SliderDragKind::Pan => {
                let mut span = (max - min).abs();
                if !span.is_finite() || span <= eps {
                    span = start_window.span().abs();
                }
                if !span.is_finite() || span <= eps {
                    span = eps;
                }

                if span >= extent_span {
                    return extent;
                }

                if max <= min {
                    max = min + span;
                } else {
                    span = max - min;
                }

                if min < extent.min {
                    let d = extent.min - min;
                    min += d;
                    max += d;
                }
                if max > extent.max {
                    let d = max - extent.max;
                    min -= d;
                    max -= d;
                }

                min = min.max(extent.min);
                max = max.min(extent.max);

                if max - min < eps {
                    min = extent.min;
                    max = (extent.min + span).min(extent.max);
                    if max - min < eps {
                        max = (min + eps).min(extent.max);
                    }
                }

                if !(max > min) {
                    return extent;
                }

                DataWindow { min, max }
            }
            SliderDragKind::HandleMin => {
                let mut out_max = max.clamp(extent.min + eps, extent.max);
                let mut out_min = min.clamp(extent.min, out_max - eps);
                if !(out_max > out_min) {
                    out_min = (out_max - eps).max(extent.min);
                    if !(out_max > out_min) {
                        out_max = (out_min + eps).min(extent.max);
                    }
                }
                DataWindow {
                    min: out_min,
                    max: out_max,
                }
            }
            SliderDragKind::HandleMax => {
                let mut out_min = min.clamp(extent.min, extent.max - eps);
                let mut out_max = max.clamp(out_min + eps, extent.max);
                if !(out_max > out_min) {
                    out_max = (out_min + eps).min(extent.max);
                    if !(out_max > out_min) {
                        out_min = (out_max - eps).max(extent.min);
                    }
                }
                DataWindow {
                    min: out_min,
                    max: out_max,
                }
            }
        }
    }

    fn y_slider_track_for_axis(&self, axis: delinea::AxisId) -> Option<Rect> {
        let plot = self.last_layout.plot;
        if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
            return None;
        }

        let band = self.last_layout.y_axes.iter().find(|b| b.axis == axis)?;

        let w = 9.0f32;
        let pad = 4.0f32;
        let x = match band.position {
            delinea::AxisPosition::Right => band.rect.origin.x.0 + band.rect.size.width.0 - w - pad,
            _ => band.rect.origin.x.0 + pad,
        };

        Some(Rect::new(
            Point::new(Px(x), plot.origin.y),
            Size::new(Px(w), plot.size.height),
        ))
    }

    fn current_window_y_for_slider(
        &mut self,
        axis: delinea::AxisId,
        extent: DataWindow,
    ) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        let window = self.with_engine(|engine| engine.state().data_window_y.get(&axis).copied());
        if let Some(window) = window {
            return window;
        }

        extent
    }

    fn slider_value_at_y(track: Rect, extent: DataWindow, px_y: f32) -> f64 {
        let height = track.size.height.0.max(1.0);
        let bottom = track.origin.y.0 + height;
        let y = px_y.clamp(track.origin.y.0, bottom);
        let y_from_bottom = bottom - y;
        delinea::engine::axis::data_at_px(extent, y_from_bottom, 0.0, height)
    }

    fn visual_map_tracks(
        &self,
    ) -> Vec<(
        delinea::VisualMapId,
        delinea::engine::model::VisualMapModel,
        Rect,
    )> {
        let Some(band) = self.last_layout.visual_map else {
            return Vec::new();
        };
        if band.size.width.0 <= 0.0 || band.size.height.0 <= 0.0 {
            return Vec::new();
        }

        let maps: Vec<(delinea::VisualMapId, delinea::engine::model::VisualMapModel)> = self
            .with_engine(|engine| {
                engine
                    .model()
                    .visual_maps
                    .iter()
                    .map(|(id, vm)| (*id, *vm))
                    .collect()
            });
        if maps.is_empty() {
            return Vec::new();
        }

        let gap = self.style.visual_map_item_gap.0.max(0.0);
        let pad = self.style.visual_map_padding.0.max(0.0);

        let total_gap = gap * (maps.len().saturating_sub(1) as f32);
        let item_h = ((band.size.height.0 - total_gap) / (maps.len() as f32)).max(1.0);

        let mut y = band.origin.y.0;
        let mut out = Vec::with_capacity(maps.len());
        for (id, vm) in maps {
            let item = Rect::new(
                Point::new(band.origin.x, Px(y)),
                Size::new(band.size.width, Px(item_h)),
            );
            y += item_h + gap;

            let track = Rect::new(
                Point::new(Px(item.origin.x.0 + pad), Px(item.origin.y.0 + pad)),
                Size::new(
                    Px((item.size.width.0 - 2.0 * pad).max(1.0)),
                    Px((item.size.height.0 - 2.0 * pad).max(1.0)),
                ),
            );
            if track.size.width.0 > 0.0 && track.size.height.0 > 0.0 {
                out.push((id, vm, track));
            }
        }
        out
    }

    fn visual_map_track_at(
        &self,
        position: Point,
    ) -> Option<(
        delinea::VisualMapId,
        delinea::engine::model::VisualMapModel,
        Rect,
    )> {
        self.visual_map_tracks()
            .into_iter()
            .find(|(_, _, track)| track.contains(position))
    }

    fn visual_map_domain_window(vm: delinea::engine::model::VisualMapModel) -> DataWindow {
        DataWindow {
            min: vm.domain.min,
            max: vm.domain.max,
        }
    }

    fn current_visual_map_window(
        &self,
        id: delinea::VisualMapId,
        vm: delinea::engine::model::VisualMapModel,
    ) -> DataWindow {
        let domain = Self::visual_map_domain_window(vm);
        let range =
            self.with_engine(|engine| engine.state().visual_map_range.get(&id).copied().flatten());
        match range {
            Some(r) => DataWindow {
                min: r.min,
                max: r.max,
            },
            None => domain,
        }
    }

    fn current_visual_map_piece_mask(
        &self,
        id: delinea::VisualMapId,
        vm: delinea::engine::model::VisualMapModel,
    ) -> u64 {
        let buckets = vm.buckets.clamp(1, 64) as u32;
        let full_mask = if buckets >= 64 {
            u64::MAX
        } else {
            (1u64 << buckets) - 1
        };
        let piece_mask = self.with_engine(|engine| {
            engine
                .state()
                .visual_map_piece_mask
                .get(&id)
                .copied()
                .flatten()
        });
        piece_mask.or(vm.initial_piece_mask).unwrap_or(full_mask) & full_mask
    }

    fn visual_map_y_at_value(track: Rect, domain: DataWindow, value: f64) -> f32 {
        let mut domain = domain;
        domain.clamp_non_degenerate();
        let span = domain.span();
        if !span.is_finite() || span <= 0.0 {
            return track.origin.y.0 + track.size.height.0;
        }
        let t = ((value - domain.min) / span).clamp(0.0, 1.0) as f32;
        track.origin.y.0 + (1.0 - t) * track.size.height.0
    }

    fn reset_view_for_axes(&mut self, x_axis: delinea::AxisId, y_axis: delinea::AxisId) {
        if self.axis_is_fixed(x_axis).is_none() {
            self.set_data_window_x(x_axis, None);
        }
        if self.axis_is_fixed(y_axis).is_none() {
            self.set_data_window_y(y_axis, None);
        }
    }

    fn fit_view_to_data_for_axes(&mut self, x_axis: delinea::AxisId, y_axis: delinea::AxisId) {
        if self.axis_is_fixed(x_axis).is_none() {
            let mut w = self.compute_axis_extent(x_axis, true);
            let (locked_min, locked_max) = self.axis_constraints(x_axis);
            w = w.apply_constraints(locked_min, locked_max);
            self.set_data_window_x(x_axis, Some(w));
        }

        if self.axis_is_fixed(y_axis).is_none() {
            let mut w = self.compute_axis_extent(y_axis, false);
            let (locked_min, locked_max) = self.axis_constraints(y_axis);
            w = w.apply_constraints(locked_min, locked_max);
            self.set_data_window_y(y_axis, Some(w));
        }
    }

    fn axis_region(layout: &ChartLayout, position: Point) -> AxisRegion {
        for axis in &layout.x_axes {
            if axis.rect.contains(position) {
                return AxisRegion::XAxis(axis.axis);
            }
        }
        for axis in &layout.y_axes {
            if axis.rect.contains(position) {
                return AxisRegion::YAxis(axis.axis);
            }
        }
        AxisRegion::Plot
    }

    fn is_button_held(button: MouseButton, buttons: fret_core::MouseButtons) -> bool {
        match button {
            MouseButton::Left => buttons.left,
            MouseButton::Right => buttons.right,
            MouseButton::Middle => buttons.middle,
            _ => false,
        }
    }

    fn apply_box_select_modifiers(
        plot_size: Size,
        start: Point,
        end: Point,
        modifiers: Modifiers,
        expand_x: Option<ModifierKey>,
        expand_y: Option<ModifierKey>,
        required: ModifiersMask,
    ) -> (Point, Point) {
        let mut start = start;
        let mut end = end;

        // Matches ImPlot's default selection modifiers:
        // - Alt: expand selection horizontally to plot edge.
        // - Shift: expand selection vertically to plot edge.
        //
        // Note: when a modifier is required to start the drag gesture (e.g. Shift+LMB alternative),
        // treat it as part of the gesture chord and do not implicitly apply edge expansion.
        if expand_x.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.x = Px(0.0);
            end.x = plot_size.width;
        }
        if expand_y.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.y = Px(0.0);
            end.y = plot_size.height;
        }

        (start, end)
    }

    fn axis_ticks_with_labels(
        model: &delinea::engine::model::ChartModel,
        axis: delinea::AxisId,
        window: DataWindow,
        count: usize,
    ) -> Vec<(f64, String)> {
        delinea::engine::axis::axis_ticks_with_labels_for(model, axis, window, count)
    }

    fn y_local_for_data_value(window: DataWindow, value: f64, plot_height_px: f32) -> f32 {
        let mut window = window;
        window.clamp_non_degenerate();

        let span = window.span();
        if !span.is_finite() || span <= 0.0 || !value.is_finite() {
            return plot_height_px;
        }

        let t = ((value - window.min) / span).clamp(0.0, 1.0) as f32;
        plot_height_px * (1.0 - t)
    }

    fn clear_tooltip_text_cache(&mut self, services: &mut dyn fret_core::UiServices) {
        self.tooltip_text.clear(services);
    }

    fn series_color(&self, series: delinea::SeriesId) -> Color {
        let order_idx = self
            .series_rank_by_id
            .get(&series)
            .copied()
            .unwrap_or_else(|| {
                self.with_engine(|engine| {
                    engine
                        .model()
                        .series_order
                        .iter()
                        .position(|id| *id == series)
                        .unwrap_or(0)
                })
            });
        let palette = &self.style.series_palette;
        palette[order_idx % palette.len()]
    }

    fn series_is_in_view_grid(
        &self,
        model: &delinea::engine::model::ChartModel,
        series: delinea::SeriesId,
    ) -> bool {
        let Some(grid) = self.grid_override else {
            return true;
        };
        let Some(series) = model.series.get(&series) else {
            return false;
        };
        model
            .axes
            .get(&series.x_axis)
            .is_some_and(|a| a.grid == grid)
    }

    fn paint_color(&self, paint: delinea::PaintId) -> Color {
        let palette = &self.style.series_palette;
        palette[(paint.0 as usize) % palette.len()]
    }

    fn legend_series_at(&self, pos: Point) -> Option<delinea::SeriesId> {
        self.legend_item_rects
            .iter()
            .find_map(|(id, r)| r.contains(pos).then_some(*id))
    }

    fn legend_selector_at(&self, pos: Point) -> Option<LegendSelectorAction> {
        self.legend_selector_rects
            .iter()
            .find_map(|(action, r)| r.contains(pos).then_some(*action))
    }

    fn legend_max_scroll_y(&self) -> Px {
        if self.legend_content_height.0 <= self.legend_view_height.0 {
            return Px(0.0);
        }
        Px(self.legend_content_height.0 - self.legend_view_height.0)
    }

    fn apply_legend_wheel_scroll(&mut self, wheel_delta_y: Px) -> bool {
        let max_scroll = self.legend_max_scroll_y();
        if max_scroll.0 <= 0.0 {
            return false;
        }

        let prev = self.legend_scroll_y;
        let speed = 0.75f32;
        let next = (self.legend_scroll_y.0 - wheel_delta_y.0 * speed).clamp(0.0, max_scroll.0);
        self.legend_scroll_y = Px(next);
        self.legend_scroll_y.0 != prev.0
    }

    fn apply_legend_select_all(&mut self) -> bool {
        let updates = self
            .with_engine(|engine| crate::legend_logic::legend_select_all_updates(engine.model()));
        if updates.is_empty() {
            return false;
        }
        self.with_engine_mut(|engine| engine.apply_action(Action::SetSeriesVisibility { updates }));
        true
    }

    fn apply_legend_select_none(&mut self) -> bool {
        let updates = self
            .with_engine(|engine| crate::legend_logic::legend_select_none_updates(engine.model()));
        if updates.is_empty() {
            return false;
        }
        self.with_engine_mut(|engine| engine.apply_action(Action::SetSeriesVisibility { updates }));
        true
    }

    fn apply_legend_invert(&mut self) -> bool {
        let updates =
            self.with_engine(|engine| crate::legend_logic::legend_invert_updates(engine.model()));
        if updates.is_empty() {
            return false;
        }
        self.with_engine_mut(|engine| engine.apply_action(Action::SetSeriesVisibility { updates }));
        true
    }

    fn apply_legend_double_click(&mut self, clicked: delinea::SeriesId) {
        let updates = self.with_engine(|engine| {
            crate::legend_logic::legend_double_click_updates(engine.model(), clicked)
        });
        if !updates.is_empty() {
            self.with_engine_mut(|engine| {
                engine.apply_action(Action::SetSeriesVisibility { updates });
            });
        }
    }

    fn apply_legend_reset(&mut self) {
        let updates =
            self.with_engine(|engine| crate::legend_logic::legend_reset_updates(engine.model()));
        if !updates.is_empty() {
            self.with_engine_mut(|engine| {
                engine.apply_action(Action::SetSeriesVisibility { updates })
            });
        }
    }

    fn apply_legend_shift_range_toggle(
        &mut self,
        anchor: delinea::SeriesId,
        clicked: delinea::SeriesId,
    ) {
        let updates = self.with_engine(|engine| {
            crate::legend_logic::legend_shift_range_toggle_updates(engine.model(), anchor, clicked)
        });
        if !updates.is_empty() {
            self.with_engine_mut(|engine| {
                engine.apply_action(Action::SetSeriesVisibility { updates })
            });
        }
    }

    fn draw_legend<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        self.legend_item_rects.clear();
        self.legend_selector_rects.clear();
        self.legend_panel_rect = None;
        self.legend_content_height = Px(0.0);
        self.legend_view_height = Px(0.0);

        let plot = self.last_layout.plot;
        if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
            return;
        }

        let series: Vec<delinea::engine::model::SeriesModel> = self.with_engine(|engine| {
            let model = engine.model();
            model
                .series_in_order()
                .filter(|s| self.series_is_in_view_grid(model, s.id))
                .cloned()
                .collect()
        });
        if series.is_empty() {
            return;
        }

        let mut key = KeyBuilder::new();
        key.mix_u64(self.last_theme_revision);
        key.mix_u64(u64::from(cx.scale_factor.to_bits()));
        key.mix_u64(u64::from(series.len() as u32));
        for s in &series {
            key.mix_u64(u64::from(s.id.0));
            key.mix_bool(s.visible);
            if let Some(name) = s.name.as_deref() {
                key.mix_str(name);
            }
        }
        self.legend_text
            .reset_if_key_changed(cx.services, key.finish());

        let text_style = TextStyle {
            size: Px(12.0),
            weight: FontWeight::NORMAL,
            ..TextStyle::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
        };

        let mut blobs: Vec<(delinea::SeriesId, TextBlobId, fret_core::TextMetrics, bool)> =
            Vec::with_capacity(series.len());

        let mut max_text_w = 1.0f32;
        let mut row_h = 1.0f32;
        for s in &series {
            let label = s
                .name
                .as_deref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("Series {}", s.id.0));
            let prepared = self
                .legend_text
                .prepare(cx.services, &label, &text_style, constraints);
            let blob = prepared.blob;
            let metrics = prepared.metrics;
            max_text_w = max_text_w.max(metrics.size.width.0.max(1.0));
            row_h = row_h.max(metrics.size.height.0.max(1.0));
            blobs.push((s.id, blob, metrics, s.visible));
        }

        let selector_text_style = TextStyle {
            size: Px(11.0),
            weight: FontWeight::MEDIUM,
            ..TextStyle::default()
        };

        let pad = self.style.legend_padding;
        let sw = self.style.legend_swatch_size.0.max(1.0);
        let sw_gap = self.style.legend_swatch_gap.0.max(0.0);
        let gap = self.style.legend_item_gap.0.max(0.0);
        let selector_gap = 8.0f32;

        let row_h = row_h.max(sw);
        let legend_w = (pad.left.0 + sw + sw_gap + max_text_w + pad.right.0).max(1.0);
        let selector_labels: [(LegendSelectorAction, &str); 3] = [
            (LegendSelectorAction::All, "All"),
            (LegendSelectorAction::None, "None"),
            (LegendSelectorAction::Invert, "Invert"),
        ];

        let mut selector_blobs: Vec<(LegendSelectorAction, TextBlobId, fret_core::TextMetrics)> =
            Vec::with_capacity(selector_labels.len());
        let mut selector_total_w = 0.0f32;
        let mut selector_h = 0.0f32;
        for (action, label) in selector_labels {
            let prepared =
                self.legend_text
                    .prepare(cx.services, label, &selector_text_style, constraints);
            let blob = prepared.blob;
            let metrics = prepared.metrics;
            selector_total_w += metrics.size.width.0.max(1.0);
            selector_h = selector_h.max(metrics.size.height.0.max(1.0));
            selector_blobs.push((action, blob, metrics));
        }
        if !selector_blobs.is_empty() {
            selector_total_w += selector_gap * (selector_blobs.len().saturating_sub(1) as f32);
        }
        let selector_h = selector_h.max(1.0);
        let selector_row_h = (selector_h + 4.0).max(1.0);

        let items_h = ((row_h + gap) * (series.len().saturating_sub(1) as f32) + row_h).max(1.0);
        let full_h = (pad.top.0 + selector_row_h + items_h + pad.bottom.0).max(1.0);

        let margin = 8.0f32;
        let min_h = (pad.top.0 + row_h + pad.bottom.0).max(1.0);
        let max_h = (plot.size.height.0 - 2.0 * margin).max(min_h);
        let legend_h = full_h.min(max_h);
        let view_h = (legend_h - selector_row_h - pad.top.0 - pad.bottom.0).max(1.0);
        self.legend_content_height = Px(items_h);
        self.legend_view_height = Px(view_h);
        self.legend_scroll_y = Px(self
            .legend_scroll_y
            .0
            .clamp(0.0, self.legend_max_scroll_y().0));

        let x0 =
            (plot.origin.x.0 + plot.size.width.0 - legend_w - margin).max(plot.origin.x.0 + margin);
        let y0 = (plot.origin.y.0 + margin).max(plot.origin.y.0 + margin);
        let legend_rect = Rect::new(
            Point::new(Px(x0), Px(y0)),
            Size::new(Px(legend_w), Px(legend_h)),
        );
        self.legend_panel_rect = Some(legend_rect);

        let legend_order = DrawOrder(self.style.draw_order.0.saturating_add(8_900));
        cx.scene.push(SceneOp::Quad {
            order: legend_order,
            rect: legend_rect,
            background: self.style.legend_background,
            border: Edges::all(self.style.legend_border_width),
            border_color: self.style.legend_border_color,
            corner_radii: Corners::all(self.style.legend_corner_radius),
        });

        cx.scene.push(SceneOp::PushClipRect { rect: legend_rect });

        // Selector row (ECharts legend.selector-like affordance).
        // Keep it non-scrolling so it stays accessible even when the legend overflows.
        let selector_y = y0 + pad.top.0;
        let selector_x0 = x0 + legend_w - pad.right.0 - selector_total_w;
        let mut sx = selector_x0;
        for (action, blob, metrics) in selector_blobs.into_iter() {
            let w = metrics.size.width.0.max(1.0);
            let rect = Rect::new(
                Point::new(Px(sx), Px(selector_y)),
                Size::new(Px(w), Px(selector_row_h)),
            );
            self.legend_selector_rects.push((action, rect));

            if self.legend_selector_hover == Some(action) {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(legend_order.0.saturating_add(1)),
                    rect,
                    background: self.style.legend_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0)),
                });
            }

            let text_y = selector_y + 0.5 * (selector_row_h - metrics.size.height.0.max(1.0));
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(legend_order.0.saturating_add(2)),
                origin: Point::new(Px(sx), Px(text_y)),
                text: blob,
                color: self.style.legend_text_color,
            });

            sx += w + selector_gap;
        }

        let items_clip = Rect::new(
            Point::new(Px(x0), Px(y0 + pad.top.0 + selector_row_h)),
            Size::new(Px(legend_w), Px(view_h)),
        );
        cx.scene.push(SceneOp::PushClipRect { rect: items_clip });

        let mut y = items_clip.origin.y.0 - self.legend_scroll_y.0;
        for (i, (series_id, blob, metrics, visible)) in blobs.into_iter().enumerate() {
            let item_rect = Rect::new(
                Point::new(Px(x0), Px(y)),
                Size::new(Px(legend_w), Px(row_h)),
            );
            self.legend_item_rects.push((series_id, item_rect));

            if self.legend_hover == Some(series_id) {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(legend_order.0.saturating_add(1 + i as u32 * 3)),
                    rect: item_rect,
                    background: self.style.legend_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let mut swatch = self.series_color(series_id);
            swatch.a = if visible { 0.9 } else { 0.25 };
            let sw_x = x0 + pad.left.0;
            let sw_y = y + 0.5 * (row_h - sw);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(legend_order.0.saturating_add(2 + i as u32 * 3)),
                rect: Rect::new(Point::new(Px(sw_x), Px(sw_y)), Size::new(Px(sw), Px(sw))),
                background: swatch,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(2.0)),
            });

            let text_x = sw_x + sw + sw_gap;
            let text_y = y + 0.5 * (row_h - metrics.size.height.0.max(1.0));
            let mut text_color = self.style.legend_text_color;
            if !visible {
                text_color.a *= 0.55;
            }
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(legend_order.0.saturating_add(3 + i as u32 * 3)),
                origin: Point::new(Px(text_x), Px(text_y)),
                text: blob,
                color: text_color,
            });

            y += row_h + gap;
        }

        cx.scene.push(SceneOp::PopClip);
        cx.scene.push(SceneOp::PopClip);
    }

    fn draw_visual_map<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let tracks = self.visual_map_tracks();
        if tracks.is_empty() {
            return;
        }

        for (i, (vm_id, vm, track)) in tracks.into_iter().enumerate() {
            let order = DrawOrder(
                self.style
                    .draw_order
                    .0
                    .saturating_add(8_600)
                    .saturating_add((i as u32).saturating_mul(20)),
            );

            cx.scene.push(SceneOp::Quad {
                order,
                rect: track,
                background: self.style.visual_map_track_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(self.style.visual_map_corner_radius),
            });

            let buckets = vm.buckets.max(1) as u32;
            let inset = 1.0f32;
            let ramp_rect = Rect::new(
                Point::new(Px(track.origin.x.0 + inset), Px(track.origin.y.0 + inset)),
                Size::new(
                    Px((track.size.width.0 - 2.0 * inset).max(1.0)),
                    Px((track.size.height.0 - 2.0 * inset).max(1.0)),
                ),
            );
            let ramp_h = ramp_rect.size.height.0.max(1.0);
            let segment_h = (ramp_h / buckets as f32).max(1.0);

            match vm.mode {
                delinea::VisualMapMode::Continuous => {
                    // Render a lightweight ramp using the bucket palette. This is not ECharts parity yet
                    // (no multi-channel mapping), but provides a stable continuous controller affordance.
                    let ramp_alpha = 0.35f32;
                    for bucket in 0..buckets {
                        let y1 = ramp_rect.origin.y.0 + ramp_h - (bucket as f32) * segment_h;
                        let y0 = (y1 - segment_h).max(ramp_rect.origin.y.0);
                        let h = (y1 - y0).max(1.0);

                        let mut c = self.paint_color(delinea::PaintId(bucket as u64));
                        c.a *= ramp_alpha;
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(1)),
                            rect: Rect::new(
                                Point::new(ramp_rect.origin.x, Px(y0)),
                                Size::new(ramp_rect.size.width, Px(h)),
                            ),
                            background: c,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }

                    let domain = Self::visual_map_domain_window(vm);
                    let window = self.current_visual_map_window(vm_id, vm);

                    let y_min = Self::visual_map_y_at_value(track, domain, window.min);
                    let y_max = Self::visual_map_y_at_value(track, domain, window.max);
                    let top = y_max.min(y_min);
                    let bottom = y_max.max(y_min);

                    let win_rect = Rect::new(
                        Point::new(track.origin.x, Px(top)),
                        Size::new(track.size.width, Px((bottom - top).max(1.0))),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(order.0.saturating_add(2)),
                        rect: win_rect,
                        background: self.style.visual_map_range_fill,
                        border: Edges::all(self.style.selection_stroke_width),
                        border_color: self.style.visual_map_range_stroke,
                        corner_radii: Corners::all(self.style.visual_map_corner_radius),
                    });

                    let handle_h = 2.0f32.max(self.style.selection_stroke_width.0);
                    let handle_color = self.style.visual_map_handle_color;
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(order.0.saturating_add(3)),
                        rect: Rect::new(
                            Point::new(track.origin.x, Px(y_min - 0.5 * handle_h)),
                            Size::new(track.size.width, Px(handle_h)),
                        ),
                        background: handle_color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(order.0.saturating_add(4)),
                        rect: Rect::new(
                            Point::new(track.origin.x, Px(y_max - 0.5 * handle_h)),
                            Size::new(track.size.width, Px(handle_h)),
                        ),
                        background: handle_color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
                delinea::VisualMapMode::Piecewise => {
                    let mask = self.current_visual_map_piece_mask(vm_id, vm);
                    let ramp_alpha_selected = 0.55f32;
                    let ramp_alpha_unselected = 0.12f32;
                    for bucket in 0..buckets {
                        let y1 = ramp_rect.origin.y.0 + ramp_h - (bucket as f32) * segment_h;
                        let y0 = (y1 - segment_h).max(ramp_rect.origin.y.0);
                        let h = (y1 - y0).max(1.0);

                        let selected = ((mask >> (bucket as u32)) & 1) == 1;
                        let alpha = if selected {
                            ramp_alpha_selected
                        } else {
                            ramp_alpha_unselected
                        };

                        let mut c = self.paint_color(delinea::PaintId(bucket as u64));
                        c.a *= alpha;
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(1)),
                            rect: Rect::new(
                                Point::new(ramp_rect.origin.x, Px(y0)),
                                Size::new(ramp_rect.size.width, Px(h)),
                            ),
                            background: c,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                }
            }
        }
    }

    fn draw_axes<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let plot = self.last_layout.plot;
        let x_axes = self.last_layout.x_axes.clone();
        let y_axes = self.last_layout.y_axes.clone();
        if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
            return;
        }

        let model = self.with_engine(|engine| engine.model().clone());

        let x_bands: Vec<(AxisBandLayout, DataWindow)> = x_axes
            .iter()
            .map(|band| (*band, self.current_window_x(band.axis)))
            .collect();
        let y_bands: Vec<(AxisBandLayout, DataWindow)> = y_axes
            .iter()
            .map(|band| (*band, self.current_window_y(band.axis)))
            .collect();

        let mut key = KeyBuilder::new();
        key.mix_u64(self.last_theme_revision);
        key.mix_u64(u64::from(cx.scale_factor.to_bits()));
        key.mix_f32_bits(plot.size.width.0);
        key.mix_f32_bits(plot.size.height.0);
        key.mix_u64(u64::from(x_bands.len() as u32));
        for (band, window) in &x_bands {
            key.mix_u64(u64::from(band.axis.0));
            key.mix_f32_bits(band.rect.origin.x.0);
            key.mix_f32_bits(band.rect.origin.y.0);
            key.mix_f32_bits(band.rect.size.width.0);
            key.mix_f32_bits(band.rect.size.height.0);
            key.mix_f64_bits(window.min);
            key.mix_f64_bits(window.max);
        }
        key.mix_u64(u64::from(y_bands.len() as u32));
        for (band, window) in &y_bands {
            key.mix_u64(u64::from(band.axis.0));
            key.mix_f32_bits(band.rect.origin.x.0);
            key.mix_f32_bits(band.rect.origin.y.0);
            key.mix_f32_bits(band.rect.size.width.0);
            key.mix_f32_bits(band.rect.size.height.0);
            key.mix_f64_bits(window.min);
            key.mix_f64_bits(window.max);
        }
        self.axis_text
            .reset_if_key_changed(cx.services, key.finish());

        let axis_order = DrawOrder(self.style.draw_order.0.saturating_add(8_500));
        let label_order = DrawOrder(self.style.draw_order.0.saturating_add(8_501));

        let line_w = self.style.axis_line_width.0.max(1.0);
        let tick_len = self.style.axis_tick_length.0.max(0.0);

        let text_style = TextStyle {
            size: Px(12.0),
            weight: FontWeight::NORMAL,
            ..TextStyle::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
        };

        let x_tick_count = (plot.size.width.0 / 80.0).round().clamp(2.0, 12.0) as usize;
        let y_tick_count = (plot.size.height.0 / 56.0).round().clamp(2.0, 12.0) as usize;

        // X axes: baseline + ticks + labels.
        for (band, window) in &x_bands {
            let baseline_y = match band.position {
                delinea::AxisPosition::Bottom => band.rect.origin.y.0,
                delinea::AxisPosition::Top => band.rect.origin.y.0 + band.rect.size.height.0,
                _ => continue,
            };

            cx.scene.push(SceneOp::Quad {
                order: axis_order,
                rect: Rect::new(
                    Point::new(band.rect.origin.x, Px(baseline_y - line_w * 0.5)),
                    Size::new(plot.size.width, Px(line_w)),
                ),
                background: self.style.axis_line_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });

            let mut last_right = f32::NEG_INFINITY;
            for (value, label) in
                Self::axis_ticks_with_labels(&model, band.axis, *window, x_tick_count)
            {
                let t = ((value - window.min) / window.span()).clamp(0.0, 1.0) as f32;
                let x_px = plot.origin.x.0 + t * plot.size.width.0;

                let tick_y = match band.position {
                    delinea::AxisPosition::Bottom => baseline_y,
                    delinea::AxisPosition::Top => baseline_y - tick_len,
                    _ => baseline_y,
                };

                cx.scene.push(SceneOp::Quad {
                    order: axis_order,
                    rect: Rect::new(
                        Point::new(Px(x_px - 0.5 * line_w), Px(tick_y)),
                        Size::new(Px(line_w), Px(tick_len)),
                    ),
                    background: self.style.axis_tick_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });

                let prepared =
                    self.axis_text
                        .prepare(cx.services, &label, &text_style, constraints);
                let blob = prepared.blob;
                let metrics = prepared.metrics;

                let label_x = x_px - metrics.size.width.0 * 0.5;
                let label_y =
                    band.rect.origin.y.0 + (band.rect.size.height.0 - metrics.size.height.0) * 0.5;

                let gap = 4.0;
                let right = label_x + metrics.size.width.0;
                if label_x >= last_right + gap {
                    cx.scene.push(SceneOp::Text {
                        order: label_order,
                        origin: Point::new(Px(label_x), Px(label_y)),
                        text: blob,
                        color: self.style.axis_label_color,
                    });
                    last_right = right;
                }
            }
        }

        // Y axes: baseline + ticks + labels.
        for (band, window) in &y_bands {
            let baseline_x = match band.position {
                delinea::AxisPosition::Left => band.rect.origin.x.0 + band.rect.size.width.0,
                delinea::AxisPosition::Right => band.rect.origin.x.0,
                _ => continue,
            };

            cx.scene.push(SceneOp::Quad {
                order: axis_order,
                rect: Rect::new(
                    Point::new(Px(baseline_x - line_w * 0.5), band.rect.origin.y),
                    Size::new(Px(line_w), plot.size.height),
                ),
                background: self.style.axis_line_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });

            let mut last_bottom = f32::NEG_INFINITY;
            for (value, label) in
                Self::axis_ticks_with_labels(&model, band.axis, *window, y_tick_count)
            {
                let t = ((value - window.min) / window.span()).clamp(0.0, 1.0) as f32;
                let y_px = plot.origin.y.0 + (1.0 - t) * plot.size.height.0;

                let (tick_x, tick_w) = match band.position {
                    delinea::AxisPosition::Left => (baseline_x - tick_len, tick_len),
                    delinea::AxisPosition::Right => (baseline_x, tick_len),
                    _ => (baseline_x, tick_len),
                };

                cx.scene.push(SceneOp::Quad {
                    order: axis_order,
                    rect: Rect::new(
                        Point::new(Px(tick_x), Px(y_px - 0.5 * line_w)),
                        Size::new(Px(tick_w), Px(line_w)),
                    ),
                    background: self.style.axis_tick_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });

                let prepared =
                    self.axis_text
                        .prepare(cx.services, &label, &text_style, constraints);
                let blob = prepared.blob;
                let metrics = prepared.metrics;

                let label_x = match band.position {
                    delinea::AxisPosition::Left => {
                        band.rect.origin.x.0
                            + (band.rect.size.width.0 - metrics.size.width.0 - 4.0).max(0.0)
                    }
                    delinea::AxisPosition::Right => band.rect.origin.x.0 + 4.0,
                    _ => band.rect.origin.x.0 + 4.0,
                };
                let label_y = y_px - metrics.size.height.0 * 0.5;

                let gap = 2.0;
                let bottom = label_y + metrics.size.height.0;
                if label_y >= last_bottom + gap {
                    cx.scene.push(SceneOp::Text {
                        order: label_order,
                        origin: Point::new(Px(label_x), Px(label_y)),
                        text: blob,
                        color: self.style.axis_label_color,
                    });
                    last_bottom = bottom;
                }
            }
        }
    }

    fn rebuild_paths_if_needed<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let marks_rev = self.with_engine(|engine| engine.output().marks.revision);
        let scale_factor_bits = cx.scale_factor.to_bits();

        if marks_rev == self.last_marks_rev && scale_factor_bits == self.last_scale_factor_bits {
            return;
        }
        self.last_marks_rev = marks_rev;
        self.last_scale_factor_bits = scale_factor_bits;

        self.path_cache.clear(cx.services);
        self.cached_paths.clear();
        self.cached_rects.clear();
        self.cached_points.clear();
        self.cached_rect_scene_ops.clear();
        self.cached_point_scene_ops.clear();

        let plot_h = self.last_layout.plot.size.height.0;
        let area_series: Vec<(delinea::SeriesId, delinea::AxisId, delinea::AreaBaseline)> = self
            .with_engine(|engine| {
                let model = engine.model();
                model
                    .series_in_order()
                    .filter(|s| s.kind == delinea::SeriesKind::Area && s.visible)
                    .filter(|s| self.series_is_in_view_grid(model, s.id))
                    .map(|s| (s.id, s.y_axis, s.area_baseline))
                    .collect()
            });

        let mut area_baseline_y_local: BTreeMap<delinea::SeriesId, f32> = BTreeMap::new();
        for (series_id, y_axis, baseline) in area_series {
            let y = match baseline {
                delinea::AreaBaseline::AxisMin => plot_h,
                delinea::AreaBaseline::Zero => {
                    let y_window = self.current_window_y(y_axis);
                    Self::y_local_for_data_value(y_window, 0.0, plot_h)
                }
                delinea::AreaBaseline::Value(value) => {
                    let y_window = self.current_window_y(y_axis);
                    Self::y_local_for_data_value(y_window, value, plot_h)
                }
            };
            area_baseline_y_local.insert(series_id, y);
        }

        let origin = self.last_layout.plot.origin;
        let (marks, model) =
            self.with_engine(|engine| (engine.output().marks.clone(), engine.model().clone()));

        self.series_rank_by_id.clear();
        for (i, series_id) in model.series_order.iter().enumerate() {
            self.series_rank_by_id.insert(*series_id, i);
        }

        #[derive(Default)]
        struct BandSegment {
            lower: Option<Range<usize>>,
            upper: Option<Range<usize>>,
            lower_id: Option<delinea::ids::MarkId>,
        }

        let mut band_segments: BTreeMap<delinea::SeriesId, Vec<BandSegment>> = BTreeMap::new();

        for node in &marks.nodes {
            if let Some(series_id) = node.source_series
                && !self.series_is_in_view_grid(&model, series_id)
            {
                continue;
            }

            if node.kind != MarkKind::Polyline {
                continue;
            }

            let MarkPayloadRef::Polyline(poly) = &node.payload else {
                continue;
            };

            let series_kind = node
                .source_series
                .and_then(|id| model.series.get(&id).map(|s| s.kind));

            let is_stacked_area = series_kind == Some(delinea::SeriesKind::Area)
                && node
                    .source_series
                    .is_some_and(|id| model.series.get(&id).is_some_and(|s| s.stack.is_some()));

            if (series_kind == Some(delinea::SeriesKind::Band) || is_stacked_area)
                && let Some(series_id) = node.source_series
            {
                let variant = delinea::ids::mark_variant(node.id);
                if variant < 1 {
                    continue;
                }
                let segment = ((variant - 1) / 2) as usize;
                let role = ((variant - 1) % 2) as u8;

                let segments = band_segments.entry(series_id).or_default();
                if segments.len() <= segment {
                    segments.resize_with(segment + 1, BandSegment::default);
                }
                let entry = &mut segments[segment];
                if role == 0 {
                    entry.lower = Some(poly.points.clone());
                    entry.lower_id = Some(node.id);
                } else {
                    entry.upper = Some(poly.points.clone());
                }
            }

            let baseline_y_local = node.source_series.and_then(|id| {
                let series = model.series.get(&id)?;
                if series.kind == delinea::SeriesKind::Area && series.stack.is_some() {
                    return None;
                }
                area_baseline_y_local.get(&id).copied()
            });

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

            let constraints = PathConstraints {
                scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
            };

            let _ = self.path_cache.prepare(
                cx.services,
                mark_path_cache_key(node.id, 0),
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                constraints,
            );

            let mut fill_prepared = false;
            if let Some(baseline_y_local) = baseline_y_local {
                let mut fill_commands: Vec<PathCommand> = Vec::with_capacity(commands.len() + 4);
                fill_commands.extend_from_slice(&commands);

                if let (Some(first), Some(last)) = (
                    marks.arena.points.get(start),
                    marks.arena.points.get(end.saturating_sub(1)),
                ) {
                    let last_x_local = last.x.0 - origin.x.0;
                    let first_x_local = first.x.0 - origin.x.0;
                    fill_commands.push(PathCommand::LineTo(fret_core::Point::new(
                        Px(last_x_local),
                        Px(baseline_y_local),
                    )));
                    fill_commands.push(PathCommand::LineTo(fret_core::Point::new(
                        Px(first_x_local),
                        Px(baseline_y_local),
                    )));
                    fill_commands.push(PathCommand::Close);

                    let _ = self.path_cache.prepare(
                        cx.services,
                        mark_path_cache_key(node.id, 1),
                        &fill_commands,
                        PathStyle::Fill(fret_core::FillStyle::default()),
                        constraints,
                    );
                    fill_prepared = true;
                }
            };
            let fill_alpha = fill_prepared.then_some(self.style.area_fill_color.a);

            let mark_id = node.id;
            self.cached_paths.insert(
                mark_id,
                CachedPath {
                    fill_alpha,
                    order: node.order.0,
                    source_series: node.source_series,
                },
            );
        }

        for (series_id, segments) in band_segments {
            for seg in segments {
                let (Some(lower_range), Some(upper_range), Some(lower_id)) =
                    (seg.lower, seg.upper, seg.lower_id)
                else {
                    continue;
                };

                if upper_range.end <= upper_range.start || lower_range.end <= lower_range.start {
                    continue;
                }
                if upper_range.end > marks.arena.points.len()
                    || lower_range.end > marks.arena.points.len()
                {
                    continue;
                }

                let upper_points = &marks.arena.points[upper_range.start..upper_range.end];
                let lower_points = &marks.arena.points[lower_range.start..lower_range.end];
                if upper_points.len() < 2 || lower_points.len() < 2 {
                    continue;
                }

                let mut fill_commands: Vec<PathCommand> =
                    Vec::with_capacity(upper_points.len() + lower_points.len() + 1);
                let first = upper_points[0];
                fill_commands.push(PathCommand::MoveTo(fret_core::Point::new(
                    Px(first.x.0 - origin.x.0),
                    Px(first.y.0 - origin.y.0),
                )));
                for p in &upper_points[1..] {
                    fill_commands.push(PathCommand::LineTo(fret_core::Point::new(
                        Px(p.x.0 - origin.x.0),
                        Px(p.y.0 - origin.y.0),
                    )));
                }
                for p in lower_points.iter().rev() {
                    fill_commands.push(PathCommand::LineTo(fret_core::Point::new(
                        Px(p.x.0 - origin.x.0),
                        Px(p.y.0 - origin.y.0),
                    )));
                }
                fill_commands.push(PathCommand::Close);

                let fill_alpha = match model.series.get(&series_id).map(|s| s.kind) {
                    Some(delinea::SeriesKind::Band) => self.style.band_fill_color.a,
                    Some(delinea::SeriesKind::Area) => self.style.area_fill_color.a,
                    _ => self.style.area_fill_color.a,
                };

                let constraints = PathConstraints {
                    scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
                };

                if self.cached_paths.contains_key(&lower_id) {
                    let _ = self.path_cache.prepare(
                        cx.services,
                        mark_path_cache_key(lower_id, 1),
                        &fill_commands,
                        PathStyle::Fill(fret_core::FillStyle::default()),
                        constraints,
                    );

                    if let Some(cached) = self.cached_paths.get_mut(&lower_id) {
                        cached.fill_alpha = Some(fill_alpha);
                    }
                }
            }
        }

        for node in &marks.nodes {
            if let Some(series_id) = node.source_series
                && !self.series_is_in_view_grid(&model, series_id)
            {
                continue;
            }

            if node.kind != MarkKind::Rect {
                continue;
            }
            let MarkPayloadRef::Rect(rects) = &node.payload else {
                continue;
            };
            let start = rects.rects.start;
            let end = rects.rects.end;
            if end <= start || end > marks.arena.rects.len() {
                continue;
            }
            self.cached_rects.reserve(end - start);
            let stroke_width = rects
                .stroke
                .as_ref()
                .map(|(_, s)| s.width)
                .filter(|w| w.0.is_finite() && w.0 > 0.0);
            for rect in &marks.arena.rects[start..end] {
                self.cached_rects.push(CachedRect {
                    rect: *rect,
                    order: node.order.0,
                    source_series: node.source_series,
                    fill: rects.fill,
                    opacity_mul: rects.opacity_mul.unwrap_or(1.0),
                    stroke_width,
                });
            }
        }

        for node in &marks.nodes {
            if let Some(series_id) = node.source_series
                && !self.series_is_in_view_grid(&model, series_id)
            {
                continue;
            }

            if node.kind != MarkKind::Points {
                continue;
            }
            let MarkPayloadRef::Points(points) = &node.payload else {
                continue;
            };
            let start = points.points.start;
            let end = points.points.end;
            if end <= start || end > marks.arena.points.len() {
                continue;
            }
            self.cached_points.reserve(end - start);
            let stroke_width = points
                .stroke
                .as_ref()
                .map(|(_, s)| s.width)
                .filter(|w| w.0.is_finite() && w.0 > 0.0);
            for p in &marks.arena.points[start..end] {
                let radius_mul = points
                    .radius_mul
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .unwrap_or(1.0);
                self.cached_points.push(CachedPoint {
                    point: *p,
                    order: node.order.0,
                    source_series: node.source_series,
                    fill: points.fill,
                    opacity_mul: points.opacity_mul.unwrap_or(1.0),
                    radius_mul,
                    stroke_width,
                });
            }
        }
    }
}

impl<H: UiHost> Widget<H> for ChartCanvas {
    fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        self.force_uncached_paint.then_some(Transform2D::IDENTITY)
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                let lock_mods_ok = !modifiers.alt && !modifiers.alt_gr && !modifiers.meta;
                let legend_mods_ok =
                    modifiers.ctrl && !modifiers.alt && !modifiers.alt_gr && !modifiers.meta;
                let legend_pos = self.last_pointer_pos;
                let in_legend = legend_pos.is_some_and(|pos| {
                    self.legend_panel_rect
                        .is_some_and(|rect| rect.contains(pos))
                }) || self.legend_hover.is_some()
                    || self.legend_selector_hover.is_some();

                if lock_mods_ok && *key == KeyCode::KeyL {
                    let Some(pos) = self.last_pointer_pos else {
                        return;
                    };

                    let toggle_pan = modifiers.shift && !modifiers.ctrl;
                    let toggle_zoom = modifiers.ctrl && !modifiers.shift;
                    let toggle_both = !toggle_pan && !toggle_zoom;

                    let layout = self.compute_layout(cx.bounds);
                    self.update_active_axes_for_position(&layout, pos);
                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };
                    match Self::axis_region(&layout, pos) {
                        AxisRegion::XAxis(axis) => {
                            if toggle_both || toggle_pan {
                                self.with_engine_mut(|engine| {
                                    engine.apply_action(Action::ToggleAxisPanLock { axis });
                                });
                            }
                            if toggle_both || toggle_zoom {
                                self.with_engine_mut(|engine| {
                                    engine.apply_action(Action::ToggleAxisZoomLock { axis });
                                });
                            }
                        }
                        AxisRegion::YAxis(axis) => {
                            if toggle_both || toggle_pan {
                                self.with_engine_mut(|engine| {
                                    engine.apply_action(Action::ToggleAxisPanLock { axis });
                                });
                            }
                            if toggle_both || toggle_zoom {
                                self.with_engine_mut(|engine| {
                                    engine.apply_action(Action::ToggleAxisZoomLock { axis });
                                });
                            }
                        }
                        AxisRegion::Plot => {
                            if toggle_both || toggle_pan {
                                self.with_engine_mut(|engine| {
                                    engine.apply_action(Action::ToggleAxisPanLock { axis: x_axis });
                                    engine.apply_action(Action::ToggleAxisPanLock { axis: y_axis });
                                });
                            }
                            if toggle_both || toggle_zoom {
                                self.with_engine_mut(|engine| {
                                    engine
                                        .apply_action(Action::ToggleAxisZoomLock { axis: x_axis });
                                    engine
                                        .apply_action(Action::ToggleAxisZoomLock { axis: y_axis });
                                });
                            }
                        }
                    }

                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    self.clear_brush();
                    self.clear_slider_drag();
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if legend_mods_ok && in_legend {
                    let mut changed = false;
                    if modifiers.shift && *key == KeyCode::KeyA {
                        changed = self.apply_legend_select_none();
                    } else if !modifiers.shift && *key == KeyCode::KeyA {
                        changed = self.apply_legend_select_all();
                    } else if !modifiers.shift && *key == KeyCode::KeyI {
                        changed = self.apply_legend_invert();
                    }

                    if changed {
                        self.legend_anchor = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                if plain && *key == KeyCode::KeyR {
                    let layout = self.compute_layout(cx.bounds);
                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };
                    self.reset_view_for_axes(x_axis, y_axis);
                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    self.clear_brush();
                    self.clear_slider_drag();
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if plain && *key == KeyCode::KeyF {
                    let layout = self.compute_layout(cx.bounds);
                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };
                    self.fit_view_to_data_for_axes(x_axis, y_axis);
                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    self.clear_brush();
                    self.clear_slider_drag();
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if plain && *key == KeyCode::KeyM {
                    let layout = self.compute_layout(cx.bounds);
                    let Some((x_axis, _y_axis)) = self.active_axes(&layout) else {
                        return;
                    };

                    self.toggle_data_window_x_filter_mode(x_axis);
                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    self.clear_brush();
                    self.clear_slider_drag();
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if plain && *key == KeyCode::KeyA {
                    self.clear_brush();
                    self.clear_slider_drag();
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            Event::Pointer(PointerEvent::Move {
                position, buttons, ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let layout = self.compute_layout(cx.bounds);
                self.update_active_axes_for_position(&layout, *position);

                let prev_series_hover = self.legend_hover;
                let prev_selector_hover = self.legend_selector_hover;
                self.legend_selector_hover = self.legend_selector_at(*position);
                self.legend_hover = if self.legend_selector_hover.is_some() {
                    None
                } else {
                    self.legend_series_at(*position)
                };
                if self.legend_hover != prev_series_hover
                    || self.legend_selector_hover != prev_selector_hover
                {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                if cx.captured == Some(cx.node) {
                    if let Some(drag) = self.visual_map_drag
                        && Self::is_button_held(MouseButton::Left, *buttons)
                    {
                        let current_value = delinea::engine::axis::data_at_y_in_rect(
                            drag.domain,
                            position.y.0,
                            drag.track,
                        );
                        let delta_value = current_value - drag.start_value;
                        let window = Self::slider_window_after_delta(
                            drag.domain,
                            drag.start_window,
                            delta_value,
                            drag.kind,
                        );
                        self.with_engine_mut(|engine| {
                            engine.apply_action(Action::SetVisualMapRange {
                                visual_map: drag.visual_map,
                                range: Some((window.min, window.max)),
                            });
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    if let Some(drag) = self.slider_drag
                        && Self::is_button_held(MouseButton::Left, *buttons)
                    {
                        let track = drag.track;
                        let extent = drag.extent;
                        let span = extent.span();
                        match drag.axis_kind {
                            SliderAxisKind::X => {
                                if track.size.width.0 > 0.0 && span.is_finite() && span > 0.0 {
                                    let x = position.x.0.clamp(
                                        track.origin.x.0,
                                        track.origin.x.0 + track.size.width.0,
                                    );
                                    let start_x = drag.start_pos.x.0.clamp(
                                        track.origin.x.0,
                                        track.origin.x.0 + track.size.width.0,
                                    );
                                    let delta_px = x - start_x;
                                    let delta_value = (delta_px / track.size.width.0) as f64 * span;

                                    let window = Self::slider_window_after_delta(
                                        extent,
                                        drag.start_window,
                                        delta_value,
                                        drag.kind,
                                    );
                                    let anchor = match drag.kind {
                                        SliderDragKind::HandleMin => WindowSpanAnchor::LockMax,
                                        SliderDragKind::HandleMax => WindowSpanAnchor::LockMin,
                                        SliderDragKind::Pan => WindowSpanAnchor::Center,
                                    };
                                    self.with_engine_mut(|engine| {
                                        engine.apply_action(Action::SetDataWindowXFromZoom {
                                            axis: drag.axis,
                                            base: drag.start_window,
                                            window,
                                            anchor,
                                        });
                                    });

                                    self.slider_drag = Some(DataZoomSliderDrag {
                                        start_pos: *position,
                                        start_window: window,
                                        ..drag
                                    });
                                    cx.invalidate_self(Invalidation::Paint);
                                    cx.request_redraw();
                                    cx.stop_propagation();
                                    return;
                                }
                            }
                            SliderAxisKind::Y => {
                                if track.size.height.0 > 0.0 && span.is_finite() && span > 0.0 {
                                    let height = track.size.height.0;
                                    let bottom = track.origin.y.0 + height;

                                    let y = position.y.0.clamp(track.origin.y.0, bottom);
                                    let start_y =
                                        drag.start_pos.y.0.clamp(track.origin.y.0, bottom);

                                    let y_from_bottom = bottom - y;
                                    let start_from_bottom = bottom - start_y;
                                    let delta_px = y_from_bottom - start_from_bottom;
                                    let delta_value = (delta_px / height) as f64 * span;

                                    let window = Self::slider_window_after_delta(
                                        extent,
                                        drag.start_window,
                                        delta_value,
                                        drag.kind,
                                    );
                                    let anchor = match drag.kind {
                                        SliderDragKind::HandleMin => WindowSpanAnchor::LockMax,
                                        SliderDragKind::HandleMax => WindowSpanAnchor::LockMin,
                                        SliderDragKind::Pan => WindowSpanAnchor::Center,
                                    };
                                    self.with_engine_mut(|engine| {
                                        engine.apply_action(Action::SetDataWindowYFromZoom {
                                            axis: drag.axis,
                                            base: drag.start_window,
                                            window,
                                            anchor,
                                        });
                                    });

                                    self.slider_drag = Some(DataZoomSliderDrag {
                                        start_pos: *position,
                                        start_window: window,
                                        ..drag
                                    });
                                    cx.invalidate_self(Invalidation::Paint);
                                    cx.request_redraw();
                                    cx.stop_propagation();
                                    return;
                                }
                            }
                        }
                        return;
                    }

                    if let Some(mut drag) = self.box_zoom_drag
                        && Self::is_button_held(drag.button, *buttons)
                    {
                        drag.current_pos = *position;
                        self.box_zoom_drag = Some(drag);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    if let Some(mut drag) = self.brush_drag
                        && Self::is_button_held(drag.button, *buttons)
                    {
                        drag.current_pos = *position;
                        self.brush_drag = Some(drag);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    if let Some(drag) = self.pan_drag
                        && buttons.left
                    {
                        let layout = self.compute_layout(cx.bounds);
                        let width = layout.plot.size.width.0;
                        let height = layout.plot.size.height.0;
                        if width <= 0.0 || height <= 0.0 {
                            return;
                        }

                        let dx = position.x.0 - drag.start_pos.x.0;
                        let dy = position.y.0 - drag.start_pos.y.0;

                        let (x_pan_locked, y_pan_locked) = self.with_engine(|engine| {
                            let x_pan_locked = engine
                                .state()
                                .axis_locks
                                .get(&drag.x_axis)
                                .copied()
                                .unwrap_or_default()
                                .pan_locked;
                            let y_pan_locked = engine
                                .state()
                                .axis_locks
                                .get(&drag.y_axis)
                                .copied()
                                .unwrap_or_default()
                                .pan_locked;
                            (x_pan_locked, y_pan_locked)
                        });

                        if drag.pan_x && self.axis_is_fixed(drag.x_axis).is_none() && !x_pan_locked
                        {
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::PanDataWindowXFromBase {
                                    axis: drag.x_axis,
                                    base: drag.start_x,
                                    delta_px: dx,
                                    viewport_span_px: width,
                                });
                            });
                        }
                        if drag.pan_y && self.axis_is_fixed(drag.y_axis).is_none() && !y_pan_locked
                        {
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::PanDataWindowYFromBase {
                                    axis: drag.y_axis,
                                    base: drag.start_y,
                                    delta_px: -dy,
                                    viewport_span_px: height,
                                });
                            });
                        }

                        self.refresh_hover_for_axis_pointer(&layout, *position);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                let hover_point = Self::axis_pointer_hover_point(&layout, *position);
                self.with_engine_mut(|engine| {
                    engine.apply_action(Action::HoverAt { point: hover_point });
                });
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                pointer_type,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let layout = self.compute_layout(cx.bounds);
                self.update_active_axes_for_position(&layout, *position);

                if *button == MouseButton::Left
                    && self.pan_drag.is_none()
                    && self.box_zoom_drag.is_none()
                    && let Some(action) = self.legend_selector_at(*position)
                {
                    let _changed = match action {
                        LegendSelectorAction::All => self.apply_legend_select_all(),
                        LegendSelectorAction::None => self.apply_legend_select_none(),
                        LegendSelectorAction::Invert => self.apply_legend_invert(),
                    };
                    self.legend_anchor = None;
                    self.legend_hover = None;
                    self.legend_selector_hover = Some(action);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *button == MouseButton::Left
                    && self.pan_drag.is_none()
                    && self.box_zoom_drag.is_none()
                    && let Some(series) = self.legend_series_at(*position)
                {
                    if *click_count >= 2 {
                        self.apply_legend_double_click(series);
                    } else {
                        if modifiers.shift
                            && let Some(anchor) = self.legend_anchor
                        {
                            self.apply_legend_shift_range_toggle(anchor, series);
                        } else {
                            let visible = self.with_engine(|engine| {
                                engine
                                    .model()
                                    .series
                                    .get(&series)
                                    .map(|s| s.visible)
                                    .unwrap_or(true)
                            });
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::SetSeriesVisible {
                                    series,
                                    visible: !visible,
                                });
                            });
                        }
                    }
                    self.legend_anchor = Some(series);
                    self.legend_hover = Some(series);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *button == MouseButton::Right
                    && self.pan_drag.is_none()
                    && self.box_zoom_drag.is_none()
                    && self
                        .legend_panel_rect
                        .is_some_and(|r| r.contains(*position))
                {
                    self.apply_legend_reset();
                    self.legend_anchor = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *pointer_type == PointerType::Mouse
                    && cx.captured.is_none()
                    && self.pan_drag.is_none()
                    && self.box_zoom_drag.is_none()
                    && self.brush_drag.is_none()
                    && self.slider_drag.is_none()
                    && let Some((vm_id, vm, track)) = self.visual_map_track_at(*position)
                    && (*button == MouseButton::Left
                        || (vm.mode == delinea::VisualMapMode::Piecewise
                            && *button == MouseButton::Right))
                {
                    let domain = Self::visual_map_domain_window(vm);
                    let click_value =
                        delinea::engine::axis::data_at_y_in_rect(domain, position.y.0, track);

                    if vm.mode == delinea::VisualMapMode::Piecewise {
                        let buckets = vm.buckets.clamp(1, 64) as u32;
                        let full_mask = if buckets >= 64 {
                            u64::MAX
                        } else {
                            (1u64 << buckets) - 1
                        };

                        let bucket =
                            delinea::visual_map::bucket_index_for_value(&vm, click_value) as u32;
                        let bit = 1u64 << bucket.min(63);
                        let current = self.current_visual_map_piece_mask(vm_id, vm);

                        let wants_reset = (*button == MouseButton::Right
                            && !modifiers.alt
                            && !modifiers.ctrl
                            && !modifiers.meta
                            && !modifiers.alt_gr)
                            || (*button == MouseButton::Left && *click_count == 2);
                        if wants_reset {
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::SetVisualMapPieceMask {
                                    visual_map: vm_id,
                                    mask: None,
                                });
                            });
                            self.visual_map_piece_anchor = None;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }

                        let is_selected = ((current >> bucket) & 1) == 1;

                        let mut next = current;
                        if modifiers.shift {
                            if let Some((anchor_vm, anchor_bucket)) = self.visual_map_piece_anchor
                                && anchor_vm == vm_id
                            {
                                let lo = anchor_bucket.min(bucket);
                                let hi = anchor_bucket.max(bucket).min(buckets.saturating_sub(1));
                                let width = hi.saturating_sub(lo).saturating_add(1);
                                let range_mask = if width >= 64 {
                                    u64::MAX
                                } else {
                                    ((1u64 << width) - 1) << lo
                                } & full_mask;

                                if is_selected {
                                    next &= !range_mask;
                                } else {
                                    next |= range_mask;
                                }
                            } else {
                                next ^= bit;
                            }
                        } else {
                            next ^= bit;
                        }
                        next &= full_mask;
                        let mask = (next != full_mask).then_some(next);
                        self.with_engine_mut(|engine| {
                            engine.apply_action(Action::SetVisualMapPieceMask {
                                visual_map: vm_id,
                                mask,
                            });
                        });
                        self.visual_map_piece_anchor = Some((vm_id, bucket));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    let current_window = self.current_visual_map_window(vm_id, vm);

                    let handle_hit_px = 8.0f32;
                    let y_min = Self::visual_map_y_at_value(track, domain, current_window.min);
                    let y_max = Self::visual_map_y_at_value(track, domain, current_window.max);
                    let (top, bottom) = (y_max.min(y_min), y_max.max(y_min));

                    let (kind, start_window) = if (position.y.0 - y_min).abs() <= handle_hit_px {
                        (SliderDragKind::HandleMin, current_window)
                    } else if (position.y.0 - y_max).abs() <= handle_hit_px {
                        (SliderDragKind::HandleMax, current_window)
                    } else if position.y.0 >= top && position.y.0 <= bottom {
                        (SliderDragKind::Pan, current_window)
                    } else {
                        let center = (current_window.min + current_window.max) * 0.5;
                        let delta = click_value - center;
                        (
                            SliderDragKind::Pan,
                            Self::slider_window_after_delta(
                                domain,
                                current_window,
                                delta,
                                SliderDragKind::Pan,
                            ),
                        )
                    };

                    self.with_engine_mut(|engine| {
                        engine.apply_action(Action::SetVisualMapRange {
                            visual_map: vm_id,
                            range: Some((start_window.min, start_window.max)),
                        });
                    });
                    self.visual_map_drag = Some(VisualMapDrag {
                        visual_map: vm_id,
                        kind,
                        track,
                        domain,
                        start_window,
                        start_value: click_value,
                    });

                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *pointer_type == PointerType::Mouse
                    && *button == MouseButton::Left
                    && *click_count == 2
                    && !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta
                {
                    let layout = self.compute_layout(cx.bounds);
                    match Self::axis_region(&layout, *position) {
                        AxisRegion::XAxis(axis) => {
                            self.active_x_axis = Some(axis);
                            if self.axis_is_fixed(axis).is_none() {
                                self.set_data_window_x(axis, None);
                            }
                        }
                        AxisRegion::YAxis(axis) => {
                            self.active_y_axis = Some(axis);
                            if self.axis_is_fixed(axis).is_none() {
                                self.set_data_window_y(axis, None);
                            }
                        }
                        AxisRegion::Plot => {
                            let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                                return;
                            };
                            self.reset_view_for_axes(x_axis, y_axis);
                        }
                    }

                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(cancel) = self.input_map.box_zoom_cancel
                    && self.box_zoom_drag.is_some()
                    && cancel.matches(*button, *modifiers)
                {
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.input_map.axis_lock_toggle.matches(*button, *modifiers) {
                    let layout = self.compute_layout(cx.bounds);
                    self.update_active_axes_for_position(&layout, *position);
                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };
                    match Self::axis_region(&layout, *position) {
                        AxisRegion::XAxis(axis) => {
                            self.active_x_axis = Some(axis);
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::ToggleAxisPanLock { axis });
                                engine.apply_action(Action::ToggleAxisZoomLock { axis });
                            });
                        }
                        AxisRegion::YAxis(axis) => {
                            self.active_y_axis = Some(axis);
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::ToggleAxisPanLock { axis });
                                engine.apply_action(Action::ToggleAxisZoomLock { axis });
                            });
                        }
                        AxisRegion::Plot => {
                            self.with_engine_mut(|engine| {
                                engine.apply_action(Action::ToggleAxisPanLock { axis: x_axis });
                                engine.apply_action(Action::ToggleAxisZoomLock { axis: x_axis });
                                engine.apply_action(Action::ToggleAxisPanLock { axis: y_axis });
                                engine.apply_action(Action::ToggleAxisZoomLock { axis: y_axis });
                            });
                        }
                    }

                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.pan_drag.is_some() || self.box_zoom_drag.is_some() {
                    return;
                }
                if self.brush_drag.is_some() {
                    return;
                }
                if self.slider_drag.is_some() {
                    return;
                }

                // Slider interaction: allow left-drag in the slider track.
                if *button == MouseButton::Left {
                    let layout = self.compute_layout(cx.bounds);
                    let region = Self::axis_region(&layout, *position);
                    match region {
                        AxisRegion::XAxis(axis) => {
                            let zoom_locked = self.with_engine(|engine| {
                                engine
                                    .state()
                                    .axis_locks
                                    .get(&axis)
                                    .copied()
                                    .unwrap_or_default()
                                    .zoom_locked
                            });
                            if zoom_locked || self.axis_is_fixed(axis).is_some() {
                                return;
                            }

                            let (locked_min, locked_max) = self.axis_constraints(axis);
                            let can_pan = locked_min.is_none() && locked_max.is_none();
                            let can_handle_min = locked_min.is_none();
                            let can_handle_max = locked_max.is_none();

                            if let Some(track) = self.x_slider_track_for_axis(axis)
                                && track.contains(*position)
                            {
                                let extent = self.compute_axis_extent_from_data(axis, true);
                                let window = self.current_window_x_for_slider(axis, extent);

                                let t0 = Self::slider_norm(extent, window.min);
                                let t1 = Self::slider_norm(extent, window.max);
                                let left = track.origin.x.0 + t0 * track.size.width.0;
                                let right = track.origin.x.0 + t1 * track.size.width.0;

                                let handle_hit_px = 7.0f32;
                                let x = position.x.0;
                                let kind = if (x - left).abs() <= handle_hit_px {
                                    SliderDragKind::HandleMin
                                } else if (x - right).abs() <= handle_hit_px {
                                    SliderDragKind::HandleMax
                                } else if x >= left && x <= right {
                                    SliderDragKind::Pan
                                } else {
                                    // Jump: center the current span around the click and drag as pan.
                                    SliderDragKind::Pan
                                };

                                if matches!(kind, SliderDragKind::Pan) && !can_pan {
                                    return;
                                }
                                if matches!(kind, SliderDragKind::HandleMin) && !can_handle_min {
                                    return;
                                }
                                if matches!(kind, SliderDragKind::HandleMax) && !can_handle_max {
                                    return;
                                }

                                let start_window = if matches!(kind, SliderDragKind::Pan)
                                    && !(x >= left && x <= right)
                                {
                                    let click_value = Self::slider_value_at(track, extent, x);
                                    let half = 0.5 * window.span();
                                    let start_window = DataWindow {
                                        min: click_value - half,
                                        max: click_value + half,
                                    };
                                    Self::slider_window_after_delta(
                                        extent,
                                        start_window,
                                        0.0,
                                        SliderDragKind::Pan,
                                    )
                                } else {
                                    window
                                };

                                self.slider_drag = Some(DataZoomSliderDrag {
                                    axis_kind: SliderAxisKind::X,
                                    axis,
                                    kind,
                                    track,
                                    extent,
                                    start_pos: *position,
                                    start_window,
                                });

                                cx.request_focus(cx.node);
                                cx.capture_pointer(cx.node);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                                return;
                            }
                        }
                        AxisRegion::YAxis(axis) => {
                            let zoom_locked = self.with_engine(|engine| {
                                engine
                                    .state()
                                    .axis_locks
                                    .get(&axis)
                                    .copied()
                                    .unwrap_or_default()
                                    .zoom_locked
                            });
                            if zoom_locked || self.axis_is_fixed(axis).is_some() {
                                return;
                            }

                            let (locked_min, locked_max) = self.axis_constraints(axis);
                            let can_pan = locked_min.is_none() && locked_max.is_none();
                            let can_handle_min = locked_min.is_none();
                            let can_handle_max = locked_max.is_none();

                            if let Some(track) = self.y_slider_track_for_axis(axis)
                                && track.contains(*position)
                            {
                                let extent = self.compute_axis_extent_from_data(axis, false);
                                let window = self.current_window_y_for_slider(axis, extent);

                                let t0 = Self::slider_norm(extent, window.min);
                                let t1 = Self::slider_norm(extent, window.max);

                                let handle_hit_px = 7.0f32;
                                let height = track.size.height.0;
                                let bottom = track.origin.y.0 + height;
                                let y_from_bottom =
                                    (bottom - position.y.0).clamp(0.0, height.max(1.0));

                                let min_handle = t0 * height;
                                let max_handle = t1 * height;

                                let kind = if (y_from_bottom - min_handle).abs() <= handle_hit_px {
                                    SliderDragKind::HandleMin
                                } else if (y_from_bottom - max_handle).abs() <= handle_hit_px {
                                    SliderDragKind::HandleMax
                                } else if y_from_bottom >= min_handle && y_from_bottom <= max_handle
                                {
                                    SliderDragKind::Pan
                                } else {
                                    // Jump: center the current span around the click and drag as pan.
                                    SliderDragKind::Pan
                                };

                                if matches!(kind, SliderDragKind::Pan) && !can_pan {
                                    return;
                                }
                                if matches!(kind, SliderDragKind::HandleMin) && !can_handle_min {
                                    return;
                                }
                                if matches!(kind, SliderDragKind::HandleMax) && !can_handle_max {
                                    return;
                                }

                                let start_window = if matches!(kind, SliderDragKind::Pan)
                                    && !(y_from_bottom >= min_handle && y_from_bottom <= max_handle)
                                {
                                    let click_value =
                                        Self::slider_value_at_y(track, extent, position.y.0);
                                    let half = 0.5 * window.span();
                                    let start_window = DataWindow {
                                        min: click_value - half,
                                        max: click_value + half,
                                    };
                                    Self::slider_window_after_delta(
                                        extent,
                                        start_window,
                                        0.0,
                                        SliderDragKind::Pan,
                                    )
                                } else {
                                    window
                                };

                                self.slider_drag = Some(DataZoomSliderDrag {
                                    axis_kind: SliderAxisKind::Y,
                                    axis,
                                    kind,
                                    track,
                                    extent,
                                    start_pos: *position,
                                    start_window,
                                });

                                cx.request_focus(cx.node);
                                cx.capture_pointer(cx.node);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                                return;
                            }
                        }
                        AxisRegion::Plot => {}
                    }
                }

                let start_box_primary = self.input_map.box_zoom.matches(*button, *modifiers);
                let start_box_alt = self
                    .input_map
                    .box_zoom_alt
                    .is_some_and(|chord| chord.matches(*button, *modifiers));
                if start_box_primary || start_box_alt {
                    let layout = self.compute_layout(cx.bounds);
                    if !layout.plot.contains(*position) {
                        return;
                    }

                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };

                    let (x_zoom_locked, y_zoom_locked) = self.with_engine(|engine| {
                        let x_zoom_locked = engine
                            .state()
                            .axis_locks
                            .get(&x_axis)
                            .copied()
                            .unwrap_or_default()
                            .zoom_locked;
                        let y_zoom_locked = engine
                            .state()
                            .axis_locks
                            .get(&y_axis)
                            .copied()
                            .unwrap_or_default()
                            .zoom_locked;
                        (x_zoom_locked, y_zoom_locked)
                    });
                    if x_zoom_locked || y_zoom_locked {
                        return;
                    }

                    if self.axis_is_fixed(x_axis).is_some() || self.axis_is_fixed(y_axis).is_some()
                    {
                        return;
                    }

                    let required_mods = if start_box_primary {
                        self.input_map.box_zoom.modifiers
                    } else {
                        self.input_map
                            .box_zoom_alt
                            .unwrap_or(self.input_map.box_zoom)
                            .modifiers
                    };

                    let start_x = self.current_window_x(x_axis);
                    let start_y = self.current_window_y(y_axis);

                    self.box_zoom_drag = Some(BoxZoomDrag {
                        x_axis,
                        y_axis,
                        button: *button,
                        required_mods,
                        start_pos: *position,
                        current_pos: *position,
                        start_x,
                        start_y,
                    });

                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.input_map.brush_select.matches(*button, *modifiers) {
                    let layout = self.compute_layout(cx.bounds);
                    if !layout.plot.contains(*position) {
                        return;
                    }

                    let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                        return;
                    };

                    let start_x = self.current_window_x(x_axis);
                    let start_y = self.current_window_y(y_axis);

                    self.brush_drag = Some(BoxZoomDrag {
                        x_axis,
                        y_axis,
                        button: *button,
                        required_mods: self.input_map.brush_select.modifiers,
                        start_pos: *position,
                        current_pos: *position,
                        start_x,
                        start_y,
                    });

                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if !self.input_map.pan.matches(*button, *modifiers) {
                    return;
                }

                let layout = self.compute_layout(cx.bounds);
                let region = Self::axis_region(&layout, *position);
                let in_plot = layout.plot.contains(*position);
                let in_axis = matches!(region, AxisRegion::XAxis(_) | AxisRegion::YAxis(_));
                if !in_plot && !in_axis {
                    return;
                }

                let Some((x_axis, y_axis)) = self.active_axes(&layout) else {
                    return;
                };
                let (x_axis, y_axis, mut pan_x, mut pan_y) = match region {
                    AxisRegion::Plot => (x_axis, y_axis, !modifiers.ctrl, !modifiers.shift),
                    AxisRegion::XAxis(axis) => (axis, y_axis, true, false),
                    AxisRegion::YAxis(axis) => (x_axis, axis, false, true),
                };

                if pan_x && self.axis_is_fixed(x_axis).is_some() {
                    pan_x = false;
                }
                if pan_y && self.axis_is_fixed(y_axis).is_some() {
                    pan_y = false;
                }

                let (x_pan_locked, y_pan_locked) = self.with_engine(|engine| {
                    let x_pan_locked = engine
                        .state()
                        .axis_locks
                        .get(&x_axis)
                        .copied()
                        .unwrap_or_default()
                        .pan_locked;
                    let y_pan_locked = engine
                        .state()
                        .axis_locks
                        .get(&y_axis)
                        .copied()
                        .unwrap_or_default()
                        .pan_locked;
                    (x_pan_locked, y_pan_locked)
                });
                if pan_x && x_pan_locked {
                    pan_x = false;
                }
                if pan_y && y_pan_locked {
                    pan_y = false;
                }
                if !pan_x && !pan_y {
                    return;
                }

                let start_x = self.current_window_x(x_axis);
                let start_y = self.current_window_y(y_axis);

                self.pan_drag = Some(PanDrag {
                    x_axis,
                    y_axis,
                    pan_x,
                    pan_y,
                    start_pos: *position,
                    start_x,
                    start_y,
                });

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                if let Some(drag) = self.box_zoom_drag
                    && drag.button == *button
                {
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }

                    let layout = self.compute_layout(cx.bounds);
                    let plot = layout.plot;
                    if let Some((x, y)) = self.selection_windows_for_drag(
                        plot,
                        drag.start_x,
                        drag.start_y,
                        drag.start_pos,
                        drag.current_pos,
                        *modifiers,
                        drag.required_mods,
                    ) {
                        let x_window = (self.axis_is_fixed(drag.x_axis).is_none()).then_some(x);
                        let y_window = (self.axis_is_fixed(drag.y_axis).is_none()).then_some(y);
                        self.with_engine_mut(|engine| {
                            engine.apply_action(Self::view_window_2d_action_from_zoom(
                                drag.x_axis,
                                drag.y_axis,
                                drag.start_x,
                                drag.start_y,
                                x_window,
                                y_window,
                            ));
                        });
                        self.refresh_hover_for_axis_pointer(&layout, *position);
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(drag) = self.brush_drag
                    && drag.button == *button
                {
                    self.brush_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }

                    let layout = self.compute_layout(cx.bounds);
                    let plot = layout.plot;
                    if let Some((x, y)) = self.selection_windows_for_drag(
                        plot,
                        drag.start_x,
                        drag.start_y,
                        drag.start_pos,
                        drag.current_pos,
                        *modifiers,
                        drag.required_mods,
                    ) {
                        self.with_engine_mut(|engine| {
                            engine.apply_action(Action::SetBrushSelection2D {
                                x_axis: drag.x_axis,
                                y_axis: drag.y_axis,
                                x,
                                y,
                            });
                        });
                    } else {
                        self.with_engine_mut(|engine| {
                            engine.apply_action(Action::ClearBrushSelection);
                        });
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.visual_map_drag.is_some() && *button == MouseButton::Left {
                    self.visual_map_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.slider_drag.is_some() && *button == MouseButton::Left {
                    self.slider_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.pan_drag.is_some() && *button == MouseButton::Left {
                    self.pan_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);

                if self
                    .legend_panel_rect
                    .is_some_and(|rect| rect.contains(*position))
                    && self.apply_legend_wheel_scroll(delta.y)
                {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let layout = self.compute_layout(cx.bounds);
                self.update_active_axes_for_position(&layout, *position);
                let plot = layout.plot;
                let width = plot.size.width.0;
                let height = plot.size.height.0;
                if width <= 0.0 || height <= 0.0 {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                if let Some(required) = self.input_map.wheel_zoom_mod
                    && !required.is_pressed(*modifiers)
                {
                    return;
                }

                // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
                let log2_scale = delta_y * 0.0025;

                let region = Self::axis_region(&layout, *position);
                let in_plot = plot.contains(*position);
                let in_axis = matches!(region, AxisRegion::XAxis(_) | AxisRegion::YAxis(_));
                if !in_plot && !in_axis {
                    return;
                }

                let local_x = (position.x.0 - plot.origin.x.0).clamp(0.0, width);
                let local_y = (position.y.0 - plot.origin.y.0).clamp(0.0, height);
                let center_x = local_x;
                let center_y_from_bottom = height - local_y;

                let Some((primary_x_axis, primary_y_axis)) = self.active_axes(&layout) else {
                    return;
                };

                let (x_axis, y_axis) = match region {
                    AxisRegion::XAxis(axis) => (axis, primary_y_axis),
                    AxisRegion::YAxis(axis) => (primary_x_axis, axis),
                    AxisRegion::Plot => (primary_x_axis, primary_y_axis),
                };

                let (zoom_x, zoom_y) = match region {
                    AxisRegion::XAxis(_) => (true, false),
                    AxisRegion::YAxis(_) => (false, true),
                    AxisRegion::Plot => (!modifiers.ctrl, !modifiers.shift),
                };

                if zoom_x && self.axis_is_fixed(x_axis).is_none() {
                    let w = self.current_window_x(x_axis);
                    self.with_engine_mut(|engine| {
                        engine.apply_action(Action::ZoomDataWindowXFromBase {
                            axis: x_axis,
                            base: w,
                            center_px: center_x,
                            log2_scale,
                            viewport_span_px: width,
                        });
                    });
                }
                if zoom_y && self.axis_is_fixed(y_axis).is_none() {
                    let w = self.current_window_y(y_axis);
                    self.with_engine_mut(|engine| {
                        engine.apply_action(Action::ZoomDataWindowYFromBase {
                            axis: y_axis,
                            base: w,
                            center_px: center_y_from_bottom,
                            log2_scale,
                            viewport_span_px: height,
                        });
                    });
                }

                self.refresh_hover_for_axis_pointer(&layout, *position);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        let theme = Theme::global(&*cx.app);
        self.sync_style_from_theme(theme);

        self.last_bounds = cx.bounds;
        self.last_layout = self.compute_layout(cx.bounds);
        self.sync_viewport(self.last_layout.plot);
        cx.available
    }

    fn prepaint(&mut self, cx: &mut PrepaintCx<'_, H>) {
        let mut measurer = NullTextMeasurer::default();

        // P0: run the engine synchronously, but allow multiple internal steps per frame so that
        // medium-sized datasets can produce the first set of marks without relying on external
        // redraw triggers.
        let start = Instant::now();
        let mut unfinished = true;
        let mut steps_ran = 0u32;
        while unfinished && steps_ran < 8 && start.elapsed() < Duration::from_millis(4) {
            let budget = if self.cached_paths.is_empty() && self.cached_rects.is_empty() {
                WorkBudget::new(262_144, 0, 32)
            } else {
                WorkBudget::new(32_768, 0, 8)
            };

            let step = self.with_engine_mut(|engine| engine.step(&mut measurer, budget));
            match step {
                Ok(step) => {
                    unfinished = step.unfinished;
                }
                Err(EngineError::MissingViewport | EngineError::MissingPlotViewport { .. }) => {
                    unfinished = false;
                }
            }
            steps_ran = steps_ran.saturating_add(1);
        }

        self.force_uncached_paint = unfinished;
        if unfinished {
            cx.request_animation_frame();
        }

        let next_key = self.sampling_window_key(self.last_layout.plot, cx.scale_factor);
        if next_key != self.last_sampling_window_key {
            cx.debug_record_chart_sampling_window_shift(next_key);
            self.last_sampling_window_key = next_key;
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let theme = Theme::global(&*cx.app);
        let style_changed = self.sync_style_from_theme(theme);
        if style_changed {
            self.last_bounds = cx.bounds;
            self.last_layout = self.compute_layout(cx.bounds);
            self.sync_viewport(self.last_layout.plot);
        }

        self.sync_linked_brush(cx);

        if self.last_bounds != cx.bounds
            || self.last_layout.plot.size.width.0 <= 0.0
            || self.last_layout.plot.size.height.0 <= 0.0
        {
            self.last_bounds = cx.bounds;
            self.last_layout = self.compute_layout(cx.bounds);
            self.sync_viewport(self.last_layout.plot);
        }

        // Advance per-frame counters for optional cache pruning.
        self.axis_text.begin_frame();
        self.tooltip_text.begin_frame();
        self.legend_text.begin_frame();
        self.path_cache.begin_frame();
        if let Some(window) = cx.window {
            let frame_id = cx.app.frame_id().0;
            let path_entries = self.path_cache.len();
            let path_stats = self.path_cache.stats();
            let path_key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-chart.canvas.paths",
            };

            let axis_text_entries = self.axis_text.len();
            let axis_text_stats = self.axis_text.stats();
            let axis_text_key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-chart.canvas.text.axis",
            };

            let tooltip_text_entries = self.tooltip_text.len();
            let tooltip_text_stats = self.tooltip_text.stats();
            let tooltip_text_key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-chart.canvas.text.tooltip",
            };

            let legend_text_entries = self.legend_text.len();
            let legend_text_stats = self.legend_text.stats();
            let legend_text_key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-chart.canvas.text.legend",
            };
            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_path_cache(path_key, frame_id, path_entries, path_stats);
                    registry.record_text_cache(
                        axis_text_key,
                        frame_id,
                        axis_text_entries,
                        axis_text_stats,
                    );
                    registry.record_text_cache(
                        tooltip_text_key,
                        frame_id,
                        tooltip_text_entries,
                        tooltip_text_stats,
                    );
                    registry.record_text_cache(
                        legend_text_key,
                        frame_id,
                        legend_text_entries,
                        legend_text_stats,
                    );
                });
        }

        self.rebuild_paths_if_needed(cx);
        self.clear_tooltip_text_cache(cx.services);
        self.publish_output(cx.app);

        if let Some(background) = self.style.background {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(self.style.draw_order.0.saturating_sub(1)),
                rect: self.last_layout.bounds,
                background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        cx.scene.push(SceneOp::PushClipRect {
            rect: self.last_layout.plot,
        });

        let brush = self.with_engine(|engine| engine.state().brush_selection_2d);
        let brush_rect_px = if let Some(brush) = brush {
            self.brush_rect_px(brush)
                .filter(|rect| rect.size.width.0 >= 1.0 && rect.size.height.0 >= 1.0)
        } else {
            None
        };

        #[derive(Clone, Copy)]
        struct SeriesSnapshot {
            x_axis: delinea::AxisId,
            y_axis: delinea::AxisId,
            kind: delinea::SeriesKind,
            stack: Option<delinea::StackId>,
        }

        let series_by_id: BTreeMap<delinea::SeriesId, SeriesSnapshot> =
            self.with_engine(|engine| {
                engine
                    .model()
                    .series
                    .iter()
                    .map(|(id, s)| {
                        (
                            *id,
                            SeriesSnapshot {
                                x_axis: s.x_axis,
                                y_axis: s.y_axis,
                                kind: s.kind,
                                stack: s.stack,
                            },
                        )
                    })
                    .collect()
            });

        let mut style_sig = KeyBuilder::new();
        style_sig.mix_f32_bits(self.style.stroke_color.r);
        style_sig.mix_f32_bits(self.style.stroke_color.g);
        style_sig.mix_f32_bits(self.style.stroke_color.b);
        style_sig.mix_f32_bits(self.style.stroke_color.a);
        style_sig.mix_f32_bits(self.style.bar_fill_alpha);
        style_sig.mix_f32_bits(self.style.scatter_fill_alpha);
        style_sig.mix_f32_bits(self.style.scatter_point_radius.0);
        for c in &self.style.series_palette {
            style_sig.mix_f32_bits(c.r);
            style_sig.mix_f32_bits(c.g);
            style_sig.mix_f32_bits(c.b);
            style_sig.mix_f32_bits(c.a);
        }
        let style_sig = style_sig.finish();

        let mut rect_key = KeyBuilder::new();
        rect_key.mix_u64(self.last_marks_rev.0);
        rect_key.mix_u64(u64::from(self.last_scale_factor_bits));
        rect_key.mix_u64(style_sig);
        rect_key.mix_u64(self.legend_hover.map(|v| v.0).unwrap_or(0));
        if let Some(brush) = brush {
            rect_key.mix_u64(1);
            rect_key.mix_u64(brush.x_axis.0);
            rect_key.mix_u64(brush.y_axis.0);
        } else {
            rect_key.mix_u64(0);
        }
        let rect_key = rect_key.finish();

        if !self.cached_rect_scene_ops.try_replay_with(
            rect_key,
            cx.scene,
            Point::new(Px(0.0), Px(0.0)),
            |_ops| {},
        ) {
            let mut ops: Vec<SceneOp> = Vec::with_capacity(self.cached_rects.len());
            for cached in &self.cached_rects {
                let base_order = self
                    .style
                    .draw_order
                    .0
                    .saturating_add(cached.order.saturating_mul(4));

                let mut fill_color = self.style.stroke_color;
                if let Some(paint) = cached.fill {
                    fill_color = self.paint_color(paint);
                    fill_color.a *= self.style.stroke_color.a;
                } else if let Some(series) = cached.source_series {
                    fill_color = self.series_color(series);
                    fill_color.a *= self.style.stroke_color.a;
                }
                fill_color.a *= cached.opacity_mul;
                if let Some(series_id) = cached.source_series {
                    let brush_dim = if brush.is_some() && series_by_id.contains_key(&series_id) {
                        0.25
                    } else {
                        1.0
                    };
                    fill_color.a *= brush_dim;
                    if let Some(hover) = self.legend_hover
                        && cached.source_series.is_some()
                        && cached.source_series != Some(hover)
                    {
                        fill_color.a *= 0.25;
                    }
                }
                fill_color.a *= self.style.bar_fill_alpha;

                let stroke_width = cached.stroke_width.unwrap_or(Px(0.0));
                let border_color = if stroke_width.0 > 0.0 {
                    fill_color
                } else {
                    Color::TRANSPARENT
                };

                ops.push(SceneOp::Quad {
                    order: DrawOrder(base_order),
                    rect: cached.rect,
                    background: fill_color,
                    border: Edges::all(stroke_width),
                    border_color,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            #[cfg(debug_assertions)]
            {
                debug_assert!(
                    ops.iter().all(|op| {
                        !matches!(
                            op,
                            SceneOp::Text { .. }
                                | SceneOp::Path { .. }
                                | SceneOp::SvgMaskIcon { .. }
                                | SceneOp::SvgImage { .. }
                        )
                    }),
                    "Cached rect scene ops must not include hosted resources without touching their caches on replay"
                );
            }

            cx.scene.replay_ops(&ops);
            self.cached_rect_scene_ops.store_ops(rect_key, ops);
        }

        let path_constraints = PathConstraints {
            scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
        };

        for (mark_id, cached) in &self.cached_paths {
            let base_order = self
                .style
                .draw_order
                .0
                .saturating_add(cached.order.saturating_mul(4));

            let mut stroke_color = self.style.stroke_color;
            if let Some(series) = cached.source_series {
                stroke_color = self.series_color(series);
                stroke_color.a *= self.style.stroke_color.a;
            }
            if let Some(series_id) = cached.source_series {
                let brush_dim = if let Some(brush) = brush
                    && let Some(series) = series_by_id.get(&series_id)
                {
                    if series.x_axis == brush.x_axis && series.y_axis == brush.y_axis {
                        0.25
                    } else {
                        0.25
                    }
                } else {
                    1.0
                };
                stroke_color.a *= brush_dim;
                if let Some(hover) = self.legend_hover
                    && cached.source_series.is_some()
                    && cached.source_series != Some(hover)
                {
                    stroke_color.a *= 0.25;
                }
            }

            if let Some(fill_alpha) = cached.fill_alpha {
                if let Some((fill, _metrics)) = self
                    .path_cache
                    .get(mark_path_cache_key(*mark_id, 1), path_constraints)
                {
                    let mut fill_color = stroke_color;
                    fill_color.a = fill_alpha;
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(base_order),
                        origin: self.last_layout.plot.origin,
                        path: fill,
                        color: fill_color,
                    });
                }
            }

            let suppress_stroke = cached.source_series.is_some_and(|series_id| {
                series_by_id
                    .get(&series_id)
                    .is_some_and(|s| s.kind == delinea::SeriesKind::Area && s.stack.is_some())
                    && delinea::ids::mark_variant(*mark_id) == 1
            });
            if !suppress_stroke {
                if let Some((stroke, _metrics)) = self
                    .path_cache
                    .get(mark_path_cache_key(*mark_id, 0), path_constraints)
                {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(base_order.saturating_add(1)),
                        origin: self.last_layout.plot.origin,
                        path: stroke,
                        color: stroke_color,
                    });
                }
            }
        }

        let mut point_key = KeyBuilder::new();
        point_key.mix_u64(self.last_marks_rev.0);
        point_key.mix_u64(u64::from(self.last_scale_factor_bits));
        point_key.mix_u64(style_sig);
        point_key.mix_u64(self.legend_hover.map(|v| v.0).unwrap_or(0));
        if let Some(brush) = brush {
            point_key.mix_u64(1);
            point_key.mix_u64(brush.x_axis.0);
            point_key.mix_u64(brush.y_axis.0);
        } else {
            point_key.mix_u64(0);
        }
        let point_key = point_key.finish();

        if !self.cached_point_scene_ops.try_replay_with(
            point_key,
            cx.scene,
            Point::new(Px(0.0), Px(0.0)),
            |_ops| {},
        ) {
            let base_point_r = self.style.scatter_point_radius.0.max(1.0);
            let point_order_bias = 2u32;
            let mut ops: Vec<SceneOp> = Vec::with_capacity(self.cached_points.len());
            for cached in &self.cached_points {
                let point_r = (base_point_r * cached.radius_mul).max(1.0);
                let base_order = self
                    .style
                    .draw_order
                    .0
                    .saturating_add(cached.order.saturating_mul(4))
                    .saturating_add(point_order_bias);

                let mut fill_color = self.style.stroke_color;
                if let Some(paint) = cached.fill {
                    fill_color = self.paint_color(paint);
                    fill_color.a *= self.style.scatter_fill_alpha;
                } else if let Some(series) = cached.source_series {
                    fill_color = self.series_color(series);
                    fill_color.a *= self.style.scatter_fill_alpha;
                }
                fill_color.a *= cached.opacity_mul;
                if let Some(series_id) = cached.source_series {
                    let brush_dim = if brush.is_some() && series_by_id.contains_key(&series_id) {
                        0.25
                    } else {
                        1.0
                    };
                    fill_color.a *= brush_dim;
                    if let Some(hover) = self.legend_hover
                        && cached.source_series.is_some()
                        && cached.source_series != Some(hover)
                    {
                        fill_color.a *= 0.25;
                    }
                }

                let stroke_width = cached.stroke_width.unwrap_or(Px(0.0));
                let border_color = if stroke_width.0 > 0.0 {
                    fill_color
                } else {
                    Color::TRANSPARENT
                };

                ops.push(SceneOp::Quad {
                    order: DrawOrder(base_order),
                    rect: Rect::new(
                        Point::new(
                            Px(cached.point.x.0 - point_r),
                            Px(cached.point.y.0 - point_r),
                        ),
                        Size::new(Px(2.0 * point_r), Px(2.0 * point_r)),
                    ),
                    background: fill_color,
                    border: Edges::all(stroke_width),
                    border_color,
                    corner_radii: Corners::all(Px(point_r)),
                });
            }

            #[cfg(debug_assertions)]
            {
                debug_assert!(
                    ops.iter().all(|op| {
                        !matches!(
                            op,
                            SceneOp::Text { .. }
                                | SceneOp::Path { .. }
                                | SceneOp::SvgMaskIcon { .. }
                                | SceneOp::SvgImage { .. }
                        )
                    }),
                    "Cached point scene ops must not include hosted resources without touching their caches on replay"
                );
            }

            cx.scene.replay_ops(&ops);
            self.cached_point_scene_ops.store_ops(point_key, ops);
        }

        if let Some(brush) = brush
            && let Some(brush_rect_px) = brush_rect_px
        {
            cx.scene.push(SceneOp::PushClipRect {
                rect: brush_rect_px,
            });

            let highlight_bias = 2u32;

            for cached in &self.cached_rects {
                let Some(series_id) = cached.source_series else {
                    continue;
                };
                let Some(series) = series_by_id.get(&series_id) else {
                    continue;
                };
                if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                    continue;
                }

                let base_order = self
                    .style
                    .draw_order
                    .0
                    .saturating_add(cached.order.saturating_mul(4));

                let mut fill_color = self.series_color(series_id);
                if let Some(paint) = cached.fill {
                    fill_color = self.paint_color(paint);
                }
                fill_color.a *= self.style.stroke_color.a;
                fill_color.a *= cached.opacity_mul;
                if let Some(hover) = self.legend_hover
                    && cached.source_series.is_some()
                    && cached.source_series != Some(hover)
                {
                    fill_color.a *= 0.25;
                }
                fill_color.a *= self.style.bar_fill_alpha;

                let stroke_width = cached.stroke_width.unwrap_or(Px(0.0));
                let border_color = if stroke_width.0 > 0.0 {
                    fill_color
                } else {
                    Color::TRANSPARENT
                };

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(base_order.saturating_add(highlight_bias)),
                    rect: cached.rect,
                    background: fill_color,
                    border: Edges::all(stroke_width),
                    border_color: border_color,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let path_constraints = PathConstraints {
                scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
            };

            for (mark_id, cached) in &self.cached_paths {
                let Some(series_id) = cached.source_series else {
                    continue;
                };
                let Some(series) = series_by_id.get(&series_id) else {
                    continue;
                };
                if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                    continue;
                }

                let base_order = self
                    .style
                    .draw_order
                    .0
                    .saturating_add(cached.order.saturating_mul(4));

                let mut stroke_color = self.series_color(series_id);
                stroke_color.a *= self.style.stroke_color.a;
                if let Some(hover) = self.legend_hover
                    && cached.source_series.is_some()
                    && cached.source_series != Some(hover)
                {
                    stroke_color.a *= 0.25;
                }

                if let Some(fill_alpha) = cached.fill_alpha {
                    if let Some((fill, _metrics)) = self
                        .path_cache
                        .get(mark_path_cache_key(*mark_id, 1), path_constraints)
                    {
                        let mut fill_color = stroke_color;
                        fill_color.a = fill_alpha;
                        cx.scene.push(SceneOp::Path {
                            order: DrawOrder(base_order.saturating_add(highlight_bias)),
                            origin: self.last_layout.plot.origin,
                            path: fill,
                            color: fill_color,
                        });
                    }
                }

                let suppress_stroke = cached.source_series.is_some_and(|series_id| {
                    series_by_id
                        .get(&series_id)
                        .is_some_and(|s| s.kind == delinea::SeriesKind::Area && s.stack.is_some())
                        && delinea::ids::mark_variant(*mark_id) == 1
                });
                if !suppress_stroke {
                    if let Some((stroke, _metrics)) = self
                        .path_cache
                        .get(mark_path_cache_key(*mark_id, 0), path_constraints)
                    {
                        cx.scene.push(SceneOp::Path {
                            order: DrawOrder(base_order.saturating_add(highlight_bias + 1)),
                            origin: self.last_layout.plot.origin,
                            path: stroke,
                            color: stroke_color,
                        });
                    }
                }
            }

            let base_point_r = self.style.scatter_point_radius.0.max(1.0);
            let point_order_bias = 2u32;
            for cached in &self.cached_points {
                let Some(series_id) = cached.source_series else {
                    continue;
                };
                let Some(series) = series_by_id.get(&series_id) else {
                    continue;
                };
                if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                    continue;
                }

                let point_r = (base_point_r * cached.radius_mul).max(1.0);
                let base_order = self
                    .style
                    .draw_order
                    .0
                    .saturating_add(cached.order.saturating_mul(4))
                    .saturating_add(point_order_bias);

                let mut fill_color = self.series_color(series_id);
                if let Some(paint) = cached.fill {
                    fill_color = self.paint_color(paint);
                }
                fill_color.a *= self.style.scatter_fill_alpha;
                fill_color.a *= cached.opacity_mul;
                if let Some(hover) = self.legend_hover
                    && cached.source_series.is_some()
                    && cached.source_series != Some(hover)
                {
                    fill_color.a *= 0.25;
                }

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(base_order.saturating_add(highlight_bias)),
                    rect: Rect::new(
                        Point::new(
                            Px(cached.point.x.0 - point_r),
                            Px(cached.point.y.0 - point_r),
                        ),
                        Size::new(Px(2.0 * point_r), Px(2.0 * point_r)),
                    ),
                    background: fill_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(point_r)),
                });
            }

            cx.scene.push(SceneOp::PopClip);
        }

        if let Some((x_axis, _y_axis)) = self.active_axes(&self.last_layout)
            && self.with_engine(|engine| {
                let dz = engine
                    .state()
                    .data_zoom_x
                    .get(&x_axis)
                    .copied()
                    .unwrap_or_default();
                dz.window.is_some() && dz.filter_mode == FilterMode::None
            })
        {
            let label = "Y bounds: global (M)";
            let text_style = TextStyle {
                size: Px(11.0),
                weight: FontWeight::NORMAL,
                ..TextStyle::default()
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
            };
            let prepared = self
                .tooltip_text
                .prepare(cx.services, label, &text_style, constraints);
            let blob = prepared.blob;

            let plot = self.last_layout.plot;
            let pad = 6.0f32;
            let order = DrawOrder(self.style.draw_order.0.saturating_add(9_050));
            cx.scene.push(SceneOp::Text {
                order,
                origin: Point::new(Px(plot.origin.x.0 + pad), Px(plot.origin.y.0 + pad)),
                text: blob,
                color: self.style.axis_tick_color,
            });
        }

        let interaction_idle = self.pan_drag.is_none() && self.box_zoom_drag.is_none();
        let axis_pointer = if interaction_idle && self.legend_hover.is_none() {
            self.with_engine(|engine| engine.output().axis_pointer.clone())
                .and_then(|axis_pointer| {
                    if let Some(grid) = self.grid_override
                        && axis_pointer.grid != Some(grid)
                    {
                        return None;
                    }
                    Some(axis_pointer)
                })
        } else {
            None
        };
        let mut axis_pointer_label_rect: Option<Rect> = None;

        if let Some(axis_pointer) = axis_pointer.as_ref() {
            let pos = axis_pointer.crosshair_px;
            let overlay_order = DrawOrder(self.style.draw_order.0.saturating_add(9_000));
            let point_order = DrawOrder(self.style.draw_order.0.saturating_add(9_001));
            let shadow_order = DrawOrder(self.style.draw_order.0.saturating_add(8_999));

            let (axis_pointer_type, axis_pointer_label_enabled, axis_pointer_label_template) = self
                .with_engine(|engine| {
                    let spec = engine.model().axis_pointer.as_ref();
                    let axis_pointer_type = spec.map(|p| p.pointer_type).unwrap_or_default();
                    let axis_pointer_label_enabled = spec.is_some_and(|p| p.label.show);
                    let axis_pointer_label_template = spec
                        .map(|p| p.label.template.clone())
                        .unwrap_or_else(|| "{value}".to_string());
                    (
                        axis_pointer_type,
                        axis_pointer_label_enabled,
                        axis_pointer_label_template,
                    )
                });
            let axis_pointer_label_template = axis_pointer_label_template.as_str();

            let plot = self.last_layout.plot;
            let crosshair_w = self.style.crosshair_width.0.max(1.0);

            let x = pos
                .x
                .0
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
            let y = pos
                .y
                .0
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);

            let (draw_x, draw_y) = match &axis_pointer.tooltip {
                delinea::TooltipOutput::Axis(axis) => match axis.axis_kind {
                    delinea::AxisKind::X => (true, false),
                    delinea::AxisKind::Y => (false, true),
                },
                delinea::TooltipOutput::Item(_) => (true, true),
            };

            let shadow = matches!(&axis_pointer.tooltip, delinea::TooltipOutput::Axis(_))
                && axis_pointer_type == delinea::AxisPointerType::Shadow;

            if shadow {
                if let Some(rect) = axis_pointer.shadow_rect_px {
                    let color = Color {
                        a: 0.08,
                        ..self.style.selection_fill
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: shadow_order,
                        rect,
                        background: color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            } else if draw_x {
                cx.scene.push(SceneOp::Quad {
                    order: overlay_order,
                    rect: Rect::new(
                        Point::new(Px(x - 0.5 * crosshair_w), plot.origin.y),
                        Size::new(Px(crosshair_w), plot.size.height),
                    ),
                    background: self.style.crosshair_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
            if !shadow && draw_y {
                cx.scene.push(SceneOp::Quad {
                    order: overlay_order,
                    rect: Rect::new(
                        Point::new(plot.origin.x, Px(y - 0.5 * crosshair_w)),
                        Size::new(plot.size.width, Px(crosshair_w)),
                    ),
                    background: self.style.crosshair_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            if axis_pointer_label_enabled {
                let union = |a: Rect, b: Rect| -> Rect {
                    let ax0 = a.origin.x.0;
                    let ay0 = a.origin.y.0;
                    let ax1 = ax0 + a.size.width.0;
                    let ay1 = ay0 + a.size.height.0;

                    let bx0 = b.origin.x.0;
                    let by0 = b.origin.y.0;
                    let bx1 = bx0 + b.size.width.0;
                    let by1 = by0 + b.size.height.0;

                    let x0 = ax0.min(bx0);
                    let y0 = ay0.min(by0);
                    let x1 = ax1.max(bx1);
                    let y1 = ay1.max(by1);

                    Rect::new(
                        Point::new(Px(x0), Px(y0)),
                        Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
                    )
                };

                let mut draw_label_for_axis =
                    |axis_id: delinea::AxisId, axis_kind: delinea::AxisKind, axis_value: f64| {
                        let band = match axis_kind {
                            delinea::AxisKind::X => self
                                .last_layout
                                .x_axes
                                .iter()
                                .find(|b| b.axis == axis_id)
                                .copied(),
                            delinea::AxisKind::Y => self
                                .last_layout
                                .y_axes
                                .iter()
                                .find(|b| b.axis == axis_id)
                                .copied(),
                        };
                        let Some(band) = band else {
                            return;
                        };

                        let default_tooltip_spec = delinea::TooltipSpecV1::default();
                        let (axis_window, axis_name, missing_value) = self.with_engine(|engine| {
                            let axis_window = engine
                                .output()
                                .axis_windows
                                .get(&axis_id)
                                .copied()
                                .unwrap_or_default();
                            let axis_name = engine
                                .model()
                                .axes
                                .get(&axis_id)
                                .and_then(|a| a.name.as_deref())
                                .unwrap_or("")
                                .to_string();
                            let missing_value = engine
                                .model()
                                .tooltip
                                .as_ref()
                                .map(|t| t.missing_value.clone())
                                .unwrap_or_else(|| default_tooltip_spec.missing_value.clone());
                            (axis_window, axis_name, missing_value)
                        });
                        let value_text = if axis_value.is_finite() {
                            self.with_engine(|engine| {
                                delinea::engine::axis::format_value_for(
                                    engine.model(),
                                    axis_id,
                                    axis_window,
                                    axis_value,
                                )
                            })
                        } else {
                            missing_value
                        };

                        let label_text = if axis_pointer_label_template == "{value}" {
                            value_text
                        } else {
                            axis_pointer_label_template
                                .replace("{value}", &value_text)
                                .replace("{axis_name}", &axis_name)
                        };

                        let text_style = TextStyle {
                            size: Px(11.0),
                            weight: FontWeight::MEDIUM,
                            ..TextStyle::default()
                        };
                        let constraints = TextConstraints {
                            max_width: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            scale_factor: cx.scale_factor,
                        };
                        let prepared = self.tooltip_text.prepare(
                            cx.services,
                            &label_text,
                            &text_style,
                            constraints,
                        );
                        let blob = prepared.blob;
                        let metrics = prepared.metrics;

                        let pad_x = 6.0f32;
                        let pad_y = 3.0f32;
                        let w = (metrics.size.width.0 + 2.0 * pad_x).max(1.0);
                        let h = (metrics.size.height.0 + 2.0 * pad_y).max(1.0);

                        let (mut box_x, mut box_y) = match axis_kind {
                            delinea::AxisKind::X => (
                                x - 0.5 * w,
                                band.rect.origin.y.0 + 0.5 * (band.rect.size.height.0 - h),
                            ),
                            delinea::AxisKind::Y => (
                                band.rect.origin.x.0 + 0.5 * (band.rect.size.width.0 - w),
                                y - 0.5 * h,
                            ),
                        };

                        let bx0 = band.rect.origin.x.0;
                        let by0 = band.rect.origin.y.0;
                        let bx1 = bx0 + band.rect.size.width.0;
                        let by1 = by0 + band.rect.size.height.0;
                        box_x = box_x.clamp(bx0, (bx1 - w).max(bx0));
                        box_y = box_y.clamp(by0, (by1 - h).max(by0));
                        let rect =
                            Rect::new(Point::new(Px(box_x), Px(box_y)), Size::new(Px(w), Px(h)));
                        axis_pointer_label_rect = Some(match axis_pointer_label_rect {
                            Some(prev) => union(prev, rect),
                            None => rect,
                        });

                        let kind_key: u32 = match axis_kind {
                            delinea::AxisKind::X => 0,
                            delinea::AxisKind::Y => 1,
                        };
                        let label_order = DrawOrder(
                            self.style
                                .draw_order
                                .0
                                .saturating_add(9_020 + kind_key.saturating_mul(4)),
                        );
                        cx.scene.push(SceneOp::Quad {
                            order: label_order,
                            rect,
                            background: self.style.tooltip_background,
                            border: Edges::all(self.style.tooltip_border_width),
                            border_color: self.style.tooltip_border_color,
                            corner_radii: Corners::all(Px(4.0)),
                        });
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(label_order.0.saturating_add(1)),
                            origin: Point::new(Px(box_x + pad_x), Px(box_y + pad_y)),
                            text: blob,
                            color: self.style.tooltip_text_color,
                        });
                    };

                match &axis_pointer.tooltip {
                    delinea::TooltipOutput::Axis(axis) => {
                        draw_label_for_axis(axis.axis, axis.axis_kind, axis.axis_value);
                    }
                    delinea::TooltipOutput::Item(item) => {
                        draw_label_for_axis(item.x_axis, delinea::AxisKind::X, item.x_value);
                        draw_label_for_axis(item.y_axis, delinea::AxisKind::Y, item.y_value);
                    }
                }
            }

            if !shadow {
                if let Some(hit) = axis_pointer.hit {
                    let r = self.style.hover_point_size.0.max(1.0);
                    cx.scene.push(SceneOp::Quad {
                        order: point_order,
                        rect: Rect::new(
                            Point::new(Px(hit.point_px.x.0 - r), Px(hit.point_px.y.0 - r)),
                            Size::new(Px(2.0 * r), Px(2.0 * r)),
                        ),
                        background: self.style.hover_point_color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }

        self.draw_legend(cx);

        if let Some(drag) = self.box_zoom_drag {
            let rect =
                rect_from_points_clamped(self.last_layout.plot, drag.start_pos, drag.current_pos);
            if rect.size.width.0 >= 1.0 && rect.size.height.0 >= 1.0 {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(self.style.draw_order.0.saturating_add(8_800)),
                    rect,
                    background: self.style.selection_fill,
                    border: Edges::all(self.style.selection_stroke_width),
                    border_color: self.style.selection_stroke,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        // DataZoom slider: render for the active bottom X axis (if present).
        if let Some((x_axis, _y_axis)) = self.active_axes(&self.last_layout)
            && let Some(track) = self.x_slider_track_for_axis(x_axis)
        {
            let extent = self.compute_axis_extent_from_data(x_axis, true);
            let window = self.current_window_x_for_slider(x_axis, extent);

            let t0 = Self::slider_norm(extent, window.min);
            let t1 = Self::slider_norm(extent, window.max);
            let left = track.origin.x.0 + t0 * track.size.width.0;
            let right = track.origin.x.0 + t1 * track.size.width.0;

            let order = DrawOrder(self.style.draw_order.0.saturating_add(8_650));
            let track_color = Color {
                a: 0.18,
                ..self.style.axis_line_color
            };
            cx.scene.push(SceneOp::Quad {
                order,
                rect: track,
                background: track_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(4.0)),
            });

            let win_rect = Rect::new(
                Point::new(Px(left.min(right)), track.origin.y),
                Size::new(Px((right - left).abs().max(1.0)), track.size.height),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(1)),
                rect: win_rect,
                background: self.style.selection_fill,
                border: Edges::all(self.style.selection_stroke_width),
                border_color: self.style.selection_stroke,
                corner_radii: Corners::all(Px(4.0)),
            });

            let handle_w = 2.0f32.max(self.style.selection_stroke_width.0);
            let handle_color = self.style.selection_stroke;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(2)),
                rect: Rect::new(
                    Point::new(Px(left - 0.5 * handle_w), track.origin.y),
                    Size::new(Px(handle_w), track.size.height),
                ),
                background: handle_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(3)),
                rect: Rect::new(
                    Point::new(Px(right - 0.5 * handle_w), track.origin.y),
                    Size::new(Px(handle_w), track.size.height),
                ),
                background: handle_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        // DataZoom slider: render for the active Y axis (if present).
        if let Some((_x_axis, y_axis)) = self.active_axes(&self.last_layout)
            && let Some(track) = self.y_slider_track_for_axis(y_axis)
        {
            let extent = self.compute_axis_extent_from_data(y_axis, false);
            let window = self.current_window_y_for_slider(y_axis, extent);

            let t0 = Self::slider_norm(extent, window.min);
            let t1 = Self::slider_norm(extent, window.max);

            let height = track.size.height.0;
            let bottom = track.origin.y.0 + height;
            let y0 = bottom - t0 * height;
            let y1 = bottom - t1 * height;

            let order = DrawOrder(self.style.draw_order.0.saturating_add(8_650));
            let track_color = Color {
                a: 0.18,
                ..self.style.axis_line_color
            };
            cx.scene.push(SceneOp::Quad {
                order,
                rect: track,
                background: track_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(4.0)),
            });

            let top = y0.min(y1);
            let bottom = y0.max(y1);
            let win_rect = Rect::new(
                Point::new(track.origin.x, Px(top)),
                Size::new(track.size.width, Px((bottom - top).abs().max(1.0))),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(1)),
                rect: win_rect,
                background: self.style.selection_fill,
                border: Edges::all(self.style.selection_stroke_width),
                border_color: self.style.selection_stroke,
                corner_radii: Corners::all(Px(4.0)),
            });

            let handle_h = 2.0f32.max(self.style.selection_stroke_width.0);
            let handle_color = self.style.selection_stroke;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(2)),
                rect: Rect::new(
                    Point::new(track.origin.x, Px(y0 - 0.5 * handle_h)),
                    Size::new(track.size.width, Px(handle_h)),
                ),
                background: handle_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(order.0.saturating_add(3)),
                rect: Rect::new(
                    Point::new(track.origin.x, Px(y1 - 0.5 * handle_h)),
                    Size::new(track.size.width, Px(handle_h)),
                ),
                background: handle_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if let Some(rect) = brush_rect_px {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(self.style.draw_order.0.saturating_add(8_700)),
                rect,
                background: self.style.selection_fill,
                border: Edges::all(self.style.selection_stroke_width),
                border_color: self.style.selection_stroke,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if let Some(drag) = self.brush_drag {
            let rect =
                rect_from_points_clamped(self.last_layout.plot, drag.start_pos, drag.current_pos);
            if rect.size.width.0 >= 1.0 && rect.size.height.0 >= 1.0 {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(self.style.draw_order.0.saturating_add(8_750)),
                    rect,
                    background: self.style.selection_fill,
                    border: Edges::all(self.style.selection_stroke_width),
                    border_color: self.style.selection_stroke,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        cx.scene.push(SceneOp::PopClip);

        self.draw_visual_map(cx);

        if let Some(axis_pointer) = axis_pointer {
            let tooltip_lines = self.with_engine(|engine| {
                self.tooltip_formatter.format_axis_pointer(
                    engine,
                    &engine.output().axis_windows,
                    &axis_pointer,
                )
            });
            if tooltip_lines.is_empty() {
                self.draw_axes(cx);
                return;
            }

            let text_style = TextStyle {
                size: Px(12.0),
                weight: FontWeight::NORMAL,
                ..TextStyle::default()
            };
            let mut header_text_style = text_style.clone();
            header_text_style.weight = FontWeight::BOLD;
            let mut value_text_style = text_style.clone();
            value_text_style.weight = FontWeight::MEDIUM;
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: effective_scale_factor(cx.scale_factor, 1.0),
            };

            let pad = self.style.tooltip_padding;
            let swatch_w = self.style.tooltip_marker_size.0.max(0.0);
            let swatch_gap = self.style.tooltip_marker_gap.0.max(0.0);
            let col_gap = self.style.tooltip_column_gap.0.max(0.0);
            let reserve_swatch =
                swatch_w > 0.0 && tooltip_lines.iter().any(|l| l.source_series.is_some());
            let swatch_space = if reserve_swatch {
                (swatch_w + swatch_gap).max(0.0)
            } else {
                0.0
            };

            enum TooltipLineLayout {
                Single {
                    blob: TextBlobId,
                    metrics: fret_core::TextMetrics,
                },
                Columns {
                    left_blob: TextBlobId,
                    left_metrics: fret_core::TextMetrics,
                    right_blob: TextBlobId,
                    right_metrics: fret_core::TextMetrics,
                },
            }

            struct PreparedTooltipLine {
                source_series: Option<delinea::SeriesId>,
                is_missing: bool,
                layout: TooltipLineLayout,
            }

            let mut prepared_lines = Vec::with_capacity(tooltip_lines.len());
            let mut max_left_w = 0.0f32;
            let mut max_right_w = 0.0f32;
            let mut max_single_w = 0.0f32;
            let mut total_h = 0.0f32;

            for line in &tooltip_lines {
                let label_style = if line.kind == crate::TooltipTextLineKind::AxisHeader {
                    &header_text_style
                } else {
                    &text_style
                };
                let value_style =
                    if line.value_emphasis && line.kind != crate::TooltipTextLineKind::AxisHeader {
                        &value_text_style
                    } else {
                        label_style
                    };

                let columns = line
                    .columns
                    .as_ref()
                    .map(|(left, right)| (left.as_str(), right.as_str()))
                    .or_else(|| crate::tooltip_layout::split_tooltip_text_for_columns(&line.text));

                if let Some((left, right)) = columns {
                    let left_prepared =
                        self.tooltip_text
                            .prepare(cx.services, left, label_style, constraints);
                    let right_prepared =
                        self.tooltip_text
                            .prepare(cx.services, right, value_style, constraints);
                    let left_blob = left_prepared.blob;
                    let left_metrics = left_prepared.metrics;
                    let right_blob = right_prepared.blob;
                    let right_metrics = right_prepared.metrics;

                    max_left_w = max_left_w.max(left_metrics.size.width.0);
                    max_right_w = max_right_w.max(right_metrics.size.width.0);
                    total_h += left_metrics
                        .size
                        .height
                        .0
                        .max(right_metrics.size.height.0)
                        .max(1.0);

                    prepared_lines.push(PreparedTooltipLine {
                        source_series: line.source_series,
                        is_missing: line.is_missing,
                        layout: TooltipLineLayout::Columns {
                            left_blob,
                            left_metrics,
                            right_blob,
                            right_metrics,
                        },
                    });
                } else {
                    let prepared = self.tooltip_text.prepare(
                        cx.services,
                        &line.text,
                        label_style,
                        constraints,
                    );
                    let blob = prepared.blob;
                    let metrics = prepared.metrics;
                    max_single_w = max_single_w.max(metrics.size.width.0);
                    total_h += metrics.size.height.0.max(1.0);
                    prepared_lines.push(PreparedTooltipLine {
                        source_series: line.source_series,
                        is_missing: line.is_missing,
                        layout: TooltipLineLayout::Single { blob, metrics },
                    });
                }
            }

            let mut w = 1.0f32;
            if max_left_w > 0.0 || max_right_w > 0.0 {
                w = w.max(max_left_w + col_gap + max_right_w);
            }
            w = w.max(max_single_w);
            w = (w + swatch_space + pad.left.0 + pad.right.0).max(1.0);
            let h = (total_h + pad.top.0 + pad.bottom.0).max(1.0);

            let bounds = self.last_layout.bounds;

            let anchor = match &axis_pointer.tooltip {
                delinea::TooltipOutput::Axis(_) => axis_pointer.crosshair_px,
                delinea::TooltipOutput::Item(_) => axis_pointer
                    .hit
                    .map(|h| h.point_px)
                    .unwrap_or(axis_pointer.crosshair_px),
            };

            let offset = 10.0f32;
            let tooltip_rect = crate::tooltip_layout::place_tooltip_rect(
                bounds,
                anchor,
                Size::new(Px(w), Px(h)),
                offset,
                axis_pointer_label_rect,
            );
            let tip_x = tooltip_rect.origin.x.0;
            let tip_y = tooltip_rect.origin.y.0;

            let tooltip_order = DrawOrder(self.style.draw_order.0.saturating_add(9_100));
            cx.scene.push(SceneOp::Quad {
                order: tooltip_order,
                rect: Rect::new(Point::new(Px(tip_x), Px(tip_y)), Size::new(Px(w), Px(h))),
                background: self.style.tooltip_background,
                border: Edges::all(self.style.tooltip_border_width),
                border_color: self.style.tooltip_border_color,
                corner_radii: Corners::all(self.style.tooltip_corner_radius),
            });

            let mut y = tip_y + pad.top.0;
            let missing_text_color = Color {
                a: (self.style.tooltip_text_color.a * 0.55).clamp(0.0, 1.0),
                ..self.style.tooltip_text_color
            };
            for (i, line) in prepared_lines.into_iter().enumerate() {
                let order_base = tooltip_order
                    .0
                    .saturating_add(1 + (i as u32).saturating_mul(3));
                let swatch_x = tip_x + pad.left.0;
                let text_x0 = swatch_x + swatch_space;

                let side = self.style.tooltip_marker_size.0.max(0.0);
                if side > 0.0
                    && reserve_swatch
                    && let Some(series) = line.source_series
                {
                    let line_height = match &line.layout {
                        TooltipLineLayout::Single { metrics, .. } => metrics.size.height.0.max(1.0),
                        TooltipLineLayout::Columns {
                            left_metrics,
                            right_metrics,
                            ..
                        } => left_metrics
                            .size
                            .height
                            .0
                            .max(right_metrics.size.height.0)
                            .max(1.0),
                    };
                    let marker_y = y + (line_height - side) * 0.5;
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(order_base),
                        rect: Rect::new(
                            Point::new(Px(swatch_x), Px(marker_y)),
                            Size::new(Px(side), Px(side)),
                        ),
                        background: self.series_color(series),
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px((side * 0.25).max(0.0))),
                    });
                }

                match line.layout {
                    TooltipLineLayout::Single { blob, metrics } => {
                        let color = if line.is_missing {
                            missing_text_color
                        } else {
                            self.style.tooltip_text_color
                        };
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(order_base.saturating_add(1)),
                            origin: Point::new(Px(text_x0), Px(y)),
                            text: blob,
                            color,
                        });
                        y += metrics.size.height.0.max(1.0);
                    }
                    TooltipLineLayout::Columns {
                        left_blob,
                        left_metrics,
                        right_blob,
                        right_metrics,
                    } => {
                        let line_height = left_metrics
                            .size
                            .height
                            .0
                            .max(right_metrics.size.height.0)
                            .max(1.0);
                        let value_x = text_x0
                            + max_left_w
                            + col_gap
                            + (max_right_w - right_metrics.size.width.0).max(0.0);

                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(order_base.saturating_add(1)),
                            origin: Point::new(Px(text_x0), Px(y)),
                            text: left_blob,
                            color: self.style.tooltip_text_color,
                        });
                        let value_color = if line.is_missing {
                            missing_text_color
                        } else {
                            self.style.tooltip_text_color
                        };
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(order_base.saturating_add(2)),
                            origin: Point::new(Px(value_x), Px(y)),
                            text: right_blob,
                            color: value_color,
                        });

                        y += line_height;
                    }
                }
            }
        }

        self.draw_axes(cx);

        // Conservative hygiene: long-lived charts should not grow text caches unbounded.
        let t = self.text_cache_prune;
        if t.max_entries > 0 && t.max_age_frames > 0 {
            self.axis_text
                .prune(cx.services, t.max_age_frames, t.max_entries);
            self.legend_text
                .prune(cx.services, t.max_age_frames, t.max_entries);
        }
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.path_cache.clear(services);
        self.cached_paths.clear();

        self.axis_text.clear(services);
        self.tooltip_text.clear(services);
        self.legend_text.clear(services);
    }
}

fn rect_from_points_clamped(bounds: Rect, a: Point, b: Point) -> Rect {
    let x0 =
        a.x.0
            .min(b.x.0)
            .clamp(bounds.origin.x.0, bounds.origin.x.0 + bounds.size.width.0);
    let x1 =
        a.x.0
            .max(b.x.0)
            .clamp(bounds.origin.x.0, bounds.origin.x.0 + bounds.size.width.0);
    let y0 =
        a.y.0
            .min(b.y.0)
            .clamp(bounds.origin.y.0, bounds.origin.y.0 + bounds.size.height.0);
    let y1 =
        a.y.0
            .max(b.y.0)
            .clamp(bounds.origin.y.0, bounds.origin.y.0 + bounds.size.height.0);

    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use delinea::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId, VisualMapId};
    use delinea::{
        AxisKind, AxisPosition, AxisRange, AxisScale, ChartSpec, DatasetSpec, FieldSpec, GridSpec,
        SeriesEncode, SeriesKind, SeriesSpec, VisualMapSpec,
    };

    #[test]
    fn legend_double_click_isolates_and_restores_all_series() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");

        let a = delinea::SeriesId::new(1);
        let b = delinea::SeriesId::new(2);

        assert!(canvas.engine().model().series.get(&a).unwrap().visible);
        assert!(canvas.engine().model().series.get(&b).unwrap().visible);

        canvas.apply_legend_double_click(b);
        assert!(!canvas.engine().model().series.get(&a).unwrap().visible);
        assert!(canvas.engine().model().series.get(&b).unwrap().visible);

        canvas.apply_legend_double_click(b);
        assert!(canvas.engine().model().series.get(&a).unwrap().visible);
        assert!(canvas.engine().model().series.get(&b).unwrap().visible);
    }

    #[test]
    fn legend_scroll_clamps_to_content_height() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        canvas.legend_content_height = Px(500.0);
        canvas.legend_view_height = Px(120.0);

        assert_eq!(canvas.legend_max_scroll_y().0, 380.0);

        assert!(canvas.apply_legend_wheel_scroll(Px(-200.0)));
        assert!(canvas.legend_scroll_y.0 > 0.0);

        canvas.apply_legend_wheel_scroll(Px(-10_000.0));
        assert_eq!(canvas.legend_scroll_y.0, 380.0);

        canvas.apply_legend_wheel_scroll(Px(10_000.0));
        assert_eq!(canvas.legend_scroll_y.0, 0.0);
    }

    #[test]
    fn legend_select_all_none_invert_update_series_visibility() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");

        let ids: Vec<_> = canvas.engine().model().series_order.clone();
        assert!(ids.len() >= 2);

        canvas.apply_legend_select_none();
        for id in &ids {
            assert!(!canvas.engine().model().series.get(id).unwrap().visible);
        }

        canvas.apply_legend_select_all();
        for id in &ids {
            assert!(canvas.engine().model().series.get(id).unwrap().visible);
        }

        canvas.with_engine_mut(|engine| {
            engine.apply_action(Action::SetSeriesVisible {
                series: ids[0],
                visible: false,
            });
        });
        canvas.apply_legend_invert();
        assert!(canvas.engine().model().series.get(&ids[0]).unwrap().visible);
        assert!(!canvas.engine().model().series.get(&ids[1]).unwrap().visible);
    }

    #[test]
    fn legend_selector_hit_test_returns_action() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        canvas.legend_selector_rects = vec![(
            LegendSelectorAction::Invert,
            Rect::new(
                Point::new(Px(10.0), Px(10.0)),
                Size::new(Px(20.0), Px(12.0)),
            ),
        )];

        assert_eq!(
            canvas.legend_selector_at(Point::new(Px(15.0), Px(15.0))),
            Some(LegendSelectorAction::Invert)
        );
        assert_eq!(
            canvas.legend_selector_at(Point::new(Px(1.0), Px(1.0))),
            None
        );
    }

    #[test]
    fn axis_pointer_hover_point_clamps_axis_band_into_plot() {
        let plot = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let layout = ChartLayout {
            bounds: plot,
            plot,
            x_axes: vec![AxisBandLayout {
                axis: AxisId::new(1),
                position: AxisPosition::Bottom,
                rect: Rect::new(
                    Point::new(Px(0.0), Px(100.0)),
                    Size::new(Px(100.0), Px(20.0)),
                ),
            }],
            y_axes: vec![
                AxisBandLayout {
                    axis: AxisId::new(2),
                    position: AxisPosition::Left,
                    rect: Rect::new(
                        Point::new(Px(-20.0), Px(0.0)),
                        Size::new(Px(20.0), Px(100.0)),
                    ),
                },
                AxisBandLayout {
                    axis: AxisId::new(3),
                    position: AxisPosition::Right,
                    rect: Rect::new(
                        Point::new(Px(100.0), Px(0.0)),
                        Size::new(Px(20.0), Px(100.0)),
                    ),
                },
            ],
            visual_map: None,
        };

        let p = ChartCanvas::axis_pointer_hover_point(&layout, Point::new(Px(50.0), Px(110.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 50.0);
        assert_eq!(p.y.0, 99.0);

        let p = ChartCanvas::axis_pointer_hover_point(&layout, Point::new(Px(-10.0), Px(25.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 1.0);
        assert_eq!(p.y.0, 25.0);

        let p = ChartCanvas::axis_pointer_hover_point(&layout, Point::new(Px(110.0), Px(75.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 99.0);
        assert_eq!(p.y.0, 75.0);
    }

    #[test]
    fn data_mapping_is_monotonic() {
        let window = DataWindow {
            min: 10.0,
            max: 20.0,
        };
        let a = delinea::engine::axis::data_at_px(window, 0.0, 0.0, 100.0);
        let b = delinea::engine::axis::data_at_px(window, 50.0, 0.0, 100.0);
        let c = delinea::engine::axis::data_at_px(window, 100.0, 0.0, 100.0);
        assert!(a < b && b < c);
        assert_eq!(a, 10.0);
        assert_eq!(c, 20.0);

        let d = delinea::engine::axis::data_at_px(window, 0.0, 0.0, 100.0);
        let e = delinea::engine::axis::data_at_px(window, 100.0, 0.0, 100.0);
        assert_eq!(d, 10.0);
        assert_eq!(e, 20.0);
    }

    #[test]
    fn rect_from_points_is_clamped_to_bounds() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(200.0)),
        );
        let a = Point::new(Px(0.0), Px(0.0));
        let b = Point::new(Px(999.0), Px(999.0));
        let rect = rect_from_points_clamped(bounds, a, b);
        assert_eq!(rect.origin, bounds.origin);
        assert_eq!(rect.size, bounds.size);
    }

    #[test]
    fn nice_ticks_include_endpoints() {
        let window = DataWindow { min: 0.2, max: 9.7 };
        let ticks = delinea::format::nice_ticks(window, 5);
        assert!(!ticks.is_empty());
        assert_eq!(*ticks.first().unwrap(), window.min);
        assert_eq!(*ticks.last().unwrap(), window.max);
    }

    #[test]
    fn series_color_is_stable() {
        let canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        let a = canvas.series_color(delinea::SeriesId::new(1));
        let b = canvas.series_color(delinea::SeriesId::new(2));
        assert_ne!(a, b);
        assert_eq!(a, canvas.series_color(delinea::SeriesId::new(1)));
    }

    #[test]
    fn series_color_respects_theme_palette_when_style_is_fixed() {
        let mut app = fret_app::App::new();
        let mut cfg = fret_ui::ThemeConfig::default();
        cfg.colors
            .insert("chart.palette.0".to_string(), "#FF0000".to_string());
        cfg.colors
            .insert("chart.palette.1".to_string(), "#00FF00".to_string());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let theme = Theme::global(&app);
        let style = ChartStyle::from_theme(theme);
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        canvas.set_style(style);

        assert_eq!(
            canvas.series_color(delinea::SeriesId::new(1)),
            theme.color_required("chart.palette.0")
        );
        assert_eq!(
            canvas.series_color(delinea::SeriesId::new(2)),
            theme.color_required("chart.palette.1")
        );
    }

    #[test]
    fn series_color_follows_series_order_not_series_id() {
        let mut app = fret_app::App::new();
        let mut cfg = fret_ui::ThemeConfig::default();
        cfg.colors
            .insert("chart.palette.0".to_string(), "#FF0000".to_string());
        cfg.colors
            .insert("chart.palette.1".to_string(), "#00FF00".to_string());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let theme = Theme::global(&app);
        let style = ChartStyle::from_theme(theme);

        let mut spec = multi_axis_spec();
        spec.series[0].id = delinea::SeriesId::new(42);
        spec.series[1].id = delinea::SeriesId::new(1);

        let mut canvas = ChartCanvas::new(spec).expect("spec should be valid");
        canvas.set_style(style);

        assert_eq!(
            canvas.series_color(delinea::SeriesId::new(42)),
            theme.color_required("chart.palette.0")
        );
        assert_eq!(
            canvas.series_color(delinea::SeriesId::new(1)),
            theme.color_required("chart.palette.1")
        );
    }

    fn multi_axis_spec() -> ChartSpec {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_left = AxisId::new(2);
        let y_right = AxisId::new(3);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);

        ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: Some(AxisPosition::Bottom),
                    scale: AxisScale::default(),
                    range: Some(AxisRange::Auto),
                },
                delinea::AxisSpec {
                    id: y_left,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Left),
                    scale: AxisScale::default(),
                    range: Some(AxisRange::Auto),
                },
                delinea::AxisSpec {
                    id: y_right,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(AxisPosition::Right),
                    scale: AxisScale::default(),
                    range: Some(AxisRange::Auto),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: SeriesId::new(1),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: SeriesId::new(2),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        }
    }

    fn multi_axis_visual_map_spec() -> ChartSpec {
        let mut spec = multi_axis_spec();
        let y_field = spec.series[0].encode.y;
        let series_id = spec.series[0].id;
        spec.visual_maps.push(VisualMapSpec {
            id: VisualMapId::new(1),
            mode: delinea::VisualMapMode::Continuous,
            dataset: None,
            series: vec![series_id],
            field: y_field,
            domain: (-1.0, 1.0),
            initial_range: Some((-0.25, 0.75)),
            initial_piece_mask: None,
            point_radius_mul_range: None,
            stroke_width_range: None,
            opacity_mul_range: None,
            buckets: 8,
            out_of_range_opacity: 0.25,
        });
        spec
    }

    #[test]
    fn primary_axes_skip_hidden_series() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        canvas
            .engine_mut()
            .apply_action(delinea::action::Action::SetSeriesVisible {
                series: delinea::SeriesId::new(1),
                visible: false,
            });

        let (_x, y) = canvas.primary_axes().expect("expected primary axes");
        assert_eq!(y, AxisId::new(3));
    }

    #[test]
    fn active_axes_prefer_last_hovered_band() {
        let mut canvas = ChartCanvas::new(multi_axis_spec()).expect("spec should be valid");
        let layout = canvas.compute_layout(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(400.0)),
        ));

        let right_band = layout
            .y_axes
            .iter()
            .find(|b| b.position == AxisPosition::Right)
            .expect("expected a right y axis band");
        let p = Point::new(
            Px(right_band.rect.origin.x.0 + 1.0),
            Px(right_band.rect.origin.y.0 + 1.0),
        );
        canvas.update_active_axes_for_position(&layout, p);

        let (x, y) = canvas.active_axes(&layout).expect("expected active axes");
        assert_eq!(x, AxisId::new(1));
        assert_eq!(y, AxisId::new(3));
    }

    #[test]
    fn visual_map_y_mapping_respects_domain_endpoints() {
        let track = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(8.0), Px(100.0)),
        );
        let domain = DataWindow {
            min: 0.0,
            max: 10.0,
        };

        let bottom = track.origin.y.0 + track.size.height.0;
        assert_eq!(
            ChartCanvas::visual_map_y_at_value(track, domain, 0.0),
            bottom
        );
        assert_eq!(
            ChartCanvas::visual_map_y_at_value(track, domain, 10.0),
            track.origin.y.0
        );
    }

    #[test]
    fn visual_map_track_applies_style_padding() {
        let mut canvas =
            ChartCanvas::new(multi_axis_visual_map_spec()).expect("spec should be valid");
        let mut style = ChartStyle::default();
        style.visual_map_band_x = Px(80.0);
        style.visual_map_padding = Px(10.0);
        canvas.set_style(style);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(400.0)),
        );
        let layout = canvas.compute_layout(bounds);
        canvas.last_layout = layout;

        let tracks = canvas.visual_map_tracks();
        assert_eq!(tracks.len(), 1);
        let (_id, _vm, track) = tracks[0];
        let outer = canvas
            .last_layout
            .visual_map
            .expect("expected a visual map band rect");
        assert_eq!(track.origin.x.0, outer.origin.x.0 + 10.0);
        assert_eq!(track.origin.y.0, outer.origin.y.0 + 10.0);
    }

    #[test]
    fn view_window_2d_action_is_atomic() {
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x = DataWindow {
            min: 10.0,
            max: 20.0,
        };
        let y = DataWindow {
            min: -5.0,
            max: 5.0,
        };

        let action = Action::SetViewWindow2D {
            x_axis,
            y_axis,
            x: Some(x),
            y: Some(y),
        };
        match action {
            Action::SetViewWindow2D {
                x_axis: ax,
                y_axis: ay,
                x: Some(wx),
                y: Some(wy),
            } => {
                assert_eq!(ax, x_axis);
                assert_eq!(ay, y_axis);
                assert_eq!(wx, x);
                assert_eq!(wy, y);
            }
            _ => panic!("expected SetViewWindow2D"),
        }
    }

    #[test]
    fn slider_window_after_delta_clamps_and_never_inverts() {
        let extent = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let start = DataWindow {
            min: 20.0,
            max: 30.0,
        };

        let left =
            ChartCanvas::slider_window_after_delta(extent, start, -999.0, SliderDragKind::Pan);
        assert_eq!(
            left,
            DataWindow {
                min: 0.0,
                max: 10.0
            }
        );

        let right =
            ChartCanvas::slider_window_after_delta(extent, start, 999.0, SliderDragKind::Pan);
        assert_eq!(
            right,
            DataWindow {
                min: 90.0,
                max: 100.0
            }
        );

        let inverted_min =
            ChartCanvas::slider_window_after_delta(extent, start, 999.0, SliderDragKind::HandleMin);
        assert!(inverted_min.max > inverted_min.min);
        assert_eq!(inverted_min.max, start.max);
        assert!(inverted_min.min >= extent.min && inverted_min.max <= extent.max);

        let inverted_max = ChartCanvas::slider_window_after_delta(
            extent,
            start,
            -999.0,
            SliderDragKind::HandleMax,
        );
        assert!(inverted_max.max > inverted_max.min);
        assert_eq!(inverted_max.min, start.min);
        assert!(inverted_max.min >= extent.min && inverted_max.max <= extent.max);
    }
}
