use super::{Renderer, WgpuContext, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn install_renderer_bootstrap(&mut self, context: WgpuContext) -> bool {
        let mut renderer = Renderer::new(&context.adapter, &context.device);

        let renderer_caps = fret_render::RendererCapabilities::from_adapter_device(
            &context.adapter,
            &context.device,
        );
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());

        renderer.set_svg_raster_budget_bytes(self.config.svg_raster_budget_bytes);
        renderer.set_intermediate_budget_bytes(self.config.renderer_intermediate_budget_bytes);
        renderer.set_path_msaa_samples(self.config.path_msaa_samples);

        let startup_async = Self::system_font_rescan_async_enabled()
            && Self::system_font_catalog_startup_async_enabled();
        // Desktop also starts from the framework-owned bundled baseline. System font discovery
        // augments it later without changing the startup baseline contract.
        let _ = super::super::super::font_catalog::initialize_desktop_startup_font_environment(
            &mut self.app,
            &mut renderer,
            self.config.text_font_families.clone(),
            startup_async,
        );

        self.context = Some(context);
        self.renderer = Some(renderer);
        self.renderer_caps = Some(renderer_caps);
        if let (Some(context), Some(renderer)) = (self.context.as_ref(), self.renderer.as_mut()) {
            self.driver.gpu_ready(&mut self.app, context, renderer);
        }

        startup_async
    }
}
