//! App-facing plot bindings that hide raw runtime model plumbing from examples.

use fret_runtime::{Model, ModelCx, ModelHost, ModelUpdateError};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use crate::declarative::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    ShadedPlotPanelProps, StemsPlotPanelProps,
};
use crate::linking::{LinkedPlotBinding, LinkedPlotMember};
use crate::models::{
    AreaPlotModel, BarsPlotModel, CandlestickPlotModel, ErrorBarsPlotModel, HeatmapPlotModel,
    Histogram2DPlotModel, HistogramPlotModel, LinePlotModel, ShadedPlotModel, StemsPlotModel,
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
        Self::new_with_state(host, model, PlotState::default())
    }

    #[track_caller]
    fn new_with_state(host: &mut impl ModelHost, model: M, state: PlotState) -> Self {
        Self {
            model: host.models_mut().insert(model),
            state: host.models_mut().insert(state),
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

    fn read_model_untracked<R>(
        &self,
        host: &impl ModelHost,
        f: impl FnOnce(&M) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.model.read_ref(host, f)
    }

    fn update_model<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut M, &mut ModelCx<'_, H>) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.model.update(host, f)
    }

    fn read_state_untracked<R>(
        &self,
        host: &impl ModelHost,
        f: impl FnOnce(&PlotState) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.state.read_ref(host, f)
    }

    fn update_state<R>(
        &self,
        host: &mut impl ModelHost,
        f: impl FnOnce(&mut PlotState) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.state.update(host, |state, _cx| f(state))
    }

    fn linked_member(&self) -> LinkedPlotMember {
        LinkedPlotMember {
            state: self.state.clone(),
            output: self.output.clone(),
        }
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

            /// Insert the plot model with caller-provided initial interaction state.
            #[track_caller]
            pub fn new_with_state(host: &mut impl ModelHost, model: $model, state: PlotState) -> Self {
                Self {
                    core: PlotPanelBindingCore::new_with_state(host, model, state),
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

            /// Read the controlled plot model without exposing the raw model handle.
            ///
            /// This is intended for diagnostics, stress harnesses, and advanced model inspection
            /// that should still keep application code on the binding surface.
            pub fn read_model_untracked<R>(
                &self,
                host: &impl ModelHost,
                f: impl FnOnce(&$model) -> R,
            ) -> Result<R, ModelUpdateError> {
                self.core.read_model_untracked(host, f)
            }

            /// Mutate the controlled plot model without exposing the raw model handle.
            ///
            /// Use this for advanced plot behaviors that own the plot model lifecycle, such as
            /// stress harnesses that deliberately alter data bounds to force path rebuilds.
            #[track_caller]
            pub fn update_model<H: ModelHost, R>(
                &self,
                host: &mut H,
                f: impl FnOnce(&mut $model, &mut ModelCx<'_, H>) -> R,
            ) -> Result<R, ModelUpdateError> {
                self.core.update_model(host, f)
            }

            /// Read caller-owned plot state without registering a UI invalidation dependency.
            ///
            /// This keeps app code on the binding surface when it needs diagnostics or event-time
            /// decisions based on view bounds, query selection, or overlays.
            pub fn read_state_untracked<R>(
                &self,
                host: &impl ModelHost,
                f: impl FnOnce(&PlotState) -> R,
            ) -> Result<R, ModelUpdateError> {
                self.core.read_state_untracked(host, f)
            }

            /// Mutate caller-owned plot state without exposing the raw model handle.
            ///
            /// Use this for advanced plot behaviors such as overlay updates, drag feedback loops,
            /// linked cursor/view coordination, or externally-controlled view bounds.
            pub fn update_state<R>(
                &self,
                host: &mut impl ModelHost,
                f: impl FnOnce(&mut PlotState) -> R,
            ) -> Result<R, ModelUpdateError> {
                self.core.update_state(host, f)
            }

            /// Build the coordinator member used by [`LinkedPlotGroup`](crate::linking::LinkedPlotGroup).
            ///
            /// The returned value is an advanced coordinator bridge; ordinary app code should keep
            /// using the binding methods instead of constructing raw plot model handles.
            pub fn linked_member(&self) -> LinkedPlotMember {
                self.core.linked_member()
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

        impl LinkedPlotBinding for $binding {
            fn linked_plot_member(&self) -> LinkedPlotMember {
                self.core.linked_member()
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

define_plot_panel_binding!(
    /// App-facing handle for an area plot panel plus its caller-owned interaction state.
    ///
    /// `AreaPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone area panel.
    AreaPlotPanelBinding,
    AreaPlotModel,
    AreaPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a shaded plot panel plus its caller-owned interaction state.
    ///
    /// `ShadedPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone shaded-band panel.
    ShadedPlotPanelBinding,
    ShadedPlotModel,
    ShadedPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a candlestick plot panel plus its caller-owned interaction state.
    ///
    /// `CandlestickPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone candlestick panel.
    CandlestickPlotPanelBinding,
    CandlestickPlotModel,
    CandlestickPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a heatmap plot panel plus its caller-owned interaction state.
    ///
    /// `HeatmapPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone heatmap panel.
    HeatmapPlotPanelBinding,
    HeatmapPlotModel,
    HeatmapPlotPanelProps
);

define_plot_panel_binding!(
    /// App-facing handle for a histogram2d plot panel plus its caller-owned interaction state.
    ///
    /// `Histogram2DPlotPanelProps` remains the component-author surface and still exposes raw model
    /// handles for advanced composition. This binding is the default app/cookbook surface for a
    /// standalone histogram2d panel.
    Histogram2DPlotPanelBinding,
    Histogram2DPlotModel,
    Histogram2DPlotPanelProps
);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_runtime::{ModelHost, ModelStore};

    use crate::cartesian::DataPoint;
    use crate::linking::{LinkedPlotGroup, PlotLinkPolicy};
    use crate::models::{
        AreaPlotModel, AreaSeries, BarSeries, BarsPlotModel, CandlestickPlotModel,
        CandlestickSeries, ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries, HeatmapPlotModel,
        Histogram2DPlotModel, HistogramPlotModel, HistogramSeries, LinePlotModel, LineSeries,
        OhlcPoint, ShadedPlotModel, ShadedSeries, StemsPlotModel, StemsSeries,
    };
    use crate::series::Series;
    use crate::state::{InfLineX, PlotOutput, PlotOverlays, PlotState};

    use super::{
        AreaPlotPanelBinding, BarsPlotPanelBinding, CandlestickPlotPanelBinding,
        ErrorBarsPlotPanelBinding, HeatmapPlotPanelBinding, Histogram2DPlotPanelBinding,
        HistogramPlotPanelBinding, LinePlotPanelBinding, ShadedPlotPanelBinding,
        StemsPlotPanelBinding,
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

    fn sample_area_model() -> AreaPlotModel {
        AreaPlotModel::from_series(vec![AreaSeries::new(
            "sample",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )])
    }

    fn sample_shaded_model() -> ShadedPlotModel {
        ShadedPlotModel::from_series(vec![ShadedSeries::new(
            "sample",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 2.0 }, DataPoint { x: 1.0, y: 3.0 }],
                true,
            ),
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
                true,
            ),
        )])
    }

    fn sample_candlestick_model() -> CandlestickPlotModel {
        CandlestickPlotModel::from_series(vec![CandlestickSeries::new_sorted(
            "sample",
            Arc::from([
                OhlcPoint {
                    x: 0.0,
                    open: 1.0,
                    high: 2.0,
                    low: 0.5,
                    close: 1.5,
                },
                OhlcPoint {
                    x: 1.0,
                    open: 1.5,
                    high: 2.5,
                    low: 1.0,
                    close: 2.0,
                },
            ]),
            true,
        )])
    }

    fn sample_heatmap_model() -> HeatmapPlotModel {
        HeatmapPlotModel::new(sample_grid_bounds(), 2, 2, [0.0, 0.5, 0.75, 1.0])
    }

    fn sample_histogram2d_model() -> Histogram2DPlotModel {
        Histogram2DPlotModel::new(sample_grid_bounds(), 2, 2, [0.0, 1.0, 2.0, 3.0])
    }

    fn sample_grid_bounds() -> crate::cartesian::DataRect {
        crate::cartesian::DataRect {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        }
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
    fn line_plot_binding_updates_model_without_exposing_model_handle() {
        let mut host = TestHost::default();

        let binding = LinePlotPanelBinding::new(&mut host, sample_line_model());
        let len = binding.update_model(&mut host, |model, _cx| {
            model.data_bounds.x_min = -1.0;
            model.series.len()
        });

        assert_eq!(len, Ok(1));
        assert_eq!(
            binding.read_model_untracked(&host, |model| model.data_bounds.x_min),
            Ok(-1.0)
        );
    }

    #[test]
    fn line_plot_binding_accepts_initial_state_without_public_raw_handles() {
        let mut host = TestHost::default();
        let initial_state = PlotState {
            overlays: PlotOverlays {
                inf_lines_x: vec![InfLineX::new(0.5)],
                ..Default::default()
            },
            ..Default::default()
        };

        let binding =
            LinePlotPanelBinding::new_with_state(&mut host, sample_line_model(), initial_state);

        assert_eq!(
            binding.read_state_untracked(&host, |state| state.overlays.inf_lines_x.len()),
            Ok(1)
        );
    }

    #[test]
    fn line_plot_binding_updates_state_without_exposing_state_model_handle() {
        let mut host = TestHost::default();

        let binding = LinePlotPanelBinding::new(&mut host, sample_line_model());
        let len = binding.update_state(&mut host, |state| {
            state.overlays.inf_lines_x.push(InfLineX::new(0.75));
            state.overlays.inf_lines_x.len()
        });

        assert_eq!(len, Ok(1));
        assert_eq!(
            binding.read_state_untracked(&host, |state| state.overlays.inf_lines_x[0].x),
            Ok(0.75)
        );
    }

    #[test]
    fn line_plot_binding_creates_linked_member_without_manual_raw_model_wiring() {
        let mut host = TestHost::default();

        let binding = LinePlotPanelBinding::new(&mut host, sample_line_model());
        let mut group = LinkedPlotGroup::new(PlotLinkPolicy::default());
        group.push_binding(&binding);

        assert_eq!(group.len(), 1);
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

    #[test]
    fn area_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = AreaPlotPanelBinding::new(&mut host, sample_area_model());
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
    fn area_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = AreaPlotPanelBinding::new(&mut host, sample_area_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 40,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 40);
    }

    #[test]
    fn shaded_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = ShadedPlotPanelBinding::new(&mut host, sample_shaded_model());
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
    fn shaded_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = ShadedPlotPanelBinding::new(&mut host, sample_shaded_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 48,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 48);
    }

    #[test]
    fn candlestick_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = CandlestickPlotPanelBinding::new(&mut host, sample_candlestick_model());
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
    fn candlestick_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = CandlestickPlotPanelBinding::new(&mut host, sample_candlestick_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 56,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 56);
    }

    #[test]
    fn heatmap_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = HeatmapPlotPanelBinding::new(&mut host, sample_heatmap_model());
        let props = binding.panel_props();

        assert!(props.state.is_some());
        assert!(props.output.is_some());
        assert!(
            host.models()
                .read(&props.model, |model| model.values.len())
                .is_ok()
        );
    }

    #[test]
    fn heatmap_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = HeatmapPlotPanelBinding::new(&mut host, sample_heatmap_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 64,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 64);
    }

    #[test]
    fn histogram2d_plot_binding_creates_props_with_state_and_output_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = Histogram2DPlotPanelBinding::new(&mut host, sample_histogram2d_model());
        let props = binding.panel_props();

        assert!(props.state.is_some());
        assert!(props.output.is_some());
        assert!(
            host.models()
                .read(&props.model, |model| model.values.len())
                .is_ok()
        );
    }

    #[test]
    fn histogram2d_plot_binding_reads_output_without_exposing_output_model_handle() {
        let mut host = TestHost::default();

        let binding = Histogram2DPlotPanelBinding::new(&mut host, sample_histogram2d_model());
        let props = binding.panel_props();
        let output = props
            .output
            .expect("binding props should include output model");
        output
            .update(&mut host, |output, _cx| {
                *output = PlotOutput {
                    revision: 72,
                    ..Default::default()
                };
            })
            .expect("output model update should succeed");

        assert_eq!(binding.output_untracked(&host).revision, 72);
    }
}
