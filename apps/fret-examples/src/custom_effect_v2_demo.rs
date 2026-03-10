//! Custom effect demo (CustomV2).
//!
//! Demonstrates the "escape hatch with a higher ceiling": a bounded custom WGSL snippet can
//! sample a single user-provided image (by `ImageId`) in addition to the effect's `src_texture`.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::scene::{
    CustomEffectImageInputV1, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep,
    ImageSamplingHint, UvRect,
};
use fret_core::{AlphaMode, Color, Corners, Edges, EffectId, ImageId, Px};
use fret_render::{
    ImageColorSpace, ImageDescriptor, Renderer, WgpuContext, write_rgba8_texture_region,
};
use fret_runtime::Model;
use fret_ui::element::{
    ContainerProps, EffectLayerProps, LayoutStyle, Length, Overflow, PositionStyle, SpacerProps,
    TextProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV2;
use fret_ui_kit::{Space, UiIntoElement};

mod act {
    fret::actions!([Reset = "custom_effect_v2_demo.reset.v1"]);
}

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: input_strength (0..1)
// - vec4s[0].y: input_debug (0 or 1)
// - vec4s[0].z: rim_strength (0..1)
// - vec4s[0].w: unused

fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let input_strength = clamp(params.vec4s[0].x, 0.0, 1.0);
  let input_debug = params.vec4s[0].y;
  let rim_strength = clamp(params.vec4s[0].z, 0.0, 1.0);

  // User input sample (filterable).
  let inp = fret_sample_input_at_pos(pos_px);

  if (input_debug > 0.5) {
    return vec4<f32>(inp.rgb, 1.0);
  }

  // Simple rim highlight using effect-local coordinates.
  let local = fret_local_px(pos_px);
  let size = render_space.size_px;
  let d = min(min(local.x, size.x - local.x), min(local.y, size.y - local.y));
  let rim = smoothstep(2.5, 0.0, d);

  // Grain-like modulation from the input image (treat as data).
  let g = (inp.r - 0.5) * input_strength;
  let rgb = clamp(src.rgb + vec3<f32>(g) + vec3<f32>(1.0) * rim * (0.06 + 0.20 * rim_strength), vec3<f32>(0.0), vec3<f32>(4.0));
  return vec4<f32>(rgb, src.a);
}
"#;

#[derive(Debug)]
struct DemoEffectPack {
    program: CustomEffectProgramV2,
    input_image_filterable: Option<ImageId>,
    input_image_non_filterable: Option<ImageId>,
}

impl DemoEffectPack {
    fn new() -> Self {
        Self {
            program: CustomEffectProgramV2::wgsl_utf8(WGSL),
            input_image_filterable: None,
            input_image_non_filterable: None,
        }
    }
}

#[derive(Debug)]
struct CustomEffectV2State {
    enabled: Model<bool>,
    use_non_filterable_input: Model<bool>,
    sampling: Model<Option<Arc<str>>>,
    sampling_open: Model<bool>,
    uv_span: Model<Vec<f32>>,
    input_strength: Model<Vec<f32>>,
    rim_strength: Model<Vec<f32>>,
    blur_radius_px: Model<Vec<f32>>,
    debug_input: Model<bool>,
}

struct CustomEffectV2View {
    st: CustomEffectV2State,
}

