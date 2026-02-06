use std::sync::{Arc, LazyLock};

use fret_core::SvgFit;
use resvg::tiny_skia::{Pixmap, Transform};

use crate::upload_counters::record_svg_upload;

pub const SMOOTH_SVG_SCALE_FACTOR: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct SvgAlphaMask {
    pub size_px: (u32, u32),
    pub alpha: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SvgRgbaImage {
    pub size_px: (u32, u32),
    /// Unpremultiplied RGBA8 pixels (matches `SceneOp::Image` expectations).
    pub rgba: Vec<u8>,
}

pub struct UploadedAlphaMask {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size_px: (u32, u32),
}

pub struct UploadedRgbaImage {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size_px: (u32, u32),
}

#[derive(Clone)]
pub struct SvgRenderer {
    usvg_options: Arc<usvg::Options<'static>>,
}

fn render_scale(
    svg_size: usvg::Size,
    target_box_px: (u32, u32),
    smooth_scale_factor: f32,
    fit: SvgFit,
) -> Result<(f32, f32, u32, u32), usvg::Error> {
    let (w, h) = target_box_px;
    if w == 0 || h == 0 {
        return Err(usvg::Error::InvalidSize);
    }

    let box_w = w as f32 * smooth_scale_factor;
    let box_h = h as f32 * smooth_scale_factor;
    let scale_x = box_w / svg_size.width();
    let scale_y = box_h / svg_size.height();
    if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(usvg::Error::InvalidSize);
    }

    let (sx, sy, out_w, out_h) = match fit {
        SvgFit::Contain => {
            let scale = scale_x.min(scale_y);
            let out_w = (svg_size.width() * scale).ceil() as u32;
            let out_h = (svg_size.height() * scale).ceil() as u32;
            (scale, scale, out_w, out_h)
        }
        SvgFit::Width => {
            let out_w = (svg_size.width() * scale_x).ceil() as u32;
            let out_h = (svg_size.height() * scale_x).ceil() as u32;
            (scale_x, scale_x, out_w, out_h)
        }
        SvgFit::Stretch => {
            let out_w = box_w.ceil() as u32;
            let out_h = box_h.ceil() as u32;
            (scale_x, scale_y, out_w, out_h)
        }
    };

    if out_w == 0 || out_h == 0 {
        return Err(usvg::Error::InvalidSize);
    }

    Ok((sx, sy, out_w, out_h))
}

