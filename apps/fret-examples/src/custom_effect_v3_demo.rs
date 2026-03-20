//! Custom effect demo (CustomV3).
//!
//! Native (desktop) authoring demo for the CustomV3 "lens" recipe. This intentionally uses the
//! action-first + view runtime path so it participates in the UI diagnostics + scripted testing
//! pipeline (`fretboard diag run`).

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_core::scene::{
    CustomEffectImageInputV1, CustomEffectPyramidRequestV1, CustomEffectSourcesV3, EffectChain,
    EffectMode, EffectParamsV1, EffectQuality, EffectStep, ImageSamplingHint, UvRect,
};
use fret_core::{AlphaMode, Color, Corners, Edges, EffectId, ImageId, Px};
use fret_render::{
    ImageColorSpace, ImageDescriptor, Renderer, RendererCapabilities, WgpuContext,
    write_rgba8_texture_region,
};
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, EffectLayerProps, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, RowProps, SpacingLength, TextProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV3;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use crate::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;

mod act {
    fret::actions!([Reset = "custom_effect_v3_demo.reset.v1"]);
}

const CUSTOM_EFFECT_V3_USER0_PROBE_WGSL: &str = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // For diagnostics: explicitly sample `user0`. When `user0` is incompatible with the CustomV3 ABI
  // (non-filterable formats + filtering sampler), the backend must bind a deterministic fallback
  // (1x1 transparent) rather than triggering a wgpu validation error.
  return fret_sample_user0_at_pos(pos_px);
}
"#;

const CUSTOM_EFFECT_V3_USER1_PROBE_WGSL: &str = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // For diagnostics: explicitly sample `user1`. When `user1` is incompatible with the CustomV3 ABI
  // (non-filterable formats + filtering sampler), the backend must bind a deterministic fallback
  // (1x1 transparent) rather than triggering a wgpu validation error.
  return fret_sample_user1_at_pos(pos_px);
}
"#;

const CUSTOM_EFFECT_V3_USER01_PROBE_WGSL: &str = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // For diagnostics: ensure both `user0` and `user1` sampling paths are exercised in the same pass.
  let a = fret_sample_user0_at_pos(pos_px);
  let b = fret_sample_user1_at_pos(pos_px);
  return 0.5 * a + 0.5 * b;
}
"#;

#[derive(Debug)]
struct DemoGlobals {
    lens_program: CustomEffectProgramV3,
    user0_probe_program: CustomEffectProgramV3,
    user1_probe_program: CustomEffectProgramV3,
    user01_probe_program: CustomEffectProgramV3,
    user0_filterable: Option<ImageId>,
    user0_non_filterable: Option<ImageId>,
    user1_filterable: Option<ImageId>,
    user1_non_filterable: Option<ImageId>,
}

impl DemoGlobals {
    fn new() -> Self {
        Self {
            lens_program: CustomEffectProgramV3::wgsl_utf8(CUSTOM_EFFECT_V3_LENS_WGSL),
            user0_probe_program: CustomEffectProgramV3::wgsl_utf8(
                CUSTOM_EFFECT_V3_USER0_PROBE_WGSL,
            ),
            user1_probe_program: CustomEffectProgramV3::wgsl_utf8(
                CUSTOM_EFFECT_V3_USER1_PROBE_WGSL,
            ),
            user01_probe_program: CustomEffectProgramV3::wgsl_utf8(
                CUSTOM_EFFECT_V3_USER01_PROBE_WGSL,
            ),
            user0_filterable: None,
            user0_non_filterable: None,
            user1_filterable: None,
            user1_non_filterable: None,
        }
    }
}

struct State {
    enabled: LocalState<bool>,
    show_user0_probe: LocalState<bool>,
    show_user1_probe: LocalState<bool>,
    use_non_filterable_user0: LocalState<bool>,
    use_non_filterable_user1: LocalState<bool>,
}

struct CustomEffectV3View;

#[derive(Clone)]
struct CustomEffectV3ViewSettings {
    enabled: bool,
    show_user0_probe: bool,
    show_user1_probe: bool,
    use_non_filterable_user0: bool,
    use_non_filterable_user1: bool,
}

fn install_demo_theme(app: &mut KernelApp) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Dark,
    );
}

