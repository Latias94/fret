use fret_core::ImageId;
use fret_core::scene::ImageSamplingHint;
use fret_ui::element::ImageProps;
use fret_ui_kit::ImageSamplingExt;

#[test]
fn image_sampling_ext_sets_sampling_hint() {
    let props = ImageProps::new(ImageId::default()).nearest();
    assert_eq!(props.sampling, ImageSamplingHint::Nearest);
}
