use crate::parley_shaper::ParleyShaper;
use crate::wrapper::wrap_word_measure_only;
use fret_core::TextInputRef;

pub(crate) fn balanced_word_wrap_width_px(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> f32 {
    if !(max_width_px.is_finite() && max_width_px > 0.0) {
        return max_width_px;
    }

    let baseline = wrap_word_measure_only(shaper, input, text_len, max_width_px, scale);
    let line_count = baseline.lines().len();
    if line_count <= 1 {
        return max_width_px;
    }

    let single_line = shaper.shape_single_line_metrics(input, scale);
    let unwrapped_width_px = single_line.width().max(0.0);
    if !(unwrapped_width_px.is_finite() && unwrapped_width_px > 0.0) {
        return max_width_px;
    }

    let target = (unwrapped_width_px / (line_count as f32))
        .clamp(0.0, max_width_px)
        .max(0.0);
    if !(target.is_finite() && target > 0.0) {
        return max_width_px;
    }

    let mut lo = target;
    let mut hi = max_width_px;

    if wrap_word_measure_only(shaper, input, text_len, hi, scale)
        .lines()
        .len()
        > line_count
    {
        return max_width_px;
    }

    for _ in 0..12 {
        let mid = (lo + hi) * 0.5;
        if !mid.is_finite() || (hi - lo).abs() <= 0.5 {
            break;
        }

        let mid_lines = wrap_word_measure_only(shaper, input, text_len, mid, scale)
            .lines()
            .len();
        if mid_lines <= line_count {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    hi.clamp(0.0, max_width_px)
}