pub fn run() -> anyhow::Result<()> {
    let builder = FretApp::new("custom-effect-v3-demo")
        .window("custom-effect-v3-demo", (1100.0, 720.0))
        .setup(install_demo_theme)
        .view::<CustomEffectV3View>()?;

    install_into(builder).run().map_err(anyhow::Error::from)
}

/// Example of a “one line install” entrypoint for consumers on the native desktop builder path.
///
/// This is the intended pattern for third-party effect libraries:
/// - keep `EffectId` renderer-scoped and runtime-assigned,
/// - register lazily and cache the returned `EffectId`,
/// - keep the authoring demo small and diagnostics-friendly.
fn install_into<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S> {
    builder
        .setup(install_app_globals)
        .install_custom_effects(register_custom_effect_v3)
        .on_gpu_ready(upload_user0_images)
}

fn install_app_globals(app: &mut KernelApp) {
    app.set_global(DemoGlobals::new());
}

fn register_custom_effect_v3(
    app: &mut KernelApp,
    effects: &mut dyn fret_core::CustomEffectService,
) {
    app.with_global_mut(DemoGlobals::new, |g, _app| {
        if let Err(err) = g.lens_program.ensure_registered(effects) {
            tracing::warn!(?err, "custom effect v3 lens registration failed");
        }
        if let Err(err) = g.user0_probe_program.ensure_registered(effects) {
            tracing::warn!(?err, "custom effect v3 user0 probe registration failed");
        }
        if let Err(err) = g.user1_probe_program.ensure_registered(effects) {
            tracing::warn!(?err, "custom effect v3 user1 probe registration failed");
        }
        if let Err(err) = g.user01_probe_program.ensure_registered(effects) {
            tracing::warn!(?err, "custom effect v3 user01 probe registration failed");
        }
    });
}

