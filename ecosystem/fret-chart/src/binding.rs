//! App-facing chart bindings that hide raw runtime model plumbing from examples.

use std::{collections::BTreeMap, sync::Arc};

use delinea::engine::window::DataWindow;
use delinea::ids::GridId;
use delinea::{ChartEngine, ChartSpec};
use fret_runtime::{Model, ModelCx, ModelHost, ModelUpdateError};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use crate::linking::{
    AxisPointerLinkAnchor, BrushSelectionLink2D, ChartLinkPolicy, ChartLinkRouter, LinkAxisKey,
    LinkedChartGroup, LinkedChartMember,
};
use crate::{ChartCanvasOutput, ChartCanvasPanelProps};

/// App-facing handle for a chart canvas panel plus its controlled engine model.
///
/// `ChartCanvasPanelProps` remains the component-author surface and still exposes raw model handles
/// for linked charts, multi-grid composition, and advanced output wiring. This binding is the
/// default app/cookbook surface for a single controlled chart canvas.
#[derive(Clone)]
pub struct ChartCanvasPanelBinding {
    spec: ChartSpec,
    engine: Model<ChartEngine>,
    output: Option<Model<ChartCanvasOutput>>,
}

impl ChartCanvasPanelBinding {
    /// Insert a chart engine and output channel into a model host and keep the matching panel spec.
    #[track_caller]
    pub fn new(host: &mut impl ModelHost, spec: ChartSpec, engine: ChartEngine) -> Self {
        Self {
            spec,
            engine: host.models_mut().insert(engine),
            output: Some(host.models_mut().insert(ChartCanvasOutput::default())),
        }
    }

    /// Build declarative panel props wired to this binding's controlled engine and output channel.
    pub fn panel_props(&self) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(self.spec.clone());
        props.engine = Some(self.engine.clone());
        if let Some(output) = self.output.as_ref() {
            props = props.output_model(output.clone());
        }
        props
    }

    /// Observe the chart engine as a paint dependency from an app view render pass.
    pub fn observe_engine_paint<'a, H, Cx>(&self, cx: &mut Cx)
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .observe_model(&self.engine, Invalidation::Paint);
    }

    /// Read the latest chart output through a layout invalidation tracked read.
    pub fn output_layout<'a, H, Cx>(&self, cx: &mut Cx) -> ChartCanvasOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        self.output
            .as_ref()
            .and_then(|output| cx.elements().get_model_cloned(output, Invalidation::Layout))
            .unwrap_or_default()
    }

    /// Read the latest chart output through a paint invalidation tracked read.
    pub fn output_paint<'a, H, Cx>(&self, cx: &mut Cx) -> ChartCanvasOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        self.output
            .as_ref()
            .and_then(|output| cx.elements().get_model_cloned(output, Invalidation::Paint))
            .unwrap_or_default()
    }

    /// Read the latest chart output without registering a UI invalidation dependency.
    ///
    /// This is intended for event handlers, diagnostics, and logging code that needs to observe
    /// chart output outside a render/layout context.
    pub fn output_untracked(&self, host: &impl ModelHost) -> ChartCanvasOutput {
        self.output
            .as_ref()
            .and_then(|output| output.read_ref(host, Clone::clone).ok())
            .unwrap_or_default()
    }

    /// Advanced bridge for component authors that already own a controlled engine model.
    ///
    /// Prefer [`Self::new`] for app code. This method exists so advanced chart coordinators can
    /// graduate to the binding surface without rebuilding already-shared chart engines.
    pub fn from_model(spec: ChartSpec, engine: Model<ChartEngine>) -> Self {
        Self {
            spec,
            engine,
            output: None,
        }
    }

    /// Advanced bridge for component authors that already own controlled engine and output models.
    ///
    /// Prefer [`Self::new`] for ordinary app code. This method keeps advanced coordinators on the
    /// binding surface when they already share output with linking, diagnostics, or overlays.
    pub fn from_models(
        spec: ChartSpec,
        engine: Model<ChartEngine>,
        output: Model<ChartCanvasOutput>,
    ) -> Self {
        Self {
            spec,
            engine,
            output: Some(output),
        }
    }
}

