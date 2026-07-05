//! App-facing chart bindings that hide raw runtime model plumbing from examples.

use delinea::{ChartEngine, ChartSpec};
use fret_runtime::{Model, ModelHost};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use crate::ChartCanvasPanelProps;

/// App-facing handle for a chart canvas panel plus its controlled engine model.
///
/// `ChartCanvasPanelProps` remains the component-author surface and still exposes raw model handles
/// for linked charts, multi-grid composition, and advanced output wiring. This binding is the
/// default app/cookbook surface for a single controlled chart canvas.
#[derive(Clone)]
pub struct ChartCanvasPanelBinding {
    spec: ChartSpec,
    engine: Model<ChartEngine>,
}

impl ChartCanvasPanelBinding {
    /// Insert a chart engine into a model host and keep the matching panel spec.
    #[track_caller]
    pub fn new(host: &mut impl ModelHost, spec: ChartSpec, engine: ChartEngine) -> Self {
        Self {
            spec,
            engine: host.models_mut().insert(engine),
        }
    }

    /// Build declarative panel props wired to this binding's controlled engine.
    pub fn panel_props(&self) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(self.spec.clone());
        props.engine = Some(self.engine.clone());
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

    /// Advanced bridge for component authors that already own a controlled engine model.
    ///
    /// Prefer [`Self::new`] for app code. This method exists so advanced chart coordinators can
    /// graduate to the binding surface without rebuilding already-shared chart engines.
    pub fn from_model(spec: ChartSpec, engine: Model<ChartEngine>) -> Self {
        Self { spec, engine }
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

    use super::ChartCanvasPanelBinding;

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
    fn chart_canvas_binding_creates_props_with_engine_without_public_raw_handle() {
        let mut host = TestHost::default();
        let spec = sample_spec();
        let engine = ChartEngine::new(spec.clone()).expect("sample spec should be valid");

        let binding = ChartCanvasPanelBinding::new(&mut host, spec.clone(), engine);
        let props = binding.panel_props();
        let engine = props
            .engine
            .expect("binding props should include controlled engine model");

        assert_eq!(props.spec.id, spec.id);
        assert_eq!(
            host.models()
                .read(&engine, |engine| engine.id())
                .expect("engine model should be readable"),
            spec.id
        );
    }
}
