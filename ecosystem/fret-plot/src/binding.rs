//! App-facing plot bindings that hide raw runtime model plumbing from examples.

use fret_runtime::{Model, ModelHost};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use crate::declarative::{
    BarsPlotPanelProps, ErrorBarsPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    StemsPlotPanelProps,
};
use crate::models::{
    BarsPlotModel, ErrorBarsPlotModel, HistogramPlotModel, LinePlotModel, StemsPlotModel,
};
use crate::state::{PlotOutput, PlotState};

#[derive(Clone)]
struct PlotPanelBindingCore<M> {
    model: Model<M>,
    state: Model<PlotState>,
    output: Model<PlotOutput>,
}

impl<M: 'static> PlotPanelBindingCore<M> {
    #[track_caller]
    fn new(host: &mut impl ModelHost, model: M) -> Self {
        Self {
            model: host.models_mut().insert(model),
            state: host.models_mut().insert(PlotState::default()),
            output: host.models_mut().insert(PlotOutput::default()),
        }
    }

    fn output_layout<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .get_model_copied(&self.output, Invalidation::Layout)
            .unwrap_or_default()
    }

    fn output_paint<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        cx.elements()
            .get_model_copied(&self.output, Invalidation::Paint)
            .unwrap_or_default()
    }

    fn output_untracked(&self, host: &impl ModelHost) -> PlotOutput {
        self.output
            .read_ref(host, |output| *output)
            .unwrap_or_default()
    }
}

macro_rules! define_plot_panel_binding {
    ($(#[$type_meta:meta])* $binding:ident, $model:ty, $props:ty) => {
        $(#[$type_meta])*
        #[derive(Clone)]
        pub struct $binding {
            core: PlotPanelBindingCore<$model>,
        }

        impl $binding {
            /// Insert the plot model, interaction state, and output channel into a model host.
            #[track_caller]
            pub fn new(host: &mut impl ModelHost, model: $model) -> Self {
                Self {
                    core: PlotPanelBindingCore::new(host, model),
                }
            }

            /// Build declarative panel props wired to this binding's model, state, and output channel.
            pub fn panel_props(&self) -> $props {
                <$props>::new(self.core.model.clone())
                    .state(self.core.state.clone())
                    .output(self.core.output.clone())
            }

            /// Read the latest plot output through a layout invalidation tracked read.
            pub fn output_layout<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
            where
                H: UiHost + 'a,
                Cx: ElementContextAccess<'a, H>,
            {
                self.core.output_layout(cx)
            }

            /// Read the latest plot output through a paint invalidation tracked read.
            pub fn output_paint<'a, H, Cx>(&self, cx: &mut Cx) -> PlotOutput
            where
                H: UiHost + 'a,
                Cx: ElementContextAccess<'a, H>,
            {
                self.core.output_paint(cx)
            }

            /// Read the latest plot output without registering a UI invalidation dependency.
            ///
            /// This is intended for event handlers, diagnostics, and logging code that needs to
            /// observe interaction output outside a render/layout context.
            pub fn output_untracked(&self, host: &impl ModelHost) -> PlotOutput {
                self.core.output_untracked(host)
            }

            /// Advanced bridge for component authors that already own raw model handles.
            ///
            /// Prefer [`Self::new`] for app code. This method exists so advanced plot coordinators
            /// can graduate to the binding surface without rebuilding already-shared plot models.
            pub fn from_models(
                model: Model<$model>,
                state: Model<PlotState>,
                output: Model<PlotOutput>,
            ) -> Self {
                Self {
                    core: PlotPanelBindingCore {
                        model,
                        state,
                        output,
                    },
                }
            }
        }
    };
}

define_plot_panel_binding!(
    /// App-facing handle for a line plot panel plus its caller-owned interaction state.
    ///
    /// `LinePlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface: callers
    /// store one handle, pass it to a panel adapter, and read plot output without naming `Model<T>`
    /// directly.
    LinePlotPanelBinding,
    LinePlotModel,
    LinePlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a histogram plot panel plus its caller-owned interaction state.
    ///
    /// `HistogramPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone histogram panel.
    HistogramPlotPanelBinding,
    HistogramPlotModel,
    HistogramPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a stems plot panel plus its caller-owned interaction state.
    ///
    /// `StemsPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone stems panel.
    StemsPlotPanelBinding,
    StemsPlotModel,
    StemsPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for an error-bars plot panel plus its caller-owned interaction state.
    ///
    /// `ErrorBarsPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone error-bars panel.
    ErrorBarsPlotPanelBinding,
    ErrorBarsPlotModel,
    ErrorBarsPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a bars plot panel plus its caller-owned interaction state.
    ///
    /// `BarsPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for
    /// standalone grouped or stacked bars panels.
    BarsPlotPanelBinding,
    BarsPlotModel,
    BarsPlotPanelProps
);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_runtime::{ModelHost, ModelStore};

    use crate::cartesian::DataPoint;
    use crate::models::{
        BarSeries, BarsPlotModel, ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries,
        HistogramPlotModel, HistogramSeries, LinePlotModel, LineSeries, StemsPlotModel,
        StemsSeries,
    };
    use crate::series::Series;
    use crate::state::PlotOutput;

    use super::{
        BarsPlotPanelBinding, ErrorBarsPlotPanelBinding, HistogramPlotPanelBinding,
        LinePlotPanelBinding, StemsPlotPanelBinding,
    };

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

    fn sample_line_model() -> LinePlotModel {
        LinePlotModel::from_series(vec![LineSeries::new(
            "sample",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )])
    }

    fn sample_histogram_model() -> HistogramPlotModel {
        HistogramPlotModel::from_series(vec![HistogramSeries::new(
            "sample",
            Arc::from([0.0, 0.5, 1.0, 1.5, 2.0]),
        )])
    }

    fn sample_stems_model() -> StemsPlotModel {
        StemsPlotModel::from_series(vec![StemsSeries::new(
            "sample",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )])
    }

    fn sample_error_bars_model() -> ErrorBarsPlotModel {
        ErrorBarsPlotModel::from_series(vec![
            ErrorBarsSeries::new(
                "sample",
                Series::from_points_sorted(
                    vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                    true,
                ),
            )
            .y_errors(Arc::from([
                ErrorBar::symmetric(0.1),
                ErrorBar::symmetric(0.2),
            ])),
        ])
    }

    fn sample_bars_model() -> BarsPlotModel {
        BarsPlotModel::from_series(vec![BarSeries::new(
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

        let binding = LinePlotPanelBinding::new(&mut host, sample_line_model());
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

        let binding = LinePlotPanelBinding::new(&mut host, sample_line_model());
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

    #[test]
    fn histogram_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = HistogramPlotPanelBinding::new(&mut host, sample_histogram_model());
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
    fn histogram_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = HistogramPlotPanelBinding::new(&mut host, sample_histogram_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 24,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 24);
    }

    #[test]
    fn stems_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = StemsPlotPanelBinding::new(&mut host, sample_stems_model());
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
    fn stems_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = StemsPlotPanelBinding::new(&mut host, sample_stems_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 12,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 12);
    }

    #[test]
    fn error_bars_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = ErrorBarsPlotPanelBinding::new(&mut host, sample_error_bars_model());
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
    fn error_bars_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = ErrorBarsPlotPanelBinding::new(&mut host, sample_error_bars_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 18,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 18);
    }

    #[test]
    fn bars_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = BarsPlotPanelBinding::new(&mut host, sample_bars_model());
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
    fn bars_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = BarsPlotPanelBinding::new(&mut host, sample_bars_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 32,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 32);
    }
}
