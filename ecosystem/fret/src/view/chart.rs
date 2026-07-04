//! App-facing chart helpers.

use delinea::engine::ChartEngine;
use fret_chart::{ChartCanvasOutput, ChartCanvasPanelProps, ChartInputMap, chart_canvas_panel};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ViewCacheProps};

use super::{AppUi, LocalState, LocalStateRawModelExt as _};

/// App-facing declarative chart canvas builder.
///
/// This keeps default app examples away from raw `Model<T>`, raw `ViewCacheProps`, and
/// `fret_chart::ChartCanvasPanelProps` while still leaving the headless `delinea` chart domain
/// explicit.
pub struct ChartCanvas {
    props: ChartCanvasPanelProps,
    contain_layout_when_bounds_known: bool,
}

impl ChartCanvas {
    pub fn new(spec: delinea::ChartSpec) -> Self {
        Self {
            props: ChartCanvasPanelProps::new(spec),
            contain_layout_when_bounds_known: false,
        }
    }

    pub fn engine(mut self, engine: &LocalState<ChartEngine>) -> Self {
        self.props.engine = Some(engine.clone_model());
        self
    }

    pub fn output(mut self, output: &LocalState<ChartCanvasOutput>) -> Self {
        self.props = self.props.output_model(output.clone_model());
        self
    }

    pub fn input_map(mut self, input_map: ChartInputMap) -> Self {
        self.props = self.props.input_map(input_map);
        self
    }

    pub fn accessibility_layer(mut self, enabled: bool) -> Self {
        self.props = self.props.accessibility_layer(enabled);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<std::sync::Arc<str>>) -> Self {
        self.props = self.props.test_id(test_id);
        self
    }

    pub fn contain_layout_when_bounds_known(mut self, contain: bool) -> Self {
        self.contain_layout_when_bounds_known = contain;
        self
    }

    #[track_caller]
    pub fn into_element<H>(self, cx: &mut AppUi<'_, '_, H>) -> AnyElement
    where
        H: UiHost + 'static,
    {
        let props = self.props;
        if self.contain_layout_when_bounds_known {
            return cx.elements().view_cache(
                ViewCacheProps::default().contain_layout_when_bounds_known(true),
                move |cx| vec![chart_canvas_panel(cx, props)],
            );
        }

        chart_canvas_panel(cx.elements(), props)
    }
}
