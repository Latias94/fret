use winit::dpi::PhysicalSize;
use winit::platform::web::WindowExtWeb;
use winit::window::Window;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn adopt_gfx_if_ready(&mut self) {
        if self.gfx.is_some() {
            return;
        }
        let pending = self.pending_gfx.borrow_mut().take();
        let Some(mut gfx) = pending else {
            return;
        };

        let renderer_caps = fret_render::RendererCapabilities::from_wgpu_context(&gfx.ctx);
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());
        self.renderer_caps = Some(renderer_caps);

        self.app
            .set_global::<fret_core::TextFontFamilyConfig>(self.config.text_font_families.clone());
        let _ = gfx
            .renderer
            .set_text_font_families(&self.config.text_font_families);

        // Web/WASM cannot access system fonts. Load our bundled defaults as soon as the renderer
        // becomes available, then seed `TextFontFamilyConfig` deterministically.
        let default_fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        let _ = gfx.renderer.add_fonts(default_fonts);

        let update = fret_runtime::apply_font_catalog_update(
            &mut self.app,
            gfx.renderer.all_font_names(),
            fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );
        let _ = gfx.renderer.set_text_font_families(&update.config);
        self.app
            .set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
                gfx.renderer.text_font_stack_key(),
            ));

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
