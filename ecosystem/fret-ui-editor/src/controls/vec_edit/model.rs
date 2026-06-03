//! VecEdit public model owner.

mod keying;
mod vec2;
mod vec3;
mod vec4;

pub use vec2::Vec2Edit;
pub use vec3::Vec3Edit;
pub use vec4::Vec4Edit;

#[cfg(test)]
mod tests {
    use super::Vec3Edit;
    use crate::primitives::NumericPresentation;
    use fret_app::App;
    use std::sync::Arc;

    #[test]
    fn vec3_edit_from_presentation_adopts_format_parse_and_chrome_affixes() {
        let mut app = App::new();
        let x = app.models_mut().insert(1.0f64);
        let y = app.models_mut().insert(2.0f64);
        let z = app.models_mut().insert(3.0f64);
        let presentation = NumericPresentation::<f64>::fixed_decimals(2)
            .with_chrome_prefix("$")
            .with_chrome_suffix("ms");

        let edit = Vec3Edit::from_presentation(x, y, z, presentation);

        assert_eq!((edit.format)(1.25).as_ref(), "1.25");
        assert_eq!((edit.parse)("1.25"), Some(1.25));
        assert_eq!(edit.options.prefix, Some(Arc::from("$")));
        assert_eq!(edit.options.suffix, Some(Arc::from("ms")));
    }
}
