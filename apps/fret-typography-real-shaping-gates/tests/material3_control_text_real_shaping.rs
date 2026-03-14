use fret_app::App;
use fret_core::{Px, TextConstraints, TextInputRef, TextOverflow, TextWrap};
use fret_render_text::ParleyShaper;
use fret_ui::Theme;
use fret_ui_material3::tokens::v30::{
    ColorSchemeOptions, TypographyOptions, theme_config_with_colors,
};

fn shaper_with_bundled_fonts() -> ParleyShaper {
    let mut shaper = ParleyShaper::new_without_system_fonts();
    let added = shaper.add_fonts(
        fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::emoji_fonts().iter())
            .chain(fret_fonts::cjk_lite_fonts().iter())
            .map(|b| b.to_vec()),
    );
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
fn material3_control_text_keeps_metrics_stable_across_fallback_runs() {
    let cfg = theme_config_with_colors(TypographyOptions::default(), ColorSchemeOptions::default());
    let mut app = App::default();
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
    let theme = Theme::global(&app).clone();

    let style = fret_ui_material3::__testing::search_bar_input_text_style(&theme);
    assert!(
        style.line_height.is_some() || style.line_height_em.is_some(),
        "expected Material3 control text styles to provide an explicit line height"
    );

    let mut shaper = shaper_with_bundled_fonts();
    let constraints = constraints_for_single_line();
    let scale = fret_render_text::effective_text_scale_factor(constraints.scale_factor);

    let expected_line_height = style
        .line_height
        .unwrap_or_else(|| Px(style.size.0 * style.line_height_em.unwrap_or(1.2)));

    let mut baseline_for = |text: &str| {
        let input = TextInputRef::plain(text, &style);
        let wrapped = fret_render_text::wrap_with_constraints(&mut shaper, input, constraints);
        let prepared =
            fret_render_text::prepare_layout_from_wrapped(text, wrapped, constraints, scale, true);

        assert_eq!(
            prepared.metrics.size.height, expected_line_height,
            "expected fixed line boxes to keep height stable: text={text:?}, metrics={:?}",
            prepared.metrics
        );
        assert_eq!(
            prepared.lines[0].layout.height, expected_line_height,
            "expected first line height to match fixed line box: text={text:?}, line={:?}",
            prepared.lines[0].layout
        );

        prepared.metrics.baseline
    };

    let baseline_ascii = baseline_for("Settings");
    let baseline_emoji = baseline_for("Settings 😄");
    let baseline_cjk = baseline_for("Settings 漢字");
    let baseline_mixed = baseline_for("Settings 😄 漢字");

    assert_eq!(baseline_ascii, baseline_emoji);
    assert_eq!(baseline_ascii, baseline_cjk);
    assert_eq!(baseline_ascii, baseline_mixed);
}