impl SvgRenderer {
    pub fn new() -> Self {
        static FONT_DB: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
            let mut db = usvg::fontdb::Database::new();
            db.load_system_fonts();
            Arc::new(db)
        });
        let default_font_resolver = usvg::FontResolver::default_font_selector();
        let font_resolver = Box::new(
            move |font: &usvg::Font, db: &mut Arc<usvg::fontdb::Database>| {
                if db.is_empty() {
                    *db = FONT_DB.clone();
                }
                default_font_resolver(font, db)
            },
        );
        let options = usvg::Options {
            font_resolver: usvg::FontResolver {
                select_font: font_resolver,
                select_fallback: usvg::FontResolver::default_fallback_selector(),
            },
            ..Default::default()
        };
        Self {
            usvg_options: Arc::new(options),
        }
    }

    pub fn render_alpha_mask_fit(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
    ) -> Result<SvgAlphaMask, usvg::Error> {
        self.render_alpha_mask_fit_mode(bytes, target_box_px, smooth_scale_factor, SvgFit::Contain)
    }

    pub fn render_alpha_mask_fit_mode(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgAlphaMask, usvg::Error> {
        let tree = usvg::Tree::from_data(bytes, &self.usvg_options)?;
        let svg_size = tree.size();
        let (sx, sy, out_w, out_h) =
            render_scale(svg_size, target_box_px, smooth_scale_factor, fit)?;

        let mut pixmap = Pixmap::new(out_w, out_h).ok_or(usvg::Error::InvalidSize)?;
        let transform = Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let alpha = pixmap
            .pixels()
            .iter()
            .map(|p| p.alpha())
            .collect::<Vec<_>>();
        Ok(SvgAlphaMask {
            size_px: (pixmap.width(), pixmap.height()),
            alpha,
        })
    }

    pub fn render_alpha_mask(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> Result<SvgAlphaMask, usvg::Error> {
        self.render_alpha_mask_fit(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR)
    }

    pub fn render_rgba_fit(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
    ) -> Result<SvgRgbaImage, usvg::Error> {
        self.render_rgba_fit_mode(bytes, target_box_px, smooth_scale_factor, SvgFit::Contain)
    }

    pub fn render_rgba_fit_mode(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgRgbaImage, usvg::Error> {
        let tree = usvg::Tree::from_data(bytes, &self.usvg_options)?;
        let svg_size = tree.size();
        let (sx, sy, out_w, out_h) =
            render_scale(svg_size, target_box_px, smooth_scale_factor, fit)?;

        let mut pixmap = Pixmap::new(out_w, out_h).ok_or(usvg::Error::InvalidSize)?;
        let transform = Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // tiny-skia pixmap stores RGBA premultiplied alpha. Our image pipeline expects
        // unpremultiplied RGBA (it premultiplies in the shader).
        let mut rgba = pixmap.take();
        debug_assert_eq!(rgba.len(), (out_w as usize) * (out_h as usize) * 4);

        for px in rgba.chunks_exact_mut(4) {
            let a = px[3] as u32;
            if a == 0 || a == 255 {
                continue;
            }
            // Unpremultiply with rounding; clamp to 0..255.
            let r = (px[0] as u32 * 255 + a / 2) / a;
            let g = (px[1] as u32 * 255 + a / 2) / a;
            let b = (px[2] as u32 * 255 + a / 2) / a;
            px[0] = r.min(255) as u8;
            px[1] = g.min(255) as u8;
            px[2] = b.min(255) as u8;
        }

        Ok(SvgRgbaImage {
            size_px: (out_w, out_h),
            rgba,
        })
    }

    pub fn render_rgba(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> Result<SvgRgbaImage, usvg::Error> {
        self.render_rgba_fit(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR)
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn upload_alpha_mask(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mask: &SvgAlphaMask,
) -> UploadedAlphaMask {
    let (w, h) = mask.size_px;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret svg alpha mask"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_row = w;
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);
    let data = if aligned_bytes_per_row == bytes_per_row {
        mask.alpha.clone()
    } else {
        let mut padded = vec![0u8; (aligned_bytes_per_row * h) as usize];
        for row in 0..h as usize {
            let src0 = row * w as usize;
            let src1 = src0 + w as usize;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + w as usize;
            padded[dst0..dst1].copy_from_slice(&mask.alpha[src0..src1]);
        }
        padded
    };

    if w > 0 && h > 0 {
        record_svg_upload(data.len());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    UploadedAlphaMask {
        texture,
        view,
        size_px: (w, h),
    }
}

pub fn upload_rgba_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &SvgRgbaImage,
) -> UploadedRgbaImage {
    let (w, h) = image.size_px;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret svg rgba"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_row = w * 4;
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);
    let data = if aligned_bytes_per_row == bytes_per_row {
        image.rgba.clone()
    } else {
        let mut padded = vec![0u8; (aligned_bytes_per_row * h) as usize];
        for row in 0..h as usize {
            let src0 = row * (w as usize) * 4;
            let src1 = src0 + (w as usize) * 4;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + (w as usize) * 4;
            padded[dst0..dst1].copy_from_slice(&image.rgba[src0..src1]);
        }
        padded
    };

    if w > 0 && h > 0 {
        record_svg_upload(data.len());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    UploadedRgbaImage {
        texture,
        view,
        size_px: (w, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_alpha_mask_has_coverage() {
        let svg = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
  <circle cx="8" cy="8" r="6" fill="black" />
</svg>
"#;
        let renderer = SvgRenderer::new();
        let mask = renderer
            .render_alpha_mask(svg.as_bytes(), (32, 32))
            .expect("render alpha mask");

        let (w, h) = mask.size_px;
        assert!(w > 0 && h > 0);
        assert_eq!(mask.alpha.len(), (w as usize) * (h as usize));
        assert!(mask.alpha.iter().any(|&a| a > 0));
        assert!(mask.alpha.iter().any(|&a| a < 255));
    }

    #[test]
    fn svg_rgba_renders_pixels() {
        let svg = r##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
  <rect x="0" y="0" width="16" height="16" fill="#ff0000" />
  <circle cx="8" cy="8" r="6" fill="#00ff00" fill-opacity="0.5" />
</svg>
"##;
        let renderer = SvgRenderer::new();
        let img = renderer
            .render_rgba(svg.as_bytes(), (32, 32))
            .expect("render rgba");

        let (w, h) = img.size_px;
        assert!(w > 0 && h > 0);
        assert_eq!(img.rgba.len(), (w as usize) * (h as usize) * 4);
        assert!(img.rgba.chunks_exact(4).any(|px| px[3] > 0));
        // The output is composited on an opaque background (the red rect), so alpha will likely
        // be fully opaque; assert color blending instead.
        assert!(img.rgba.chunks_exact(4).any(|px| px[1] > 0 && px[0] < 255));
    }
}
