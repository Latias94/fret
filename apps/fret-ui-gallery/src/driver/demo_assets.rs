use super::*;

#[derive(Debug, Clone)]
pub(crate) struct UiGalleryImageSourceDemoAssets {
    pub wide_png: fret_ui_assets::ImageSource,
    pub tall_png: fret_ui_assets::ImageSource,
    pub square_png: fret_ui_assets::ImageSource,
    pub pixel_png: fret_ui_assets::ImageSource,
}

impl UiGalleryDriver {
    pub(crate) const AVATAR_DEMO_IMAGE_WIDTH: u32 = 96;
    pub(crate) const AVATAR_DEMO_IMAGE_HEIGHT: u32 = 96;
    pub(crate) const AVATAR_DEMO_IMAGE_RETRY_MAX: u8 = 8;

    pub(crate) const IMAGE_FIT_DEMO_WIDE_SIZE: (u32, u32) = (320, 180);
    pub(crate) const IMAGE_FIT_DEMO_TALL_SIZE: (u32, u32) = (180, 320);
    pub(crate) const IMAGE_FIT_DEMO_STREAMING_SIZE: (u32, u32) = (320, 200);
    pub(crate) const IMAGE_SAMPLING_DEMO_PIXEL_SIZE: (u32, u32) = (16, 16);

