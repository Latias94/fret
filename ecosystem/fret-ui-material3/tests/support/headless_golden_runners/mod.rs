pub(crate) mod carousel_item;
pub(crate) mod list;
pub(crate) mod navigation;
pub(crate) mod overlays;
pub(crate) mod slider;
pub(crate) mod text_field;

pub(crate) fn scale_segment(scale_factor: f32) -> &'static str {
    if (scale_factor - 1.0).abs() < 1e-6 {
        "scale1_0"
    } else if (scale_factor - 1.25).abs() < 1e-6 {
        "scale1_25"
    } else if (scale_factor - 2.0).abs() < 1e-6 {
        "scale2_0"
    } else {
        panic!("unsupported scale factor: {scale_factor}");
    }
}
