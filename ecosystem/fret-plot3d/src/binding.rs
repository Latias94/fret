//! App-facing Plot3D bindings that hide raw runtime model plumbing from examples.

use fret_core::RenderTargetId;
use fret_runtime::{Model, ModelCx, ModelHost, ModelUpdateError};

use crate::declarative::{Plot3dModel, Plot3dPanelProps, Plot3dViewport};

#[derive(Clone)]
pub struct Plot3dPanelBinding {
    model: Model<Plot3dModel>,
}

impl Plot3dPanelBinding {
    /// Insert the Plot3D model into a model host from an initial viewport contract.
    #[track_caller]
    pub fn new(host: &mut impl ModelHost, viewport: Plot3dViewport) -> Self {
        Self::new_model(host, Plot3dModel { viewport })
    }

    /// Insert a caller-provided Plot3D model into a model host.
    #[track_caller]
    pub fn new_model(host: &mut impl ModelHost, model: Plot3dModel) -> Self {
        Self {
            model: host.models_mut().insert(model),
        }
    }

    /// Build declarative panel props wired to this binding's model.
    pub fn panel_props(&self) -> Plot3dPanelProps {
        Plot3dPanelProps::new(self.model.clone())
    }

    /// Read the current viewport without registering a UI invalidation dependency.
    ///
    /// This is intended for render-target allocation code that runs outside the UI render pass.
    pub fn viewport_untracked(&self, host: &impl ModelHost) -> Plot3dViewport {
        self.model
            .read_ref(host, |model| model.viewport)
            .unwrap_or_default()
    }

    /// Synchronize the engine-owned render target identity and pixel size into the panel model.
    ///
    /// Returns `true` only when the model changed.
    #[track_caller]
    pub fn sync_viewport_target(
        &self,
        host: &mut impl ModelHost,
        target: RenderTargetId,
        target_px_size: (u32, u32),
    ) -> Result<bool, ModelUpdateError> {
        let current = self.model.read_ref(host, |model| model.viewport)?;
        if current.target == target && current.target_px_size == target_px_size {
            return Ok(false);
        }

        self.model.update(host, |model, _cx| {
            model.viewport.target = target;
            model.viewport.target_px_size = target_px_size;
            true
        })
    }

    /// Read the controlled Plot3D model without exposing the raw model handle.
    pub fn read_model_untracked<R>(
        &self,
        host: &impl ModelHost,
        f: impl FnOnce(&Plot3dModel) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.model.read_ref(host, f)
    }

    /// Mutate the controlled Plot3D model without exposing the raw model handle.
    #[track_caller]
    pub fn update_model<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut Plot3dModel, &mut ModelCx<'_, H>) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.model.update(host, f)
    }

    /// Advanced bridge for component authors that already own a raw Plot3D model handle.
    ///
    /// Prefer [`Self::new`] for app code. This method exists so advanced viewport coordinators can
    /// graduate to the binding surface without rebuilding already-shared models.
    pub fn from_model(model: Model<Plot3dModel>) -> Self {
        Self { model }
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{RenderTargetId, ViewportFit};
    use fret_runtime::{ModelHost, ModelStore};

    use super::*;

    #[derive(Default)]
    struct TestHost {
        models: ModelStore,
    }

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    fn sample_viewport() -> Plot3dViewport {
        Plot3dViewport {
            target: RenderTargetId::default(),
            target_px_size: (640, 360),
            fit: ViewportFit::Contain,
            opacity: 0.75,
        }
    }

    #[test]
    fn plot3d_binding_creates_panel_props_without_public_raw_handles() {
        let mut host = TestHost::default();

        let binding = Plot3dPanelBinding::new(&mut host, sample_viewport());
        let props = binding.panel_props();

        assert_eq!(
            host.models()
                .read(&props.model, |model| model.viewport.target_px_size)
                .unwrap(),
            (640, 360)
        );
        assert_eq!(binding.viewport_untracked(&host).opacity, 0.75);
    }

    #[test]
    fn plot3d_binding_syncs_viewport_target_only_when_changed() {
        let mut host = TestHost::default();
        let binding = Plot3dPanelBinding::new(&mut host, sample_viewport());
        let props = binding.panel_props();
        let initial_revision = props.model.revision(&host);

        assert!(
            !binding
                .sync_viewport_target(&mut host, RenderTargetId::default(), (640, 360))
                .unwrap()
        );
        assert_eq!(
            props.model.revision(&host),
            initial_revision,
            "no-op viewport target sync should not dirty the Plot3D model"
        );

        let next_target = RenderTargetId::default();
        assert!(
            binding
                .sync_viewport_target(&mut host, next_target, (1280, 720))
                .unwrap()
        );

        let viewport = binding.viewport_untracked(&host);
        assert_eq!(viewport.target, next_target);
        assert_eq!(viewport.target_px_size, (1280, 720));
        assert!(
            props.model.revision(&host) > initial_revision,
            "changed viewport target sync should dirty the Plot3D model"
        );
    }
}
