use anyhow::Context as _;
use fret_app::App;
use fret_core::{AppWindowId, Event, KeyCode};
use fret_launch::{EngineFrameUpdate, ImportedViewportRenderTarget};
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
use fret_launch::runner::windows_mf_video as wmf;

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
    texture: Option<wgpu::Texture>,

    #[cfg(target_os = "windows")]
    mf: Option<wmf::MfVideoReader>,
    #[cfg(target_os = "windows")]
    mf_dx12: Option<wmf::Dx12GpuCopySession>,
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
        texture: None,
        mode: ExternalVideoImportsMode::CheckerGpu,
        #[cfg(target_os = "windows")]
        mf: None,
        #[cfg(target_os = "windows")]
        mf_dx12: None,
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
        gap: fret_core::Px(12.0),
        padding: fret_core::Edges::all(fret_core::Px(16.0)),
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
        st.texture = None;
        st.target_px_size = (1, 1);
        return update;
    }

    #[cfg(target_os = "windows")]
    {
        if st.mode == ExternalVideoImportsMode::MfVideoCpuUpload && st.mf.is_none() {
            if let Ok(path) = std::env::var("FRET_MF_VIDEO_PATH")
                && !path.trim().is_empty()
            {
                match wmf::MfVideoReader::new(path) {
                    Ok(reader) => st.mf = Some(reader),
                    Err(err) => {
                        tracing::warn!(?err, "failed to initialize MF reader; falling back");
                        st.mode = ExternalVideoImportsMode::CheckerGpu;
                    }
                }
            } else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; staying on checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
            }
        }
    }

    // Decide target size:
    // - checker mode: follow viewport size (stress the contract-path update shapes),
    // - MF mode: follow decoded frame size (avoid per-frame rescale churn in M2A).
    let mut decoded_frame: Option<((u32, u32), u32, Vec<u8>)> = None;
    let desired = match st.mode {
        ExternalVideoImportsMode::CheckerGpu => st.desired_target_px_size,
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoCpuUpload => {
            if let Some(reader) = st.mf.as_mut() {
                if let Some(frame) = reader.read_next().ok().flatten() {
                    decoded_frame = Some((frame.size, frame.bytes_per_row, frame.bgra8));
                    frame.size
                } else {
                    st.desired_target_px_size
                }
            } else {
                st.desired_target_px_size
            }
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoDx12GpuCopy => st
            .mf_dx12
            .as_ref()
            .map(|v| v.size())
            .unwrap_or(st.desired_target_px_size),
    };

    let needs_realloc = st.texture.is_none() || st.target_px_size != desired;
    if needs_realloc {
        // Allocate the shared-allocation texture as *linear* BGRA8, but expose an sRGB view for
        // the UI contract-path surface. This avoids relying on backend-specific support for
        // wrapping SRGB-format resources in D3D11On12 interop.
        let view_formats = [wgpu::TextureFormat::Bgra8UnormSrgb];
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("external video imports texture"),
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(st.target.format()),
            ..Default::default()
        });

        if !st.target.is_registered() {
            let _ = st.target.ensure_registered(renderer, view.clone(), desired);
        }

        st.texture = Some(texture);
        st.target_px_size = desired;
        #[cfg(target_os = "windows")]
        {
            st.mf_dx12 = None;
        }
    }

    let texture = st.texture.as_ref().expect("texture allocated");
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(st.target.format()),
        ..Default::default()
    });

    let mut metadata = RenderTargetMetadata::default();
    match st.mode {
        ExternalVideoImportsMode::CheckerGpu => {
            st.target.push_update_with_deterministic_fallback(
                &mut update,
                view.clone(),
                st.target_px_size,
                metadata,
                RenderTargetIngestStrategy::Owned,
                &[RenderTargetIngestStrategy::Owned],
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
            if let Some(reader) = st.mf.as_ref() {
                metadata.color_encoding = reader.color_encoding();
            }

            // Stage M2A: request the v2 ceiling, but deterministically degrade to CPU upload.
            // The requested/effective split is visible in perf bundles.
            st.target.push_update_with_deterministic_fallback(
                &mut update,
                view.clone(),
                st.target_px_size,
                metadata,
                RenderTargetIngestStrategy::ExternalZeroCopy,
                &[RenderTargetIngestStrategy::CpuUpload],
            );

            if let Some((size, bytes_per_row, bgra8)) = decoded_frame.as_ref() {
                // The helper pads rows to wgpu alignment; the bytes are BGRA8 matching the texture format.
                fret_render::write_rgba8_texture_region(
                    &context.queue,
                    texture,
                    (0, 0),
                    *size,
                    *bytes_per_row,
                    bgra8,
                );
            }
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoDx12GpuCopy => {
            // Stage M2B (shared allocation / GPU copy): request v2's ceiling, but deterministically
            // degrade to a GPU copy into a renderer-owned texture on capable backends.
            let Some(path) = std::env::var("FRET_MF_VIDEO_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; falling back to checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
                return update;
            };

            if st.mf_dx12.is_none() {
                match wmf::Dx12GpuCopySession::new(context, texture, &path)
                    .context("init MF DX12 GPU copy session")
                {
                    Ok(v) => st.mf_dx12 = Some(v),
                    Err(err) => {
                        tracing::warn!(
                            ?err,
                            "MF DX12 GPU copy mode requested but backend/session init failed; falling back"
                        );
                        st.mf_dx12 = None;
                        st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                        return update;
                    }
                }
            }

            let Some(session) = st.mf_dx12.as_mut() else {
                st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                return update;
            };

            // If the session's decoded frame size disagrees with the currently allocated
            // texture size, do not attempt a copy yet. Let the next frame reallocate the
            // target to match the decoded size deterministically.
            if session.size() != st.target_px_size {
                return update;
            }

            let tick = match session.tick(context, texture) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(?err, "MF DX12 GPU copy tick failed; falling back");
                    st.mf_dx12 = None;
                    st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                    return update;
                }
            };

            let (size, color_encoding) = match tick {
                wmf::Dx12GpuCopyTick::Copied {
                    size,
                    color_encoding,
                } => (size, color_encoding),
                wmf::Dx12GpuCopyTick::EndOfStream => {
                    st.mf_dx12 = None;
                    return update;
                }
            };

            metadata.color_encoding = color_encoding;
            st.target.push_update_with_deterministic_fallback(
                &mut update,
                view.clone(),
                size,
                metadata,
                RenderTargetIngestStrategy::ExternalZeroCopy,
                &[RenderTargetIngestStrategy::GpuCopy],
            );
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
