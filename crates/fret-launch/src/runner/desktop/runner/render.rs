use super::*;
use fret_core::Scene;
use tracing::error;

pub(super) fn validate_scene_if_enabled(scene: &Scene) {
    if std::env::var_os("FRET_VALIDATE_SCENE").is_none() {
        return;
    }

    if let Err(err) = scene.validate() {
        error!(
            index = err.index,
            op = ?err.op,
            kind = ?err.kind,
            error = %err,
            "scene validation failed (set FRET_VALIDATE_SCENE_PANIC=1 to panic)"
        );

        if std::env::var_os("FRET_VALIDATE_SCENE_PANIC").is_some() {
            panic!("scene validation failed: {err}");
        }
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn init_renderdoc_if_needed(&mut self) {
        if self.renderdoc.is_some() {
            return;
        }

        let enabled = std::env::var_os("FRET_RENDERDOC")
            .filter(|v| !v.is_empty())
            .is_some()
            || std::env::var_os("FRET_RENDERDOC_DLL")
                .filter(|v| !v.is_empty())
                .is_some();

        if !enabled {
            return;
        }

        self.renderdoc = RenderDocCapture::try_init();
        if self.renderdoc.is_some() {
            tracing::info!("renderdoc capture enabled");
        } else {
            tracing::warn!(
                "renderdoc capture requested but renderdoc API is unavailable (set FRET_RENDERDOC_DLL to renderdoc.dll path)"
            );
        }
    }

    pub(super) fn ui_services_mut<'a>(
        renderer: &'a mut Option<Renderer>,
        no_services: &'a mut NoUiServices,
    ) -> &'a mut dyn UiServices {
        match renderer.as_mut() {
            Some(renderer) => renderer as &mut dyn UiServices,
            None => no_services as &mut dyn UiServices,
        }
    }

    pub(super) fn resize_surface(
        &mut self,
        window: fret_core::AppWindowId,
        width: u32,
        height: u32,
    ) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let Some(surface) = state.surface.as_mut() else {
            return;
        };
        let (cur_w, cur_h) = surface.size();
        if cur_w == width.max(1) && cur_h == height.max(1) {
            return;
        }
        surface.resize(&context.device, width, height);
    }
}