/// App-facing handle for one shared chart engine rendered through multiple grid panels.
///
/// Unlike [`ChartCanvasPanelBinding`], this binding deliberately does not allocate a default output
/// model. Multiple grid panels plus one overlay panel would otherwise race to publish one
/// "current" output. Linked/aggregated chart output should get its own explicit contract.
#[derive(Clone)]
pub struct ChartCanvasMultiGridBinding {
    spec: ChartSpec,
    engine: Model<ChartEngine>,
    grids: Vec<GridId>,
}

impl ChartCanvasMultiGridBinding {
    /// Insert a shared chart engine into a model host and keep the grid order for panel rendering.
    #[track_caller]
    pub fn new(
        host: &mut impl ModelHost,
        spec: ChartSpec,
        engine: ChartEngine,
        grids: impl IntoIterator<Item = GridId>,
    ) -> Self {
        Self {
            spec,
            engine: host.models_mut().insert(engine),
            grids: grids.into_iter().collect(),
        }
    }

    /// Grid IDs rendered by this binding, in caller-provided order.
    pub fn grids(&self) -> &[GridId] {
        &self.grids
    }

    /// Build panel props for one grid viewport, wired to the shared engine model.
    pub fn grid_panel_props(&self, grid: GridId) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(self.spec.clone()).grid_view(grid);
        props.engine = Some(self.engine.clone());
        props
    }

    /// Build overlay-only panel props wired to the shared engine model.
    pub fn overlay_panel_props(&self) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(self.spec.clone()).overlay_only();
        props.engine = Some(self.engine.clone());
        props
    }

    /// Observe the shared chart engine as a paint dependency from an app view render pass.
    pub fn observe_engine_paint<'a, H, Cx>(&self, cx: &mut Cx)
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .observe_model(&self.engine, Invalidation::Paint);
    }
}

/// Shared linked-chart state handle for diagnostics and orchestration code.
///
/// This is a read-only companion to [`ChartCanvasLinkedGroupBinding`]. It lets diagnostics inspect
/// the shared domain-window state without exposing the raw model handle to app examples.
#[derive(Clone)]
pub struct ChartCanvasLinkedStateBinding {
    domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
}

impl ChartCanvasLinkedStateBinding {
    /// Read linked domain windows without registering a UI invalidation dependency.
    pub fn domain_windows_untracked(
        &self,
        host: &impl ModelHost,
    ) -> BTreeMap<LinkAxisKey, Option<DataWindow>> {
        self.domain_windows
            .read_ref(host, Clone::clone)
            .unwrap_or_default()
    }
}

/// App-facing coordinator for linked chart panels.
///
/// The lower-level linking machinery still exists for component authors, but app examples should
/// usually keep linked brush, axis-pointer, domain-window, engine, and output model plumbing behind
/// this binding.
pub struct ChartCanvasLinkedGroupBinding {
    linked: LinkedChartGroup,
    shared_brush: Model<Option<BrushSelectionLink2D>>,
    shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
}

impl ChartCanvasLinkedGroupBinding {
    /// Insert linked-chart shared state into a model host.
    #[track_caller]
    pub fn new(host: &mut impl ModelHost, policy: ChartLinkPolicy) -> Self {
        let shared_brush = host.models_mut().insert(None::<BrushSelectionLink2D>);
        let shared_axis_pointer = host.models_mut().insert(None::<AxisPointerLinkAnchor>);
        let shared_domain_windows = host
            .models_mut()
            .insert(BTreeMap::<LinkAxisKey, Option<DataWindow>>::default());
        let linked = LinkedChartGroup::new(
            policy,
            shared_brush.clone(),
            shared_axis_pointer.clone(),
            shared_domain_windows.clone(),
        );

        Self {
            linked,
            shared_brush,
            shared_axis_pointer,
            shared_domain_windows,
        }
    }

