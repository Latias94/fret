//! Web/WASM authoring template demo for Custom Effect V2 (LUT input image).
//!
//! This demo exists to keep the WebGPU path honest and provide a simple LUT-based CustomV2 example:
//! - register a CustomV2 program in `gpu_ready`,
//! - upload and register a filterable LUT texture as the v2 input image (data texture),
//! - render a small `EffectLayer` in `Backdrop` mode so the grade is visually obvious.
//!
//! Keys:
//! - `V`: toggle the lens surface
//! - `R`: reset controls

use std::sync::Arc;

use fret_app::{App, Effect};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::scene::{
    CustomEffectImageInputV1, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep,
    ImageSamplingHint, Paint, UvRect,
};
use fret_core::{
    AppWindowId, Corners, CustomEffectDescriptorV2, CustomEffectService, Edges, EffectId, ImageId,
    KeyCode, Px,
};
use fret_launch::{WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_render::{
    ImageColorSpace, ImageDescriptor, Renderer, RendererCapabilities, WgpuContext,
    write_rgba8_texture_region,
};
use fret_runtime::{Model, PlatformCapabilities};
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, EffectLayerProps, Elements, FlexProps, LayoutStyle,
    Length, MainAlign, Overflow, SpacerProps, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::{Space, UiExt};
use fret_ui_shadcn as shadcn;

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: intensity (0..1)
// - vec4s[0].y: contrast01 (0..1) mapped to ~[0.5..1.5]
// - vec4s[0].z: input_debug (0 or 1)
// - vec4s[0].w: unused

fn unpremul(p: vec4<f32>) -> vec3<f32> {
  if (p.a <= 1e-6) {
    return vec3<f32>(0.0);
  }
  return p.rgb / p.a;
}

fn sample_lut(rgb: vec3<f32>) -> vec3<f32> {
  // LUT encoding:
  // - input_texture is a 2D texture with dimensions (N*N, N).
  // - B selects the slice; R/G select coordinates inside the slice.
  let dims_u = textureDimensions(input_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return rgb;
  }

  let width = f32(dims_u.x);
  let height = f32(dims_u.y);
  let n = height; // expected N, with width = N*N

  let c = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));

  let b = c.b * (n - 1.0);
  let b0 = floor(b);
  let b1 = min(b0 + 1.0, n - 1.0);
  let fb = fract(b);

  let half_u = 0.5 / width;
  let half_v = 0.5 / height;
  let v = mix(0.0 + half_v, 1.0 - half_v, c.g);

  let u0_min = (b0 * n) / width + half_u;
  let u0_max = ((b0 + 1.0) * n) / width - half_u;
  let u1_min = (b1 * n) / width + half_u;
  let u1_max = ((b1 + 1.0) * n) / width - half_u;

  let u0 = mix(u0_min, u0_max, c.r);
  let u1 = mix(u1_min, u1_max, c.r);

  let c0 = fret_sample_input(vec2<f32>(u0, v));
  let c1 = fret_sample_input(vec2<f32>(u1, v));
  return mix(c0.rgb, c1.rgb, fb);
}

