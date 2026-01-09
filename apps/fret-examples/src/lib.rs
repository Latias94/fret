#[cfg(not(target_arch = "wasm32"))]
pub mod alloc_profile;

pub(crate) mod hotpatch;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_native_demo<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: fret_app::App,
    driver: D,
) -> anyhow::Result<()> {
    use anyhow::Context as _;
    use fret_bootstrap::BootstrapBuilder;

    BootstrapBuilder::new(app, driver)
        .configure(move |c| {
            *c = config;
        })
        .with_default_settings_json()
        .context("load .fret/settings.json")?
        .register_icon_pack(fret_icons_lucide::register_icons)
        .run()
        .map_err(anyhow::Error::from)
}

pub mod area_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod assets_demo;
pub mod bars_demo;
pub mod candlestick_demo;
pub mod chart_demo;
pub mod components_gallery;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_arbitration_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_demo;
pub mod drag_demo;
pub mod error_bars_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod first_frame_smoke_demo;
pub mod grouped_bars_demo;
pub mod heatmap_demo;
pub mod histogram2d_demo;
pub mod histogram_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod image_upload_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod ime_smoke_demo;
pub mod inf_lines_demo;
pub mod linked_cursor_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod markdown_demo;
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
pub mod table_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod table_stress_demo;
pub mod tags_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod virtual_list_stress_demo;

#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod node_graph_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod node_graph_domain_demo;
