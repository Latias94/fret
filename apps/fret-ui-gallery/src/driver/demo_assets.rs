use fret::assets::{AssetBundleId, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry};
use fret_app::App;

pub(crate) const UI_GALLERY_DEMO_ASSET_BUNDLE_NAME: &str = "ui-gallery-demo-assets";
pub(crate) const UI_GALLERY_SHARED_MEDIA_PREVIEW_KEY: &str = "shared/media-preview.jpg";
pub(crate) const UI_GALLERY_CARD_EVENT_COVER_KEY: &str = "card/event-cover.jpg";
pub(crate) const UI_GALLERY_ASPECT_RATIO_LANDSCAPE_KEY: &str = "aspect-ratio/landscape.jpg";
pub(crate) const UI_GALLERY_ASPECT_RATIO_PORTRAIT_KEY: &str = "aspect-ratio/portrait.jpg";
pub(crate) const UI_GALLERY_PROFILE_SQUARE_KEY: &str = "shared/profile-square.jpg";
pub(crate) const UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_KEY: &str = "image-object-fit/sampling.png";
#[cfg(any(test, feature = "gallery-dev"))]
pub(crate) const UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_KEY: &str = "ai/attachments/landscape.jpg";
#[cfg(any(test, feature = "gallery-dev"))]
pub(crate) const UI_GALLERY_AI_ATTACHMENT_PORTRAIT_KEY: &str = "ai/attachments/portrait.jpg";

const UI_GALLERY_SHARED_MEDIA_PREVIEW_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/test-1.jpg");
const UI_GALLERY_CARD_EVENT_COVER_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/test.jpg");
const UI_GALLERY_ASPECT_RATIO_LANDSCAPE_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/aspect-ratio-landscape.jpg");
const UI_GALLERY_ASPECT_RATIO_PORTRAIT_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/aspect-ratio-portrait.jpg");
const UI_GALLERY_PROFILE_SQUARE_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/avatar-square.jpg");
const UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/image-object-fit-sampling.png");
#[cfg(any(test, feature = "gallery-dev"))]
const UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/aspect-ratio-landscape.jpg");
#[cfg(any(test, feature = "gallery-dev"))]
const UI_GALLERY_AI_ATTACHMENT_PORTRAIT_BYTES: &[u8] =
    include_bytes!("../../../../assets/textures/aspect-ratio-portrait.jpg");
#[cfg(not(any(test, feature = "gallery-dev")))]
const UI_GALLERY_DEMO_ASSET_ENTRIES: [StaticAssetEntry; 6] = [
    StaticAssetEntry::new(
        UI_GALLERY_SHARED_MEDIA_PREVIEW_KEY,
        AssetRevision(1),
        UI_GALLERY_SHARED_MEDIA_PREVIEW_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_CARD_EVENT_COVER_KEY,
        AssetRevision(1),
        UI_GALLERY_CARD_EVENT_COVER_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_ASPECT_RATIO_LANDSCAPE_KEY,
        AssetRevision(1),
        UI_GALLERY_ASPECT_RATIO_LANDSCAPE_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_ASPECT_RATIO_PORTRAIT_KEY,
        AssetRevision(1),
        UI_GALLERY_ASPECT_RATIO_PORTRAIT_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_PROFILE_SQUARE_KEY,
        AssetRevision(1),
        UI_GALLERY_PROFILE_SQUARE_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_KEY,
        AssetRevision(1),
        UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_BYTES,
    )
    .with_media_type("image/png"),
];
#[cfg(any(test, feature = "gallery-dev"))]
const UI_GALLERY_DEMO_ASSET_ENTRIES: [StaticAssetEntry; 8] = [
    StaticAssetEntry::new(
        UI_GALLERY_SHARED_MEDIA_PREVIEW_KEY,
        AssetRevision(1),
        UI_GALLERY_SHARED_MEDIA_PREVIEW_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_CARD_EVENT_COVER_KEY,
        AssetRevision(1),
        UI_GALLERY_CARD_EVENT_COVER_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_ASPECT_RATIO_LANDSCAPE_KEY,
        AssetRevision(1),
        UI_GALLERY_ASPECT_RATIO_LANDSCAPE_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_ASPECT_RATIO_PORTRAIT_KEY,
        AssetRevision(1),
        UI_GALLERY_ASPECT_RATIO_PORTRAIT_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_PROFILE_SQUARE_KEY,
        AssetRevision(1),
        UI_GALLERY_PROFILE_SQUARE_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_KEY,
        AssetRevision(1),
        UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_BYTES,
    )
    .with_media_type("image/png"),
    StaticAssetEntry::new(
        UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_KEY,
        AssetRevision(1),
        UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_BYTES,
    )
    .with_media_type("image/jpeg"),
    StaticAssetEntry::new(
        UI_GALLERY_AI_ATTACHMENT_PORTRAIT_KEY,
        AssetRevision(1),
        UI_GALLERY_AI_ATTACHMENT_PORTRAIT_BYTES,
    )
    .with_media_type("image/jpeg"),
];

