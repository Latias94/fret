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
fn radius_at(centered: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (centered.x >= 0.0) {
    if (centered.y <= 0.0) { return radii.y; }
    return radii.z;
  }
  if (centered.y <= 0.0) { return radii.x; }
  return radii.w;
}

fn sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let corner = abs(centered) - (half_size - vec2<f32>(radius));
  let outside = length(max(corner, vec2<f32>(0.0))) - radius;
  let inside = min(max(corner.x, corner.y), 0.0);
  return outside + inside;
}

fn grad_sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
  let corner = abs(centered) - (half_size - vec2<f32>(radius));
  if (corner.x >= 0.0 || corner.y >= 0.0) {
    return sign(centered) * normalize(max(corner, vec2<f32>(0.0)) + vec2<f32>(1.0e-6, 0.0));
  }
  let grad_x = select(0.0, 1.0, corner.y <= corner.x);
  return sign(centered) * vec2<f32>(grad_x, 1.0 - grad_x);
}

fn circle_map(x: f32) -> f32 {
  let xx = clamp(x, 0.0, 1.0);
  return 1.0 - sqrt(max(1.0 - xx * xx, 0.0));
}

fn hash01(p: vec2<f32>) -> f32 {
  // Deterministic, derivative-free "hash noise" for subtle grain.
  let u = dot(p, vec2<f32>(12.9898, 78.233));
  return fract(sin(u) * 43758.5453);
}

fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  // params.vec4s[0]:
  // - x: refraction_height_px (controls edge thickness)
  // - y: refraction_amount_px (controls displacement)
  // - z: pyramid_level
  // - w: frost_mix (0..1)
  let refraction_height_px = clamp(params.vec4s[0].x, 0.0, 96.0);
  let refraction_amount_px = clamp(params.vec4s[0].y, 0.0, 96.0);
  let pyramid_level = u32(clamp(params.vec4s[0].z, 0.0, 6.0));
  let frost_mix = clamp(params.vec4s[0].w, 0.0, 1.0);

  // params.vec4s[1]:
  // - x: corner_radius_px
  // - y: depth_effect (0..1)
  // - z: dispersion (0..1)
  // - w: highlight_alpha (0..1)
  let corner_radius_px = clamp(params.vec4s[1].x, 0.0, 256.0);
  let depth_effect = clamp(params.vec4s[1].y, 0.0, 1.0);
  let dispersion = clamp(params.vec4s[1].z, 0.0, 1.0);
  let highlight_alpha = clamp(params.vec4s[1].w, 0.0, 1.0);

  // params.vec4s[2]:
  // - x: inner_shadow_alpha (0..1)
  // - y: inner_shadow_radius_px
  // - z: vignette_strength (0..1)
  // - w: noise_alpha (0..0.1)
  let inner_shadow_alpha = clamp(params.vec4s[2].x, 0.0, 1.0);
  let inner_shadow_radius_px = clamp(params.vec4s[2].y, 0.0, 96.0);
  let vignette_strength = clamp(params.vec4s[2].z, 0.0, 1.0);
  let noise_alpha = clamp(params.vec4s[2].w, 0.0, 0.1);

  // params.vec4s[3]: tint (premul-ish RGB + alpha)
  let tint = vec4<f32>(
    clamp(params.vec4s[3].x, 0.0, 1.0),
    clamp(params.vec4s[3].y, 0.0, 1.0),
    clamp(params.vec4s[3].z, 0.0, 1.0),
    clamp(params.vec4s[3].w, 0.0, 1.0)
  );

  let local = fret_local_px(pos_px);
  let size = max(render_space.size_px, vec2<f32>(1.0));
  let half_size = size * 0.5;
  let centered = local - half_size;

  // Rounded-rect SDF (AndroidLiquidGlass-like).
  let radii = vec4<f32>(corner_radius_px, corner_radius_px, corner_radius_px, corner_radius_px);
  let radius = radius_at(centered, radii);
  let sd = sd_rounded_rect(centered, half_size, radius);
  let inside_px = clamp(-sd, 0.0, 4096.0);
  let inside01 = select(0.0, inside_px / max(refraction_height_px, 1.0), refraction_height_px > 0.0);

  // Edge weight (1 at edge -> 0 at center).
  let edge = 1.0 - smoothstep(0.15, 0.95, inside01);

  // Frosted source: prior blur (`src`) + optional extra pyramid sampling.
  let pyr = fret_sample_src_pyramid_at_pos(pyramid_level, pos_px);
  let frosted = mix(src, pyr, frost_mix);

  // Refraction direction from SDF gradient (optionally with "depth" pull toward center).
  let grad_radius = min(radius * 1.5, min(half_size.x, half_size.y));
  let g0 = grad_sd_rounded_rect(centered, half_size, grad_radius);
  let g1 = normalize(g0 + depth_effect * normalize(centered + vec2<f32>(1.0e-6, 0.0)));

  // Displacement magnitude (circle-map taper like AndroidLiquidGlass).
  let d = circle_map(1.0 - inside01) * refraction_amount_px;
  let refract = d * g1;

  // Chromatic dispersion (3 taps; cheaper than the full 7-tap Android shader).
  let disp_k = dispersion * abs((centered.x * centered.y) / max(half_size.x * half_size.y, 1.0));
  let disp = refract * disp_k;
  let raw_r = fret_sample_src_raw_at_pos(pos_px + refract + disp);
  let raw_g = fret_sample_src_raw_at_pos(pos_px + refract);
  let raw_b = fret_sample_src_raw_at_pos(pos_px + refract - disp);
  let raw = vec4<f32>(raw_r.r, raw_g.g, raw_b.b, raw_g.a);

  // Combine: crisp edge refraction over frosted center.
  var out_rgb = mix(frosted.rgb, raw.rgb, edge);

  // Specular-like highlight driven by edge normal (AndroidLiquidGlass highlight flavor).
  let light = normalize(vec2<f32>(-0.55, -0.85));
  let ndotl = abs(dot(g1, light));
  let hl = pow(ndotl, 2.8) * highlight_alpha * edge;
  out_rgb = out_rgb + vec3<f32>(1.0, 1.0, 1.0) * hl;

  // Inner shadow (darken near the edge).
  let shadow = (1.0 - smoothstep(0.0, max(inner_shadow_radius_px, 1.0), inside_px)) * inner_shadow_alpha;
  out_rgb = mix(out_rgb, out_rgb * (1.0 - 0.35 * shadow), 1.0);

  // Vignette (subtle center lift + edge falloff).
  let t = centered / max(half_size, vec2<f32>(1.0));
  let v = clamp(length(t), 0.0, 1.0);
  let vig = vignette_strength * smoothstep(0.35, 1.0, v);
  out_rgb = mix(out_rgb, out_rgb * (1.0 - 0.25 * vig), 1.0);

  // Tint + subtle grain.
  out_rgb = mix(out_rgb, mix(out_rgb, tint.rgb, tint.a), 1.0);
  let n = hash01(floor(pos_px) + vec2<f32>(17.0, 91.0)) - 0.5;
  out_rgb = out_rgb + vec3<f32>(n) * noise_alpha;

  return vec4<f32>(out_rgb, 1.0);
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
            let params = EffectParamsV1 {
                vec4s: [
                    // (refraction_height_px, refraction_amount_px, pyramid_level, frost_mix)
                    [22.0, 34.0, 3.0, 0.75],
                    // (corner_radius_px, depth_effect, dispersion, highlight_alpha)
                    [24.0, 0.18, 0.55, 0.32],
                    // (inner_shadow_alpha, inner_shadow_radius_px, vignette_strength, noise_alpha)
                    [0.22, 28.0, 0.25, 0.012],
                    // tint (rgb + alpha)
                    [1.0, 1.0, 1.0, 0.10],
                ],
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
                    radius_px: Px(18.0),
                    downsample: 2,
                },
                EffectStep::CustomV3 {
                    id: effect,
                    params,
                    max_sample_offset_px: Px(40.0),
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
                scene.push(SceneOp::PushClipRRect {
                    rect: lens,
                    corner_radii: Corners::all(Px(24.0)),
                });
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
                        a: 0.12,
                    }),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT),
                    corner_radii: Corners::all(Px(24.0)),
                });
                scene.push(SceneOp::PopEffect);
                scene.push(SceneOp::PopClip);

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
