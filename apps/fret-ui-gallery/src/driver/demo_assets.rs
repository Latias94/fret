use super::*;
use fret::assets::{AssetBundleId, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry};

impl UiGalleryDriver {
    pub(crate) const AVATAR_DEMO_IMAGE_WIDTH: u32 = 96;
    pub(crate) const AVATAR_DEMO_IMAGE_HEIGHT: u32 = 96;
    pub(crate) const AVATAR_DEMO_IMAGE_RETRY_MAX: u8 = 8;

    pub(crate) const IMAGE_FIT_DEMO_WIDE_SIZE: (u32, u32) = (320, 180);
    pub(crate) const IMAGE_FIT_DEMO_TALL_SIZE: (u32, u32) = (180, 320);

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
}

pub(crate) const UI_GALLERY_DEMO_ASSET_BUNDLE_NAME: &str = "ui-gallery-demo-assets";
pub(crate) const UI_GALLERY_CARD_EVENT_COVER_KEY: &str = "card/event-cover.jpg";

const UI_GALLERY_CARD_EVENT_COVER_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/test.jpg");
const UI_GALLERY_DEMO_ASSET_ENTRIES: [StaticAssetEntry; 1] = [StaticAssetEntry::new(
    UI_GALLERY_CARD_EVENT_COVER_KEY,
    AssetRevision(1),
    UI_GALLERY_CARD_EVENT_COVER_BYTES,
)
.with_media_type("image/jpeg")];

pub(crate) fn ui_gallery_demo_asset_bundle() -> AssetBundleId {
    AssetBundleId::package(UI_GALLERY_DEMO_ASSET_BUNDLE_NAME)
}

pub(crate) fn ui_gallery_card_event_cover_request() -> AssetRequest {
    AssetRequest::new(AssetLocator::bundle(
        ui_gallery_demo_asset_bundle(),
        UI_GALLERY_CARD_EVENT_COVER_KEY,
    ))
}

pub(crate) fn install_gallery_demo_asset_bundle(app: &mut App) {
    fret::assets::register_bundle_entries(
        app,
        ui_gallery_demo_asset_bundle(),
        UI_GALLERY_DEMO_ASSET_ENTRIES,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_gallery_demo_asset_bundle_registers_card_cover_locator() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let resolved =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_card_event_cover_request())
                .expect("expected UI Gallery demo asset bundle to resolve");
        assert_eq!(
            resolved.locator,
            ui_gallery_card_event_cover_request().locator
        );
        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(resolved.bytes.as_ref(), UI_GALLERY_CARD_EVENT_COVER_BYTES);
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );
    }
}