pub fn run() -> anyhow::Result<()> {
    let builder = FretApp::new("custom-effect-v2-demo")
        .window("custom-effect-v2-demo", (1100.0, 720.0))
        .setup(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .view::<CustomEffectV2View>()?;

    install_into(builder).run().map_err(anyhow::Error::from)
}

/// Example of a “one line install” entrypoint for consumers on the native desktop builder path.
///
/// This is the intended pattern for third-party component/effect libraries:
/// - keep `EffectId` renderer-scoped and runtime-assigned,
/// - register lazily and cache the returned `EffectId`,
/// - upload/register any input textures on GPU-ready.
fn install_into<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S> {
    builder
        .setup(install_app_globals)
        .install_custom_effects(register_custom_effect)
        .on_gpu_ready(upload_input_image)
}

fn install_app_globals(app: &mut KernelApp) {
    app.set_global(DemoEffectPack::new());
}

fn register_custom_effect(app: &mut KernelApp, effects: &mut dyn fret_core::CustomEffectService) {
    app.with_global_mut(DemoEffectPack::new, |pack, _app| {
        pack.program
            .ensure_registered(effects)
            .expect("custom effect v2 registration must succeed on wgpu backends");
    });
}

fn upload_input_image(app: &mut KernelApp, context: &WgpuContext, renderer: &mut Renderer) {
    let size = (64u32, 64u32);
    let filterable_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v2_demo input texture"),
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
    for y in 0..size.1 {
        for x in 0..size.0 {
            let i = ((y * size.0 + x) * 4) as usize;
            let check = ((x ^ y) & 1) as u8;
            let r = if check == 0 { 20u8 } else { 235u8 };
            let g = ((x * 4) & 0xff) as u8;
            let b = ((y * 4) & 0xff) as u8;
            bytes[i] = r;
            bytes[i + 1] = g;
            bytes[i + 2] = b;
            bytes[i + 3] = 255;
        }
    }

    write_rgba8_texture_region(
        &context.queue,
        &filterable_texture,
        (0, 0),
        size,
        size.0 * 4,
        &bytes,
    );

    let view = filterable_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let filterable_image = renderer.register_image(ImageDescriptor {
        view,
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let non_filterable_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v2_demo non-filterable input texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Rgba32Float is a non-filterable float format in wgpu; sampling it with the CustomV2 ABI
        // (which uses filtering samplers) should deterministically fall back to a 1x1 transparent
        // texture rather than triggering a validation error.
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = non_filterable_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let non_filterable_image = renderer.register_image(ImageDescriptor {
        view,
        size: (1, 1),
        format: wgpu::TextureFormat::Rgba32Float,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    app.with_global_mut(DemoEffectPack::new, |pack, _app| {
        pack.input_image_filterable = Some(filterable_image);
        pack.input_image_non_filterable = Some(non_filterable_image);
    });
}

impl CustomEffectV2State {
    fn reset(models: &mut fret_runtime::ModelStore, st: &CustomEffectV2State) {
        let _ = models.update(&st.enabled, |v| *v = true);
        let _ = models.update(&st.use_non_filterable_input, |v| *v = false);
        let _ = models.update(&st.sampling, |v| *v = Some(Arc::from("linear")));
        let _ = models.update(&st.uv_span, |v| *v = vec![0.25]);
        let _ = models.update(&st.input_strength, |v| *v = vec![0.35]);
        let _ = models.update(&st.rim_strength, |v| *v = vec![0.65]);
        let _ = models.update(&st.blur_radius_px, |v| *v = vec![10.0]);
        let _ = models.update(&st.debug_input, |v| *v = false);
    }
}

impl View for CustomEffectV2View {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            st: CustomEffectV2State {
                enabled: app.models_mut().insert(true),
                use_non_filterable_input: app.models_mut().insert(false),
                sampling: app.models_mut().insert(Some(Arc::from("linear"))),
                sampling_open: app.models_mut().insert(false),
                uv_span: app.models_mut().insert(vec![0.25]),
                input_strength: app.models_mut().insert(vec![0.35]),
                rim_strength: app.models_mut().insert(vec![0.65]),
                blur_radius_px: app.models_mut().insert(vec![10.0]),
                debug_input: app.models_mut().insert(false),
            },
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        cx.on_action_notify_models::<act::Reset>({
            let st = self.clone_for_reset();
            move |models| {
                CustomEffectV2State::reset(models, &st);
                true
            }
        });

        view(cx, &mut self.st)
    }
}

impl CustomEffectV2View {
    fn clone_for_reset(&self) -> CustomEffectV2State {
        CustomEffectV2State {
            enabled: self.st.enabled.clone(),
            use_non_filterable_input: self.st.use_non_filterable_input.clone(),
            sampling: self.st.sampling.clone(),
            sampling_open: self.st.sampling_open.clone(),
            uv_span: self.st.uv_span.clone(),
            input_strength: self.st.input_strength.clone(),
            rim_strength: self.st.rim_strength.clone(),
            blur_radius_px: self.st.blur_radius_px.clone(),
            debug_input: self.st.debug_input.clone(),
        }
    }
}

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    let mut c = fret_ui_kit::colors::linear_from_hex_rgb(
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    );
    c.a = a.clamp(0.0, 1.0);
    c
}

fn watch_first_f32(
    cx: &mut ElementContext<'_, KernelApp>,
    model: &Model<Vec<f32>>,
    default: f32,
) -> f32 {
    cx.watch_model(model)
        .layout()
        .read_ref(|v| v.first().copied().unwrap_or(default))
        .ok()
        .unwrap_or(default)
}

fn sampling_hint(value: &str) -> ImageSamplingHint {
    match value.trim().to_ascii_lowercase().as_str() {
        "nearest" => ImageSamplingHint::Nearest,
        "linear" => ImageSamplingHint::Linear,
        "default" => ImageSamplingHint::Default,
        _ => ImageSamplingHint::Default,
    }
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut CustomEffectV2State) -> Elements {
    let (effect, filterable_input_image, non_filterable_input_image) = {
        let pack = cx.app.global::<DemoEffectPack>();
        (
            pack.and_then(|p| p.program.id()),
            pack.and_then(|p| p.input_image_filterable),
            pack.and_then(|p| p.input_image_non_filterable),
        )
    };
    let Some(effect) = effect else {
        return vec![shadcn::typography::h3(cx, "Custom effects unavailable")].into();
    };

    let enabled = cx.watch_model(&st.enabled).layout().value_or(true);
    let use_non_filterable_input = cx
        .watch_model(&st.use_non_filterable_input)
        .layout()
        .value_or(false);
    let input_image = if use_non_filterable_input {
        non_filterable_input_image
    } else {
        filterable_input_image
    };
    let sampling_value = cx
        .watch_model(&st.sampling)
        .layout()
        .read_ref(|v| v.clone())
        .ok()
        .and_then(|v| v.as_ref().map(|s| s.to_string()))
        .unwrap_or_else(|| "linear".to_string());
    let uv_span = watch_first_f32(cx, &st.uv_span, 0.25);
    let input_strength = watch_first_f32(cx, &st.input_strength, 0.35);
    let rim_strength = watch_first_f32(cx, &st.rim_strength, 0.65);
    let blur_radius_px = watch_first_f32(cx, &st.blur_radius_px, 10.0);
    let debug_input = cx.watch_model(&st.debug_input).layout().value_or(false);

    let inspector = inspector(
        cx,
        st,
        &sampling_value,
        uv_span,
        input_strength,
        rim_strength,
        blur_radius_px,
    );
    let stage = stage(
        cx,
        enabled,
        effect,
        input_image,
        sampling_hint(&sampling_value),
        uv_span,
        input_strength,
        rim_strength,
        blur_radius_px,
        debug_input,
    );

    let root = ui::h_flex(move |_cx| [inspector, stage])
        .size_full()
        .items_stretch()
        .gap(Space::N0)
        .into_element(cx);

    vec![root].into()
}

fn stage(
    cx: &mut ElementContext<'_, KernelApp>,
    enabled: bool,
    effect: EffectId,
    input_image: Option<ImageId>,
    sampling: ImageSamplingHint,
    uv_span: f32,
    input_strength: f32,
    rim_strength: f32,
    blur_radius_px: f32,
    debug_input: bool,
) -> AnyElement {
    let lenses = lens_row(
        cx,
        enabled,
        effect,
        input_image,
        sampling,
        uv_span,
        input_strength,
        rim_strength,
        blur_radius_px,
        debug_input,
    );

    let title = shadcn::typography::h3(cx, "Custom Effect V2 (CustomV2)");
    let subtitle = shadcn::typography::muted(
        cx,
        "CustomV2 can sample one user-provided ImageId (e.g. noise/LUT/normal map).",
    );

    let stripes = ui::h_flex(|cx| {
        (0..10)
            .map(|i| {
                let t = (i as f32) / 9.0;
                let c = Color {
                    r: (t * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                    g: ((t + 0.33) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                    b: ((t + 0.66) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                    a: 1.0,
                };

                let mut stripe_layout = LayoutStyle::default();
                stripe_layout.flex.grow = 1.0;
                stripe_layout.size.height = Length::Fill;

                cx.container(
                    ContainerProps {
                        layout: stripe_layout,
                        background: Some(c),
                        ..Default::default()
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )
            })
            .collect::<Vec<_>>()
    })
    .size_full()
    .gap(Space::N0)
    .items_stretch()
    .into_element(cx);

    let mut stripes_layer_layout = LayoutStyle::default();
    stripes_layer_layout.position = PositionStyle::Absolute;
    stripes_layer_layout.inset.left = Some(Px(0.0)).into();
    stripes_layer_layout.inset.right = Some(Px(0.0)).into();
    stripes_layer_layout.inset.top = Some(Px(0.0)).into();
    stripes_layer_layout.inset.bottom = Some(Px(0.0)).into();

    let mut stage_layout = LayoutStyle::default();
    stage_layout.size.width = Length::Fill;
    stage_layout.size.height = Length::Fill;
    stage_layout.flex.grow = 1.0;

    cx.container(
        ContainerProps {
            layout: stage_layout,
            ..Default::default()
        },
        move |cx| {
            let stripes = cx.container(
                ContainerProps {
                    layout: stripes_layer_layout,
                    ..Default::default()
                },
                move |_cx| vec![stripes],
            );

            let mut header_layout = LayoutStyle::default();
            header_layout.size.width = Length::Fill;

            let header = cx.container(
                ContainerProps {
                    layout: header_layout,
                    padding: Edges::all(Px(12.0)).into(),
                    background: Some(srgb(0, 0, 0, 0.38)),
                    corner_radii: Corners::all(Px(12.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        ui::v_flex(|_cx| [title, subtitle])
                            .gap(Space::N1)
                            .into_element(cx),
                    ]
                },
            );

            let mut content_layout = LayoutStyle::default();
            content_layout.size.width = Length::Fill;
            content_layout.size.height = Length::Fill;

            let content = cx.container(
                ContainerProps {
                    layout: content_layout,
                    padding: Edges {
                        left: Px(24.0),
                        right: Px(24.0),
                        top: Px(20.0),
                        bottom: Px(24.0),
                    }
                    .into(),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        ui::v_flex(move |_cx| [header, lenses])
                            .gap(Space::N4)
                            .items_start()
                            .into_element(cx),
                    ]
                },
            );

            vec![stripes, content]
        },
    )
}

fn lens_row(
    cx: &mut ElementContext<'_, KernelApp>,
    enabled: bool,
    effect: EffectId,
    input_image: Option<ImageId>,
    sampling: ImageSamplingHint,
    uv_span: f32,
    input_strength: f32,
    rim_strength: f32,
    blur_radius_px: f32,
    debug_input: bool,
) -> AnyElement {
    let radius = Px(24.0);
    ui::h_flex(move |cx| {
        [
            plain_lens(cx, "Plain (no effect)", radius),
            if enabled {
                custom_effect_lens(
                    cx,
                    "CustomV2 lens",
                    effect,
                    input_image,
                    sampling,
                    uv_span,
                    input_strength,
                    rim_strength,
                    blur_radius_px,
                    debug_input,
                )
            } else {
                plain_lens(cx, "CustomV2 lens (disabled)", radius)
            },
        ]
    })
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

fn lens_shell(
    cx: &mut ElementContext<'_, KernelApp>,
    label: Arc<str>,
    radius: Px,
    body: AnyElement,
) -> AnyElement {
    let mut outer_layout = LayoutStyle::default();
    outer_layout.size.width = Length::Px(Px(380.0));
    outer_layout.size.height = Length::Px(Px(240.0));
    outer_layout.overflow = Overflow::Clip;

    cx.container(
        ContainerProps {
            layout: outer_layout,
            corner_radii: Corners::all(radius),
            border: Edges::all(Px(1.0)),
            border_color: Some(srgb(255, 255, 255, 0.24)),
            ..Default::default()
        },
        move |cx| {
            let mut label_layout = LayoutStyle::default();
            label_layout.position = PositionStyle::Absolute;
            label_layout.inset.left = Some(Px(12.0)).into();
            label_layout.inset.top = Some(Px(12.0)).into();

            let title = cx.text_props(TextProps {
                layout: Default::default(),
                text: label.clone(),
                style: None,
                color: Some(srgb(255, 255, 255, 0.92)),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let pill = cx.container(
                ContainerProps {
                    layout: label_layout,
                    padding: Edges {
                        left: Px(10.0),
                        right: Px(10.0),
                        top: Px(6.0),
                        bottom: Px(6.0),
                    }
                    .into(),
                    background: Some(srgb(10, 12, 18, 0.32)),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(srgb(255, 255, 255, 0.18)),
                    corner_radii: Corners::all(Px(999.0)),
                    ..Default::default()
                },
                move |cx| vec![title.into_element(cx)],
            );

            vec![body, pill]
        },
    )
}

fn plain_lens(
    cx: &mut ElementContext<'_, KernelApp>,
    label: impl Into<Arc<str>>,
    radius: Px,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let body = cx.container(
        ContainerProps {
            layout,
            background: Some(srgb(15, 23, 42, 0.35)),
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    );

    lens_shell(cx, label.into(), radius, body)
}

fn custom_effect_lens(
    cx: &mut ElementContext<'_, KernelApp>,
    label: impl Into<Arc<str>>,
    effect: EffectId,
    input_image: Option<ImageId>,
    sampling: ImageSamplingHint,
    uv_span: f32,
    input_strength: f32,
    rim_strength: f32,
    blur_radius_px: f32,
    debug_input: bool,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let uv_span = uv_span.clamp(0.05, 1.0);
    let u0 = (0.5 - uv_span * 0.5).clamp(0.0, 1.0);
    let v0 = (0.5 - uv_span * 0.5).clamp(0.0, 1.0);
    let u1 = (0.5 + uv_span * 0.5).clamp(0.0, 1.0);
    let v1 = (0.5 + uv_span * 0.5).clamp(0.0, 1.0);
    let uv = UvRect { u0, v0, u1, v1 };

    let params = EffectParamsV1 {
        vec4s: [
            [
                input_strength.clamp(0.0, 1.0),
                if debug_input { 1.0 } else { 0.0 },
                rim_strength.clamp(0.0, 1.0),
                0.0,
            ],
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
        ],
    };

    let mut steps = Vec::new();
    let blur_radius_px = blur_radius_px.clamp(0.0, 48.0);
    if blur_radius_px > 0.0 {
        steps.push(EffectStep::GaussianBlur {
            radius_px: Px(blur_radius_px),
            downsample: 2,
        });
    }
    steps.push(EffectStep::CustomV2 {
        id: effect,
        params,
        max_sample_offset_px: Px(0.0),
        input_image: input_image.map(|image| CustomEffectImageInputV1 {
            image,
            uv,
            sampling,
        }),
    });
    let chain = EffectChain::from_steps(&steps).sanitize();

    let layer = cx.effect_layer_props(
        EffectLayerProps {
            layout,
            mode: EffectMode::Backdrop,
            chain,
            quality: EffectQuality::Auto,
        },
        |_cx| Vec::<AnyElement>::new(),
    );

    lens_shell(cx, label.into(), Px(24.0), layer)
}

fn inspector(
    cx: &mut ElementContext<'_, KernelApp>,
    st: &mut CustomEffectV2State,
    sampling_value: &str,
    uv_span: f32,
    input_strength: f32,
    rim_strength: f32,
    blur_radius_px: f32,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let enabled_model = st.enabled.clone();
    let non_filterable_model = st.use_non_filterable_input.clone();
    let sampling_model = st.sampling.clone();
    let sampling_open_model = st.sampling_open.clone();
    let uv_span_model = st.uv_span.clone();
    let input_strength_model = st.input_strength.clone();
    let rim_strength_model = st.rim_strength.clone();
    let blur_radius_model = st.blur_radius_px.clone();
    let debug_model = st.debug_input.clone();

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(Px(380.0));
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
            let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String| {
                ui::h_flex(move |cx| {
                    [
                        shadcn::Label::new(label).into_element(cx),
                        cx.spacer(SpacerProps::default()),
                        shadcn::Badge::new(value)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx)
            };

            let header = shadcn::CardHeader::new([
                shadcn::CardTitle::new("Custom Effect V2").into_element(cx),
                shadcn::CardDescription::new(
                    "CustomV2 adds a single user image input (ImageId + UvRect + SamplingHint).",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let sampling_row = ui::v_flex(move |cx| {
                [
                    label_row(cx, "Input sampling", sampling_value.to_string()),
                    shadcn::Select::new(sampling_model.clone(), sampling_open_model.clone())
                        .value(shadcn::SelectValue::new().placeholder("Pick sampling"))
                        .items([
                            shadcn::SelectItem::new("default", "Default"),
                            shadcn::SelectItem::new("linear", "Linear"),
                            shadcn::SelectItem::new("nearest", "Nearest"),
                        ])
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let uv_span_row = ui::v_flex(move |cx| {
                [
                    label_row(
                        cx,
                        "Input UV span",
                        format!("{:.2}", uv_span.clamp(0.05, 1.0)),
                    ),
                    shadcn::Slider::new(uv_span_model.clone())
                        .range(0.05, 1.0)
                        .step(0.01)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let strength_row = ui::v_flex(move |cx| {
                [
                    label_row(
                        cx,
                        "Input strength",
                        format!("{:.2}", input_strength.clamp(0.0, 1.0)),
                    ),
                    shadcn::Slider::new(input_strength_model.clone())
                        .range(0.0, 1.0)
                        .step(0.01)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let rim_row = ui::v_flex(move |cx| {
                [
                    label_row(
                        cx,
                        "Rim strength",
                        format!("{:.2}", rim_strength.clamp(0.0, 1.0)),
                    ),
                    shadcn::Slider::new(rim_strength_model.clone())
                        .range(0.0, 1.0)
                        .step(0.01)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let blur_row = ui::v_flex(move |cx| {
                [
                    label_row(
                        cx,
                        "Backdrop blur (px)",
                        format!("{:.1}", blur_radius_px.clamp(0.0, 48.0)),
                    ),
                    shadcn::Slider::new(blur_radius_model.clone())
                        .range(0.0, 32.0)
                        .step(0.5)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let content = shadcn::CardContent::new([ui::v_flex(move |cx| {
                [
                    ui::h_flex(|cx| {
                        [
                            shadcn::Switch::new(enabled_model.clone())
                                .a11y_label("Enable custom effect v2")
                                .test_id("custom-effect-v2.enabled")
                                .into_element(cx),
                            shadcn::Label::new("Enable").into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                    ui::h_flex(|cx| {
                        [
                            shadcn::Switch::new(non_filterable_model.clone())
                                .a11y_label("Use non-filterable input image (expect fallback)")
                                .test_id("custom-effect-v2.use-non-filterable-input")
                                .into_element(cx),
                            shadcn::Label::new("Non-filterable input (fallback)").into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                    sampling_row,
                    uv_span_row,
                    strength_row,
                    rim_row,
                    blur_row,
                    ui::h_flex(|cx| {
                        [
                            shadcn::Switch::new(debug_model.clone())
                                .a11y_label("Show input image")
                                .test_id("custom-effect-v2.debug-input")
                                .into_element(cx),
                            shadcn::Label::new("Show input").into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                    shadcn::Button::new("Reset")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .action(act::Reset)
                        .test_id("custom-effect-v2.reset")
                        .into_element(cx),
                ]
            })
            .gap(Space::N3)
            .items_stretch()
            .into_element(cx)])
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
