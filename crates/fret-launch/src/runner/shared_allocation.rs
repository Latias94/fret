use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharedAllocationExportError {
    #[error("shared allocation export is not supported on this backend")]
    UnsupportedBackend,
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub mod dx12 {
    use super::SharedAllocationExportError;
    use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Resource};

    #[derive(Debug)]
    pub struct Dx12SharedAllocationWriteGuard {
        device: wgpu::Device,
        queue: wgpu::Queue,
        texture: wgpu::Texture,
        restore_uses: wgpu::wgt::TextureUses,
        queue_raw: ID3D12CommandQueue,
        resource: ID3D12Resource,
        finished: bool,
    }

    impl Dx12SharedAllocationWriteGuard {
        /// Begin a native write into a renderer-owned `wgpu::Texture` on the DX12 backend.
        ///
        /// Contract:
        /// - The returned `queue_raw()` must be the queue used by the native writer.
        /// - The caller must ensure any native command lists are submitted before `finish()` (or drop).
        pub fn begin(
            ctx: &fret_render::WgpuContext,
            texture: &wgpu::Texture,
            pre_uses: wgpu::wgt::TextureUses,
        ) -> Result<Self, SharedAllocationExportError> {
            let (queue_guard, tex_guard) = match (
                unsafe { ctx.queue.as_hal::<wgpu::hal::dx12::Api>() },
                unsafe { texture.as_hal::<wgpu::hal::dx12::Api>() },
            ) {
                (Some(queue_guard), Some(tex_guard)) => (queue_guard, tex_guard),
                _ => return Err(SharedAllocationExportError::UnsupportedBackend),
            };

            let mut enc = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("shared allocation dx12 interop pre-transition"),
                });
            enc.transition_resources(
                std::iter::empty(),
                std::iter::once(wgpu::wgt::TextureTransition {
                    texture,
                    selector: None,
                    state: pre_uses,
                }),
            );
            ctx.queue.submit([enc.finish()]);

            let queue_raw = queue_guard.as_raw().clone();
            let resource = unsafe { tex_guard.raw_resource() }.clone();

            Ok(Self {
                device: ctx.device.clone(),
                queue: ctx.queue.clone(),
                texture: texture.clone(),
                restore_uses: wgpu::wgt::TextureUses::RESOURCE,
                queue_raw,
                resource,
                finished: false,
            })
        }

        pub fn queue_raw(&self) -> &ID3D12CommandQueue {
            &self.queue_raw
        }

        pub fn resource(&self) -> &ID3D12Resource {
            &self.resource
        }

        pub fn finish(mut self) {
            self.finish_inner();
            self.finished = true;
        }

        fn finish_inner(&self) {
            let mut enc = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("shared allocation dx12 interop post-transition"),
                });
            enc.transition_resources(
                std::iter::empty(),
                std::iter::once(wgpu::wgt::TextureTransition {
                    texture: &self.texture,
                    selector: None,
                    state: self.restore_uses,
                }),
            );
            self.queue.submit([enc.finish()]);
        }
    }

    impl Drop for Dx12SharedAllocationWriteGuard {
        fn drop(&mut self) {
            if self.finished {
                return;
            }
            self.finish_inner();
        }
    }
}
