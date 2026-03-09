use std::sync::Arc;
use std::time::Duration;

use fret_app::{App, Model};
use fret_core::{AppWindowId, Axis, Corners, Edges, Px, SemanticsRole};
use fret_ui::Theme;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, Overflow, PressableA11y,
    PressableProps, SpacingEdges, SpacingLength,
};
use fret_ui_kit::declarative::prelude::{ActionHooksExt as _, ModelWatchExt as _};

#[derive(Debug, Clone)]
struct MinimalState {
    count: Model<u64>,
}

#[derive(Debug, Clone, Copy)]
struct MinimalFlags {
    no_text: bool,
    no_pressable: bool,
}

fn env_bool(name: &str) -> bool {
    std::env::var_os(name)
        .and_then(|v| v.into_string().ok())
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "enabled"
            )
        })
        .unwrap_or(false)
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var_os(name)
        .and_then(|v| v.into_string().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
}

fn dump_wgpu_report(label: &str, instance: &wgpu::Instance) {
    let Some(report) = instance.generate_report() else {
        return;
    };

    let hub = report.hub_report();
    eprintln!(
        "wgpu_report label={label} adapters={} devices={} queues={} encoders={} buffers={} textures={} views={} samplers={} shader_modules={} render_pipelines={} compute_pipelines={}",
        hub.adapters.num_allocated + hub.adapters.num_kept_from_user,
        hub.devices.num_allocated + hub.devices.num_kept_from_user,
        hub.queues.num_allocated + hub.queues.num_kept_from_user,
        hub.command_encoders.num_allocated + hub.command_encoders.num_kept_from_user,
        hub.buffers.num_allocated + hub.buffers.num_kept_from_user,
        hub.textures.num_allocated + hub.textures.num_kept_from_user,
        hub.texture_views.num_allocated + hub.texture_views.num_kept_from_user,
        hub.samplers.num_allocated + hub.samplers.num_kept_from_user,
        hub.shader_modules.num_allocated + hub.shader_modules.num_kept_from_user,
        hub.render_pipelines.num_allocated + hub.render_pipelines.num_kept_from_user,
        hub.compute_pipelines.num_allocated + hub.compute_pipelines.num_kept_from_user,
    );
}

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn init_window(app: &mut App, _window: AppWindowId) -> MinimalState {
    let count = app.models_mut().insert(0u64);
    MinimalState { count }
}

