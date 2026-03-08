//! Web/WASM authoring template demo for Custom Effect V2 (starter).
//!
//! This demo exists to keep the WebGPU path honest and provide a small parameter harness:
//! - register a CustomV2 program in `gpu_ready`,
//! - upload and register a filterable `ImageId` as the v2 user input,
//! - expose a minimal inspector UI to validate wiring for:
//!   - mode/quality,
//!   - input sampling + `UvRect`,
//!   - a single mix parameter + debug switch.
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
use fret_core::{AppWindowId, Corners, Edges, EffectId, ImageId, KeyCode, Px};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
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
use fret_ui_kit::custom_effects::CustomEffectProgramV2;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::on_activate_request_redraw;
use fret_ui_kit::ui;
use fret_ui_kit::{Space, UiExt};
use fret_ui_shadcn as shadcn;

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: mix01 (0..1)
// - vec4s[0].y: input_debug (0 or 1)
// - vec4s[0].zw: unused

fn fret_custom_effect(tex: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let mix01 = clamp(params.vec4s[0].x, 0.0, 1.0);
  let input_debug = params.vec4s[0].y;

  // User input sample (filterable). Treat it as a data/utility texture.
  let inp = fret_sample_input_at_pos(pos_px);

  if (input_debug > 0.5) {
    return vec4<f32>(inp.rgb, 1.0);
  }

  // `tex` is premultiplied; preserve alpha while mixing in the input color.
  let a = tex.a;
  let overlay = vec4<f32>(inp.rgb * a, a);
  return mix(tex, overlay, mix01);
}
"#;

#[derive(Debug)]
struct DemoEffectPack {
    program: CustomEffectProgramV2,
    effect: Option<EffectId>,
    input_image: Option<ImageId>,
}

impl DemoEffectPack {
    fn new() -> Self {
        Self {
            program: CustomEffectProgramV2::wgsl_utf8(WGSL),
            effect: None,
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
    mix01: Model<Vec<f32>>,
    debug_input: Model<bool>,
}

pub struct CustomEffectV2IdentityWebWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    show: fret_runtime::Model<bool>,
    controls: DemoControls,
}

#[derive(Default)]
pub struct CustomEffectV2IdentityWebDriver;

impl CustomEffectV2IdentityWebDriver {
    fn srgb(r: u8, g: u8, b: u8, a: f32) -> fret_core::Color {
        let mut c = fret_ui_kit::colors::linear_from_hex_rgb(
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        );
        c.a = a.clamp(0.0, 1.0);
        c
    }

