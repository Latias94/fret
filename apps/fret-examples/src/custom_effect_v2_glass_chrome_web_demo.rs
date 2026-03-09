//! Web/WASM authoring template demo for Custom Effect V2 (glass/chrome highlight).
//!
//! Goals:
//! - keep the WebGPU path honest for CustomV2,
//! - show a "higher ceiling" recipe where the v2 user input is a normal/noise map,
//! - keep authoring copy/paste-friendly (no ecosystem recipes).
//!
//! Keys:
//! - `V`: toggle the lens surface
//! - `R`: reset controls

use std::sync::Arc;

use fret_app::{App, Effect};
use fret_core::scene::{
    CustomEffectImageInputV1, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep,
    ImageSamplingHint, Paint, UvRect,
};
use fret_core::{AppWindowId, Corners, Edges, ImageId, KeyCode, Px};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_render::{
    ImageColorSpace, ImageDescriptor, Renderer, RendererCapabilities, WgpuContext,
    write_rgba8_texture_region,
};
use fret_runtime::{Model, PlatformCapabilities};
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, EffectLayerProps, Elements, FlexProps, LayoutStyle,
    Length, MainAlign, Overflow, PositionStyle, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_kit::custom_effects::CustomEffectProgramV2;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::on_activate_request_redraw;
use fret_ui_kit::{Space, UiExt};
use fret_ui_shadcn as shadcn;

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: strength (0..2)
// - vec4s[0].y: shininess (1..128)
// - vec4s[0].z: mix01 (0..1)
// - vec4s[0].w: input_debug (0 or 1)

fn unpremul(p: vec4<f32>) -> vec3<f32> {
  if (p.a <= 1e-6) { return vec3<f32>(0.0); }
  return p.rgb / p.a;
}

fn decode_normal(inp: vec4<f32>) -> vec3<f32> {
  var n = inp.rgb * 2.0 - vec3<f32>(1.0);
  let l2 = max(dot(n, n), 1e-6);
  n = n * inversesqrt(l2);
  return n;
}

fn fret_custom_effect(tex: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength = clamp(params.vec4s[0].x, 0.0, 2.0);
  let shininess = clamp(params.vec4s[0].y, 1.0, 128.0);
  let mix01 = clamp(params.vec4s[0].z, 0.0, 1.0);
  let input_debug = params.vec4s[0].w;

  let inp = fret_sample_input_at_pos(pos_px);
  if (input_debug > 0.5) {
    return vec4<f32>(inp.rgb, 1.0);
  }

  let a = tex.a;
  let src_u = unpremul(tex);

  let n = decode_normal(inp);
  let light_dir = normalize(vec3<f32>(0.35, -0.45, 0.82));
  let ndotl = max(dot(n, light_dir), 0.0);
  let spec = pow(ndotl, shininess);

  let highlight = vec3<f32>(0.85, 0.93, 1.0) * (spec * strength);
  let out_u = clamp(src_u + highlight, vec3<f32>(0.0), vec3<f32>(4.0));
  let out_premul = vec4<f32>(out_u * a, a);
  return mix(tex, out_premul, mix01);
}
"#;

#[derive(Debug)]
struct DemoEffectPack {
    program: CustomEffectProgramV2,
    input_image: Option<ImageId>,
}

impl DemoEffectPack {
    fn new() -> Self {
        Self {
            program: CustomEffectProgramV2::wgsl_utf8(WGSL),
            input_image: None,
        }
    }
}

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
    strength: Model<Vec<f32>>,
    shininess: Model<Vec<f32>>,
    mix01: Model<Vec<f32>>,
    debug_input: Model<bool>,
}

pub struct CustomEffectV2GlassChromeWebWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    show: Model<bool>,
    controls: DemoControls,
}

#[derive(Default)]
pub struct CustomEffectV2GlassChromeWebDriver;

impl CustomEffectV2GlassChromeWebDriver {
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

    fn effect_mode(value: &str) -> EffectMode {
        match value {
            "filter_content" => EffectMode::FilterContent,
            _ => EffectMode::Backdrop,
        }
    }

    fn effect_quality(value: &str) -> EffectQuality {
        match value {
            "low" => EffectQuality::Low,
            "auto" => EffectQuality::Auto,
            _ => EffectQuality::High,
        }
    }