pub(crate) fn ui_gallery_demo_asset_bundle() -> AssetBundleId {
    AssetBundleId::package(UI_GALLERY_DEMO_ASSET_BUNDLE_NAME)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_shared_media_preview_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_SHARED_MEDIA_PREVIEW_KEY)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_card_event_cover_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_CARD_EVENT_COVER_KEY)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_aspect_ratio_landscape_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_ASPECT_RATIO_LANDSCAPE_KEY)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_aspect_ratio_portrait_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_ASPECT_RATIO_PORTRAIT_KEY)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_profile_square_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_PROFILE_SQUARE_KEY)
}

#[allow(dead_code)]
pub(crate) fn ui_gallery_image_object_fit_sampling_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_KEY)
}

#[cfg(any(test, feature = "gallery-dev"))]
#[allow(dead_code)]
pub(crate) fn ui_gallery_ai_attachment_landscape_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_KEY)
}

#[cfg(any(test, feature = "gallery-dev"))]
#[allow(dead_code)]
pub(crate) fn ui_gallery_ai_attachment_portrait_request() -> AssetRequest {
    ui_gallery_demo_bundle_request(UI_GALLERY_AI_ATTACHMENT_PORTRAIT_KEY)
}

#[allow(dead_code)]
fn ui_gallery_demo_bundle_request(key: &'static str) -> AssetRequest {
    AssetRequest::new(AssetLocator::bundle(ui_gallery_demo_asset_bundle(), key))
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
    fn install_gallery_demo_asset_bundle_registers_shared_media_preview_locator() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let resolved =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_shared_media_preview_request())
                .expect("expected UI Gallery shared media preview asset to resolve");
        assert_eq!(
            resolved.locator,
            ui_gallery_shared_media_preview_request().locator
        );
        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(
            resolved.bytes.as_ref(),
            UI_GALLERY_SHARED_MEDIA_PREVIEW_BYTES
        );
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );
    }

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

    #[test]
    fn install_gallery_demo_asset_bundle_registers_aspect_ratio_preview_locators() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let landscape =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_aspect_ratio_landscape_request())
                .expect("expected Aspect Ratio landscape preview asset to resolve");
        assert_eq!(
            landscape.locator,
            ui_gallery_aspect_ratio_landscape_request().locator
        );
        assert_eq!(landscape.revision, AssetRevision(1));
        assert_eq!(
            landscape.bytes.as_ref(),
            UI_GALLERY_ASPECT_RATIO_LANDSCAPE_BYTES
        );
        assert_eq!(
            landscape
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );

        let portrait =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_aspect_ratio_portrait_request())
                .expect("expected Aspect Ratio portrait preview asset to resolve");
        assert_eq!(
            portrait.locator,
            ui_gallery_aspect_ratio_portrait_request().locator
        );
        assert_eq!(portrait.revision, AssetRevision(1));
        assert_eq!(
            portrait.bytes.as_ref(),
            UI_GALLERY_ASPECT_RATIO_PORTRAIT_BYTES
        );
        assert_eq!(
            portrait
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );
    }

    #[test]
    fn install_gallery_demo_asset_bundle_registers_profile_square_locator() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let resolved = fret_runtime::resolve_asset_bytes(&app, &ui_gallery_profile_square_request())
            .expect("expected profile square preview asset to resolve");
        assert_eq!(resolved.locator, ui_gallery_profile_square_request().locator);
        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(resolved.bytes.as_ref(), UI_GALLERY_PROFILE_SQUARE_BYTES);
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );
    }

    #[test]
    fn install_gallery_demo_asset_bundle_registers_sampling_locator() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let resolved = fret_runtime::resolve_asset_bytes(
            &app,
            &ui_gallery_image_object_fit_sampling_request(),
        )
        .expect("expected image object-fit sampling asset to resolve");
        assert_eq!(
            resolved.locator,
            ui_gallery_image_object_fit_sampling_request().locator
        );
        assert_eq!(resolved.revision, AssetRevision(1));
        assert_eq!(resolved.bytes.as_ref(), UI_GALLERY_IMAGE_OBJECT_FIT_SAMPLING_BYTES);
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/png")
        );
    }

    #[test]
    fn install_gallery_demo_asset_bundle_registers_ai_attachment_preview_locators() {
        let mut app = App::new();

        install_gallery_demo_asset_bundle(&mut app);

        let landscape =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_ai_attachment_landscape_request())
                .expect("expected landscape attachment preview asset to resolve");
        assert_eq!(
            landscape.locator,
            ui_gallery_ai_attachment_landscape_request().locator
        );
        assert_eq!(landscape.revision, AssetRevision(1));
        assert_eq!(
            landscape.bytes.as_ref(),
            UI_GALLERY_AI_ATTACHMENT_LANDSCAPE_BYTES
        );
        assert_eq!(
            landscape
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );

        let portrait =
            fret_runtime::resolve_asset_bytes(&app, &ui_gallery_ai_attachment_portrait_request())
                .expect("expected portrait attachment preview asset to resolve");
        assert_eq!(
            portrait.locator,
            ui_gallery_ai_attachment_portrait_request().locator
        );
        assert_eq!(portrait.revision, AssetRevision(1));
        assert_eq!(
            portrait.bytes.as_ref(),
            UI_GALLERY_AI_ATTACHMENT_PORTRAIT_BYTES
        );
        assert_eq!(
            portrait
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some("image/jpeg")
        );
    }
}
