#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret_app::App;
use fret_core::{AlphaMode, AppWindowId, ImageColorSpace, ImageId, Px, TextAlign, TextOverflow, TextWrap};
use fret_render::{ImageDescriptor, Renderer, WgpuContext, write_rgba8_texture_region};
use fret_ui::ElementContext;
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

struct ImageHeavyMemoryState;

pub fn run() -> anyhow::Result<()> {
    fret_bootstrap::ui_app("image-heavy-memory-demo", init_window, view)
        .init_app(fret_bootstrap::install_default_i18n_backend)
        .with_main_window("image_heavy_memory_demo", (980.0, 720.0))
        .on_gpu_ready(upload_images)
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(_app: &mut App, _window: AppWindowId) -> ImageHeavyMemoryState {
    ImageHeavyMemoryState
}

fn upload_images(app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
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

fn view(
    cx: &mut ElementContext<'_, App>,
    _st: &mut ImageHeavyMemoryState,
) -> fret_ui::element::Elements {
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
            images.images.iter().map(|&image| {
                let mut props = ImageProps::new(image);
                props.layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(128.0)),
                        height: Length::Px(Px(128.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                cx.image_props(props)
            })
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

    fret_ui::element::Elements::from_iter([header, scroll])
}
