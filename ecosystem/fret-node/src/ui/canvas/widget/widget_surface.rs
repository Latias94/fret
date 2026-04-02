use super::*;

#[path = "widget_surface/builders.rs"]
mod builders;
#[path = "widget_surface/constants.rs"]
mod constants;
#[path = "widget_surface/construct.rs"]
mod construct;
#[path = "widget_surface/fit_view.rs"]
mod fit_view;
#[path = "widget_surface/runtime.rs"]
mod runtime;
#[path = "widget_surface/sync.rs"]
mod sync;

impl NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    pub fn new(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        editor_config: Model<NodeGraphEditorConfig>,
    ) -> Self {
        Self::new_with_middleware(
            graph,
            view_state,
            editor_config,
            NoopNodeGraphCanvasMiddleware,
        )
    }
}
