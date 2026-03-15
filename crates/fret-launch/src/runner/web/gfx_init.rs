use winit::dpi::PhysicalSize;
use winit::platform::web::WindowExtWeb;
use winit::window::Window;

use super::{WinitAppDriver, WinitRunner};

#[cfg(target_arch = "wasm32")]
fn bundled_font_role_name(role: fret_fonts::BundledFontRole) -> &'static str {
    match role {
        fret_fonts::BundledFontRole::UiSans => "UiSans",
        fret_fonts::BundledFontRole::UiSerif => "UiSerif",
        fret_fonts::BundledFontRole::UiMonospace => "UiMonospace",
        fret_fonts::BundledFontRole::EmojiFallback => "EmojiFallback",
        fret_fonts::BundledFontRole::CjkFallback => "CjkFallback",
    }
}

#[cfg(target_arch = "wasm32")]
fn bundled_generic_family_name(family: fret_fonts::BundledGenericFamily) -> &'static str {
    match family {
        fret_fonts::BundledGenericFamily::Sans => "Sans",
        fret_fonts::BundledGenericFamily::Serif => "Serif",
        fret_fonts::BundledGenericFamily::Monospace => "Monospace",
    }
}

#[cfg(target_arch = "wasm32")]
fn default_bundled_font_baseline_snapshot() -> fret_runtime::BundledFontBaselineSnapshot {
    let profile = fret_fonts::default_profile();
    fret_runtime::BundledFontBaselineSnapshot::bundled_profile(
        profile.name,
        fret_fonts::bundled_asset_bundle().as_str(),
        profile
            .faces
            .iter()
            .map(|face| face.asset_key.to_string())
            .collect(),
        profile
            .provided_roles
            .iter()
            .map(|role| bundled_font_role_name(*role).to_string())
            .collect(),
        profile
            .guaranteed_generic_families
            .iter()
            .map(|family| bundled_generic_family_name(*family).to_string())
            .collect(),
    )
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn adopt_gfx_if_ready(&mut self) {
        if self.gfx.is_some() {
            return;
        }
        let pending = self.pending_gfx.borrow_mut().take();
        let Some(mut gfx) = pending else {
            return;
        };

        let renderer_caps = fret_render::RendererCapabilities::from_adapter_device(
            &gfx.ctx.adapter,
            &gfx.ctx.device,
        );
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());
        self.renderer_caps = Some(renderer_caps);

        let _ = super::super::font_catalog::publish_bundled_font_baseline_snapshot(
            &mut self.app,
            default_bundled_font_baseline_snapshot(),
        );

        // Web/WASM cannot access system fonts. Load our bundled defaults as soon as the renderer
        // becomes available, then let the runtime font bootstrap policy fill missing UI families.
        let default_fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        let _ = gfx.renderer.add_fonts(default_fonts);

        // Font catalog refresh trigger (ADR 0258): initial renderer availability (adopt gfx).
        let _update = super::super::font_catalog::initialize_startup_font_environment(
            &mut self.app,
            &mut gfx.renderer,
            self.config.text_font_families.clone(),
            super::super::font_catalog::StartupFontEnvironmentMode::WebBundledSync,
        );

        self.gfx = Some(gfx);
    }

    pub(super) fn ensure_gpu_ready_hook(&mut self) {
        if self.gpu_ready_called {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        self.driver
            .gpu_ready(&mut self.app, &gfx.ctx, &mut gfx.renderer);
        self.gpu_ready_called = true;
    }

    pub(super) fn desired_surface_size(window: &dyn Window) -> Option<PhysicalSize<u32>> {
        let canvas: web_sys::HtmlCanvasElement = window.canvas()?.clone();
        let web_window = web_sys::window()?;
        let dpr = web_window.device_pixel_ratio().max(1.0);
        let css_w = canvas.client_width().max(0) as f64;
        let css_h = canvas.client_height().max(0) as f64;
        let physical = PhysicalSize::new(
            (css_w * dpr).round().max(1.0) as u32,
            (css_h * dpr).round().max(1.0) as u32,
        );

        if canvas.width() != physical.width {
            canvas.set_width(physical.width);
        }
        if canvas.height() != physical.height {
            canvas.set_height(physical.height);
        }

        Some(physical)
    }
}