fn fret_custom_effect(tex: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let intensity = clamp(params.vec4s[0].x, 0.0, 1.0);
  let contrast01 = clamp(params.vec4s[0].y, 0.0, 1.0);
  let input_debug = params.vec4s[0].z;

  if (input_debug > 0.5) {
    let inp = fret_sample_input_at_pos(pos_px);
    return vec4<f32>(inp.rgb, 1.0);
  }

  let a = tex.a;
  let src = unpremul(tex);
  var graded = sample_lut(src);

  let contrast = mix(0.5, 1.5, contrast01);
  graded = clamp((graded - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));

  let out_rgb = mix(src, graded, intensity);
  return vec4<f32>(out_rgb * a, a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct DemoEffect(Option<EffectId>);

#[derive(Debug, Clone, Copy)]
struct DemoInputImage(Option<ImageId>);

#[derive(Debug, Clone)]
struct DemoControls {
    enabled: Model<bool>,
    mode: Model<Option<Arc<str>>>,
    mode_open: Model<bool>,
    quality: Model<Option<Arc<str>>>,
    quality_open: Model<bool>,
    sampling: Model<Option<Arc<str>>>,
    sampling_open: Model<bool>,
    uv_span: Model<Vec<f32>>,
    strength_px: Model<Vec<f32>>,
    max_sample_offset_px: Model<Vec<f32>>,
    tint_strength: Model<Vec<f32>>,
    blur_radius_px: Model<Vec<f32>>,
    blur_downsample: Model<Vec<f32>>,
    lens_corner_radius_px: Model<Vec<f32>>,
    tile_corner_radius_px: Model<Vec<f32>>,
    debug_input: Model<bool>,
}

pub struct CustomEffectV2LutWebWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    show: fret_runtime::Model<bool>,
    controls: DemoControls,
}

#[derive(Default)]
pub struct CustomEffectV2LutWebDriver;

impl CustomEffectV2LutWebDriver {
    fn srgb(r: u8, g: u8, b: u8, a: f32) -> fret_core::Color {
        fret_core::Color {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: a.clamp(0.0, 1.0),
        }
    }

    fn with_alpha(mut c: fret_core::Color, a: f32) -> fret_core::Color {
        c.a = a.clamp(0.0, 1.0);
        c
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> CustomEffectV2LutWebWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let show = app.models_mut().insert(true);

        let controls = DemoControls {
            enabled: app.models_mut().insert(true),
            mode: app.models_mut().insert(Some(Arc::from("backdrop"))),
            mode_open: app.models_mut().insert(false),
            quality: app.models_mut().insert(Some(Arc::from("high"))),
            quality_open: app.models_mut().insert(false),
            sampling: app.models_mut().insert(Some(Arc::from("linear"))),
            sampling_open: app.models_mut().insert(false),
            uv_span: app.models_mut().insert(vec![1.0]),
            strength_px: app.models_mut().insert(vec![0.85]),
            max_sample_offset_px: app.models_mut().insert(vec![0.0]),
            tint_strength: app.models_mut().insert(vec![0.5]),
            blur_radius_px: app.models_mut().insert(vec![0.0]),
            blur_downsample: app.models_mut().insert(vec![1.0]),
            lens_corner_radius_px: app.models_mut().insert(vec![24.0]),
            tile_corner_radius_px: app.models_mut().insert(vec![18.0]),
            debug_input: app.models_mut().insert(false),
        };

        CustomEffectV2LutWebWindowState {
            ui,
            root: None,
            show,
            controls,
        }
    }

    fn watch_first_f32(
        cx: &mut ElementContext<'_, App>,
        model: &Model<Vec<f32>>,
        default: f32,
    ) -> f32 {
        cx.watch_model(model)
            .paint()
            .read_ref(|v| v.first().copied().unwrap_or(default))
            .ok()
            .unwrap_or(default)
    }

    fn watch_opt_string(
        cx: &mut ElementContext<'_, App>,
        model: &Model<Option<Arc<str>>>,
        default: &str,
    ) -> String {
        cx.watch_model(model)
            .paint()
            .read_ref(|v| v.as_ref().map(|s| s.to_string()))
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    }

    fn sampling_hint(value: &str) -> ImageSamplingHint {
        match value.trim().to_ascii_lowercase().as_str() {
            "nearest" => ImageSamplingHint::Nearest,
            "linear" => ImageSamplingHint::Linear,
            "default" => ImageSamplingHint::Default,
            _ => ImageSamplingHint::Default,
        }
    }

    fn effect_mode(value: &str) -> EffectMode {
        match value.trim().to_ascii_lowercase().as_str() {
            "filter_content" => EffectMode::FilterContent,
            "backdrop" => EffectMode::Backdrop,
            _ => EffectMode::Backdrop,
        }
    }

    fn effect_quality(value: &str) -> EffectQuality {
        match value.trim().to_ascii_lowercase().as_str() {
            "low" => EffectQuality::Low,
            "medium" => EffectQuality::Medium,
            "high" => EffectQuality::High,
            "auto" => EffectQuality::Auto,
            _ => EffectQuality::Auto,
        }
    }

    fn reset_controls(app: &mut App, controls: &DemoControls) {
        let _ = app.models_mut().update(&controls.enabled, |v| *v = true);
        let _ = app
            .models_mut()
            .update(&controls.mode, |v| *v = Some(Arc::from("backdrop")));
        let _ = app
            .models_mut()
            .update(&controls.quality, |v| *v = Some(Arc::from("high")));
        let _ = app
            .models_mut()
            .update(&controls.sampling, |v| *v = Some(Arc::from("linear")));
        let _ = app
            .models_mut()
            .update(&controls.uv_span, |v| *v = vec![1.0]);
        let _ = app
            .models_mut()
            .update(&controls.strength_px, |v| *v = vec![0.85]);
        let _ = app
            .models_mut()
            .update(&controls.max_sample_offset_px, |v| *v = vec![0.0]);
        let _ = app
            .models_mut()
            .update(&controls.tint_strength, |v| *v = vec![0.5]);
        let _ = app
            .models_mut()
            .update(&controls.blur_radius_px, |v| *v = vec![0.0]);
        let _ = app
            .models_mut()
            .update(&controls.blur_downsample, |v| *v = vec![1.0]);
        let _ = app
            .models_mut()
            .update(&controls.lens_corner_radius_px, |v| *v = vec![24.0]);
        let _ = app
            .models_mut()
            .update(&controls.tile_corner_radius_px, |v| *v = vec![18.0]);
        let _ = app
            .models_mut()
            .update(&controls.debug_input, |v| *v = false);
    }

    fn install_custom_effect_and_input(
        app: &mut App,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) {
        let effect = renderer
            .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(WGSL))
            .ok();
        app.set_global(DemoEffect(effect));

        // 3D LUT encoded as 2D:
        // - width = N*N
        // - height = N
        let n = 16u32;
        let size = (n * n, n);
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("custom_effect_v2_lut_web_demo lut texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut bytes = vec![0u8; (size.0 * size.1 * 4) as usize];
        let m = [
            [1.10f32, -0.05f32, 0.00f32],
            [-0.03f32, 1.05f32, -0.02f32],
            [0.08f32, -0.04f32, 1.18f32],
        ];
        let denom = (n - 1) as f32;
        for g in 0..n {
            for b in 0..n {
                for r in 0..n {
                    let x = b * n + r;
                    let y = g;
                    let i = ((y * size.0 + x) * 4) as usize;

                    let rf = (r as f32) / denom;
                    let gf = (g as f32) / denom;
                    let bf = (b as f32) / denom;

                    let out_r = (m[0][0] * rf + m[0][1] * gf + m[0][2] * bf).clamp(0.0, 1.0);
                    let out_g = (m[1][0] * rf + m[1][1] * gf + m[1][2] * bf).clamp(0.0, 1.0);
                    let out_b = (m[2][0] * rf + m[2][1] * gf + m[2][2] * bf).clamp(0.0, 1.0);

                    bytes[i] = (out_r * 255.0).round() as u8;
                    bytes[i + 1] = (out_g * 255.0).round() as u8;
                    bytes[i + 2] = (out_b * 255.0).round() as u8;
                    bytes[i + 3] = 255;
                }
            }
        }

        write_rgba8_texture_region(&context.queue, &texture, (0, 0), size, size.0 * 4, &bytes);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let image = renderer.register_image(ImageDescriptor {
            view,
            size,
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_space: ImageColorSpace::Linear,
            alpha_mode: fret_core::AlphaMode::Opaque,
        });
        app.set_global(DemoInputImage(Some(image)));
    }

    fn stage_tile(
        cx: &mut ElementContext<'_, App>,
        color: fret_core::Color,
        left: Px,
        top: Px,
        w: Px,
        h: Px,
        corner_radius_px: Px,
    ) -> AnyElement {
        let mut layout = LayoutStyle::default();
        layout.position = fret_ui::element::PositionStyle::Absolute;
        layout.inset.left = Some(left).into();
        layout.inset.top = Some(top).into();
        layout.size.width = Length::Px(w);
        layout.size.height = Length::Px(h);

        cx.container(
            ContainerProps {
                layout,
                background: Some(color),
                corner_radii: Corners::all(corner_radius_px),
                border: Edges::all(Px(1.0)),
                border_paint: Some(Paint::Solid(Self::with_alpha(
                    Self::srgb(255, 255, 255, 1.0),
                    0.12,
                ))),
                ..Default::default()
            },
            |_cx| Vec::<AnyElement>::new(),
        )
    }

    fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let caps = cx.app.global::<RendererCapabilities>().cloned();
        let supported = caps.map(|c| c.custom_effect_v2_user_image).unwrap_or(false);

        let effect = cx.app.global::<DemoEffect>().copied().and_then(|x| x.0);
        let input_image = cx.app.global::<DemoInputImage>().copied().and_then(|x| x.0);

        let enabled = cx.watch_model(&controls.enabled).paint().copied_or(true);
        let mode_value = Self::watch_opt_string(cx, &controls.mode, "backdrop");
        let quality_value = Self::watch_opt_string(cx, &controls.quality, "high");
        let sampling_value = Self::watch_opt_string(cx, &controls.sampling, "linear");
        let uv_span = Self::watch_first_f32(cx, &controls.uv_span, 1.0).clamp(0.05, 1.0);
        let intensity = Self::watch_first_f32(cx, &controls.strength_px, 0.85).clamp(0.0, 1.0);
        let max_sample_offset_px =
            Self::watch_first_f32(cx, &controls.max_sample_offset_px, 0.0).clamp(0.0, 96.0);
        let contrast01 = Self::watch_first_f32(cx, &controls.tint_strength, 0.5).clamp(0.0, 1.0);
        let blur_radius_px =
            Self::watch_first_f32(cx, &controls.blur_radius_px, 0.0).clamp(0.0, 48.0);
        let blur_downsample = Self::watch_first_f32(cx, &controls.blur_downsample, 1.0)
            .round()
            .clamp(1.0, 4.0) as u32;
        let lens_corner_radius_px =
            Self::watch_first_f32(cx, &controls.lens_corner_radius_px, 24.0).clamp(0.0, 64.0);
        let debug_input = cx
            .watch_model(&controls.debug_input)
            .paint()
            .copied_or(false);

        let radius = Px(lens_corner_radius_px);

        let mut outer_layout = LayoutStyle::default();
        outer_layout.size.width = Length::Px(Px(420.0));
        outer_layout.size.height = Length::Px(Px(280.0));
        outer_layout.overflow = Overflow::Clip;

        let mut body_layout = LayoutStyle::default();
        body_layout.size.width = Length::Fill;
        body_layout.size.height = Length::Fill;

        let body = if !enabled {
            cx.container(
                ContainerProps {
                    layout: body_layout,
                    background: Some(Self::with_alpha(theme.color_token("muted"), 0.25)),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            )
        } else if let (true, Some(effect)) = (supported, effect) {
            let params = EffectParamsV1 {
                vec4s: [
                    [
                        intensity,
                        contrast01,
                        if debug_input { 1.0 } else { 0.0 },
                        0.0,
                    ],
                    [0.0; 4],
                    [0.0; 4],
                    [0.0; 4],
                ],
            };

            let half = uv_span * 0.5;
            let uv = UvRect {
                u0: 0.5 - half,
                v0: 0.5 - half,
                u1: 0.5 + half,
                v1: 0.5 + half,
            };
            let sampling = Self::sampling_hint(&sampling_value);

            let chain = EffectChain::from_steps(&[
                EffectStep::GaussianBlur {
                    radius_px: Px(blur_radius_px),
                    downsample: blur_downsample,
                },
                EffectStep::CustomV2 {
                    id: effect,
                    params,
                    max_sample_offset_px: Px(max_sample_offset_px),
                    input_image: input_image.map(|image| CustomEffectImageInputV1 {
                        image,
                        uv,
                        sampling,
                    }),
                },
            ])
            .sanitize();

            cx.effect_layer_props(
                EffectLayerProps {
                    layout: body_layout,
                    mode: Self::effect_mode(&mode_value),
                    chain,
                    quality: Self::effect_quality(&quality_value),
                },
                |_cx| Vec::<AnyElement>::new(),
            )
        } else {
            let mut msg_layout = LayoutStyle::default();
            msg_layout.size.width = Length::Fill;
            msg_layout.size.height = Length::Fill;

            cx.container(
                ContainerProps {
                    layout: msg_layout,
                    background: Some(Self::with_alpha(theme.color_token("muted"), 0.35)),
                    ..Default::default()
                },
                |cx| {
                    let text = cx.text_props(TextProps {
                        layout: Default::default(),
                        text: "CustomV2 unsupported on this adapter/backend".into(),
                        style: None,
                        color: Some(theme.color_token("muted_foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: fret_core::TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    });
                    vec![text]
                },
            )
        };

        cx.container(
            ContainerProps {
                layout: outer_layout,
                corner_radii: Corners::all(radius),
                border: Edges::all(Px(1.0)),
                border_paint: Some(Paint::Solid(Self::with_alpha(
                    Self::srgb(255, 255, 255, 1.0),
                    0.18,
                ))),
                ..Default::default()
            },
            move |cx| {
                let mut badge_layout = LayoutStyle::default();
                badge_layout.position = fret_ui::element::PositionStyle::Absolute;
                badge_layout.inset.left = Some(Px(12.0)).into();
                badge_layout.inset.top = Some(Px(12.0)).into();

                let badge_text = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: "Custom Effect V2 (LUT)".into(),
                    style: None,
                    color: Some(Self::srgb(255, 255, 255, 0.92)),
                    align: fret_core::TextAlign::Start,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                });

                let badge = cx.container(
                    ContainerProps {
                        layout: badge_layout,
                        padding: Edges {
                            left: Px(10.0),
                            right: Px(10.0),
                            top: Px(6.0),
                            bottom: Px(6.0),
                        }
                        .into(),
                        background: Some(Self::srgb(10, 12, 18, 0.35)),
                        border: Edges::all(Px(1.0)),
                        border_paint: Some(Paint::Solid(Self::with_alpha(
                            Self::srgb(255, 255, 255, 1.0),
                            0.16,
                        ))),
                        corner_radii: Corners::all(Px(999.0)),
                        ..Default::default()
                    },
                    move |_cx| vec![badge_text],
                );

                vec![body, badge]
            },
        )
    }

    fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let mode_value = Self::watch_opt_string(cx, &controls.mode, "backdrop");
        let quality_value = Self::watch_opt_string(cx, &controls.quality, "high");
        let sampling_value = Self::watch_opt_string(cx, &controls.sampling, "linear");

        let uv_span = Self::watch_first_f32(cx, &controls.uv_span, 1.0).clamp(0.05, 1.0);
        let intensity = Self::watch_first_f32(cx, &controls.strength_px, 0.85).clamp(0.0, 1.0);
        let max_sample_offset_px =
            Self::watch_first_f32(cx, &controls.max_sample_offset_px, 0.0).clamp(0.0, 96.0);
        let contrast01 = Self::watch_first_f32(cx, &controls.tint_strength, 0.5).clamp(0.0, 1.0);
        let contrast = 0.5 + contrast01;
        let blur_radius_px =
            Self::watch_first_f32(cx, &controls.blur_radius_px, 0.0).clamp(0.0, 48.0);
        let blur_downsample = Self::watch_first_f32(cx, &controls.blur_downsample, 1.0)
            .round()
            .clamp(1.0, 4.0) as u32;
        let lens_corner_radius_px =
            Self::watch_first_f32(cx, &controls.lens_corner_radius_px, 24.0).clamp(0.0, 64.0);
        let tile_corner_radius_px =
            Self::watch_first_f32(cx, &controls.tile_corner_radius_px, 18.0).clamp(0.0, 64.0);

        let reset_controls = controls.clone();
        let reset: fret_ui::action::OnActivate = Arc::new(move |host, acx, _reason| {
            let models = host.models_mut();
            let _ = models.update(&reset_controls.enabled, |v| *v = true);
            let _ = models.update(&reset_controls.mode, |v| *v = Some(Arc::from("backdrop")));
            let _ = models.update(&reset_controls.quality, |v| *v = Some(Arc::from("high")));
            let _ = models.update(&reset_controls.sampling, |v| *v = Some(Arc::from("linear")));
            let _ = models.update(&reset_controls.uv_span, |v| *v = vec![1.0]);
            let _ = models.update(&reset_controls.strength_px, |v| *v = vec![0.85]);
            let _ = models.update(&reset_controls.max_sample_offset_px, |v| *v = vec![0.0]);
            let _ = models.update(&reset_controls.tint_strength, |v| *v = vec![0.5]);
            let _ = models.update(&reset_controls.blur_radius_px, |v| *v = vec![0.0]);
            let _ = models.update(&reset_controls.blur_downsample, |v| *v = vec![1.0]);
            let _ = models.update(&reset_controls.lens_corner_radius_px, |v| *v = vec![24.0]);
            let _ = models.update(&reset_controls.tile_corner_radius_px, |v| *v = vec![18.0]);
            let _ = models.update(&reset_controls.debug_input, |v| *v = false);
            host.request_redraw(acx.window);
        });

        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Px(Px(420.0));
        layout.size.height = Length::Fill;
        layout.flex.shrink = 0.0;

        cx.container(
            ContainerProps {
                layout,
                padding: Edges::all(Px(16.0)).into(),
                background: Some(theme.color_token("background")),
                border: Edges {
                    left: Px(0.0),
                    right: Px(1.0),
                    top: Px(0.0),
                    bottom: Px(0.0),
                },
                border_color: Some(theme.color_token("border")),
                ..Default::default()
            },
            move |cx| {
                let label_row = |cx: &mut ElementContext<'_, App>, label: &str, value: String| {
                    shadcn::stack::hstack(
                        cx,
                        shadcn::stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center(),
                        move |cx| {
                            vec![
                                shadcn::Label::new(label).into_element(cx),
                                cx.spacer(SpacerProps::default()),
                                shadcn::Badge::new(value)
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let header = shadcn::CardHeader::new([
                    shadcn::CardTitle::new("Custom Effect V2 (LUT)").into_element(cx),
                    shadcn::CardDescription::new(
                        "Authoring template: CustomV2 input image used as a 3D LUT encoded in 2D.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                let mode_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Effect mode", mode_value.clone()),
                            shadcn::Select::new(controls.mode.clone(), controls.mode_open.clone())
                                .placeholder("Pick mode")
                                .items([
                                    shadcn::SelectItem::new("backdrop", "Backdrop"),
                                    shadcn::SelectItem::new("filter_content", "FilterContent"),
                                ])
                                .into_element(cx),
                        ]
                    },
                );

                let quality_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Effect quality", quality_value.clone()),
                            shadcn::Select::new(
                                controls.quality.clone(),
                                controls.quality_open.clone(),
                            )
                            .placeholder("Pick quality")
                            .items([
                                shadcn::SelectItem::new("auto", "Auto"),
                                shadcn::SelectItem::new("low", "Low"),
                                shadcn::SelectItem::new("medium", "Medium"),
                                shadcn::SelectItem::new("high", "High"),
                            ])
                            .into_element(cx),
                        ]
                    },
                );

                let sampling_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Input sampling", sampling_value.clone()),
                            shadcn::Select::new(
                                controls.sampling.clone(),
                                controls.sampling_open.clone(),
                            )
                            .placeholder("Pick sampling")
                            .items([
                                shadcn::SelectItem::new("default", "Default"),
                                shadcn::SelectItem::new("linear", "Linear"),
                                shadcn::SelectItem::new("nearest", "Nearest"),
                            ])
                            .into_element(cx),
                        ]
                    },
                );

                let uv_span_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Input UV span", format!("{uv_span:.2}")),
                            shadcn::Slider::new(controls.uv_span.clone())
                                .range(0.05, 1.0)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let strength_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Intensity", format!("{intensity:.2}")),
                            shadcn::Slider::new(controls.strength_px.clone())
                                .range(0.0, 1.0)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let max_sample_offset_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(
                                cx,
                                "Max sample offset (px)",
                                format!("{max_sample_offset_px:.1}"),
                            ),
                            shadcn::Slider::new(controls.max_sample_offset_px.clone())
                                .range(0.0, 96.0)
                                .step(0.5)
                                .into_element(cx),
                        ]
                    },
                );

                let tint_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Contrast", format!("{contrast:.2}x")),
                            shadcn::Slider::new(controls.tint_strength.clone())
                                .range(0.0, 1.0)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let blur_radius_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Blur radius (px)", format!("{blur_radius_px:.1}")),
                            shadcn::Slider::new(controls.blur_radius_px.clone())
                                .range(0.0, 32.0)
                                .step(0.5)
                                .into_element(cx),
                        ]
                    },
                );

                let blur_downsample_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(cx, "Blur downsample", format!("{blur_downsample}x")),
                            shadcn::Slider::new(controls.blur_downsample.clone())
                                .range(1.0, 4.0)
                                .step(1.0)
                                .into_element(cx),
                        ]
                    },
                );

                let lens_corner_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(
                                cx,
                                "Lens corner radius (px)",
                                format!("{lens_corner_radius_px:.1}"),
                            ),
                            shadcn::Slider::new(controls.lens_corner_radius_px.clone())
                                .range(0.0, 48.0)
                                .step(0.5)
                                .into_element(cx),
                        ]
                    },
                );

                let tile_corner_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    move |cx| {
                        vec![
                            label_row(
                                cx,
                                "Tile corner radius (px)",
                                format!("{tile_corner_radius_px:.1}"),
                            ),
                            shadcn::Slider::new(controls.tile_corner_radius_px.clone())
                                .range(0.0, 48.0)
                                .step(0.5)
                                .into_element(cx),
                        ]
                    },
                );

                let content = shadcn::CardContent::new([shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default()
                        .gap(Space::N3)
                        .items_stretch(),
                    move |cx| {
                        let supported = cx
                            .app
                            .global::<RendererCapabilities>()
                            .map(|c| c.custom_effect_v2_user_image)
                            .unwrap_or(false);
                        vec![
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(controls.enabled.clone())
                                            .a11y_label("Enable the effect layer")
                                            .test_id("custom-effect-v2-lut-web.enabled")
                                            .into_element(cx),
                                        shadcn::Label::new("Enable").into_element(cx),
                                    ]
                                },
                            ),
                            mode_row,
                            quality_row,
                            sampling_row,
                            uv_span_row,
                            strength_row,
                            max_sample_offset_row,
                            tint_row,
                            blur_radius_row,
                            blur_downsample_row,
                            shadcn::Separator::new().into_element(cx),
                            lens_corner_row,
                            tile_corner_row,
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(controls.debug_input.clone())
                                            .a11y_label("Show the input image")
                                            .test_id("custom-effect-v2-lut-web.debug-input")
                                            .into_element(cx),
                                        shadcn::Label::new("Show input").into_element(cx),
                                    ]
                                },
                            ),
                            shadcn::Button::new("Reset")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .on_activate(reset.clone())
                                .test_id("custom-effect-v2-lut-web.reset")
                                .into_element(cx),
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                move |cx| {
                                    vec![
                                        shadcn::Label::new("Supported").into_element(cx),
                                        cx.spacer(SpacerProps::default()),
                                        shadcn::Badge::new(format!("{supported}"))
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .into_element(cx),
                                    ]
                                },
                            ),
                            shadcn::CardDescription::new(
                                "Keys: V toggle surface, R reset controls.",
                            )
                            .into_element(cx),
                        ]
                    },
                )])
                .into_element(cx);

                vec![
                    shadcn::Card::new([header, content])
                        .ui()
                        .w_full()
                        .into_element(cx),
                ]
            },
        )
    }

    fn render_root(
        cx: &mut ElementContext<'_, App>,
        show: fret_runtime::Model<bool>,
        controls: DemoControls,
    ) -> Elements {
        cx.observe_model(&show, Invalidation::Layout);
        let visible = cx.app.models().read(&show, |v| *v).unwrap_or(true);
        let theme = Theme::global(&*cx.app).snapshot();

        let mut fill = LayoutStyle::default();
        fill.size.width = Length::Fill;
        fill.size.height = Length::Fill;
        fill.overflow = Overflow::Clip;

        let mut row = FlexProps {
            layout: fill,
            direction: fret_core::Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        };
        row.layout.size.width = Length::Fill;
        row.layout.size.height = Length::Fill;

        vec![cx.flex(row, move |cx| {
            let inspector = Self::inspector(cx, &controls);

            let mut stage_layout = LayoutStyle::default();
            stage_layout.size.width = Length::Fill;
            stage_layout.size.height = Length::Fill;
            stage_layout.overflow = Overflow::Clip;

            let controls_for_stage = controls.clone();
            let stage = cx.container(
                ContainerProps {
                    layout: stage_layout,
                    background: Some(Self::srgb(7, 10, 18, 1.0)),
                    ..Default::default()
                },
                move |cx| {
                    let mut items: Vec<AnyElement> = Vec::new();

                    let tile_corner_radius_px = Px(Self::watch_first_f32(
                        cx,
                        &controls_for_stage.tile_corner_radius_px,
                        18.0,
                    ));

                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb(24, 160, 255, 0.25),
                        Px(48.0),
                        Px(40.0),
                        Px(220.0),
                        Px(140.0),
                        tile_corner_radius_px,
                    ));
                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb(245, 158, 11, 0.22),
                        Px(320.0),
                        Px(96.0),
                        Px(260.0),
                        Px(160.0),
                        tile_corner_radius_px,
                    ));
                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb(34, 197, 94, 0.18),
                        Px(140.0),
                        Px(240.0),
                        Px(300.0),
                        Px(180.0),
                        tile_corner_radius_px,
                    ));
                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb(168, 85, 247, 0.16),
                        Px(520.0),
                        Px(280.0),
                        Px(260.0),
                        Px(160.0),
                        tile_corner_radius_px,
                    ));

                    let mut hint_layout = LayoutStyle::default();
                    hint_layout.position = fret_ui::element::PositionStyle::Absolute;
                    hint_layout.inset.left = Some(Px(16.0)).into();
                    hint_layout.inset.bottom = Some(Px(16.0)).into();

                    items.push(cx.text_props(TextProps {
                        layout: hint_layout,
                        text:
                            "Press V to toggle the demo surface. Press R to reset controls.".into(),
                        style: None,
                        color: Some(Self::with_alpha(theme.color_token("foreground"), 0.55)),
                        align: fret_core::TextAlign::Start,
                        wrap: fret_core::TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    }));

                    if !visible {
                        return items;
                    }

                    let mut overlay_fill_container = LayoutStyle::default();
                    overlay_fill_container.position = fret_ui::element::PositionStyle::Absolute;
                    overlay_fill_container.inset.left = Some(Px(0.0)).into();
                    overlay_fill_container.inset.top = Some(Px(0.0)).into();
                    overlay_fill_container.inset.right = Some(Px(0.0)).into();
                    overlay_fill_container.inset.bottom = Some(Px(0.0)).into();
                    overlay_fill_container.size.width = Length::Fill;
                    overlay_fill_container.size.height = Length::Fill;

                    let mut overlay_fill_center = LayoutStyle::default();
                    overlay_fill_center.position = fret_ui::element::PositionStyle::Absolute;
                    overlay_fill_center.inset.left = Some(Px(0.0)).into();
                    overlay_fill_center.inset.top = Some(Px(0.0)).into();
                    overlay_fill_center.inset.right = Some(Px(0.0)).into();
                    overlay_fill_center.inset.bottom = Some(Px(0.0)).into();
                    overlay_fill_center.size.width = Length::Fill;
                    overlay_fill_center.size.height = Length::Fill;

                    let mut center = FlexProps {
                        layout: overlay_fill_center,
                        direction: fret_core::Axis::Horizontal,
                        gap: SpacingLength::Px(Px(0.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    };
                    center.layout.size.width = Length::Fill;
                    center.layout.size.height = Length::Fill;

                    let controls_for_lens = controls_for_stage.clone();
                    let overlay = cx.container(
                        ContainerProps {
                            layout: overlay_fill_container,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.flex(center, move |cx| vec![Self::lens(cx, &controls_for_lens)]),
                            ]
                        },
                    );
                    items.push(overlay);
                    items
                },
            );

            vec![inspector, stage]
        })]
        .into()
    }
}

