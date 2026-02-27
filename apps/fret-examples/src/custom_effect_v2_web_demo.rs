//! Web/WASM smoke demo for Custom Effect V2.
//!
//! This demo exists to keep the WebGPU path honest:
//! - register a CustomV2 program in `gpu_ready`,
//! - upload and register a filterable `ImageId` as the v2 user input,
//! - render a small `EffectLayer` in `Backdrop` mode so the effect is visually obvious.

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
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, EffectLayerProps, Elements, FlexProps, LayoutStyle,
    Length, MainAlign, Overflow, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_shadcn as shadcn;

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: strength_px (0..24)
// - vec4s[0].y: tint_strength (0..1)
// - vec4s[0].z: input_debug (0 or 1)
// - vec4s[0].w: unused

fn sample_src_premul_bilinear(p_px: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(src_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  // Convert from pixel-center coordinates to texel coordinates for manual bilinear sampling.
  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(src_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(src_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(src_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(src_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength_px = clamp(params.vec4s[0].x, 0.0, 24.0);
  let tint_strength = clamp(params.vec4s[0].y, 0.0, 1.0);
  let input_debug = params.vec4s[0].z;

  // User input sample (filterable). Treat it as a data texture.
  let inp = fret_sample_input_at_pos(pos_px);

  if (input_debug > 0.5) {
    return vec4<f32>(inp.rgb, 1.0);
  }

  // Warp driven by the input image (two channels). Use the blue channel as a falloff so
  // the center stays stable while edges distort more.
  let n = vec2<f32>(inp.r * 2.0 - 1.0, inp.g * 2.0 - 1.0);
  let amp = clamp(1.0 - inp.b, 0.0, 1.0);
  let offset_px = n * (strength_px * amp);
  let warped = sample_src_premul_bilinear(pos_px + offset_px);

  // Subtle tint so the effect is visible even on low-frequency backgrounds.
  let tint = vec3<f32>(0.10, 0.18, 0.30) * (0.35 + 0.65 * inp.b) * tint_strength;
  return vec4<f32>(clamp(warped.rgb + tint, vec3<f32>(0.0), vec3<f32>(4.0)), warped.a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct DemoEffect(Option<EffectId>);

#[derive(Debug, Clone, Copy)]
struct DemoInputImage(Option<ImageId>);

pub struct CustomEffectV2WebWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    show: fret_runtime::Model<bool>,
}

#[derive(Default)]
pub struct CustomEffectV2WebDriver;

impl CustomEffectV2WebDriver {
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

    fn build_ui(app: &mut App, window: AppWindowId) -> CustomEffectV2WebWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let show = app.models_mut().insert(true);

        CustomEffectV2WebWindowState {
            ui,
            root: None,
            show,
        }
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

        let size = (64u32, 64u32);
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("custom_effect_v2_web_demo input texture"),
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

                // Smooth "normal-map like" data texture:
                // - R/G encode a signed vector field in [-1, 1]
                // - B encodes a falloff term (higher near center).
                let fx = (x as f32 + 0.5) / (size.0 as f32) * 2.0 - 1.0;
                let fy = (y as f32 + 0.5) / (size.1 as f32) * 2.0 - 1.0;
                let r2 = fx * fx + fy * fy;
                let falloff = (-r2 * 2.5).exp().clamp(0.0, 1.0);

                // Bump gradient (unnormalized).
                let dh_dx = -fx * falloff;
                let dh_dy = -fy * falloff;
                let inv_len = 1.0 / (dh_dx * dh_dx + dh_dy * dh_dy + 1.0).sqrt();
                let nx = (dh_dx * inv_len).clamp(-1.0, 1.0);
                let ny = (dh_dy * inv_len).clamp(-1.0, 1.0);

                let r = ((nx * 0.5 + 0.5) * 255.0).round() as u8;
                let g = ((ny * 0.5 + 0.5) * 255.0).round() as u8;
                let b = (falloff * 255.0).round() as u8;

                bytes[i] = r;
                bytes[i + 1] = g;
                bytes[i + 2] = b;
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
        app.set_global(DemoInputImage(Some(image)));
    }

    fn stage_tile(
        cx: &mut ElementContext<'_, App>,
        color: fret_core::Color,
        left: Px,
        top: Px,
        w: Px,
        h: Px,
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
                corner_radii: Corners::all(Px(18.0)),
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

    fn lens(cx: &mut ElementContext<'_, App>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let caps = cx.app.global::<RendererCapabilities>().cloned();
        let supported = caps.map(|c| c.custom_effect_v2_user_image).unwrap_or(false);

        let effect = cx.app.global::<DemoEffect>().copied().and_then(|x| x.0);
        let input_image = cx.app.global::<DemoInputImage>().copied().and_then(|x| x.0);

        let radius = Px(24.0);

        let mut outer_layout = LayoutStyle::default();
        outer_layout.size.width = Length::Px(Px(420.0));
        outer_layout.size.height = Length::Px(Px(280.0));
        outer_layout.overflow = Overflow::Clip;

        let mut body_layout = LayoutStyle::default();
        body_layout.size.width = Length::Fill;
        body_layout.size.height = Length::Fill;

        let body = if let (true, Some(effect)) = (supported, effect) {
            let strength_px = 14.0;
            let params = EffectParamsV1 {
                vec4s: [[strength_px, 0.8, 0.0, 0.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            };
            let chain = EffectChain::from_steps(&[
                EffectStep::GaussianBlur {
                    radius_px: Px(12.0),
                    downsample: 1,
                },
                EffectStep::CustomV2 {
                    id: effect,
                    params,
                    max_sample_offset_px: Px(strength_px),
                    input_image: input_image.map(|image| CustomEffectImageInputV1 {
                        image,
                        uv: UvRect::FULL,
                        sampling: ImageSamplingHint::Linear,
                    }),
                },
            ])
            .sanitize();

            cx.effect_layer_props(
                EffectLayerProps {
                    layout: body_layout,
                    mode: EffectMode::Backdrop,
                    chain,
                    quality: EffectQuality::High,
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
                    text: "Custom Effect V2 (WebGPU)".into(),
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

    fn render_root(cx: &mut ElementContext<'_, App>, show: fret_runtime::Model<bool>) -> Elements {
        cx.observe_model(&show, Invalidation::Layout);
        let visible = cx.app.models().read(&show, |v| *v).unwrap_or(true);
        let theme = Theme::global(&*cx.app).snapshot();

        let mut fill = LayoutStyle::default();
        fill.size.width = Length::Fill;
        fill.size.height = Length::Fill;
        fill.overflow = Overflow::Clip;

        let stage = cx.container(
            ContainerProps {
                layout: fill,
                background: Some(Self::srgb(7, 10, 18, 1.0)),
                ..Default::default()
            },
            move |cx| {
                let mut items: Vec<AnyElement> = Vec::new();

                items.push(Self::stage_tile(
                    cx,
                    Self::srgb(24, 160, 255, 0.25),
                    Px(48.0),
                    Px(40.0),
                    Px(220.0),
                    Px(140.0),
                ));
                items.push(Self::stage_tile(
                    cx,
                    Self::srgb(245, 158, 11, 0.22),
                    Px(320.0),
                    Px(96.0),
                    Px(260.0),
                    Px(160.0),
                ));
                items.push(Self::stage_tile(
                    cx,
                    Self::srgb(34, 197, 94, 0.18),
                    Px(140.0),
                    Px(240.0),
                    Px(300.0),
                    Px(180.0),
                ));
                items.push(Self::stage_tile(
                    cx,
                    Self::srgb(168, 85, 247, 0.16),
                    Px(520.0),
                    Px(280.0),
                    Px(260.0),
                    Px(160.0),
                ));

                let mut hint_layout = LayoutStyle::default();
                hint_layout.position = fret_ui::element::PositionStyle::Absolute;
                hint_layout.inset.left = Some(Px(16.0)).into();
                hint_layout.inset.bottom = Some(Px(16.0)).into();

                items.push(cx.text_props(TextProps {
                    layout: hint_layout,
                    text: "Press V to toggle the demo surface.".into(),
                    style: None,
                    color: Some(Self::with_alpha(theme.color_token("foreground"), 0.55)),
                    align: fret_core::TextAlign::Start,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                }));

                items
            },
        );

        if !visible {
            return vec![stage].into();
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

        let overlay = cx.container(
            ContainerProps {
                layout: overlay_fill_container,
                ..Default::default()
            },
            move |cx| vec![cx.flex(center, |cx| vec![Self::lens(cx)])],
        );

        vec![stage, overlay].into()
    }
}

impl WinitAppDriver for CustomEffectV2WebDriver {
    type WindowState = CustomEffectV2WebWindowState;

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

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("custom-effect-v2-web", |cx| {
                    Self::render_root(cx, show.clone())
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
        main_window_title: "fret-demo custom_effect_v2_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    CustomEffectV2WebDriver::default()
}