    /// Add a chart panel to the linked group and return its app-facing panel binding.
    #[track_caller]
    pub fn push_panel(
        &mut self,
        host: &mut impl ModelHost,
        spec: ChartSpec,
        engine: ChartEngine,
        router: ChartLinkRouter,
    ) -> ChartCanvasLinkedPanelBinding {
        let engine = host.models_mut().insert(engine);
        let output = host.models_mut().insert(ChartCanvasOutput::default());
        self.linked.push(LinkedChartMember {
            router,
            output: output.clone(),
        });

        ChartCanvasLinkedPanelBinding {
            spec,
            engine,
            output,
            shared_brush: self.shared_brush.clone(),
            shared_axis_pointer: self.shared_axis_pointer.clone(),
            shared_domain_windows: self.shared_domain_windows.clone(),
        }
    }

    /// Tick the linked group and propagate pending link output into shared state.
    pub fn tick<H: UiHost>(&mut self, host: &mut H) -> bool {
        self.linked.tick(host)
    }

    /// Clone a read-only diagnostics handle for the shared linked state.
    pub fn shared_state(&self) -> ChartCanvasLinkedStateBinding {
        ChartCanvasLinkedStateBinding {
            domain_windows: self.shared_domain_windows.clone(),
        }
    }

    /// Read linked domain windows without registering a UI invalidation dependency.
    pub fn domain_windows_untracked(
        &self,
        host: &impl ModelHost,
    ) -> BTreeMap<LinkAxisKey, Option<DataWindow>> {
        self.shared_state().domain_windows_untracked(host)
    }
}

/// App-facing handle for one chart panel owned by a linked chart group.
#[derive(Clone)]
pub struct ChartCanvasLinkedPanelBinding {
    spec: ChartSpec,
    engine: Model<ChartEngine>,
    output: Model<ChartCanvasOutput>,
    shared_brush: Model<Option<BrushSelectionLink2D>>,
    shared_axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    shared_domain_windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
}

impl ChartCanvasLinkedPanelBinding {
    /// Build declarative panel props wired to this linked panel and its shared link state.
    pub fn panel_props(&self) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(self.spec.clone())
            .output_model(self.output.clone())
            .linked_brush(self.shared_brush.clone())
            .linked_axis_pointer(self.shared_axis_pointer.clone())
            .linked_domain_windows(self.shared_domain_windows.clone());
        props.engine = Some(self.engine.clone());
        props
    }

    /// Build declarative panel props and attach a stable test id.
    pub fn panel_props_with_test_id(&self, test_id: impl Into<Arc<str>>) -> ChartCanvasPanelProps {
        self.panel_props().test_id(test_id)
    }

    /// Observe the chart engine as a paint dependency from an app view render pass.
    pub fn observe_engine_paint<'a, H, Cx>(&self, cx: &mut Cx)
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .observe_model(&self.engine, Invalidation::Paint);
    }

    /// Read the latest chart output without registering a UI invalidation dependency.
    pub fn output_untracked(&self, host: &impl ModelHost) -> ChartCanvasOutput {
        self.output.read_ref(host, Clone::clone).unwrap_or_default()
    }

    /// Read the controlled chart engine through the host's normal model read path.
    pub fn read_engine<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut H, &ChartEngine) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.engine.read(host, f)
    }

    /// Mutate the controlled chart engine through the host's normal model update path.
    #[track_caller]
    pub fn update_engine<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut ChartEngine, &mut ModelCx<'_, H>) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.engine.update(host, f)
    }
}

