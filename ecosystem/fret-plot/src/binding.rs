//! App-facing plot bindings that hide raw runtime model plumbing from examples.

use fret_runtime::{Model, ModelHost};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use crate::declarative::LinePlotPanelProps;
use crate::models::LinePlotModel;
use crate::state::{PlotOutput, PlotState};

/// App-facing handle for a line plot panel plus its caller-owned interaction state.
///
/// `LinePlotPanelProps` remains the component-author surface and still exposes raw model handles
/// for advanced composition. This binding is the default app/cookbook surface: callers store one
/// handle, pass it to a panel adapter, and read plot output without naming `Model<T>` directly.
#[derive(Clone)]
pub struct LinePlotPanelBinding {
    model: Model<LinePlotModel>,
    state: Model<PlotState>,
    output: Model<PlotOutput>,
}

impl LinePlotPanelBinding {
    /// Insert the plot model, interaction state, and output channel into a model host.
    #[track_caller]
    pub fn new(host: &mut impl ModelHost, model: LinePlotModel) -> Self {
        Self {
            model: host.models_mut().insert(model),
            state: host.models_mut().insert(PlotState::default()),
            output: host.models_mut().insert(PlotOutput::default()),
        }
    }

    /// Build declarative panel props wired to this binding's model, state, and output channel.
    pub fn panel_props(&self) -> LinePlotPanelProps {
        LinePlotPanelProps::new(self.model.clone())
            .state(self.state.clone())
            .output(self.output.clone())
    }

    /// Read the latest plot output through a layout invalidation tracked read.
    pub fn output_layout<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .get_model_copied(&self.output, Invalidation::Layout)
            .unwrap_or_default()
    }

    /// Read the latest plot output through a paint invalidation tracked read.
    pub fn output_paint<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .get_model_copied(&self.output, Invalidation::Paint)
            .unwrap_or_default()
    }

    /// Read the latest plot output without registering a UI invalidation dependency.
    ///
    /// This is intended for event handlers, diagnostics, and logging code that needs to observe
    /// interaction output outside a render/layout context.
    pub fn output_untracked(&self, host: &impl ModelHost) -> PlotOutput {
        self.output
            .read_ref(host, |output| *output)
            .unwrap_or_default()
    }

    /// Advanced bridge for component authors that already own raw model handles.
    ///
    /// Prefer [`Self::new`] for app code. This method exists so advanced plot coordinators can
    /// graduate to the binding surface without rebuilding already-shared plot models.
    pub fn from_models(
        model: Model<LinePlotModel>,
        state: Model<PlotState>,
        output: Model<PlotOutput>,
    ) -> Self {
        Self {
            model,
            state,
            output,
        }
    }
}

#[cfg(test)]
mod tests {
    use fret_runtime::{ModelHost, ModelStore};

    use crate::cartesian::DataPoint;
    use crate::models::{LinePlotModel, LineSeries};
    use crate::series::Series;
    use crate::state::PlotOutput;

    use super::LinePlotPanelBinding;

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

    fn sample_model() -> LinePlotModel {
        LinePlotModel::from_series(vec![LineSeries::new(
            "sample",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )])
    }

    #[test]
    fn line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = LinePlotPanelBinding::new(&mut host, sample_model());
        let props = binding.panel_props();

        assert!(props.state.is_some());
        assert!(props.output.is_some());
        assert!(
            host.models()
                .read(&props.model, |model| model.series.len())
                .is_ok()
        );
    }

    #[test]
    fn line_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = LinePlotPanelBinding::new(&mut host, sample_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 42,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 42);
    }
}