    pub(crate) fn ensure_image_source_demo_assets_installed(app: &mut App) {
        if app.global::<UiGalleryImageSourceDemoAssets>().is_some() {
            return;
        }

        // Encode a few tiny demo images to PNG and load them through the ecosystem `ImageSource`
        // path. This exercises the decode/load + `ImageAssetCache` integration without requiring
        // external files.
        let wide_rgba = Self::generate_fit_demo_image_rgba8(
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.0,
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.1,
            (120, 190, 255),
        );
        let tall_rgba = Self::generate_fit_demo_image_rgba8(
            Self::IMAGE_FIT_DEMO_TALL_SIZE.0,
            Self::IMAGE_FIT_DEMO_TALL_SIZE.1,
            (255, 160, 120),
        );
        let square_rgba = Self::generate_avatar_demo_image_rgba8(
            Self::AVATAR_DEMO_IMAGE_WIDTH,
            Self::AVATAR_DEMO_IMAGE_HEIGHT,
        );
        let pixel_rgba = Self::generate_pixel_demo_image_rgba8(
            Self::IMAGE_SAMPLING_DEMO_PIXEL_SIZE.0,
            Self::IMAGE_SAMPLING_DEMO_PIXEL_SIZE.1,
        );

        let wide_png = Self::encode_rgba8_png_bytes(
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.0,
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.1,
            &wide_rgba,
        );
        let tall_png = Self::encode_rgba8_png_bytes(
            Self::IMAGE_FIT_DEMO_TALL_SIZE.0,
            Self::IMAGE_FIT_DEMO_TALL_SIZE.1,
            &tall_rgba,
        );
        let square_png = Self::encode_rgba8_png_bytes(
            Self::AVATAR_DEMO_IMAGE_WIDTH,
            Self::AVATAR_DEMO_IMAGE_HEIGHT,
            &square_rgba,
        );
        let pixel_png = Self::encode_rgba8_png_bytes(
            Self::IMAGE_SAMPLING_DEMO_PIXEL_SIZE.0,
            Self::IMAGE_SAMPLING_DEMO_PIXEL_SIZE.1,
            &pixel_rgba,
        );

        app.set_global(UiGalleryImageSourceDemoAssets {
            wide_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(wide_png)),
            tall_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(tall_png)),
            square_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(square_png)),
            pixel_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(pixel_png)),
        });
    }

    pub(crate) fn enqueue_avatar_demo_image_register(
        app: &mut App,
        window: AppWindowId,
        token: ImageUploadToken,
    ) {
        app.push_effect(Effect::ImageRegisterRgba8 {
            window,
            token,
            width: Self::AVATAR_DEMO_IMAGE_WIDTH,
            height: Self::AVATAR_DEMO_IMAGE_HEIGHT,
            bytes: Self::generate_avatar_demo_image_rgba8(
                Self::AVATAR_DEMO_IMAGE_WIDTH,
                Self::AVATAR_DEMO_IMAGE_HEIGHT,
            ),
            color_info: ImageColorInfo::srgb_rgba(),
            alpha_mode: AlphaMode::Opaque,
        });
    }

    pub(crate) fn enqueue_image_fit_demo_image_register(
        app: &mut App,
        window: AppWindowId,
        token: ImageUploadToken,
        size: (u32, u32),
        accent: (u8, u8, u8),
    ) {
        app.push_effect(Effect::ImageRegisterRgba8 {
            window,
            token,
            width: size.0,
            height: size.1,
            bytes: Self::generate_fit_demo_image_rgba8(size.0, size.1, accent),
            color_info: ImageColorInfo::srgb_rgba(),
            alpha_mode: AlphaMode::Opaque,
        });
    }

    fn generate_avatar_demo_image_rgba8(width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        let w = (width.saturating_sub(1)).max(1) as f32;
        let h = (height.saturating_sub(1)).max(1) as f32;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let fx = x as f32 / w;
                let fy = y as f32 / h;

                let cx = fx - 0.5;
                let cy = fy - 0.5;
                let d = (cx * cx + cy * cy).sqrt().min(1.0);
                let highlight = (1.0 - d).powf(1.6);

                let r = (40.0 + 140.0 * fx + 60.0 * highlight).min(255.0) as u8;
                let g = (55.0 + 110.0 * (1.0 - fy) + 70.0 * highlight).min(255.0) as u8;
                let b = (90.0 + 110.0 * (0.5 + 0.5 * (fx - fy)).abs() + 80.0 * highlight).min(255.0)
                    as u8;

                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }

        out
    }

    fn generate_fit_demo_image_rgba8(width: u32, height: u32, accent: (u8, u8, u8)) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        let w = (width.saturating_sub(1)).max(1) as f32;
        let h = (height.saturating_sub(1)).max(1) as f32;

        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let fx = x as f32 / w;
                let fy = y as f32 / h;

                let mut r = (20.0 + (accent.0 as f32) * (0.25 + 0.75 * fx)) as u8;
                let mut g = (20.0 + (accent.1 as f32) * (0.25 + 0.75 * (1.0 - fy))) as u8;
                let mut b =
                    (20.0 + (accent.2 as f32) * (0.25 + 0.75 * (0.5 + 0.5 * (fx - fy)))) as u8;

                let border = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;
                if border {
                    r = 245;
                    g = 245;
                    b = 245;
                }

                let dx = (x as i32 - cx).abs();
                let dy = (y as i32 - cy).abs();
                if dx <= 1 || dy <= 1 {
                    r = 10;
                    g = 10;
                    b = 10;
                }

                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }

        out
    }

    fn generate_pixel_demo_image_rgba8(width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;

                let border = x == 0 || y == 0 || x + 1 == width || y + 1 == height;
                let checker = ((x ^ y) & 1) == 0;

                let (mut r, mut g, mut b) = if border {
                    (10u8, 10u8, 10u8)
                } else if x == y || x + y + 1 == width {
                    (245u8, 50u8, 50u8)
                } else if checker {
                    (255u8, 255u8, 255u8)
                } else {
                    (35u8, 35u8, 35u8)
                };

                if x >= width / 2 && y < height / 2 {
                    r = r.saturating_add(0);
                    g = g.saturating_add(80);
                    b = b.saturating_add(0);
                }

                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }

        out
    }

    fn encode_rgba8_png_bytes(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::{ColorType, ImageEncoder as _};

        let mut out: Vec<u8> = Vec::new();
        PngEncoder::new(&mut out)
            .write_image(rgba, width, height, ColorType::Rgba8.into())
            .expect("png encode must succeed for demo bytes");
        out
    }
}