    fn with_alpha(mut c: fret_core::Color, a: f32) -> fret_core::Color {
        c.a = a.clamp(0.0, 1.0);
        c
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> CustomEffectV2IdentityWebWindowState {
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
            mix01: app.models_mut().insert(vec![0.65]),
            debug_input: app.models_mut().insert(false),
        };

        CustomEffectV2IdentityWebWindowState {
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
            .update(&controls.mix01, |v| *v = vec![0.65]);
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
            // Note: CustomV2 registration is capability-gated. Keep this demo resilient: store
            // `None` if unsupported so the UI can render a helpful message.
            pack.effect = pack.program.ensure_registered(renderer).ok();

            let size = (64u32, 64u32);
            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("custom_effect_v2_identity_web_demo input texture"),
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

                    // A colorful, high-frequency data/utility texture so sampling + UvRect are obvious.
                    let u = (x as f32 + 0.5) / (size.0 as f32);
                    let v = (y as f32 + 0.5) / (size.1 as f32);
                    let cell = (((x >> 3) ^ (y >> 3)) & 1) as f32;
                    let stripe = (((x + y) >> 2) & 1) as f32;

                    let r = (0.10 + 0.85 * cell) * (0.25 + 0.75 * u);
                    let g = (0.10 + 0.85 * (1.0 - cell)) * (0.25 + 0.75 * v);
                    let b = (0.15 + 0.80 * stripe) * (0.35 + 0.65 * (1.0 - (u - 0.5).abs() * 2.0));

                    bytes[i] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
                    bytes[i + 1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
                    bytes[i + 2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
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

        let pack = cx.app.global::<DemoEffectPack>();
        let effect = pack.and_then(|p| p.effect);
        let input_image = pack.and_then(|p| p.input_image);

        let enabled = cx.watch_model(&controls.enabled).paint().copied_or(true);
        let mode_value = Self::watch_opt_string(cx, &controls.mode, "backdrop");
        let quality_value = Self::watch_opt_string(cx, &controls.quality, "high");
        let sampling_value = Self::watch_opt_string(cx, &controls.sampling, "linear");
        let uv_span = Self::watch_first_f32(cx, &controls.uv_span, 1.0).clamp(0.05, 1.0);
        let mix01 = Self::watch_first_f32(cx, &controls.mix01, 0.65).clamp(0.0, 1.0);
        let debug_input = cx
            .watch_model(&controls.debug_input)
            .paint()
            .copied_or(false);

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
                    background: Some(Self::with_alpha(theme.color_token("muted"), 0.25)),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            )
        } else if let (true, Some(effect)) = (supported, effect) {
            let params = EffectParamsV1 {
                vec4s: [
                    [mix01, if debug_input { 1.0 } else { 0.0 }, 0.0, 0.0],
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
                    text: "Custom Effect V2 (Starter)".into(),
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
        let mix01 = Self::watch_first_f32(cx, &controls.mix01, 0.65).clamp(0.0, 1.0);

        let reset_controls = controls.clone();
        let reset = on_activate_request_redraw(move |host| {
            let models = host.models_mut();
            let _ = models.update(&reset_controls.enabled, |v| *v = true);
            let _ = models.update(&reset_controls.mode, |v| *v = Some(Arc::from("backdrop")));
            let _ = models.update(&reset_controls.quality, |v| *v = Some(Arc::from("high")));
            let _ = models.update(&reset_controls.sampling, |v| *v = Some(Arc::from("linear")));
            let _ = models.update(&reset_controls.uv_span, |v| *v = vec![1.0]);
            let _ = models.update(&reset_controls.mix01, |v| *v = vec![0.65]);
            let _ = models.update(&reset_controls.debug_input, |v| *v = false);
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
                    ui::h_row(|cx| {
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
                    shadcn::CardTitle::new("Custom Effect V2 (Starter)").into_element(cx),
                    shadcn::CardDescription::new(
                        "Copy/paste template: v2 input image + sampling + UvRect + params.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                let mode_row = ui::v_flex(move |cx| {
                    vec![
                        label_row(cx, "Effect mode", mode_value.clone()),
                        shadcn::Select::new(controls.mode.clone(), controls.mode_open.clone())
                            .value(shadcn::SelectValue::new().placeholder("Pick mode"))
                            .items([
                                shadcn::SelectItem::new("backdrop", "Backdrop"),
                                shadcn::SelectItem::new("filter_content", "FilterContent"),
                            ])
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                let quality_row = ui::v_flex(move |cx| {
                    vec![
                        label_row(cx, "Effect quality", quality_value.clone()),
                        shadcn::Select::new(
                            controls.quality.clone(),
                            controls.quality_open.clone(),
                        )
                        .value(shadcn::SelectValue::new().placeholder("Pick quality"))
                        .items([
                            shadcn::SelectItem::new("auto", "Auto"),
                            shadcn::SelectItem::new("low", "Low"),
                            shadcn::SelectItem::new("medium", "Medium"),
                            shadcn::SelectItem::new("high", "High"),
                        ])
                        .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                let sampling_row = ui::v_flex(move |cx| {
                    vec![
                        label_row(cx, "Input sampling", sampling_value.clone()),
                        shadcn::Select::new(
                            controls.sampling.clone(),
                            controls.sampling_open.clone(),
                        )
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
                    vec![
                        label_row(cx, "Input UV span", format!("{uv_span:.2}")),
                        shadcn::Slider::new(controls.uv_span.clone())
                            .range(0.05, 1.0)
                            .step(0.01)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                let mix_row = ui::v_flex(move |cx| {
                    vec![
                        label_row(cx, "Mix", format!("{mix01:.2}")),
                        shadcn::Slider::new(controls.mix01.clone())
                            .range(0.0, 1.0)
                            .step(0.01)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                let content = shadcn::CardContent::new([ui::v_flex(move |cx| {
                    let supported = cx
                        .app
                        .global::<RendererCapabilities>()
                        .map(|c| c.custom_effect_v2_user_image)
                        .unwrap_or(false);
                    vec![
                        ui::h_row(|cx| {
                            [
                                shadcn::Switch::new(controls.enabled.clone())
                                    .a11y_label("Enable the effect layer")
                                    .test_id("custom-effect-v2-identity-web.enabled")
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
                        uv_span_row,
                        mix_row,
                        shadcn::Separator::new().into_element(cx),
                        ui::h_row(|cx| {
                            [
                                shadcn::Switch::new(controls.debug_input.clone())
                                    .a11y_label("Show the input image")
                                    .test_id("custom-effect-v2-identity-web.debug-input")
                                    .into_element(cx),
                                shadcn::Label::new("Show input").into_element(cx),
                            ]
                        })
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx),
                        shadcn::Button::new("Reset")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .on_activate(reset.clone())
                            .test_id("custom-effect-v2-identity-web.reset")
                            .into_element(cx),
                        ui::h_row(move |cx| {
                            [
                                shadcn::Label::new("Supported").into_element(cx),
                                cx.spacer(SpacerProps::default()),
                                shadcn::Badge::new(format!("{supported}"))
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx),
                            ]
                        })
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx),
                        shadcn::CardDescription::new("Keys: V toggle surface, R reset controls.")
                            .into_element(cx),
                    ]
                })
                .gap(Space::N3)
                .items_stretch()])
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

                    let tile_corner_radius_px = Px(18.0);

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
                        text: "Press V to toggle the lens. Press R to reset controls.".into(),
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

fn handle_model_changes(
    _driver: &mut CustomEffectV2IdentityWebDriver,
    context: fret_launch::WinitWindowContext<'_, CustomEffectV2IdentityWebWindowState>,
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
    _driver: &mut CustomEffectV2IdentityWebDriver,
    context: fret_launch::WinitWindowContext<'_, CustomEffectV2IdentityWebWindowState>,
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

fn create_window_state(
    _driver: &mut CustomEffectV2IdentityWebDriver,
    app: &mut App,
    window: AppWindowId,
) -> CustomEffectV2IdentityWebWindowState {
    CustomEffectV2IdentityWebDriver::build_ui(app, window)
}

fn gpu_ready(
    _driver: &mut CustomEffectV2IdentityWebDriver,
    app: &mut App,
    context: &WgpuContext,
    renderer: &mut Renderer,
) {
    app.set_global(PlatformCapabilities::default());
    CustomEffectV2IdentityWebDriver::install_custom_effect_and_input(app, context, renderer);
}

fn handle_event(
    _driver: &mut CustomEffectV2IdentityWebDriver,
    context: WinitEventContext<'_, CustomEffectV2IdentityWebWindowState>,
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
        CustomEffectV2IdentityWebDriver::reset_controls(app, &state.controls);
        app.request_redraw(window);
    }

    state.ui.dispatch_event(app, services, event);
}

fn render(
    _driver: &mut CustomEffectV2IdentityWebDriver,
    context: WinitRenderContext<'_, CustomEffectV2IdentityWebWindowState>,
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

    let diag_enabled =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _| svc.is_enabled());
    state.ui.set_debug_enabled(diag_enabled);

    let show = state.show.clone();
    let controls = state.controls.clone();

    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
        .render_root("custom-effect-v2-identity-web", |cx| {
            CustomEffectV2IdentityWebDriver::render_root(cx, show.clone(), controls.clone())
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
            services,
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

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<
        CustomEffectV2IdentityWebDriver,
        CustomEffectV2IdentityWebWindowState,
    >,
) {
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.gpu_ready = Some(gpu_ready);
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
        main_window_title: "fret-demo custom_effect_v2_identity_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_fn_driver()
-> FnDriver<CustomEffectV2IdentityWebDriver, CustomEffectV2IdentityWebWindowState> {
    FnDriver::new(
        CustomEffectV2IdentityWebDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
}