impl WinitAppDriver for CustomEffectV2LutWebDriver {
    type WindowState = CustomEffectV2LutWebWindowState;

    fn handle_model_changes(
        &mut self,
        context: fret_launch::WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        let fret_launch::WinitWindowContext {
            app, state, window, ..
        } = context;

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_model_changes(window, changed);
        });
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: fret_launch::WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        let fret_launch::WinitWindowContext {
            app, state, window, ..
        } = context;

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.record_global_changes(app, window, changed);
        });
        state.ui.propagate_global_changes(app, changed);
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        app.set_global(PlatformCapabilities::default());
        Self::install_custom_effect_and_input(app, context, renderer);
    }

    fn handle_event(
        &mut self,
        context: WinitEventContext<'_, Self::WindowState>,
        event: &fret_core::Event,
    ) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        let diag_enabled =
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _| svc.is_enabled());
        state.ui.set_debug_enabled(diag_enabled);

        let consumed = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            if !svc.is_enabled() {
                return false;
            }
            if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                return true;
            }
            svc.maybe_intercept_event_for_picking(app, window, event)
        });
        if consumed {
            return;
        }

        if let fret_core::Event::KeyDown { key, .. } = event
            && *key == KeyCode::KeyV
        {
            let _ = app.models_mut().update(&state.show, |v| *v = !*v);
            app.request_redraw(window);
        }
        if let fret_core::Event::KeyDown { key, .. } = event
            && *key == KeyCode::KeyR
        {
            Self::reset_controls(app, &state.controls);
            app.request_redraw(window);
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
            ..
        } = context;

        let diag_enabled =
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _| svc.is_enabled());
        state.ui.set_debug_enabled(diag_enabled);

        let show = state.show.clone();
        let controls = state.controls.clone();

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("custom-effect-v2-lut-web", |cx| {
                    Self::render_root(cx, show.clone(), controls.clone())
                });

        state.ui.set_root(root);
        state.root = Some(root);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot_arc();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&mut state.ui),
                semantics_snapshot.as_deref(),
            )
        });

        if drive.request_redraw {
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }
        if injected_any {
            state.ui.request_semantics_snapshot();
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &mut state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo custom_effect_v2_lut_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    CustomEffectV2LutWebDriver::default()
}
