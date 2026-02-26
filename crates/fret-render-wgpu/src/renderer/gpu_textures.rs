pub(super) struct GpuTextures {
    mask_image_identity_texture: wgpu::Texture,
    mask_image_identity_uploaded: bool,

    material_catalog_texture: wgpu::Texture,
    material_catalog_uploaded: bool,

    custom_effect_input_fallback_texture: wgpu::Texture,
    custom_effect_input_fallback_uploaded: bool,
}

impl GpuTextures {
    pub(super) fn new(
        mask_image_identity_texture: wgpu::Texture,
        material_catalog_texture: wgpu::Texture,
        custom_effect_input_fallback_texture: wgpu::Texture,
    ) -> Self {
        Self {
            mask_image_identity_texture,
            mask_image_identity_uploaded: false,
            material_catalog_texture,
            material_catalog_uploaded: false,
            custom_effect_input_fallback_texture,
            custom_effect_input_fallback_uploaded: false,
        }
    }

    pub(super) fn ensure_mask_image_identity_uploaded(&mut self, queue: &wgpu::Queue) {
        if self.mask_image_identity_uploaded {
            return;
        }

        // Use a 1x1 `R8Unorm` texture filled with 1.0 coverage as the default mask-image source.
        // This keeps `Mask::Image` deterministic even if an image source disappears between
        // encoding and rendering.
        let bytes_per_row = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let mut bytes = vec![0u8; bytes_per_row as usize];
        bytes[0] = 255;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.mask_image_identity_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.mask_image_identity_uploaded = true;
    }

    pub(super) fn ensure_material_catalog_uploaded(&mut self, queue: &wgpu::Queue) {
        if self.material_catalog_uploaded {
            return;
        }

        // Layer 0: hash noise (portable and deterministic).
        // Layer 1: Bayer 8x8 repeated (portable and deterministic).
        let w = 64u32;
        let h = 64u32;
        let bytes_per_pixel = 4usize;
        let bytes_per_row = (w as usize) * bytes_per_pixel;

        fn bayer8x8(x: u32, y: u32) -> u8 {
            const M: [[u8; 8]; 8] = [
                [0, 48, 12, 60, 3, 51, 15, 63],
                [32, 16, 44, 28, 35, 19, 47, 31],
                [8, 56, 4, 52, 11, 59, 7, 55],
                [40, 24, 36, 20, 43, 27, 39, 23],
                [2, 50, 14, 62, 1, 49, 13, 61],
                [34, 18, 46, 30, 33, 17, 45, 29],
                [10, 58, 6, 54, 9, 57, 5, 53],
                [42, 26, 38, 22, 41, 25, 37, 21],
            ];
            M[(y & 7) as usize][(x & 7) as usize]
        }

        fn hash_noise_u8(x: u32, y: u32) -> u8 {
            let mut v = x ^ (y.wrapping_mul(0x9e3779b9));
            v ^= v >> 16;
            v = v.wrapping_mul(0x7feb352d);
            v ^= v >> 15;
            v = v.wrapping_mul(0x846ca68b);
            v ^= v >> 16;
            (v & 0xff) as u8
        }

        for layer in 0..2u32 {
            let mut rgba = vec![0u8; (w as usize) * (h as usize) * bytes_per_pixel];
            for yy in 0..h {
                for xx in 0..w {
                    let v = match layer {
                        0 => hash_noise_u8(xx, yy),
                        _ => bayer8x8(xx, yy).saturating_mul(4),
                    };
                    let i = (yy as usize) * bytes_per_row + (xx as usize) * bytes_per_pixel;
                    rgba[i] = v;
                    rgba[i + 1] = v;
                    rgba[i + 2] = v;
                    rgba[i + 3] = 255;
                }
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.material_catalog_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.material_catalog_uploaded = true;
    }

    pub(super) fn ensure_custom_effect_input_fallback_uploaded(&mut self, queue: &wgpu::Queue) {
        if self.custom_effect_input_fallback_uploaded {
            return;
        }

        // Use a 1x1 `Rgba8Unorm` texture filled with zeros as the deterministic fallback for
        // missing/disabled CustomV2 input images.
        let bytes_per_row = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let bytes = vec![0u8; bytes_per_row as usize];

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.custom_effect_input_fallback_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.custom_effect_input_fallback_uploaded = true;
    }
}
