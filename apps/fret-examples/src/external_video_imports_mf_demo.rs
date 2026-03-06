use anyhow::Context as _;
use fret_app::App;
use fret_core::{AppWindowId, Event, KeyCode};
use fret_launch::{
    EngineFrameUpdate,
    imported_viewport_target::{
        ImportedViewportFallbackUpdate, ImportedViewportFallbacks, ImportedViewportRenderTarget,
    },
};
use fret_render::RendererCapabilities;
use fret_render::{
    RenderTargetColorSpace, RenderTargetIngestStrategy, RenderTargetMetadata, Renderer, WgpuContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{
    ContainerProps, CrossAlign, Elements, FlexProps, LayoutStyle, Length, MainAlign,
    ViewportSurfaceProps,
};
use fret_ui::{ElementContext, Invalidation, Theme};

fn env_flag_default_false(name: &str) -> bool {
    let Ok(raw) = std::env::var(name) else {
        return false;
    };
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
use fret_launch::media::windows_mf_video as wmf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternalVideoImportsMode {
    CheckerGpu,
    #[cfg(target_os = "windows")]
    MfVideoCpuUpload,
    #[cfg(target_os = "windows")]
    MfVideoDx12GpuCopy,
}

struct ExternalVideoImportsState {
    show: fret_runtime::Model<bool>,
    mode: ExternalVideoImportsMode,

    target: ImportedViewportRenderTarget,
    target_px_size: (u32, u32),
    desired_target_px_size: (u32, u32),
    checker_texture: Option<wgpu::Texture>,

    #[cfg(target_os = "windows")]
    mf_importer: Option<wmf::MfVideoNativeExternalImporter>,
}

fn init_window(app: &mut App, _window: AppWindowId) -> ExternalVideoImportsState {
    ExternalVideoImportsState {
        show: app.models_mut().insert(true),
        // Use BGRA to align with Media Foundation's RGB32 output (little-endian BGRA).
        target: ImportedViewportRenderTarget::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
        ),
        target_px_size: (1, 1),
        desired_target_px_size: (1280, 720),
        checker_texture: None,
        mode: ExternalVideoImportsMode::CheckerGpu,
        #[cfg(target_os = "windows")]
        mf_importer: None,
    }
}

fn on_event(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut ExternalVideoImportsState,
    event: &Event,
) {
    if let Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyV
    {
        let _ = app.models_mut().update(&st.show, |v| *v = !*v);
        app.request_redraw(window);
    }

    if let Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyI
    {
        st.mode = match st.mode {
            ExternalVideoImportsMode::CheckerGpu => {
                #[cfg(target_os = "windows")]
                {
                    ExternalVideoImportsMode::MfVideoCpuUpload
                }
                #[cfg(not(target_os = "windows"))]
                {
                    ExternalVideoImportsMode::CheckerGpu
                }
            }
            #[cfg(target_os = "windows")]
            ExternalVideoImportsMode::MfVideoCpuUpload => {
                if env_flag_default_false("FRET_EXTV2_MF_DX12_GPU_COPY") {
                    ExternalVideoImportsMode::MfVideoDx12GpuCopy
                } else {
                    ExternalVideoImportsMode::CheckerGpu
                }
            }
            #[cfg(target_os = "windows")]
            ExternalVideoImportsMode::MfVideoDx12GpuCopy => ExternalVideoImportsMode::CheckerGpu,
        };
        #[cfg(target_os = "windows")]
        {
            st.mf_importer = None;
        }
        st.checker_texture = None;
        app.request_redraw(window);
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsState) -> Elements {
    cx.observe_model(&st.show, Invalidation::Layout);

    let scale_factor = cx.environment_scale_factor(Invalidation::Layout);
    let w_px = (cx.bounds.size.width.0.max(1.0) * scale_factor).round() as u32;
    let h_px = (cx.bounds.size.height.0.max(1.0) * scale_factor).round() as u32;
    st.desired_target_px_size = (w_px.max(1).min(4096), h_px.max(1).min(4096));

    let show = cx.app.models().read(&st.show, |v| *v).unwrap_or(true);

    let theme = Theme::global(&*cx.app).snapshot();

    let mut fill = LayoutStyle::default();
    fill.size.width = Length::Fill;
    fill.size.height = Length::Fill;

    let mut panel_layout = LayoutStyle::default();
    panel_layout.size.width = Length::Px(fret_core::Px(980.0));
    panel_layout.size.height = Length::Px(fret_core::Px(720.0));

    let mut row = FlexProps {
        layout: fill,
        direction: fret_core::Axis::Horizontal,
        gap: fret_ui::element::SpacingLength::Px(fret_core::Px(12.0)),
        padding: fret_core::Edges::all(fret_core::Px(16.0)).into(),
        justify: MainAlign::Start,
        align: CrossAlign::Start,
        wrap: false,
    };
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Fill;

    let target = st.target.id();
    let target_px_size = st.target_px_size;

    vec![
        cx.container(
            ContainerProps {
                layout: fill,
                background: Some(theme.color_token("background")),
                ..Default::default()
            },
            |cx| {
                vec![cx.flex(row, |cx| {
                    vec![
                        cx.container(
                            ContainerProps {
                                layout: panel_layout,
                                border: fret_core::Edges::all(fret_core::Px(1.0)),
                                border_paint: Some(fret_core::scene::Paint::Solid(
                                    theme.color_token("border"),
                                )),
                                background: Some(theme.color_token("muted")),
                                corner_radii: fret_core::Corners::all(fret_core::Px(10.0)),
                                ..Default::default()
                            },
                            |cx| {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                vec![
                                    cx.viewport_surface_props(ViewportSurfaceProps {
                                        layout,
                                        target,
                                        target_px_size,
                                        fit: fret_core::ViewportFit::Contain,
                                        opacity: if show { 1.0 } else { 0.0 },
                                    })
                                    .test_id("external-video-imports-mf-surface"),
                                ]
                            },
                        )
                        .test_id("external-video-imports-mf-root"),
                    ]
                })]
            },
        )
        .test_id("external-video-imports-mf-app"),
    ]
    .into()
}

fn record_engine_frame(
    app: &mut App,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut ExternalVideoImportsState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _scale_factor: f32,
    _tick_id: fret_runtime::TickId,
    frame_id: fret_runtime::FrameId,
) -> EngineFrameUpdate {
    let show = app.models().read(&st.show, |v| *v).unwrap_or(true);
    let mut update = EngineFrameUpdate::default();

    if !show {
        st.target.push_unregister(&mut update);
        st.checker_texture = None;
        st.target_px_size = (1, 1);
        #[cfg(target_os = "windows")]
        {
            st.mf_importer = None;
        }
        return update;
    }

    let metadata = RenderTargetMetadata::default();
    match st.mode {
        ExternalVideoImportsMode::CheckerGpu => {
            let desired = st.desired_target_px_size;
            let needs_realloc = st.checker_texture.is_none() || st.target_px_size != desired;
            if needs_realloc {
                let view_formats = [wgpu::TextureFormat::Bgra8UnormSrgb];
                let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("external video imports checker texture"),
                    size: wgpu::Extent3d {
                        width: desired.0.max(1),
                        height: desired.1.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &view_formats,
                });
                st.checker_texture = Some(texture);
                st.target_px_size = desired;
            }

            let texture = st
                .checker_texture
                .as_ref()
                .expect("checker texture allocated");
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(st.target.format()),
                ..Default::default()
            });

            if !st.target.is_registered() {
                let _ = st
                    .target
                    .ensure_registered(renderer, view.clone(), st.target_px_size);
            }

            st.target.push_update_with_fallbacks(
                &mut update,
                RenderTargetIngestStrategy::Owned,
                ImportedViewportFallbacks {
                    owned: Some(ImportedViewportFallbackUpdate::new(
                        view.clone(),
                        st.target_px_size,
                        metadata,
                        None,
                    )),
                    ..Default::default()
                },
            );

            // A tiny animated clear tint is enough to keep the contract-path hot.
            let t = frame_id.0 as f32 * (1.0 / 60.0);
            let pulse = (t * 0.5).sin() * 0.5 + 0.5;
            let color = wgpu::Color {
                r: 0.06 + pulse as f64 * 0.02,
                g: 0.08 + pulse as f64 * 0.03,
                b: 0.11 + pulse as f64 * 0.04,
                a: 1.0,
            };
            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("external video imports checker encoder"),
                    });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("external video imports checker pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                let _ = &mut pass;
            }
            update.push_command_buffer(encoder.finish());
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoCpuUpload => {
            let Some(path) = std::env::var("FRET_MF_VIDEO_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; falling back to checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
                st.mf_importer = None;
                return update;
            };

            let recreate = st
                .mf_importer
                .as_ref()
                .map(|v| v.path() != path || v.prefer_dx12_gpu_copy())
                .unwrap_or(true);
            if recreate {
                st.mf_importer = Some(wmf::MfVideoNativeExternalImporter::new(path, false));
            }

            let caps = app
                .global::<RendererCapabilities>()
                .expect("renderer capabilities must be set before record_engine_frame");
            let frame = st
                .mf_importer
                .as_ref()
                .expect("mf importer created")
                .frame();

            match st.target.push_native_external_import_update(
                renderer,
                &mut update,
                context,
                &caps,
                frame,
            ) {
                Ok(()) => {
                    if let Some(size) = st.mf_importer.as_ref().and_then(|v| v.last_size()) {
                        st.target_px_size = size;
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        ?err,
                        "MF CPU upload native adapter import failed; falling back to checker mode"
                    );
                    st.mode = ExternalVideoImportsMode::CheckerGpu;
                    st.mf_importer = None;
                }
            }
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoDx12GpuCopy => {
            let Some(path) = std::env::var("FRET_MF_VIDEO_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; falling back to checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
                st.mf_importer = None;
                return update;
            };

            let recreate = st
                .mf_importer
                .as_ref()
                .map(|v| v.path() != path || !v.prefer_dx12_gpu_copy())
                .unwrap_or(true);
            if recreate {
                st.mf_importer = Some(wmf::MfVideoNativeExternalImporter::new(path, true));
            }

            let caps = app
                .global::<RendererCapabilities>()
                .expect("renderer capabilities must be set before record_engine_frame");
            let frame = st
                .mf_importer
                .as_ref()
                .expect("mf importer created")
                .frame();

            match st.target.push_native_external_import_update(
                renderer,
                &mut update,
                context,
                &caps,
                frame,
            ) {
                Ok(()) => {
                    if let Some(size) = st.mf_importer.as_ref().and_then(|v| v.last_size()) {
                        st.target_px_size = size;
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        ?err,
                        "MF DX12 GPU copy native adapter import failed; falling back to CPU upload mode"
                    );
                    st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                    st.mf_importer = None;
                }
            }
        }
    }

    app.push_effect(fret_app::Effect::RequestAnimationFrame(window));
    update
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let builder = fret::app_with_hooks("external-video-imports-mf", init_window, view, |driver| {
        driver
            .on_event(on_event)
            .record_engine_frame(record_engine_frame)
    })?
    .init_app(|app| {
        app.set_global(PlatformCapabilities::default());
    })
    .with_main_window(
        "fret-demo external_video_imports_mf_demo (V toggles visibility, I toggles source)",
        (980.0, 720.0),
    );

    builder.run().context("run external_video_imports_mf_demo")
}
