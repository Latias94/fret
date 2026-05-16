#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_core::scene::{ImageSamplingHint, UvRect};
use fret_core::{ImageId, Px, Size, ViewportFit};
use fret_ui::UiHost;
use fret_ui_kit::imui::{ImageItemOptions, ImageItemVariant, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn image_item_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let image = ImageId::default();
    let size = Size::new(Px(48.0), Px(32.0));

    let plain = ui.image_item("image.preview", image, size);
    let _ = plain.hovered();

    let configured = ui.image_item_with_options(
        "image.configured",
        image,
        size,
        ImageItemOptions {
            fit: ViewportFit::Contain,
            sampling: ImageSamplingHint::Nearest,
            opacity: 0.75,
            uv: Some(UvRect::FULL),
            a11y_label: Some(Arc::from("Preview texture")),
            test_id: Some(Arc::from("imui-image.preview")),
            ..Default::default()
        },
    );
    let _ = configured.id();

    let button = ui.image_button("image.button", image, size);
    let _ = button.clicked();

    let button_with_options = ui.image_button_with_options(
        "image.button.options",
        image,
        size,
        ImageItemOptions::button()
            .fit(ViewportFit::Cover)
            .sampling(ImageSamplingHint::Linear)
            .opacity(0.6)
            .with_a11y_label("Open texture")
            .with_test_id("imui-image.button"),
    );
    let _ = button_with_options.context_menu_requested();
}

#[test]
fn image_item_option_defaults_are_plain_non_focusable_images() {
    let options = ImageItemOptions::default();
    assert!(options.enabled);
    assert!(!options.focusable);
    assert_eq!(options.variant, ImageItemVariant::Image);
    assert_eq!(options.fit, ViewportFit::Stretch);
    assert_eq!(options.sampling, ImageSamplingHint::Default);
    assert_eq!(options.opacity, 1.0);
    assert!(options.uv.is_none());
    assert!(options.a11y_label.is_none());
    assert!(options.test_id.is_none());
}

#[test]
fn image_item_button_defaults_are_focusable_buttons() {
    let options = ImageItemOptions::button();
    assert!(options.enabled);
    assert!(options.focusable);
    assert_eq!(options.variant, ImageItemVariant::Button);
}
