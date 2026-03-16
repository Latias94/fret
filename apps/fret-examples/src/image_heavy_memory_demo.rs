#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*};
use fret_core::{
    AlphaMode, AppWindowId, ImageColorSpace, ImageId, Px, TextAlign, TextOverflow, TextWrap,
};
use fret_render::{ImageDescriptor, Renderer, WgpuContext, write_rgba8_texture_region};
use fret_ui::element::{
    FlexProps, ImageProps, LayoutStyle, Length, ScrollProps, SizeStyle, SpacingEdges,
    SpacingLength, TextProps,
};

#[derive(Debug, Clone)]
struct ImageHeavyImages {
    images: Arc<[ImageId]>,
    texture_size_px: u32,
    estimated_rgba8_bytes: u64,
}

impl Default for ImageHeavyImages {
    fn default() -> Self {
        Self {
            images: Arc::from(Vec::<ImageId>::new()),
            texture_size_px: 0,
            estimated_rgba8_bytes: 0,
        }
    }
}

struct ImageHeavyMemoryView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("image-heavy-memory-demo")
        .window("image_heavy_memory_demo", (980.0, 720.0))
        .view_with_hooks::<ImageHeavyMemoryView>(|driver| {
            driver.record_engine_frame(record_engine_frame)
        })?
        .setup(fret_bootstrap::install_default_i18n_backend)
        .on_gpu_ready(upload_images)
        .run()
        .map_err(anyhow::Error::from)
}

impl View for ImageHeavyMemoryView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        render_view(cx.elements())
    }
}

fn record_engine_frame(
    app: &mut KernelApp,
    _window: AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    _st: &mut fret::advanced::view::ViewWindowState<ImageHeavyMemoryView>,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _dt_s: f32,
    _tick_id: fret_runtime::TickId,
    frame_id: fret_runtime::FrameId,
) -> fret_launch::EngineFrameUpdate {
    let drop_after_frames: Option<u64> = std::env::var("FRET_IMAGE_HEAVY_DEMO_DROP_AFTER_FRAMES")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|n| *n > 0);
    if drop_after_frames.is_none() {
        return fret_launch::EngineFrameUpdate::default();
    }
    let drop_after_frames = drop_after_frames.unwrap_or(0);

    let poll_after_drop = std::env::var("FRET_IMAGE_HEAVY_DEMO_POLL_AFTER_DROP")
        .ok()
        .is_some_and(|v| v.trim() != "0");

    let dropped = app.with_global_mut_untracked(ImageHeavyImages::default, |g, _app| {
        if g.images.is_empty() {
            return false;
        }
        if frame_id.0 < drop_after_frames {
            return false;
        }

        for &image in g.images.iter() {
            let _ = renderer.unregister_image(image);
        }
        *g = ImageHeavyImages::default();
        true
    });

    if dropped && poll_after_drop {
        let _ = context.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }

    fret_launch::EngineFrameUpdate::default()
}

fn upload_images(app: &mut KernelApp, context: &WgpuContext, renderer: &mut Renderer) {
    let count: usize = std::env::var("FRET_IMAGE_HEAVY_DEMO_COUNT")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(24)
        .clamp(1, 256);

    let size_px: u32 = std::env::var("FRET_IMAGE_HEAVY_DEMO_SIZE_PX")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(1024)
        .clamp(64, 4096);

    let bytes_per_row = size_px.saturating_mul(4);
    let byte_len = (bytes_per_row as usize).saturating_mul(size_px as usize);

    // Single shared upload buffer to avoid inflating CPU-side memory while still growing GPU
    // allocations (IOSurface/IOAccelerator on macOS).
    let mut rgba = vec![0u8; byte_len];
    for (i, chunk) in rgba.chunks_exact_mut(4).enumerate() {
        let x = (i as u32) % size_px;
        let y = (i as u32) / size_px;
        chunk[0] = (x & 0xFF) as u8;
        chunk[1] = (y & 0xFF) as u8;
        chunk[2] = ((x ^ y) & 0xFF) as u8;
        chunk[3] = 0xFF;
    }

    let mut out: Vec<ImageId> = Vec::with_capacity(count);
    for i in 0..count {
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("image_heavy_memory_demo texture {i}")),
            size: wgpu::Extent3d {
                width: size_px,
                height: size_px,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        write_rgba8_texture_region(
            &context.queue,
            &texture,
            (0, 0),
            (size_px, size_px),
            bytes_per_row,
            &rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let image = renderer.register_image(ImageDescriptor {
            view,
            size: (size_px, size_px),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_space: ImageColorSpace::Srgb,
            alpha_mode: AlphaMode::Premultiplied,
        });
        out.push(image);
    }

    let estimated_rgba8_bytes = (count as u128)
        .saturating_mul(size_px as u128)
        .saturating_mul(size_px as u128)
        .saturating_mul(4)
        .min(u64::MAX as u128) as u64;

    app.with_global_mut_untracked(ImageHeavyImages::default, |g, _app| {
        *g = ImageHeavyImages {
            images: Arc::from(out.into_boxed_slice()),
            texture_size_px: size_px,
            estimated_rgba8_bytes,
        };
    });
}

fn render_view(cx: &mut UiCx<'_>) -> Ui {
    let images = cx
        .app
        .with_global_mut_untracked(ImageHeavyImages::default, |g, _app| g.clone());

    let header = cx.text_props(TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        text: Arc::from(format!(
            "image-heavy memory demo: images={} texture_size_px={} estimated_rgba8_bytes={}",
            images.images.len(),
            images.texture_size_px,
            images.estimated_rgba8_bytes
        )),
        style: None,
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    });

    let grid = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            wrap: true,
            gap: SpacingLength::Px(Px(8.0)),
            padding: SpacingEdges::all(SpacingLength::Px(Px(16.0))),
            ..Default::default()
        },
        |cx| {
            let mut children = Vec::with_capacity(images.images.len());
            for &image in images.images.iter() {
                let mut props = ImageProps::new(image);
                props.layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(128.0)),
                        height: Length::Px(Px(128.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                children.push(cx.image_props(props));
            }
            children
        },
    );

    let scroll = cx.scroll(
        ScrollProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![grid],
    );

    Ui::from_iter([header, scroll])
}
