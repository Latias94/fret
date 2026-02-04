//! Immediate-mode adapters for embedding `fret-plot` retained canvases into an `imui` tree.
//!
//! This module is intentionally tiny: it provides glue for authoring frontends that implement
//! `UiWriter` without coupling to a concrete `ImUi` type.

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_ui::UiHost;
use fret_ui::element::LayoutStyle;

/// Options for embedding a plot canvas in an imui tree.
#[derive(Debug, Clone)]
pub struct PlotCanvasImUiOptions {
    /// Layout for the host element (defaults to fill + grow).
    pub layout: LayoutStyle,
}

impl Default for PlotCanvasImUiOptions {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        layout.flex.grow = 1.0;
        Self { layout }
    }
}

/// Embed a retained `LinePlotCanvas` into an imui output list.
///
/// Notes:
/// - This adapter uses the feature-gated retained bridge (`unstable-retained-bridge`).
/// - Provide stable identity by wrapping the call in `ui.id(...)` / `ui.keyed(...)` when rendering
///   inside dynamic collections.
#[track_caller]
pub fn line_plot_canvas_with<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    options: PlotCanvasImUiOptions,
    canvas: impl Fn() -> crate::retained::LinePlotCanvas + 'static,
) {
    let canvas: Arc<dyn Fn() -> crate::retained::LinePlotCanvas> = Arc::new(canvas);

    let props = fret_ui::retained_bridge::RetainedSubtreeProps {
        layout: options.layout,
        factory: fret_ui::retained_bridge::RetainedSubtreeFactory::new::<H>({
            let canvas = canvas.clone();
            move |ui_tree| crate::retained::LinePlotCanvas::create_node(ui_tree, (canvas)())
        }),
    };

    let element = ui.with_cx_mut(|cx| cx.retained_subtree(props));
    ui.add(element);
}

/// Convenience wrapper that uses default options (fill + grow layout).
#[track_caller]
pub fn line_plot_canvas<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    canvas: impl Fn() -> crate::retained::LinePlotCanvas + 'static,
) {
    line_plot_canvas_with(ui, PlotCanvasImUiOptions::default(), canvas);
}
