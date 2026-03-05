use crate::RenderError;

fn env_var_trimmed(name: &str) -> Option<String> {
    let raw = std::env::var(name).ok()?;
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_env_u32(name: &str) -> Option<u32> {
    let raw = env_var_trimmed(name)?;
    raw.parse::<u32>().ok()
}

fn present_mode_from_env(caps: &wgpu::SurfaceCapabilities) -> Option<wgpu::PresentMode> {
    let raw = env_var_trimmed("FRET_RENDER_WGPU_SURFACE_PRESENT_MODE")?;
    let token = raw.to_ascii_lowercase().replace(['-', ' '], "_");
    let desired = match token.as_str() {
        "auto_vsync" | "autovsync" => wgpu::PresentMode::AutoVsync,
        "auto_no_vsync" | "autonovsync" | "auto_novsync" => wgpu::PresentMode::AutoNoVsync,
        "fifo" => wgpu::PresentMode::Fifo,
        "fifo_relaxed" | "fiforelaxed" => wgpu::PresentMode::FifoRelaxed,
        "immediate" => wgpu::PresentMode::Immediate,
        "mailbox" => wgpu::PresentMode::Mailbox,
        _ => return None,
    };

    // Only accept modes supported by the surface capabilities.
    caps.present_modes.iter().copied().find(|m| *m == desired)
}

pub struct SurfaceState<'window> {
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'window> SurfaceState<'window> {
    pub fn new_with_usage(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        surface: wgpu::Surface<'window>,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, RenderError> {
        let capabilities = surface.get_capabilities(adapter);
        if capabilities.formats.is_empty() {
            return Err(RenderError::SurfaceNoFormats);
        }
        if capabilities.present_modes.is_empty() {
            return Err(RenderError::SurfaceNoPresentModes);
        }
        if capabilities.alpha_modes.is_empty() {
            return Err(RenderError::SurfaceNoAlphaModes);
        }

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let present_mode =
            present_mode_from_env(&capabilities).unwrap_or(capabilities.present_modes[0]);
        let desired_maximum_frame_latency =
            parse_env_u32("FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY")
                .unwrap_or(2)
                .clamp(1, 8);

        let config = wgpu::SurfaceConfiguration {
            usage,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            desired_maximum_frame_latency,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(device, &config);

        Ok(Self { surface, config })
    }

    pub fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        surface: wgpu::Surface<'window>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        Self::new_with_usage(
            adapter,
            device,
            surface,
            width,
            height,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(device, &self.config);
    }

    pub fn get_current_frame_view(
        &self,
    ) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok((frame, view))
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn present_with(
        &self,
        queue: &wgpu::Queue,
        build_commands: impl FnOnce(&wgpu::TextureView) -> Vec<wgpu::CommandBuffer>,
    ) -> Result<(), RenderError> {
        let (frame, view) = self
            .get_current_frame_view()
            .map_err(|source| RenderError::SurfaceAcquireFailed { source })?;

        let cmd_buffers = build_commands(&view);
        queue.submit(cmd_buffers);
        frame.present();
        Ok(())
    }
}
