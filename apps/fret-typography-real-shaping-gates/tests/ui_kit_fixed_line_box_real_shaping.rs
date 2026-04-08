use fret_core::{FontId, Px, TextConstraints, TextInputRef, TextOverflow, TextWrap};
use fret_render_text::ParleyShaper;

fn shaper_with_bundled_fonts() -> ParleyShaper {
    let mut shaper = ParleyShaper::new_without_system_fonts();
    let added = shaper.add_fonts(fret_fonts::test_support::face_blobs(
        fret_fonts::bootstrap_profile()
            .faces
            .iter()
            .chain(fret_fonts_emoji::default_profile().faces.iter())
            .chain(fret_fonts_cjk::default_profile().faces.iter()),
    ));
    assert!(added > 0, "expected bundled fonts to load");
    shaper
}

fn constraints_for_single_line() -> TextConstraints {
    TextConstraints {
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        ..Default::default()
    }
}

#[test]
fn fixed_line_box_keeps_metrics_height_and_baseline_stable_across_fallback_runs() {
    let mut shaper = shaper_with_bundled_fonts();

    let size = Px(13.0);
    let line_height = Px(20.0);
    let style = fret_ui_kit::typography::fixed_line_box_style(FontId::ui(), size, line_height);
    let constraints = constraints_for_single_line();
    let scale = fret_render_text::effective_text_scale_factor(constraints.scale_factor);

    let mut baseline_for = |text: &str| {
        let input = TextInputRef::plain(text, &style);
        let wrapped = fret_render_text::wrap_with_constraints(&mut shaper, input, constraints);
        let prepared =
            fret_render_text::prepare_layout_from_wrapped(text, wrapped, constraints, scale, true);

        assert_eq!(
            prepared.metrics().size.height,
            line_height,
            "expected fixed line boxes to keep height stable: text={text:?}, metrics={:?}",
            prepared.metrics()
        );
        assert_eq!(
            prepared.lines()[0].layout().height(),
            line_height,
            "expected first line height to match fixed line box: text={text:?}, line={:?}",
            prepared.lines()[0].layout()
        );

        prepared.metrics().baseline
    };

    let baseline_ascii = baseline_for("Settings");
    let baseline_emoji = baseline_for("Settings 😄");
    let baseline_cjk = baseline_for("Settings 漢字");
    let baseline_mixed = baseline_for("Settings 😄 漢字");

    assert_eq!(baseline_ascii, baseline_emoji);
    assert_eq!(baseline_ascii, baseline_cjk);
    assert_eq!(baseline_ascii, baseline_mixed);
}
