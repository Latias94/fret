use fret_core::{Px, TextStyle};
use fret_ui::Theme;

fn lerp_px(a: Px, b: Px, t: f32) -> Px {
    let t = t.clamp(0.0, 1.0);
    Px(a.0 + (b.0 - a.0) * t)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

pub fn is_floated(progress: f32) -> bool {
    progress >= 0.5
}

pub fn material_floating_label_offsets(progress: f32) -> (Px, Px) {
    let y = lerp_px(Px(18.0), Px(6.0), progress);
    let x = Px(16.0);
    (x, y)
}

pub fn interpolated_label_text_style(
    theme: &Theme,
    progress: f32,
    large_key: &'static str,
    small_key: &'static str,
) -> Option<TextStyle> {
    let large = theme.text_style_by_key(large_key)?;
    let small = theme.text_style_by_key(small_key)?;

    if large.font != small.font || large.weight != small.weight || large.slant != small.slant {
        return Some(if is_floated(progress) { small } else { large });
    }

    let size = lerp_px(large.size, small.size, progress);
    let line_height = match (large.line_height, small.line_height) {
        (Some(a), Some(b)) => Some(lerp_px(a, b, progress)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let letter_spacing_em = match (large.letter_spacing_em, small.letter_spacing_em) {
        (Some(a), Some(b)) => Some(lerp_f32(a, b, progress)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    Some(TextStyle {
        font: large.font,
        size,
        weight: large.weight,
        slant: large.slant,
        line_height,
        letter_spacing_em,
    })
}

pub fn material_floating_label_text_style(theme: &Theme, progress: f32) -> Option<TextStyle> {
    interpolated_label_text_style(
        theme,
        progress,
        "md.sys.typescale.body-large",
        "md.sys.typescale.body-small",
    )
}
