use std::sync::{Arc, LazyLock};

use resvg::tiny_skia::{Pixmap, Transform};

pub const SMOOTH_SVG_SCALE_FACTOR: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct SvgAlphaMask {
    pub size_px: (u32, u32),
    pub alpha: Vec<u8>,
}

pub struct UploadedAlphaMask {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size_px: (u32, u32),
}

#[derive(Clone)]
pub struct SvgRenderer {
    usvg_options: Arc<usvg::Options<'static>>,
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
        let (w, h) = target_box_px;
        if w == 0 || h == 0 {
            return Err(usvg::Error::InvalidSize);
        }

        let tree = usvg::Tree::from_data(bytes, &self.usvg_options)?;
        let svg_size = tree.size();
        let box_w = w as f32 * smooth_scale_factor;
        let box_h = h as f32 * smooth_scale_factor;
        let scale_x = box_w / svg_size.width();
        let scale_y = box_h / svg_size.height();
        let scale = scale_x.min(scale_y);
        if !scale.is_finite() || scale <= 0.0 {
            return Err(usvg::Error::InvalidSize);
        }

        let out_w = (svg_size.width() * scale) as u32;
        let out_h = (svg_size.height() * scale) as u32;
        if out_w == 0 || out_h == 0 {
            return Err(usvg::Error::InvalidSize);
        }

        let mut pixmap = Pixmap::new(out_w, out_h).ok_or(usvg::Error::InvalidSize)?;
        let transform = Transform::from_scale(scale, scale);
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
}