    fn sampling_hint(value: &str) -> ImageSamplingHint {
        match value {
            "nearest" => ImageSamplingHint::Nearest,
            "linear" => ImageSamplingHint::Linear,
            _ => ImageSamplingHint::Default,
        }
    }

    fn srgb_hex(hex: u32, a: f32) -> fret_core::Color {
        let mut c = fret_core::Color::from_srgb_hex_rgb(hex);
        c.a = a.clamp(0.0, 1.0);
        c
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
            .update(&controls.strength, |v| *v = vec![0.95]);
        let _ = app
            .models_mut()
            .update(&controls.shininess, |v| *v = vec![36.0]);
        let _ = app.models_mut().update(&controls.mix01, |v| *v = vec![1.0]);
        let _ = app
            .models_mut()
            .update(&controls.debug_input, |v| *v = false);
    }

    fn install_custom_effect_and_input(
        app: &mut App,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) {
        app.with_global_mut(DemoEffectPack::new, |pack, _app| {
            let _ = pack.program.ensure_registered(renderer);

            let size = (96u32, 96u32);
            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("custom_effect_v2_glass_chrome_web_demo input texture"),
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

            fn hash_u32(mut x: u32) -> u32 {
                x ^= x >> 16;
                x = x.wrapping_mul(0x7FEB_352D);
                x ^= x >> 15;
                x = x.wrapping_mul(0x846C_A68B);
                x ^= x >> 16;
                x
            }

            let mut bytes = vec![0u8; (size.0 * size.1 * 4) as usize];
            let cx = (size.0 as f32 - 1.0) * 0.5;
            let cy = (size.1 as f32 - 1.0) * 0.5;
            for y in 0..size.1 {
                for x in 0..size.0 {
                    let i = ((y * size.0 + x) * 4) as usize;

                    let dx = (x as f32 - cx) / cx;
                    let dy = (y as f32 - cy) / cy;
                    let r2 = (dx * dx + dy * dy).min(1.0);

                    let mut nx = dx;
                    let mut ny = dy;
                    let mut nz = (1.0 - 0.65 * r2).max(0.05);

                    let h = hash_u32((x.wrapping_mul(73856093)) ^ (y.wrapping_mul(19349663)));
                    let n01 = (h as f32) / (u32::MAX as f32);
                    let noise = (n01 - 0.5) * 0.20;
                    nx += noise * 0.6;
                    ny -= noise * 0.4;
                    nz += noise * 0.8;

                    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-6);
                    nx /= len;
                    ny /= len;
                    nz /= len;

                    let r = (nx * 0.5 + 0.5).clamp(0.0, 1.0);
                    let g = (ny * 0.5 + 0.5).clamp(0.0, 1.0);
                    let b = (nz * 0.5 + 0.5).clamp(0.0, 1.0);

                    bytes[i] = (r * 255.0).round() as u8;
                    bytes[i + 1] = (g * 255.0).round() as u8;
                    bytes[i + 2] = (b * 255.0).round() as u8;
                    bytes[i + 3] = 255;
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
            pack.input_image = Some(image);
        });
    }

    fn label_row(cx: &mut ElementContext<'_, App>, label: &str, value: String) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        ui::h_flex(|cx| {
            [
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: label.into(),
                    style: None,
                    color: Some(theme.color_token("muted_foreground")),
                    align: fret_core::TextAlign::Start,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                }),
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: value.into(),
                    style: None,
                    color: Some(theme.color_token("foreground")),
                    align: fret_core::TextAlign::End,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                }),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_between()
        .into_element(cx)
    }

    fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let caps = cx.app.global::<RendererCapabilities>().cloned();
        let supported = caps.map(|c| c.custom_effect_v2_user_image).unwrap_or(false);

        let pack = cx.app.global::<DemoEffectPack>();
        let effect = pack.and_then(|p| p.program.id());
        let input_image = pack.and_then(|p| p.input_image);

        let enabled = cx.watch_model(&controls.enabled).paint().value_or(true);
        let mode_value = Self::watch_opt_string(cx, &controls.mode, "backdrop");
        let quality_value = Self::watch_opt_string(cx, &controls.quality, "high");
        let sampling_value = Self::watch_opt_string(cx, &controls.sampling, "linear");
        let uv_span = Self::watch_first_f32(cx, &controls.uv_span, 1.0).clamp(0.05, 1.0);
        let strength = Self::watch_first_f32(cx, &controls.strength, 0.95).clamp(0.0, 2.0);
        let shininess = Self::watch_first_f32(cx, &controls.shininess, 36.0).clamp(1.0, 128.0);
        let mix01 = Self::watch_first_f32(cx, &controls.mix01, 1.0).clamp(0.0, 1.0);
        let debug_input = cx
            .watch_model(&controls.debug_input)
            .paint()
            .value_or(false);

        let radius = Px(24.0);

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
                    background: Some(theme.color_token("muted")),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            )
        } else if let (true, Some(effect)) = (supported, effect) {
            let params = EffectParamsV1 {
                vec4s: [
                    [
                        strength,
                        shininess,
                        mix01,
                        if debug_input { 1.0 } else { 0.0 },
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

            let chain = EffectChain::from_steps(&[EffectStep::CustomV2 {
                id: effect,
                params,
                max_sample_offset_px: Px(0.0),
                input_image: input_image.map(|image| CustomEffectImageInputV1 {
                    image,
                    uv,
                    sampling,
                }),
            }])
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
            cx.container(
                ContainerProps {
                    layout: body_layout,
                    background: Some(theme.color_token("muted")),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.text_props(TextProps {
                        layout: Default::default(),
                        text: "CustomV2 unsupported on this adapter/backend".into(),
                        style: None,
                        color: Some(theme.color_token("muted_foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: fret_core::TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    })]
                },
            )
        };

        cx.container(
            ContainerProps {
                layout: outer_layout,
                corner_radii: Corners::all(radius),
                border: Edges::all(Px(1.0)),
                border_paint: Some(Paint::Solid(fret_core::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.18,
                })),
                ..Default::default()
            },
            move |_cx| vec![body],
        )
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
        layout.position = PositionStyle::Absolute;
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
                border_paint: Some(Paint::Solid(fret_core::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.10,
                })),
                ..Default::default()
            },
            |_cx| Vec::<AnyElement>::new(),
        )
    }

    fn controls_panel(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement {
        let enabled_model = controls.enabled.clone();
        let mode_model = controls.mode.clone();
        let mode_open_model = controls.mode_open.clone();
        let quality_model = controls.quality.clone();
        let quality_open_model = controls.quality_open.clone();
        let sampling_model = controls.sampling.clone();
        let sampling_open_model = controls.sampling_open.clone();
        let uv_span_model = controls.uv_span.clone();
        let strength_model = controls.strength.clone();
        let shininess_model = controls.shininess.clone();
        let mix01_model = controls.mix01.clone();
        let debug_input_model = controls.debug_input.clone();

        let uv_span = Self::watch_first_f32(cx, &controls.uv_span, 1.0).clamp(0.05, 1.0);
        let strength = Self::watch_first_f32(cx, &controls.strength, 0.95).clamp(0.0, 2.0);
        let shininess = Self::watch_first_f32(cx, &controls.shininess, 36.0).clamp(1.0, 128.0);
        let mix01 = Self::watch_first_f32(cx, &controls.mix01, 1.0).clamp(0.0, 1.0);
        let debug_input = cx
            .watch_model(&controls.debug_input)
            .paint()
            .value_or(false);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("CustomV2 (Glass/Chrome)").into_element(cx),
            shadcn::CardDescription::new("Normal/noise-map driven highlight").into_element(cx),
        ])
        .into_element(cx);

        let mode_row = ui::h_flex(move |cx| {
            [
                shadcn::Label::new("Mode").into_element(cx),
                shadcn::Select::new(mode_model.clone(), mode_open_model.clone())
                    .value(shadcn::SelectValue::new().placeholder("Pick mode"))
                    .items([
                        shadcn::SelectItem::new("backdrop", "Backdrop"),
                        shadcn::SelectItem::new("filter_content", "FilterContent"),
                    ])
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_between()
        .into_element(cx);

        let quality_row = ui::h_flex(move |cx| {
            [
                shadcn::Label::new("Quality").into_element(cx),
                shadcn::Select::new(quality_model.clone(), quality_open_model.clone())
                    .value(shadcn::SelectValue::new().placeholder("Pick quality"))
                    .items([
                        shadcn::SelectItem::new("high", "High"),
                        shadcn::SelectItem::new("auto", "Auto"),
                        shadcn::SelectItem::new("low", "Low"),
                    ])
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_between()
        .into_element(cx);

        let sampling_row = ui::h_flex(move |cx| {
            [
                shadcn::Label::new("Input sampling").into_element(cx),
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
        .items_center()
        .justify_between()
        .into_element(cx);

        let uv_span_row = ui::v_flex(move |cx| {
            [
                Self::label_row(cx, "Uv span", format!("{uv_span:.2}")),
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
                Self::label_row(cx, "Strength", format!("{strength:.2}")),
                shadcn::Slider::new(strength_model.clone())
                    .range(0.0, 2.0)
                    .step(0.01)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let shininess_row = ui::v_flex(move |cx| {
            [
                Self::label_row(cx, "Shininess", format!("{shininess:.0}")),
                shadcn::Slider::new(shininess_model.clone())
                    .range(1.0, 128.0)
                    .step(1.0)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let mix_row = ui::v_flex(move |cx| {
            [
                Self::label_row(cx, "Mix", format!("{mix01:.2}")),
                shadcn::Slider::new(mix01_model.clone())
                    .range(0.0, 1.0)
                    .step(0.01)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let debug_row = ui::h_flex(move |cx| {
            [
                shadcn::Switch::new(debug_input_model.clone())
                    .a11y_label("Debug input texture")
                    .test_id("customv2.glass_chrome.debug_input")
                    .into_element(cx),
                shadcn::Label::new(format!("Debug input ({debug_input})")).into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let reset_controls = controls.clone();
        let reset = on_activate_request_redraw(move |host| {
            let models = host.models_mut();
            let _ = models.update(&reset_controls.enabled, |v| *v = true);
            let _ = models.update(&reset_controls.mode, |v| *v = Some(Arc::from("backdrop")));
            let _ = models.update(&reset_controls.quality, |v| *v = Some(Arc::from("high")));
            let _ = models.update(&reset_controls.sampling, |v| *v = Some(Arc::from("linear")));
            let _ = models.update(&reset_controls.uv_span, |v| *v = vec![1.0]);
            let _ = models.update(&reset_controls.strength, |v| *v = vec![0.95]);
            let _ = models.update(&reset_controls.shininess, |v| *v = vec![36.0]);
            let _ = models.update(&reset_controls.mix01, |v| *v = vec![1.0]);
            let _ = models.update(&reset_controls.debug_input, |v| *v = false);
        });

        let content = shadcn::CardContent::new([ui::v_flex(move |cx| {
            [
                ui::h_flex(|cx| {
                    [
                        shadcn::Switch::new(enabled_model.clone())
                            .a11y_label("Enable glass/chrome effect")
                            .test_id("customv2.glass_chrome.enabled")
                            .into_element(cx),
                        shadcn::Label::new("Enable").into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                mode_row,
                quality_row,
                sampling_row,
                shadcn::Separator::new().into_element(cx),
                uv_span_row,
                strength_row,
                shininess_row,
                mix_row,
                debug_row,
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_activate(reset.clone())
                    .test_id("customv2.glass_chrome.reset")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_stretch()
        .into_element(cx)])
        .into_element(cx);

        shadcn::Card::new([header, content])
            .ui()
            .w_full()
            .into_element(cx)
    }

    fn render_root(
        cx: &mut ElementContext<'_, App>,
        show: Model<bool>,
        controls: DemoControls,
    ) -> Elements {
        cx.observe_model(&show, Invalidation::Layout);
        let visible = cx.app.models().read(&show, |v| *v).unwrap_or(true);

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
            let inspector = Self::controls_panel(cx, &controls);

            let mut stage_layout = LayoutStyle::default();
            stage_layout.size.width = Length::Fill;
            stage_layout.size.height = Length::Fill;
            stage_layout.overflow = Overflow::Clip;

            let controls_for_stage = controls.clone();
            let stage = cx.container(
                ContainerProps {
                    layout: stage_layout,
                    background: Some(Self::srgb_hex(0x08_0a_0f, 1.0)),
                    ..Default::default()
                },
                move |cx| {
                    let mut items: Vec<AnyElement> = Vec::new();
                    let tile_radius = Px(18.0);

                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb_hex(0x1a_9e_ff, 0.22),
                        Px(48.0),
                        Px(40.0),
                        Px(220.0),
                        Px(140.0),
                        tile_radius,
                    ));
                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb_hex(0xf5_9e_1a, 0.18),
                        Px(320.0),
                        Px(96.0),
                        Px(260.0),
                        Px(160.0),
                        tile_radius,
                    ));
                    items.push(Self::stage_tile(
                        cx,
                        Self::srgb_hex(0x21_d6_59, 0.16),
                        Px(140.0),
                        Px(248.0),
                        Px(280.0),
                        Px(120.0),
                        tile_radius,
                    ));

                    if visible {
                        items.push(Self::lens(cx, &controls_for_stage));
                    }

                    items
                },
            );

            vec![inspector, stage]
        })]
        .into()
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> CustomEffectV2GlassChromeWebWindowState {
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
            strength: app.models_mut().insert(vec![0.95]),
            shininess: app.models_mut().insert(vec![36.0]),
            mix01: app.models_mut().insert(vec![1.0]),
            debug_input: app.models_mut().insert(false),
        };

        CustomEffectV2GlassChromeWebWindowState {
            ui,
            root: None,
            show,
            controls,
        }
    }
}

fn init(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    app: &mut App,
    _main_window: AppWindowId,
) {
    // The runner is expected to populate this, but keep a placeholder so `global::<...>()`
    // calls remain non-panicking before `gpu_ready`.
    app.set_global(PlatformCapabilities::default());
}

fn create_window_state(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    app: &mut App,
    window: AppWindowId,
) -> CustomEffectV2GlassChromeWebWindowState {
    CustomEffectV2GlassChromeWebDriver::build_ui(app, window)
}

fn gpu_ready(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    app: &mut App,
    context: &WgpuContext,
    renderer: &mut Renderer,
) {
    app.set_global(RendererCapabilities::from_wgpu_context(context));
    CustomEffectV2GlassChromeWebDriver::install_custom_effect_and_input(app, context, renderer);
}

fn handle_model_changes(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    context: fret_launch::WinitWindowContext<'_, CustomEffectV2GlassChromeWebWindowState>,
    changed: &[fret_app::ModelId],
) {
    let fret_launch::WinitWindowContext { app, state, .. } = context;
    state.ui.propagate_model_changes(app, changed);
}

fn handle_global_changes(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    context: fret_launch::WinitWindowContext<'_, CustomEffectV2GlassChromeWebWindowState>,
    changed: &[std::any::TypeId],
) {
    let fret_launch::WinitWindowContext { app, state, .. } = context;
    state.ui.propagate_global_changes(app, changed);
}

fn handle_event(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    context: WinitEventContext<'_, CustomEffectV2GlassChromeWebWindowState>,
    event: &fret_core::Event,
) {
    let WinitEventContext {
        app,
        services,
        window,
        state,
        ..
    } = context;

    if let fret_core::Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyV
    {
        let _ = app.models_mut().update(&state.show, |v| *v = !*v);
        app.request_redraw(window);
    }
    if let fret_core::Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyR
    {
        CustomEffectV2GlassChromeWebDriver::reset_controls(app, &state.controls);
        app.request_redraw(window);
    }

    state.ui.dispatch_event(app, services, event);
}

fn render(
    _driver: &mut CustomEffectV2GlassChromeWebDriver,
    context: WinitRenderContext<'_, CustomEffectV2GlassChromeWebWindowState>,
) {
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

    let show = state.show.clone();
    let controls = state.controls.clone();

    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
        .render_root("custom-effect-v2-glass-chrome-web", |cx| {
            CustomEffectV2GlassChromeWebDriver::render_root(cx, show.clone(), controls.clone())
        });

    state.ui.set_root(root);
    state.root = Some(root);

    scene.clear();
    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);

    // Keep the demo responsive to live sliders without relying on external diagnostics.
    app.push_effect(Effect::RequestAnimationFrame(window));
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<
        CustomEffectV2GlassChromeWebDriver,
        CustomEffectV2GlassChromeWebWindowState,
    >,
) {
    hooks.init = Some(init);
    hooks.gpu_ready = Some(gpu_ready);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
}

pub fn build_app() -> App {
    let mut app = App::new();
    shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
    // Install the demo pack early so consumers can treat it like a “one line install” library.
    app.set_global(DemoEffectPack::new());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo custom_effect_v2_glass_chrome_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_fn_driver()
-> FnDriver<CustomEffectV2GlassChromeWebDriver, CustomEffectV2GlassChromeWebWindowState> {
    FnDriver::new(
        CustomEffectV2GlassChromeWebDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
}
