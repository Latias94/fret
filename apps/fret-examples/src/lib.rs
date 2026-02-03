#[cfg(not(target_arch = "wasm32"))]
pub mod alloc_profile;

pub(crate) mod hotpatch;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_native_demo<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: fret_app::App,
    driver: D,
) -> anyhow::Result<()> {
    fret_kit::run_native_demo(config, app, driver).map_err(anyhow::Error::from)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod alpha_mode_demo;
pub mod area_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod assets_demo;
pub mod bars_demo;
pub mod candlestick_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod canvas_datagrid_stress_demo;
pub mod category_line_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod chart_declarative_demo;
pub mod chart_demo;
pub mod chart_multi_axis_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod chart_stress_demo;
pub mod cjk_conformance_demo;
pub mod components_gallery;
#[cfg(not(target_arch = "wasm32"))]
pub mod datatable_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod date_picker_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_arbitration_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_demo;
pub mod drag_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod echarts_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod echarts_multi_grid_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod effects_demo;
pub mod emoji_conformance_demo;
pub mod error_bars_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod first_frame_smoke_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod form_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod gizmo3d_demo;
pub mod grouped_bars_demo;
pub mod heatmap_demo;
pub mod histogram2d_demo;
pub mod histogram_demo;
pub mod horizontal_bars_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod image_upload_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod ime_smoke_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_editor_proof_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_hello_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod imui_node_graph_demo;
pub mod inf_lines_demo;
pub mod linked_cursor_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod markdown_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod node_graph_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod node_graph_domain_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub(crate) mod node_graph_tuning_overlay;
#[cfg(not(target_arch = "wasm32"))]
pub mod plot3d_demo;
pub mod plot_demo;
pub mod plot_image_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod plot_stress_demo;
pub mod shaded_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod sonner_demo;
pub mod stacked_bars_demo;
pub mod stairs_demo;
pub mod stems_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_i420_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_image_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_nv12_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod table_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod table_stress_demo;
pub mod tags_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_foreign_iced_style_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_interop_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_interop_kit_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_mvu_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_mvu_interop_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod virtual_list_stress_demo;
