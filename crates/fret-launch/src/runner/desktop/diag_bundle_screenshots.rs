use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub struct DiagBundleScreenshotCapture {
    enabled: bool,
    diag_dir: PathBuf,
    last_attempted_dir: Option<PathBuf>,
}

impl DiagBundleScreenshotCapture {
    pub fn from_env() -> Self {
        let enabled = env_flag_default_false("FRET_DIAG_SCREENSHOT");
        let diag_dir = std::env::var_os("FRET_DIAG_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/fret-diag"));
        Self {
            enabled,
            diag_dir,
            last_attempted_dir: None,
        }
    }

    pub fn surface_usage(&self) -> wgpu::TextureUsages {
        if self.enabled {
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        } else {
            wgpu::TextureUsages::RENDER_ATTACHMENT
        }
    }

    pub fn poll_request_dir(&mut self) -> Option<PathBuf> {
        if !self.enabled {
            return None;
        }

        let latest_rel = std::fs::read_to_string(self.diag_dir.join("latest.txt")).ok()?;
        let latest_rel = latest_rel.trim();
        if latest_rel.is_empty() {
            return None;
        }
        let dir = self.diag_dir.join(latest_rel);
        if !dir.is_dir() {
            return None;
        }

        if !dir.join("screenshot.request").is_file() {
            return None;
        }
        if dir.join("frame.bmp").is_file() {
            return None;
        }

        if self.last_attempted_dir.as_ref().is_some_and(|d| d == &dir) {
            return None;
        }
        self.last_attempted_dir = Some(dir.clone());
        Some(dir)
    }

    pub fn begin_readback(
        &self,
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Option<(PendingScreenshotReadback, wgpu::CommandBuffer)> {
        PendingScreenshotReadback::begin(device, texture, format, size)
    }

    pub fn finish_and_write_bmp(
        &self,
        device: &wgpu::Device,
        pending: PendingScreenshotReadback,
        out_dir: &Path,
        format: wgpu::TextureFormat,
    ) -> std::io::Result<()> {
        let out_path = out_dir.join("frame.bmp");
        let size = pending.size;
        let pixels = pending.finish_readback(device).ok();
        let Some(pixels) = pixels else {
            return Ok(());
        };
        let bmp = encode_frame_as_bmp(&pixels, size, format);
        let Some(bmp) = bmp else {
            return Ok(());
        };
        std::fs::write(out_path, bmp)?;
        std::fs::write(out_dir.join("screenshot.done"), b"ok\n")?;
        Ok(())
    }
}

pub struct PendingScreenshotReadback {
    buffer: wgpu::Buffer,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    pub size: (u32, u32),
}

impl PendingScreenshotReadback {
    fn begin(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Option<(Self, wgpu::CommandBuffer)> {
        if size.0 == 0 || size.1 == 0 {
            return None;
        }

        if bytes_per_pixel(format)? != 4 {
            return None;
        }

        let (width, height) = size;
        let unpadded_bytes_per_row = width * 4;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256) * 256;
        let buffer_size = padded_bytes_per_row as u64 * height as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret diag screenshot readback buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret diag screenshot readback encoder"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let cmd = encoder.finish();
        Some((
            Self {
                buffer,
                padded_bytes_per_row,
                unpadded_bytes_per_row,
                size,
            },
            cmd,
        ))
    }

    fn finish_readback(self, device: &wgpu::Device) -> Result<Vec<u8>, ()> {
        let (_width, height) = self.size;

        let slice = self.buffer.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().map_err(|_| ())?.map_err(|_| ())?;

        let mapped = slice.get_mapped_range();
        let mut pixels = vec![0u8; (self.unpadded_bytes_per_row * height) as usize];
        for row in 0..height as usize {
            let src = row * self.padded_bytes_per_row as usize;
            let dst = row * self.unpadded_bytes_per_row as usize;
            pixels[dst..dst + self.unpadded_bytes_per_row as usize]
                .copy_from_slice(&mapped[src..src + self.unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        self.buffer.unmap();

        Ok(pixels)
    }
}

fn bytes_per_pixel(format: wgpu::TextureFormat) -> Option<u32> {
    match format {
        wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb => Some(4),
        _ => None,
    }
}

fn encode_frame_as_bmp(
    pixels: &[u8],
    size: (u32, u32),
    format: wgpu::TextureFormat,
) -> Option<Vec<u8>> {
    let (width, height) = size;
    if width == 0 || height == 0 {
        return None;
    }
    let expected_len = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if pixels.len() < expected_len {
        return None;
    }

    let dib_header_size: u32 = 40;
    let file_header_size: u32 = 14;
    let pixel_offset: u32 = file_header_size + dib_header_size;
    let file_size: u32 = pixel_offset.saturating_add(expected_len as u32);

    let mut out = Vec::with_capacity(pixel_offset as usize + expected_len);

    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&pixel_offset.to_le_bytes());

    out.extend_from_slice(&dib_header_size.to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&(-(height as i32)).to_le_bytes()); // top-down
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&(expected_len as u32).to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes()); // x ppm
    out.extend_from_slice(&0i32.to_le_bytes()); // y ppm
    out.extend_from_slice(&0u32.to_le_bytes()); // colors used
    out.extend_from_slice(&0u32.to_le_bytes()); // important colors

    match format {
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
            out.extend_from_slice(&pixels[..expected_len]);
        }
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
            for px in pixels[..expected_len].chunks_exact(4) {
                out.push(px[2]); // B
                out.push(px[1]); // G
                out.push(px[0]); // R
                out.push(px[3]); // A
            }
        }
        _ => return None,
    }

    Some(out)
}

fn env_flag_default_false(name: &str) -> bool {
    let Ok(v) = std::env::var(name) else {
        return false;
    };
    let v = v.trim().to_ascii_lowercase();
    if v.is_empty() {
        return true;
    }
    !matches!(v.as_str(), "0" | "false" | "no" | "off")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bmp_header_is_well_formed() {
        let pixels = vec![0u8; 4 * 2 * 3];
        let bmp =
            encode_frame_as_bmp(&pixels, (2, 3), wgpu::TextureFormat::Rgba8UnormSrgb).expect("bmp");
        assert_eq!(&bmp[..2], b"BM");
        assert_eq!(u32::from_le_bytes(bmp[10..14].try_into().unwrap()), 54);
        assert_eq!(bmp.len(), 54 + pixels.len());
    }
}
