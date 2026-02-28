//! Web/WASM inspector demo for Custom Effect V3.
//!
//! This is a minimal authoring-oriented demo:
//! - Registers a CustomV3 program (WGSL) at `gpu_ready`.
//! - Requests `src_raw` + a bounded `src_pyramid` in the effect step.
//! - Uses `fret_sample_src_raw_at_pos` and `fret_sample_src_pyramid_at_pos` to illustrate the
//!   “raw + multi-scale blur” ceiling bump for liquid-glass-like recipes.

use fret_app::{App, Effect};
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, CustomEffectPyramidRequestV1, CustomEffectSourcesV3, DrawOrder, EffectChain, EffectMode,
    EffectParamsV1, EffectQuality, EffectStep, Paint, SceneOp,
};
use fret_core::{CustomEffectDescriptorV3, CustomEffectService as _, EffectId, Event};
use fret_launch::{WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;

const WGSL: &str = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength_px = clamp(params.vec4s[0].x, 0.0, 64.0);
  let center_blur_mix = clamp(params.vec4s[0].y, 0.0, 1.0);
  let pyramid_level = u32(clamp(params.vec4s[0].z, 0.0, 6.0));

  // Use the renderer-provided "local" coordinate space to build a soft radial edge mask.
  let local = fret_local_px(pos_px);
  let size = max(render_space.size_px, vec2<f32>(1.0));
  let t = local / size;
  let d = length(t - vec2<f32>(0.5, 0.5));
  let edge = smoothstep(0.35, 0.50, d);

  // Raw refraction stays crisp.
  let dir = normalize((t - vec2<f32>(0.5, 0.5)) * vec2<f32>(1.0, 1.0) + vec2<f32>(1.0e-6, 0.0));
  let raw = fret_sample_src_raw_at_pos(pos_px + dir * strength_px * edge);

  // Base frosted source comes from the prior blur step (src argument).
  let frosted = src;

  // Optional extra blur from the pyramid (derived from src_raw).
  let pyr = fret_sample_src_pyramid_at_pos(pyramid_level, pos_px);
  let center = mix(frosted, pyr, center_blur_mix * (1.0 - edge));

  // Blend crisp edge refraction over the frosted center.
  let out_rgb = mix(center.rgb, raw.rgb, edge);
  return vec4<f32>(out_rgb, center.a);
}
"#;

#[derive(Debug, Default)]
pub struct CustomEffectV3WebDriver {
    effect: Option<EffectId>,
}

#[derive(Debug, Default)]
pub struct CustomEffectV3WebWindowState;

impl CustomEffectV3WebDriver {
    fn time_seconds() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now() * 0.001)
            .unwrap_or(0.0)
    }
}

