use super::{WgpuContext, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn publish_wgpu_adapter_selection_diagnostics(&mut self, context: &WgpuContext) {
        if let Some(raw) = std::env::var_os("FRET_WGPU_BACKEND")
            && !raw.is_empty()
        {
            tracing::info!(requested = ?raw, "wgpu backend requested");
        }

        let info = context.adapter.get_info();
        tracing::info!(
            backend = ?info.backend,
            name = info.name,
            driver = info.driver,
            driver_info = info.driver_info,
            vendor = info.vendor,
            device = info.device,
            "wgpu adapter selected"
        );

        let downlevel = context.adapter.get_downlevel_capabilities();
        if !downlevel.is_webgpu_compliant() {
            tracing::warn!(
                flags = ?downlevel.flags,
                "wgpu adapter is downlevel (not fully WebGPU compliant)"
            );
        }

        if context.init_diagnostics.allow_fallback || context.init_diagnostics.attempts.len() > 1 {
            tracing::info!(
                attempts = ?context.init_diagnostics.attempts,
                "wgpu init attempts"
            );
        }

        self.app
            .set_global::<fret_render::WgpuAdapterSelectionSnapshot>(
                fret_render::WgpuAdapterSelectionSnapshot::from_context(context),
            );
    }
}
