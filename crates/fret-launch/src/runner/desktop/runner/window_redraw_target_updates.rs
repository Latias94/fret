use fret_render::Renderer;
use tracing::error;

use super::RenderTargetUpdate;

pub(super) fn apply_window_redraw_target_updates(
    renderer: &mut Renderer,
    target_updates: Vec<RenderTargetUpdate>,
) {
    for update in target_updates {
        match update {
            RenderTargetUpdate::Update { id, desc } => {
                if !renderer.update_render_target(id, desc) {
                    error!(
                        ?id,
                        "engine frame update tried to update unknown render target"
                    );
                }
            }
            RenderTargetUpdate::Unregister { id } => {
                if !renderer.unregister_render_target(id) {
                    error!(
                        ?id,
                        "engine frame update tried to unregister unknown render target"
                    );
                }
            }
        }
    }
}
