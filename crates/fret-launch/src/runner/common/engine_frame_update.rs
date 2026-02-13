use std::{any::Any, fmt};

/// A per-frame keepalive token carried from driver code to the runner submission boundary.
///
/// This exists to support truly-ephemeral imported resources (e.g. platform-backed external
/// textures) whose validity may not be fully captured by a `wgpu::TextureView` handle alone.
///
/// The runner must keep these tokens alive until engine + UI command buffers have been submitted
/// for the frame (ADR 0234 D3).
pub struct EngineFrameKeepalive(Box<dyn Any>);

impl EngineFrameKeepalive {
    pub fn new<T: 'static>(value: T) -> Self {
        Self(Box::new(value))
    }
}

pub enum RenderTargetUpdate {
    Update {
        id: fret_core::RenderTargetId,
        desc: fret_render::RenderTargetDescriptor,
    },
    Unregister {
        id: fret_core::RenderTargetId,
    },
}

impl RenderTargetUpdate {
    pub fn update(
        id: fret_core::RenderTargetId,
        desc: fret_render::RenderTargetDescriptor,
    ) -> Self {
        Self::Update { id, desc }
    }

    pub fn unregister(id: fret_core::RenderTargetId) -> Self {
        Self::Unregister { id }
    }
}

impl fmt::Debug for RenderTargetUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Update { id, desc } => f
                .debug_struct("Update")
                .field("id", id)
                .field("size", &desc.size)
                .field("format", &desc.format)
                .field("color_space", &desc.color_space)
                .field("view", &"<wgpu::TextureView>")
                .finish(),
            Self::Unregister { id } => f.debug_struct("Unregister").field("id", id).finish(),
        }
    }
}

#[derive(Default)]
pub struct EngineFrameUpdate {
    pub target_updates: Vec<RenderTargetUpdate>,
    pub command_buffers: Vec<wgpu::CommandBuffer>,
    pub keepalive: Vec<EngineFrameKeepalive>,
}

impl EngineFrameUpdate {
    pub fn push_command_buffer(&mut self, cb: wgpu::CommandBuffer) {
        self.command_buffers.push(cb);
    }

    pub fn push_keepalive<T: 'static>(&mut self, value: T) {
        self.keepalive.push(EngineFrameKeepalive::new(value));
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: fret_render::RenderTargetDescriptor,
    ) {
        self.target_updates
            .push(RenderTargetUpdate::Update { id, desc });
    }

    pub fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) {
        self.target_updates
            .push(RenderTargetUpdate::Unregister { id });
    }
}