fn upload_user0_images(app: &mut KernelApp, context: &WgpuContext, renderer: &mut Renderer) {
    let filterable_size = (1u32, 1u32);
    let filterable_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v3_demo user0 filterable"),
        size: wgpu::Extent3d {
            width: filterable_size.0,
            height: filterable_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture_region(
        &context.queue,
        &filterable_texture,
        (0, 0),
        filterable_size,
        filterable_size.0 * 4,
        &[255, 0, 0, 255],
    );
    let view = filterable_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let user0_filterable = renderer.register_image(ImageDescriptor {
        view,
        size: filterable_size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let non_filterable_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v3_demo user0 non-filterable"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Rgba32Float is a non-filterable float format in wgpu; sampling it with the CustomV3 ABI
        // (which uses filtering samplers) should deterministically fall back to a 1x1 transparent
        // texture rather than triggering a validation error.
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = non_filterable_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let user0_non_filterable = renderer.register_image(ImageDescriptor {
        view,
        size: (1, 1),
        format: wgpu::TextureFormat::Rgba32Float,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let user1_size = (1u32, 1u32);
    let user1_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v3_demo user1 filterable"),
        size: wgpu::Extent3d {
            width: user1_size.0,
            height: user1_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture_region(
        &context.queue,
        &user1_texture,
        (0, 0),
        user1_size,
        user1_size.0 * 4,
        &[0, 255, 0, 255],
    );
    let view = user1_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let user1_filterable = renderer.register_image(ImageDescriptor {
        view,
        size: user1_size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let user1_non_filterable_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_effect_v3_demo user1 non-filterable"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = user1_non_filterable_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let user1_non_filterable = renderer.register_image(ImageDescriptor {
        view,
        size: (1, 1),
        format: wgpu::TextureFormat::Rgba32Float,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    app.with_global_mut(DemoGlobals::new, |g, _app| {
        g.user0_filterable = Some(user0_filterable);
        g.user0_non_filterable = Some(user0_non_filterable);
        g.user1_filterable = Some(user1_filterable);
        g.user1_non_filterable = Some(user1_non_filterable);
    });
}

impl State {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            enabled: cx.state().local_init(|| true),
            show_user0_probe: cx.state().local_init(|| false),
            show_user1_probe: cx.state().local_init(|| false),
            use_non_filterable_user0: cx.state().local_init(|| false),
            use_non_filterable_user1: cx.state().local_init(|| false),
        }
    }
}

impl View for CustomEffectV3View {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let mut st = State::new(cx);

        cx.actions().local(&st.enabled).set::<act::Reset>(true);
        cx.actions()
            .local(&st.show_user0_probe)
            .set::<act::Reset>(false);
        cx.actions()
            .local(&st.show_user1_probe)
            .set::<act::Reset>(false);
        cx.actions()
            .local(&st.use_non_filterable_user0)
            .set::<act::Reset>(false);
        cx.actions()
            .local(&st.use_non_filterable_user1)
            .set::<act::Reset>(false);

        view(cx, &mut st)
    }
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut State) -> ViewElements {
    // Animations make refraction far easier to see than static gradients.
    // Hold a continuous-frames lease so the backdrop moves without user input.
    let _frames = cx.begin_continuous_frames();

    let (
        lens_effect,
        user0_probe_effect,
        user1_probe_effect,
        user01_probe_effect,
        user0_filterable,
        user0_non_filterable,
        user1_filterable,
        user1_non_filterable,
    ) = {
        let globals = cx.app.global::<DemoGlobals>();
        (
            globals.and_then(|g| g.lens_program.id()),
            globals.and_then(|g| g.user0_probe_program.id()),
            globals.and_then(|g| g.user1_probe_program.id()),
            globals.and_then(|g| g.user01_probe_program.id()),
            globals.and_then(|g| g.user0_filterable),
            globals.and_then(|g| g.user0_non_filterable),
            globals.and_then(|g| g.user1_filterable),
            globals.and_then(|g| g.user1_non_filterable),
        )
    };
    let supported = cx
        .app
        .global::<RendererCapabilities>()
        .is_some_and(|caps| caps.custom_effect_v3);
    let Some(lens_effect) = lens_effect else {
        let msg = if supported {
            "CustomV3 is unavailable (registration failed)"
        } else {
            "CustomV3 is unsupported on this backend"
        };
        return vec![shadcn::raw::typography::h3(msg).into_element(cx)].into();
    };

    let view_settings: CustomEffectV3ViewSettings = cx.data().selector_layout(
        (
            &st.enabled,
            &st.show_user0_probe,
            &st.show_user1_probe,
            &st.use_non_filterable_user0,
            &st.use_non_filterable_user1,
        ),
        |(
            enabled,
            show_user0_probe,
            show_user1_probe,
            use_non_filterable_user0,
            use_non_filterable_user1,
        )| CustomEffectV3ViewSettings {
            enabled,
            show_user0_probe,
            show_user1_probe,
            use_non_filterable_user0,
            use_non_filterable_user1,
        },
    );
    let user0_image = if view_settings.use_non_filterable_user0 {
        user0_non_filterable
    } else {
        user0_filterable
    };
    let user1_image = if view_settings.use_non_filterable_user1 {
        user1_non_filterable
    } else {
        user1_filterable
    };

    let stage = stage(
        cx,
        st,
        view_settings.enabled,
        view_settings.show_user0_probe,
        view_settings.use_non_filterable_user0,
        lens_effect,
        user0_probe_effect,
        view_settings.show_user1_probe,
        view_settings.use_non_filterable_user1,
        user1_probe_effect,
        user01_probe_effect,
        user0_image,
        user1_image,
    )
    .into_element(cx);

    let mut root_layout = LayoutStyle::default();
    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;

    let root = cx.container(
        ContainerProps {
            layout: root_layout,
            ..Default::default()
        },
        move |_cx| vec![stage],
    );

    vec![root].into()
}

fn stage(
    cx: &mut UiCx<'_>,
    st: &mut State,
    enabled: bool,
    show_user0_probe: bool,
    use_non_filterable_user0: bool,
    lens_effect: EffectId,
    user0_probe_effect: Option<EffectId>,
    show_user1_probe: bool,
    use_non_filterable_user1: bool,
    user1_probe_effect: Option<EffectId>,
    user01_probe_effect: Option<EffectId>,
    user0_image: Option<ImageId>,
    user1_image: Option<ImageId>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let backdrop = animated_backdrop(cx).into_element(cx);
    let lenses = lens_row(
        cx,
        enabled,
        show_user0_probe,
        lens_effect,
        user0_probe_effect,
        show_user1_probe,
        user1_probe_effect,
        user01_probe_effect,
        user0_image,
        user1_image,
    );

    let title = shadcn::raw::typography::h3("Custom Effect V3 (CustomV3)").into_element(cx);
    let subtitle = shadcn::raw::typography::muted(
        "V3 can request renderer sources: src_raw + an optional bounded pyramid (for liquid glass ceilings).",
    )
    .into_element(cx);
    let controls = stage_controls(
        cx,
        st,
        enabled,
        show_user0_probe,
        show_user1_probe,
        use_non_filterable_user0,
        use_non_filterable_user1,
    )
    .into_element(cx);

    let mut header_layout = LayoutStyle::default();
    header_layout.size.width = Length::Fill;

    let header = cx.container(
        ContainerProps {
            layout: header_layout,
            padding: Edges::all(Px(12.0)).into(),
            background: Some(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.38,
            }),
            corner_radii: Corners::all(Px(12.0)),
            ..Default::default()
        },
        move |cx| {
            vec![
                ui::v_flex(|_cx| [title, subtitle, controls])
                    .gap(fret_ui_kit::Space::N1)
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
            let lenses = lenses.into_element(cx);
            vec![
                ui::v_flex(move |_cx| [header, lenses])
                    .gap(fret_ui_kit::Space::N4)
                    .items_start()
                    .into_element(cx),
            ]
        },
    );

    let mut stage_layout = LayoutStyle::default();
    stage_layout.size.width = Length::Fill;
    stage_layout.size.height = Length::Fill;

    cx.container(
        ContainerProps {
            layout: stage_layout,
            ..Default::default()
        },
        move |_cx| vec![backdrop, content],
    )
}

fn stage_controls(
    cx: &mut UiCx<'_>,
    st: &mut State,
    enabled: bool,
    show_user0_probe: bool,
    show_user1_probe: bool,
    use_non_filterable_user0: bool,
    use_non_filterable_user1: bool,
) -> impl IntoUiElement<KernelApp> + use<> {
    let enabled_model = st.enabled.clone_model();
    let show_user0_probe_model = st.show_user0_probe.clone_model();
    let show_user1_probe_model = st.show_user1_probe.clone_model();
    let use_non_filterable_user0_model = st.use_non_filterable_user0.clone_model();
    let use_non_filterable_user1_model = st.use_non_filterable_user1.clone_model();

    ui::h_row(move |cx| {
        let mut out: Vec<AnyElement> = Vec::new();

        out.push(
            shadcn::Switch::new(enabled_model.clone())
                .a11y_label("Enable CustomV3 lens")
                .test_id("custom-effect-v3.enabled")
                .into_element(cx),
        );
        out.push(shadcn::Label::new(if enabled { "Enabled" } else { "Disabled" }).into_element(cx));

        out.push(
            shadcn::Switch::new(show_user0_probe_model.clone())
                .a11y_label("Show CustomV3 user0 probe")
                .test_id("custom-effect-v3.show-user0-probe")
                .into_element(cx),
        );
        out.push(
            shadcn::Label::new(if show_user0_probe {
                "User0 probe"
            } else {
                "User0 off"
            })
            .into_element(cx),
        );

        out.push(
            shadcn::Switch::new(show_user1_probe_model.clone())
                .a11y_label("Show CustomV3 user1 probe")
                .test_id("custom-effect-v3.show-user1-probe")
                .into_element(cx),
        );
        out.push(
            shadcn::Label::new(if show_user1_probe {
                "User1 probe"
            } else {
                "User1 off"
            })
            .into_element(cx),
        );

        out.push(
            shadcn::Switch::new(use_non_filterable_user0_model.clone())
                .a11y_label("Use non-filterable user0 image (expect fallback)")
                .test_id("custom-effect-v3.use-non-filterable-user0")
                .into_element(cx),
        );
        out.push(
            shadcn::Label::new(if use_non_filterable_user0 {
                "Non-filterable user0"
            } else {
                "Filterable user0"
            })
            .into_element(cx),
        );

        out.push(
            shadcn::Switch::new(use_non_filterable_user1_model.clone())
                .a11y_label("Use non-filterable user1 image (expect fallback)")
                .test_id("custom-effect-v3.use-non-filterable-user1")
                .into_element(cx),
        );
        out.push(
            shadcn::Label::new(if use_non_filterable_user1 {
                "Non-filterable user1"
            } else {
                "Filterable user1"
            })
            .into_element(cx),
        );

        out.push(
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Secondary)
                .action(act::Reset)
                .test_id("custom-effect-v3.reset")
                .into_element(cx),
        );

        out
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx)
}

fn animated_backdrop(cx: &mut UiCx<'_>) -> impl IntoUiElement<KernelApp> + use<> {
    let viewport = cx.environment_viewport_bounds(Invalidation::Paint);
    let w = viewport.size.width.0.max(1.0);
    let h = viewport.size.height.0.max(1.0);

    // Use frame-driven motion so the demo stays deterministic under `diag` scripts.
    let t = (cx.frame_id.0 as f32) * (1.0 / 60.0);

    let cols = 18u32;
    let rows = 10u32;
    let tile_w = (w / cols as f32).max(1.0);
    let tile_h = (h / rows as f32).max(1.0);

    let mut layout = LayoutStyle::default();
    layout.position = PositionStyle::Absolute;
    layout.inset.left = Some(Px(0.0)).into();
    layout.inset.right = Some(Px(0.0)).into();
    layout.inset.top = Some(Px(0.0)).into();
    layout.inset.bottom = Some(Px(0.0)).into();

    cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();

            // High-frequency, slowly varying tile colors give the lens something to refract.
            for iy in 0..rows {
                for ix in 0..cols {
                    let x = ix as f32 * tile_w;
                    let y = iy as f32 * tile_h;
                    let phase = t * 0.75 + (ix as f32) * 0.27 + (iy as f32) * 0.19;
                    let c = Color {
                        r: 0.14 + 0.12 * phase.sin().abs(),
                        g: 0.12 + 0.14 * (phase * 1.3).cos().abs(),
                        b: 0.18 + 0.12 * (phase * 0.9).sin().abs(),
                        a: 1.0,
                    };

                    let mut tile_layout = LayoutStyle::default();
                    tile_layout.position = PositionStyle::Absolute;
                    tile_layout.inset.left = Some(Px(x)).into();
                    tile_layout.inset.top = Some(Px(y)).into();
                    tile_layout.size.width = Length::Px(Px(tile_w + 1.0));
                    tile_layout.size.height = Length::Px(Px(tile_h + 1.0));

                    out.push(
                        cx.container(
                            ContainerProps {
                                layout: tile_layout,
                                background: Some(c),
                                ..Default::default()
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        )
                        .into(),
                    );
                }
            }

            // Moving bars create crisp edges; refraction becomes obvious even in stills.
            let bar_w = 56.0;
            let stride = 120.0;
            let speed = 140.0;
            let count = ((w / stride).ceil() as u32).max(10) + 2;
            for i in 0..count {
                let x = ((i as f32) * stride + t * speed).rem_euclid(w + stride) - stride;

                let mut bar_layout = LayoutStyle::default();
                bar_layout.position = PositionStyle::Absolute;
                bar_layout.inset.left = Some(Px(x)).into();
                bar_layout.inset.top = Some(Px(0.0)).into();
                bar_layout.size.width = Length::Px(Px(bar_w));
                bar_layout.size.height = Length::Fill;

                out.push(
                    cx.container(
                        ContainerProps {
                            layout: bar_layout,
                            background: Some(Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 0.04,
                            }),
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    )
                    .into(),
                );
            }

            // A soft moving blob provides local contrast and lets you judge dispersion.
            let blob_r = 140.0;
            let blob_x = (w * 0.65 + (t * 0.7).sin() * (w * 0.18)).clamp(0.0, w);
            let blob_y = (h * 0.28 + (t * 0.9).cos() * (h * 0.12)).clamp(0.0, h);

            let mut blob_layout = LayoutStyle::default();
            blob_layout.position = PositionStyle::Absolute;
            blob_layout.inset.left = Some(Px(blob_x - blob_r)).into();
            blob_layout.inset.top = Some(Px(blob_y - blob_r)).into();
            blob_layout.size.width = Length::Px(Px(blob_r * 2.0));
            blob_layout.size.height = Length::Px(Px(blob_r * 2.0));

            out.push(
                cx.container(
                    ContainerProps {
                        layout: blob_layout,
                        background: Some(Color {
                            r: 0.96,
                            g: 0.92,
                            b: 0.25,
                            a: 0.20,
                        }),
                        corner_radii: Corners::all(Px(blob_r)),
                        ..Default::default()
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )
                .into(),
            );

            out
        },
    )
}

fn lens_row(
    cx: &mut UiCx<'_>,
    enabled: bool,
    show_user0_probe: bool,
    lens_effect: EffectId,
    user0_probe_effect: Option<EffectId>,
    show_user1_probe: bool,
    user1_probe_effect: Option<EffectId>,
    user01_probe_effect: Option<EffectId>,
    user0_image: Option<ImageId>,
    user1_image: Option<ImageId>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let radius = Px(24.0);
    let lens_w = Px(360.0);
    let lens_h = Px(260.0);

    let mut row_layout = LayoutStyle::default();
    row_layout.size.width = Length::Fill;

    cx.row(
        RowProps {
            layout: row_layout,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            gap: SpacingLength::Px(Px(36.0)),
            ..Default::default()
        },
        move |cx| {
            let left_lens = plain_lens(cx, "Plain (no effect)", radius, lens_w, lens_h)
                .into_element(cx)
                .test_id("custom-effect-v3-demo.lens_left");

            let right_lens = if show_user0_probe && show_user1_probe {
                match (user01_probe_effect, user0_image, user1_image) {
                    (Some(effect), Some(user0), Some(user1)) => custom_effect_user01_probe_lens(
                        cx,
                        "CustomV3 user0+user1 probe",
                        effect,
                        user0,
                        user1,
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                    _ => plain_lens(
                        cx,
                        "CustomV3 user0+user1 probe (unavailable)",
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                }
            } else if show_user0_probe {
                match (user0_probe_effect, user0_image) {
                    (Some(effect), Some(image)) => custom_effect_user0_probe_lens(
                        cx,
                        "CustomV3 user0 probe",
                        effect,
                        image,
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                    _ => plain_lens(
                        cx,
                        "CustomV3 user0 probe (unavailable)",
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                }
            } else if show_user1_probe {
                match (user1_probe_effect, user1_image) {
                    (Some(effect), Some(image)) => custom_effect_user1_probe_lens(
                        cx,
                        "CustomV3 user1 probe",
                        effect,
                        image,
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                    _ => plain_lens(
                        cx,
                        "CustomV3 user1 probe (unavailable)",
                        radius,
                        lens_w,
                        lens_h,
                    )
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right"),
                }
            } else if enabled {
                custom_effect_lens(cx, "CustomV3 lens", lens_effect, radius, lens_w, lens_h)
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right")
            } else {
                plain_lens(cx, "CustomV3 lens (disabled)", radius, lens_w, lens_h)
                    .into_element(cx)
                    .test_id("custom-effect-v3-demo.lens_right")
            };

            vec![left_lens, right_lens]
        },
    )
}

fn lens_shell(
    cx: &mut UiCx<'_>,
    title: &'static str,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
    with_effect: Option<EffectChain>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let mut lens_layout = LayoutStyle::default();
    lens_layout.size.width = Length::Px(lens_w);
    lens_layout.size.height = Length::Px(lens_h);
    lens_layout.overflow = Overflow::Clip;

    let mut fill_layout = LayoutStyle::default();
    fill_layout.size.width = Length::Fill;
    fill_layout.size.height = Length::Fill;

    let mut chrome_layout = fill_layout;
    chrome_layout.position = PositionStyle::Absolute;
    chrome_layout.inset.left = Some(Px(0.0)).into();
    chrome_layout.inset.right = Some(Px(0.0)).into();
    chrome_layout.inset.top = Some(Px(0.0)).into();
    chrome_layout.inset.bottom = Some(Px(0.0)).into();

    let mut label_layout = LayoutStyle::default();
    label_layout.position = PositionStyle::Absolute;
    label_layout.inset.left = Some(Px(14.0)).into();
    label_layout.inset.top = Some(Px(12.0)).into();

    cx.container(
        ContainerProps {
            layout: lens_layout,
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| {
            let effect_layer = with_effect.map(|chain| {
                cx.effect_layer_props(
                    EffectLayerProps {
                        layout: fill_layout,
                        mode: EffectMode::Backdrop,
                        chain,
                        quality: EffectQuality::Auto,
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )
            });

            let chrome = cx.container(
                ContainerProps {
                    layout: chrome_layout,
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.08,
                    }),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.12,
                    }),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            );

            let label = cx.text_props(TextProps {
                layout: label_layout,
                text: Arc::from(title),
                style: None,
                color: Some(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.92,
                }),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let mut out = Vec::new();
            if let Some(layer) = effect_layer {
                out.push(layer);
            }
            out.push(chrome);
            out.push(label);
            out
        },
    )
}

fn plain_lens(
    cx: &mut UiCx<'_>,
    title: &'static str,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> impl IntoUiElement<KernelApp> + use<> {
    lens_shell(cx, title, radius, lens_w, lens_h, None)
}

fn custom_effect_lens(
    cx: &mut UiCx<'_>,
    title: &'static str,
    effect: EffectId,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> impl IntoUiElement<KernelApp> + use<> {
    let sf = cx.environment_scale_factor(Invalidation::Paint).max(1.0e-6);
    let params = EffectParamsV1 {
        vec4s: [
            // (refraction_height_px, refraction_amount_px, pyramid_level, frost_mix)
            [22.0 * sf, 34.0 * sf, 3.0, 0.75],
            // (corner_radius_px, depth_effect, dispersion, dispersion_quality)
            // - dispersion_quality: 0 = 3-tap, 1 = 7-tap Android-like.
            [radius.0 * sf, 0.18, 0.55, 1.0],
            // (noise_alpha, reserved, reserved, reserved)
            [0.012, 0.0, 0.0, 0.0],
            // tint (rgb + alpha)
            [1.0, 1.0, 1.0, 0.08],
        ],
    };
    let max_sample_offset_px =
        crate::effect_authoring::custom_effect_v3_lens_max_sample_offset_px(34.0, 0.55);

    let chain = EffectChain::from_steps(&[
        EffectStep::GaussianBlur {
            radius_px: Px(18.0),
            downsample: 2,
        },
        EffectStep::CustomV3 {
            id: effect,
            params,
            max_sample_offset_px,
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: true,
                pyramid: Some(CustomEffectPyramidRequestV1 {
                    max_levels: 6,
                    max_radius_px: Px(32.0),
                }),
            },
        },
    ])
    .sanitize();

    lens_shell(cx, title, radius, lens_w, lens_h, Some(chain))
}

fn custom_effect_user0_probe_lens(
    cx: &mut UiCx<'_>,
    title: &'static str,
    effect: EffectId,
    user0_image: ImageId,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> impl IntoUiElement<KernelApp> + use<> {
    let chain = EffectChain::from_steps(&[EffectStep::CustomV3 {
        id: effect,
        params: EffectParamsV1::ZERO,
        max_sample_offset_px: Px(0.0),
        user0: Some(CustomEffectImageInputV1 {
            image: user0_image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Linear,
        }),
        user1: None,
        sources: CustomEffectSourcesV3 {
            want_raw: false,
            pyramid: None,
        },
    }])
    .sanitize();

    lens_shell(cx, title, radius, lens_w, lens_h, Some(chain))
}

fn custom_effect_user1_probe_lens(
    cx: &mut UiCx<'_>,
    title: &'static str,
    effect: EffectId,
    user1_image: ImageId,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> impl IntoUiElement<KernelApp> + use<> {
    let chain = EffectChain::from_steps(&[EffectStep::CustomV3 {
        id: effect,
        params: EffectParamsV1::ZERO,
        max_sample_offset_px: Px(0.0),
        user0: None,
        user1: Some(CustomEffectImageInputV1 {
            image: user1_image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Linear,
        }),
        sources: CustomEffectSourcesV3 {
            want_raw: false,
            pyramid: None,
        },
    }])
    .sanitize();

    lens_shell(cx, title, radius, lens_w, lens_h, Some(chain))
}

fn custom_effect_user01_probe_lens(
    cx: &mut UiCx<'_>,
    title: &'static str,
    effect: EffectId,
    user0_image: ImageId,
    user1_image: ImageId,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> impl IntoUiElement<KernelApp> + use<> {
    let chain = EffectChain::from_steps(&[EffectStep::CustomV3 {
        id: effect,
        params: EffectParamsV1::ZERO,
        max_sample_offset_px: Px(0.0),
        user0: Some(CustomEffectImageInputV1 {
            image: user0_image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Linear,
        }),
        user1: Some(CustomEffectImageInputV1 {
            image: user1_image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Linear,
        }),
        sources: CustomEffectSourcesV3 {
            want_raw: false,
            pyramid: None,
        },
    }])
    .sanitize();

    lens_shell(cx, title, radius, lens_w, lens_h, Some(chain))
}