fn view<'a>(
    cx: &mut fret_ui::ElementContext<'a, App>,
    st: &mut MinimalState,
) -> fret_ui::element::Elements {
    let flags = MinimalFlags {
        no_text: env_bool("FRET_MINIMAL_NO_TEXT"),
        no_pressable: env_bool("FRET_MINIMAL_NO_PRESSABLE"),
    };

    let count = if flags.no_text {
        0
    } else {
        cx.watch_model(&st.count).layout().value_or_default()
    };
    let theme = Theme::global(&*cx.app);
    let surface_bg = theme.colors.surface_background;
    let panel_bg = theme.colors.panel_background;
    let hover_bg = theme.colors.hover_background;
    let pressed_bg = theme.colors.selection_background;
    let panel_border = theme.colors.panel_border;

    let mut root = ContainerProps::default();
    root.layout = fill_layout();
    root.background = Some(surface_bg);
    root.padding = SpacingEdges::all(SpacingLength::Px(Px(16.0)));

    let element: AnyElement = cx.container(root, move |cx| {
        let mut flex = FlexProps::default();
        flex.layout = fill_layout();
        flex.direction = Axis::Vertical;
        flex.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
        flex.layout.overflow = Overflow::Clip;

        vec![cx.flex(flex, move |cx| {
            let title = if flags.no_text {
                None
            } else {
                Some(cx.text("fret-minimal baseline"))
            };
            let count_el = if flags.no_text {
                None
            } else {
                let count_label: Arc<str> = Arc::from(format!("count: {count}"));
                Some(cx.text(count_label))
            };

            let mut pressable_layout = LayoutStyle::default();
            pressable_layout.size.width = Length::Px(Px(160.0));

            let button = if flags.no_pressable {
                None
            } else {
                Some(cx.pressable(
                    PressableProps {
                        layout: pressable_layout,
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Increment counter")),
                            test_id: Some(Arc::from("fret-minimal.increment")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, pressable_state| {
                        cx.pressable_update_model(&st.count, |v| *v = v.saturating_add(1));

                        let bg = if pressable_state.pressed {
                            pressed_bg
                        } else if pressable_state.hovered {
                            hover_bg
                        } else {
                            panel_bg
                        };

                        let mut props = ContainerProps::default();
                        props.layout.size.width = Length::Fill;
                        props.padding = SpacingEdges::all(SpacingLength::Px(Px(10.0)));
                        props.background = Some(bg);
                        props.border = Edges::all(Px(1.0));
                        props.border_color = Some(panel_border);
                        props.corner_radii = Corners::all(Px(8.0));

                        let label = if flags.no_text {
                            None
                        } else {
                            Some(cx.text("Increment"))
                        };

                        vec![cx.container(props, move |_cx| label)]
                    },
                ))
            };

            let mut out = Vec::new();
            if let Some(title) = title {
                out.push(title);
            }
            if let Some(count_el) = count_el {
                out.push(count_el);
            }
            if let Some(button) = button {
                out.push(button);
            }
            out
        })]
    });

    element.into()
}

fn main() -> anyhow::Result<()> {
    let frame_interval_ms = env_u64("FRET_MINIMAL_FRAME_INTERVAL_MS");
    let accessibility_enabled = if std::env::var_os("FRET_MINIMAL_ACCESSIBILITY_ENABLED").is_some()
    {
        env_bool("FRET_MINIMAL_ACCESSIBILITY_ENABLED")
    } else {
        true
    };
    let path_msaa_samples = env_u64("FRET_MINIMAL_PATH_MSAA_SAMPLES")
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(4);

    let renderer_intermediate_budget_mb = env_u64("FRET_MINIMAL_RENDERER_INTERMEDIATE_BUDGET_MB");
    let svg_raster_budget_mb = env_u64("FRET_MINIMAL_SVG_RASTER_BUDGET_MB");
    let streaming_staging_budget_mb = env_u64("FRET_MINIMAL_STREAMING_STAGING_BUDGET_MB");
    let streaming_upload_budget_mb = env_u64("FRET_MINIMAL_STREAMING_UPLOAD_BUDGET_MB_PER_FRAME");
    let wgpu_report_enabled = env_bool("FRET_MINIMAL_WGPU_REPORT");

    fret_bootstrap::ui_app("fret-minimal", init_window, view)
        .on_gpu_ready(move |_app, context, _renderer| {
            if !wgpu_report_enabled {
                return;
            }

            let info = context.adapter.get_info();
            eprintln!(
                "wgpu_adapter backend={:?} device_type={:?} name={}",
                info.backend, info.device_type, info.name
            );

            dump_wgpu_report("gpu_ready", &context.instance);

            let instance = context.instance.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(10));
                dump_wgpu_report("t+10s", &instance);
            });
        })
        .configure(move |config| {
            if let Some(ms) = frame_interval_ms {
                config.frame_interval = Duration::from_millis(ms);
            }
            config.accessibility_enabled = accessibility_enabled;
            config.path_msaa_samples = path_msaa_samples.max(1);

            if let Some(mb) = renderer_intermediate_budget_mb {
                config.renderer_intermediate_budget_bytes = mb.saturating_mul(1024 * 1024);
            }
            if let Some(mb) = svg_raster_budget_mb {
                config.svg_raster_budget_bytes = mb.saturating_mul(1024 * 1024);
            }
            if let Some(mb) = streaming_staging_budget_mb {
                config.streaming_staging_budget_bytes = mb.saturating_mul(1024 * 1024);
            }
            if let Some(mb) = streaming_upload_budget_mb {
                config.streaming_upload_budget_bytes_per_frame = mb.saturating_mul(1024 * 1024);
            }
        })
        .with_main_window("fret-minimal", (560.0, 360.0))
        .run()?;
    Ok(())
}