#[cfg(test)]
mod tests {
    use delinea::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId};
    use delinea::{
        AxisKind, ChartEngine, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode,
        SeriesKind, SeriesSpec,
    };
    use fret_runtime::{ModelHost, ModelStore};

    use crate::ChartCanvasPanelMode;

    use super::{
        ChartCanvasLinkedGroupBinding, ChartCanvasMultiGridBinding, ChartCanvasPanelBinding,
    };
    use crate::ChartCanvasOutput;

    #[derive(Default)]
    struct TestHost {
        models: ModelStore,
    }

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    fn sample_spec() -> ChartSpec {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
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
                from: None,
                transforms: Vec::new(),
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![SeriesSpec {
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
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        }
    }

    #[test]
    fn chart_canvas_binding_creates_props_with_engine_and_output_without_public_raw_handle() {
        let mut host = TestHost::default();
        let spec = sample_spec();
        let engine = ChartEngine::new(spec.clone()).expect("sample spec should be valid");

        let binding = ChartCanvasPanelBinding::new(&mut host, spec.clone(), engine);
        let props = binding.panel_props();
        let engine = props
            .engine
            .expect("binding props should include controlled engine model");
        let output = props
            .output_model
            .expect("binding props should include controlled output model");

        assert_eq!(props.spec.id, spec.id);
        assert_eq!(
            host.models()
                .read(&engine, |engine| engine.id())
                .expect("engine model should be readable"),
            spec.id
        );
        assert_eq!(
            host.models()
                .read(&output, |output| output.clone())
                .expect("output model should be readable"),
            binding.output_untracked(&host)
        );
    }

    #[test]
    fn chart_canvas_multi_grid_binding_creates_grid_and_overlay_props_without_output_model() {
        let mut host = TestHost::default();
        let mut spec = sample_spec();
        let grid_1 = GridId::new(1);
        let grid_2 = GridId::new(2);
        spec.grids.push(GridSpec { id: grid_2 });
        let engine = ChartEngine::new(spec.clone()).expect("sample spec should be valid");

        let binding =
            ChartCanvasMultiGridBinding::new(&mut host, spec.clone(), engine, [grid_1, grid_2]);
        let grid_props = binding.grid_panel_props(grid_2);
        let overlay_props = binding.overlay_panel_props();

        assert_eq!(binding.grids(), &[grid_1, grid_2]);
        assert_eq!(grid_props.spec.id, spec.id);
        assert_eq!(grid_props.mode, ChartCanvasPanelMode::GridView(grid_2));
        assert_eq!(overlay_props.mode, ChartCanvasPanelMode::Overlay);
        assert!(grid_props.engine.is_some());
        assert!(overlay_props.engine.is_some());
        assert!(grid_props.output_model.is_none());
        assert!(overlay_props.output_model.is_none());
    }

    #[test]
    fn chart_canvas_linked_group_binding_creates_panel_props_without_public_raw_handles() {
        let mut host = TestHost::default();
        let spec = sample_spec();
        let engine = ChartEngine::new(spec.clone()).expect("sample spec should be valid");
        let mut linked = ChartCanvasLinkedGroupBinding::new(
            &mut host,
            crate::ChartLinkPolicy {
                brush: true,
                axis_pointer: true,
                domain_windows: true,
            },
        );

        let panel = linked.push_panel(
            &mut host,
            spec.clone(),
            engine,
            crate::ChartLinkRouter::from_spec(&spec),
        );
        let props = panel.panel_props_with_test_id("chart-linked-panel");

        assert_eq!(props.spec.id, spec.id);
        assert_eq!(props.test_id.as_deref(), Some("chart-linked-panel"));
        assert!(props.engine.is_some());
        assert!(props.output_model.is_some());
        assert!(props.linked_brush_model.is_some());
        assert!(props.linked_axis_pointer_model.is_some());
        assert!(props.linked_domain_windows_model.is_some());
        assert_eq!(panel.output_untracked(&host), ChartCanvasOutput::default());
        assert!(linked.domain_windows_untracked(&host).is_empty());
        assert!(
            linked
                .shared_state()
                .domain_windows_untracked(&host)
                .is_empty()
        );
    }
}