impl WinitAppDriver for CustomEffectV3WebDriver {
    type WindowState = CustomEffectV3WebWindowState;

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, renderer: &mut Renderer) {
        if self.effect.is_some() {
            return;
        }
        let Ok(effect) =
            renderer.register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(WGSL))
        else {
            return;
        };
        self.effect = Some(effect);
    }

    fn create_window_state(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        Self::WindowState::default()
    }

    fn handle_event(&mut self, _context: WinitEventContext<'_, Self::WindowState>, _event: &Event) {
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            window,
            bounds,
            scene,
            ..
        } = context;

        let w = bounds.size.width.0.max(1.0);
        let h = bounds.size.height.0.max(1.0);
        let secs = Self::time_seconds() as f32;

        scene.clear();

        fn union_rect(a: Rect, b: Rect) -> Rect {
            let ax0 = a.origin.x;
            let ay0 = a.origin.y;
            let ax1 = a.origin.x + a.size.width;
            let ay1 = a.origin.y + a.size.height;

            let bx0 = b.origin.x;
            let by0 = b.origin.y;
            let bx1 = b.origin.x + b.size.width;
            let by1 = b.origin.y + b.size.height;

            let x0 = ax0.min(bx0);
            let y0 = ay0.min(by0);
            let x1 = ax1.max(bx1);
            let y1 = ay1.max(by1);

            Rect::new(
                Point::new(x0, y0),
                Size::new((x1 - x0).max(Px(0.0)), (y1 - y0).max(Px(0.0))),
            )
        }

        // Background: a simple animated color grid.
        let cols = 10u32;
        let rows = 7u32;
        let tile_w = (w / cols as f32).max(1.0);
        let tile_h = (h / rows as f32).max(1.0);
        for iy in 0..rows {
            for ix in 0..cols {
                let x = ix as f32 * tile_w;
                let y = iy as f32 * tile_h;
                let phase = secs * 0.7 + (ix as f32) * 0.35 + (iy as f32) * 0.22;
                let r = 0.12 + 0.10 * phase.sin().abs();
                let g = 0.10 + 0.12 * (phase * 1.3).cos().abs();
                let b = 0.16 + 0.10 * (phase * 0.9).sin().abs();
                scene.push(SceneOp::Quad {
                    order: DrawOrder(iy * cols + ix),
                    rect: Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(tile_w), Px(tile_h))),
                    background: Paint::Solid(Color { r, g, b, a: 1.0 }),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT),
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        if let Some(effect) = self.effect {
            let strength_px = 18.0;
            let params = EffectParamsV1 {
                vec4s: [[strength_px, 0.65, 2.0, 0.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            };

            // Two lenses in one backdrop source group:
            // - Demonstrates renderer-provided `src_raw` + bounded `src_pyramid` sharing.
            // - Allows multiple glass surfaces to reuse a single raw snapshot (and optionally a pyramid).
            let lens_w = w.min(980.0) * 0.38;
            let lens_h = h.min(720.0) * 0.42;
            let gap = (w * 0.04).max(12.0);
            let total_w = lens_w * 2.0 + gap;
            let start_x = (w - total_w) * 0.5;
            let lens_y = (h - lens_h) * 0.5;

            let lens_a = Rect::new(
                Point::new(Px(start_x), Px(lens_y)),
                Size::new(Px(lens_w), Px(lens_h)),
            );
            let lens_b = Rect::new(
                Point::new(Px(start_x + lens_w + gap), Px(lens_y)),
                Size::new(Px(lens_w), Px(lens_h)),
            );
            let group_bounds = union_rect(lens_a, lens_b);

            scene.push(SceneOp::PushBackdropSourceGroupV1 {
                bounds: group_bounds,
                pyramid: Some(CustomEffectPyramidRequestV1 {
                    max_levels: 6,
                    max_radius_px: Px(32.0),
                }),
                quality: EffectQuality::Auto,
            });

            let chain = EffectChain::from_steps(&[
                EffectStep::GaussianBlur {
                    radius_px: Px(12.0),
                    downsample: 2,
                },
                EffectStep::CustomV3 {
                    id: effect,
                    params,
                    max_sample_offset_px: Px(strength_px + 2.0),
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

            for (i, lens) in [lens_a, lens_b].into_iter().enumerate() {
                let base = 10_000 + i as u32 * 8;
                scene.push(SceneOp::PushEffect {
                    bounds: lens,
                    mode: EffectMode::Backdrop,
                    chain: chain.clone(),
                    quality: EffectQuality::Auto,
                });
                scene.push(SceneOp::Quad {
                    order: DrawOrder(base),
                    rect: lens,
                    background: Paint::Solid(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.06,
                    }),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT),
                    corner_radii: Corners::all(Px(24.0)),
                });
                scene.push(SceneOp::PopEffect);

                // Outline highlight.
                scene.push(SceneOp::Quad {
                    order: DrawOrder(base + 1),
                    rect: lens,
                    background: Paint::Solid(Color::TRANSPARENT),
                    border: Edges::all(Px(1.0)),
                    border_paint: Paint::Solid(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.18,
                    }),
                    corner_radii: Corners::all(Px(24.0)),
                });
            }

            scene.push(SceneOp::PopBackdropSourceGroup);
        } else {
            // Fallback: show lens bounds even if CustomV3 registration failed.
            let lens_w = w.min(980.0) * 0.48;
            let lens_h = h.min(720.0) * 0.42;
            let lens_x = (w - lens_w) * 0.5;
            let lens_y = (h - lens_h) * 0.5;
            let lens = Rect::new(
                Point::new(Px(lens_x), Px(lens_y)),
                Size::new(Px(lens_w), Px(lens_h)),
            );
            scene.push(SceneOp::Quad {
                order: DrawOrder(10_000),
                rect: lens,
                background: Paint::Solid(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.25,
                }),
                border: Edges::all(Px(1.0)),
                border_paint: Paint::Solid(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.8,
                }),
                corner_radii: Corners::all(Px(24.0)),
            });
        }

        app.request_redraw(window);
        app.push_effect(Effect::RequestAnimationFrame(window));
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo custom_effect_v3_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    CustomEffectV3WebDriver::default()
}
