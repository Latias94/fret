use super::{TextBlobKey, TextShapeKey, spans_paint_fingerprint, subpixel_mask_to_alpha};
use fret_core::{
    AttributedText, Color, DecorationLineStyle, FontWeight, Px, TextConstraints, TextOverflow,
    TextPaintStyle, TextShapingStyle, TextSpan, TextStyle, TextWrap, UnderlineStyle,
};
use fret_render_text::cache_keys::{TextMeasureKey, spans_shaping_fingerprint};
use fret_render_text::spans::{ResolvedSpan, paint_span_for_text_range, sanitize_spans_for_text};
use std::sync::Arc;

fn pending_upload_bytes_for_key(text: &super::TextSystem, key: super::GlyphKey) -> Vec<u8> {
    let atlas = text.atlas_runtime.atlas(key.kind);
    let entry = atlas.entry(key).expect("expected atlas entry after ensure");
    atlas
        .pending_upload_bytes_for_entry(entry)
        .expect("expected pending upload for ensured glyph")
}

fn reset_bundled_only_font_runtime(text: &mut super::TextSystem) {
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;
}

#[test]
fn subpixel_mask_to_alpha_uses_channel_max() {
    let data = vec![
        10u8, 3u8, 4u8, 0u8, //
        1u8, 200u8, 2u8, 0u8,
    ];
    assert_eq!(subpixel_mask_to_alpha(&data), vec![10u8, 200u8]);
}

#[test]
fn paint_span_for_text_range_is_directional_across_span_boundary() {
    let spans = vec![
        ResolvedSpan {
            start: 0,
            end: 3,
            slot: 0,
            fg: None,
            underline: None,
            strikethrough: None,
        },
        ResolvedSpan {
            start: 3,
            end: 6,
            slot: 1,
            fg: None,
            underline: None,
            strikethrough: None,
        },
    ];

    // Cluster spans the boundary (2..4). We cannot split the glyph, so we pick a deterministic
    // representative index based on direction.
    assert_eq!(paint_span_for_text_range(&spans, &(2..4), false), Some(0));
    assert_eq!(paint_span_for_text_range(&spans, &(2..4), true), Some(1));

    // Synthetic 0-length ranges (e.g. ellipsis mapping) should inherit the preceding style.
    assert_eq!(paint_span_for_text_range(&spans, &(3..3), false), Some(0));
    assert_eq!(paint_span_for_text_range(&spans, &(3..3), true), Some(0));
}

#[test]
fn all_font_names_is_sorted_and_deduped() {
    // This is intentionally platform-dependent; we only assert structural invariants.
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);
    let names = text.all_font_names();

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for name in &names {
        assert!(
            seen.insert(name.to_ascii_lowercase()),
            "expected case-insensitive dedupe for {name:?}"
        );
    }

    for w in names.windows(2) {
        assert!(
            w[0].to_ascii_lowercase() <= w[1].to_ascii_lowercase(),
            "expected case-insensitive sort"
        );
    }
}

#[test]
fn text_blob_key_includes_typography_fields() {
    let constraints = TextConstraints {
        max_width: Some(Px(120.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 2.0,
    };

    let base = TextStyle::default();
    let k0 = TextBlobKey::new("hello", &base, constraints, 1);

    let mut style = base.clone();
    style.weight = FontWeight::BOLD;
    let k_weight = TextBlobKey::new("hello", &style, constraints, 1);
    assert_ne!(k0, k_weight);

    let mut style = base.clone();
    style.line_height = Some(Px(18.0));
    let k_line_height = TextBlobKey::new("hello", &style, constraints, 1);
    assert_ne!(k0, k_line_height);

    let mut style = base.clone();
    style.letter_spacing_em = Some(0.05);
    let k_tracking = TextBlobKey::new("hello", &style, constraints, 1);
    assert_ne!(k0, k_tracking);
}

#[test]
fn text_blob_key_includes_font_fallback_policy() {
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let base = TextStyle::default();
    let k0 = TextBlobKey::new("hello", &base, constraints, 1);
    let k1 = TextBlobKey::new("hello", &base, constraints, 2);
    assert_ne!(k0, k1);
}

#[test]
fn text_locale_changes_font_stack_key() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let k0 = text.font_stack_key();
    assert!(text.set_text_locale(Some("en-US")));
    let k1 = text.font_stack_key();
    assert_ne!(k0, k1);

    assert!(!text.set_text_locale(Some("en-US")));
    assert_eq!(k1, text.font_stack_key());

    assert!(text.set_text_locale(Some("zh-CN")));
    let k2 = text.font_stack_key();
    assert_ne!(k1, k2);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn text_rescan_system_fonts_is_noop_when_environment_unchanged() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let k0 = text.font_stack_key();
    let font_db_revision0 = text.font_runtime.font_db_revision;
    let cache_resets0 = text.frame_perf.cache_resets;

    let seed = text
        .system_font_rescan_seed()
        .expect("expected system font rescan to be available");
    let result = seed.run();

    assert!(
        !text.apply_system_font_rescan_result(result),
        "expected apply to short-circuit when the environment is unchanged"
    );
    assert_eq!(
        text.font_stack_key(),
        k0,
        "expected no-op rescan apply to keep TextFontStackKey stable"
    );
    assert_eq!(
        text.font_runtime.font_db_revision, font_db_revision0,
        "expected no-op rescan apply to avoid bumping the font DB revision"
    );
    assert_eq!(
        text.frame_perf.cache_resets, cache_resets0,
        "expected no-op rescan apply to avoid cache-reset churn"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn text_rescan_system_fonts_noop_preserves_generic_injection() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let generic = parley::fontique::GenericFamily::UiSansSerif;
    let baseline = text.parley_shaper.generic_family_ids(generic);

    let names = text.all_font_names();
    let requested = names
        .iter()
        .take(1024)
        .find(|name| {
            text.parley_shaper
                .resolve_family_id(name)
                .is_some_and(|id| !baseline.contains(&id))
        })
        .cloned()
        .or_else(|| names.first().cloned())
        .expect("expected at least one system font family name");

    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![requested.clone()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let requested_id = text
        .parley_shaper
        .resolve_family_id(&requested)
        .expect("family id after config apply");
    assert_eq!(
        text.parley_shaper.generic_family_ids(generic).first(),
        Some(&requested_id),
        "expected UI sans stack to be injected with the configured family"
    );

    let k0 = text.font_stack_key();
    let font_db_revision0 = text.font_runtime.font_db_revision;
    let cache_resets0 = text.frame_perf.cache_resets;
    let seed = text
        .system_font_rescan_seed()
        .expect("expected system font rescan to be available");
    let result = seed.run();

    assert!(
        !text.apply_system_font_rescan_result(result),
        "expected apply to short-circuit when the environment is unchanged"
    );

    let requested_id_after = text
        .parley_shaper
        .resolve_family_id(&requested)
        .expect("family id after rescan");
    assert_eq!(
        text.parley_shaper.generic_family_ids(generic).first(),
        Some(&requested_id_after),
        "expected UI sans stack injection to remain intact across a no-op rescan apply"
    );
    assert_eq!(
        requested_id_after, requested_id,
        "expected no-op rescan apply to preserve resolved family identity"
    );
    assert_eq!(
        text.font_stack_key(),
        k0,
        "expected no-op rescan apply to keep TextFontStackKey stable"
    );
    assert_eq!(
        text.font_runtime.font_db_revision, font_db_revision0,
        "expected no-op rescan apply to avoid bumping the font DB revision"
    );
    assert_eq!(
        text.frame_perf.cache_resets, cache_resets0,
        "expected no-op rescan apply to avoid cache-reset churn"
    );
}

#[test]
fn text_measure_key_ignores_width_for_wrap_none() {
    let style = TextStyle::default();

    let a = TextConstraints {
        max_width: Some(Px(120.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let b = TextConstraints {
        max_width: Some(Px(320.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    assert_eq!(
        TextMeasureKey::new(&style, a, 7),
        TextMeasureKey::new(&style, b, 7)
    );
}

#[test]
fn text_measure_key_includes_width_for_wrap_word() {
    let style = TextStyle::default();

    let a = TextConstraints {
        max_width: Some(Px(120.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let b = TextConstraints {
        max_width: Some(Px(320.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    assert_ne!(
        TextMeasureKey::new(&style, a, 7),
        TextMeasureKey::new(&style, b, 7)
    );
}

#[test]
fn text_measure_key_includes_width_for_wrap_grapheme() {
    let style = TextStyle::default();

    let a = TextConstraints {
        max_width: Some(Px(120.0)),
        wrap: TextWrap::Grapheme,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let b = TextConstraints {
        max_width: Some(Px(320.0)),
        wrap: TextWrap::Grapheme,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    assert_ne!(
        TextMeasureKey::new(&style, a, 7),
        TextMeasureKey::new(&style, b, 7)
    );
}

#[test]
fn sanitize_spans_extends_missing_tail() {
    let text = "hello";
    let spans = vec![TextSpan {
        len: 2,
        paint: fret_core::TextPaintStyle {
            fg: Some(fret_core::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            ..Default::default()
        },
        ..Default::default()
    }];

    let sanitized = sanitize_spans_for_text(text, &spans).expect("sanitized spans");
    let rich = fret_core::AttributedText {
        text: Arc::<str>::from(text),
        spans: sanitized.clone(),
    };

    assert!(rich.is_valid());
    assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
    assert_eq!(sanitized.len(), 2);
    assert_eq!(sanitized[0].len, 2);
    assert!(sanitized[0].paint.fg.is_some());
    assert_eq!(sanitized[1].len, 3);
    assert_eq!(sanitized[1].paint.fg, None);
}

#[test]
fn sanitize_spans_truncates_overflowing_last_span() {
    let text = "hello";
    let spans = vec![TextSpan {
        len: 999,
        paint: fret_core::TextPaintStyle {
            fg: Some(fret_core::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            ..Default::default()
        },
        ..Default::default()
    }];

    let sanitized = sanitize_spans_for_text(text, &spans).expect("sanitized spans");
    let rich = fret_core::AttributedText {
        text: Arc::<str>::from(text),
        spans: sanitized.clone(),
    };

    assert!(rich.is_valid());
    assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
    assert_eq!(sanitized.len(), 1);
    assert_eq!(sanitized[0].len, text.len());
    assert!(sanitized[0].paint.fg.is_some());
}

#[test]
fn sanitize_spans_snaps_to_char_boundaries() {
    let text = "aé";
    assert_eq!(text.len(), 3);

    let spans = vec![
        TextSpan {
            len: 2,
            paint: fret_core::TextPaintStyle {
                fg: Some(fret_core::Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        TextSpan {
            len: 1,
            paint: fret_core::TextPaintStyle {
                fg: Some(fret_core::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    let sanitized = sanitize_spans_for_text(text, &spans).expect("sanitized spans");
    let rich = fret_core::AttributedText {
        text: Arc::<str>::from(text),
        spans: sanitized.clone(),
    };

    assert!(rich.is_valid());
    assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
    assert_eq!(sanitized.len(), 2);
    assert_eq!(sanitized[0].len, 1);
    assert_eq!(sanitized[1].len, 2);
    assert_eq!(sanitized[0].paint.fg, spans[0].paint.fg);
    assert_eq!(sanitized[1].paint.fg, spans[1].paint.fg);
}

#[test]
fn sanitize_spans_returns_none_for_noop_full_span() {
    let text = "hello";
    let spans = vec![TextSpan::new(text.len())];
    assert!(sanitize_spans_for_text(text, &spans).is_none());
}

#[test]
fn sanitize_spans_treats_axis_overrides_as_non_noop() {
    let text = "hello";
    let spans = vec![TextSpan {
        len: text.len(),
        shaping: TextShapingStyle::default().with_axis("wght", 300.0),
        paint: Default::default(),
    }];
    assert!(sanitize_spans_for_text(text, &spans).is_some());
}

#[test]
fn sanitize_spans_treats_feature_overrides_as_non_noop() {
    let text = "hello";
    let spans = vec![TextSpan {
        len: text.len(),
        shaping: TextShapingStyle::default().with_feature("calt", 0),
        paint: Default::default(),
    }];
    assert!(sanitize_spans_for_text(text, &spans).is_some());
}

#[test]
fn multiline_metrics_are_pixel_snapped_under_non_integer_scale_factor() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let content = {
        let mut out = String::new();
        for _ in 0..200 {
            out.push_str("The quick brown fox jumps over the lazy dog. ");
        }
        out
    };

    let scale_factor = 1.25_f32;
    let constraints = TextConstraints {
        max_width: Some(Px(120.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let style = TextStyle {
        font: fret_core::FontId::monospace(),
        size: Px(13.0),
        ..Default::default()
    };

    let (blob_id, metrics) = text.prepare(&content, &style, constraints);
    let blob = text.blob(blob_id).expect("prepared blob");
    let lines = blob.shape.lines.as_ref();
    assert!(lines.len() > 10, "expected multi-line layout");

    let is_pixel_aligned = |logical: Px| {
        let px = logical.0 * scale_factor;
        (px - px.round()).abs() < 1e-3
    };

    assert!(
        is_pixel_aligned(metrics.baseline),
        "expected baseline to align to device pixels under fractional scale"
    );

    let mut prev_y_px = -1.0_f32;
    for line in lines {
        let y_px = line.y_top.0 * scale_factor;
        let h_px = line.height.0 * scale_factor;
        let baseline_px = line.y_baseline.0 * scale_factor;
        assert!(
            (y_px - y_px.round()).abs() < 1e-3,
            "expected y_top to be pixel-aligned, got {y_px}"
        );
        assert!(
            (h_px - h_px.round()).abs() < 1e-3 && h_px > 0.0,
            "expected line height to be positive and pixel-aligned, got {h_px}"
        );
        assert!(
            y_px + 0.5 >= prev_y_px,
            "expected non-decreasing y_top across lines"
        );
        prev_y_px = y_px;
        assert!(
            (baseline_px - baseline_px.round()).abs() < 1e-3,
            "expected per-line baseline to be pixel-aligned, got {baseline_px}"
        );
    }
}

#[test]
fn wrapped_measure_matches_prepare_under_fractional_scale_factor() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let content =
        "This window starts on top of A's overlap target. Then click A's 'Activate' button.";
    let scale_factor = 1.5_f32;
    let constraints = TextConstraints {
        max_width: Some(Px(160.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let style = TextStyle {
        font: fret_core::FontId::monospace(),
        size: Px(13.0),
        ..Default::default()
    };

    let measured = text.measure(content, &style, constraints);
    let (_blob_id, prepared) = text.prepare(content, &style, constraints);

    let eps = 0.01_f32;
    assert!(
        (measured.size.width.0 - prepared.size.width.0).abs() <= eps,
        "expected measure width to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.size.height.0 - prepared.size.height.0).abs() <= eps,
        "expected measure height to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.baseline.0 - prepared.baseline.0).abs() <= eps,
        "expected measure baseline to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
}

#[test]
fn glyph_cache_key_tracks_scale_factor_below_one() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Keep this test deterministic: bundled fonts only (no system font discovery).
    reset_bundled_only_font_runtime(&mut text);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let content = "mmmm";
    let style = TextStyle {
        font: fret_core::FontId::family("Inter"),
        size: Px(16.0),
        ..Default::default()
    };

    let constraints_1x = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let constraints_half = TextConstraints {
        scale_factor: 0.5,
        ..constraints_1x
    };

    let (blob_a, _metrics_a) = text.prepare(content, &style, constraints_1x);
    let (blob_b, _metrics_b) = text.prepare(content, &style, constraints_half);

    let a = text.blob(blob_a).expect("prepared blob (scale=1.0)");
    let b = text.blob(blob_b).expect("prepared blob (scale=0.5)");

    let ga = a.shape.glyphs.first().expect("expected at least one glyph");
    let gb = b.shape.glyphs.first().expect("expected at least one glyph");

    let size_a = f32::from_bits(ga.key.size_bits);
    let size_b = f32::from_bits(gb.key.size_bits);

    let ratio = size_b / size_a.max(1.0);
    assert!(
        (ratio - 0.5).abs() <= 0.15,
        "expected glyph cache key font size to scale with constraints.scale_factor; size_a={size_a} size_b={size_b} ratio={ratio}"
    );

    text.release(blob_a);
    text.release(blob_b);
}

#[test]
fn grapheme_wrapped_measure_matches_prepare_under_fractional_scale_factor() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let content = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let scale_factor = 1.5_f32;
    let constraints = TextConstraints {
        max_width: Some(Px(60.0)),
        wrap: TextWrap::Grapheme,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let style = TextStyle {
        font: fret_core::FontId::monospace(),
        size: Px(13.0),
        ..Default::default()
    };

    let measured = text.measure(content, &style, constraints);
    let (_blob_id, prepared) = text.prepare(content, &style, constraints);

    let eps = 0.01_f32;
    assert!(
        (measured.size.width.0 - prepared.size.width.0).abs() <= eps,
        "expected measure width to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.size.height.0 - prepared.size.height.0).abs() <= eps,
        "expected measure height to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.baseline.0 - prepared.baseline.0).abs() <= eps,
        "expected measure baseline to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
}

#[test]
fn max_content_width_round_trip_does_not_force_wrapping_under_fractional_scale_factor() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let scale_factor = 1.5_f32;
    let base_constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let style = TextStyle {
        font: fret_core::FontId::default(),
        size: Px(20.0),
        weight: fret_core::FontWeight::SEMIBOLD,
        ..Default::default()
    };

    let contents = ["Demo", "Description", "Choice Card"];
    for content in contents {
        let max_content = text.measure(content, &style, base_constraints);
        assert!(
            max_content.size.width.0 > 0.0 && max_content.size.height.0 > 0.0,
            "expected non-empty metrics for {content:?}: {max_content:?}"
        );

        let tight_constraints = TextConstraints {
            max_width: Some(max_content.size.width),
            ..base_constraints
        };
        let measured_tight = text.measure(content, &style, tight_constraints);
        let (_blob, prepared_tight) = text.prepare(content, &style, tight_constraints);

        let eps = 0.01_f32;
        assert!(
            measured_tight.size.height.0 <= max_content.size.height.0 + eps,
            "expected max-content width round-trip not to introduce wrapping in measure (scale={scale_factor}); content={content:?} max_content={max_content:?} measured_tight={measured_tight:?}"
        );
        assert!(
            prepared_tight.size.height.0 <= max_content.size.height.0 + eps,
            "expected max-content width round-trip not to introduce wrapping in prepare (scale={scale_factor}); content={content:?} max_content={max_content:?} prepared_tight={prepared_tight:?}"
        );
    }
}

#[test]
fn grapheme_wrapped_measure_attributed_matches_prepare_under_fractional_scale_factor() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let content = Arc::<str>::from(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let split = content.len() / 2;
    assert!(content.is_char_boundary(split));

    let spans = vec![
        TextSpan {
            len: split,
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle {
                fg: Some(Color {
                    r: 0.9,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                }),
                ..Default::default()
            },
        },
        TextSpan {
            len: content.len() - split,
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle {
                underline: Some(UnderlineStyle {
                    color: None,
                    style: fret_core::DecorationLineStyle::Solid,
                }),
                ..Default::default()
            },
        },
    ];

    let rich = AttributedText::new(content, Arc::<[TextSpan]>::from(spans));

    let scale_factor = 1.5_f32;
    let constraints = TextConstraints {
        max_width: Some(Px(60.0)),
        wrap: TextWrap::Grapheme,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let style = TextStyle {
        font: fret_core::FontId::monospace(),
        size: Px(13.0),
        ..Default::default()
    };

    let measured = text.measure_attributed(&rich, &style, constraints);
    let (_blob_id, prepared) = text.prepare_attributed(&rich, &style, constraints);

    let eps = 0.01_f32;
    assert!(
        (measured.size.width.0 - prepared.size.width.0).abs() <= eps,
        "expected measure width to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.size.height.0 - prepared.size.height.0).abs() <= eps,
        "expected measure height to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
    assert!(
        (measured.baseline.0 - prepared.baseline.0).abs() <= eps,
        "expected measure baseline to match prepare (scale={scale_factor}), got measured={:?} prepared={:?}",
        measured,
        prepared
    );
}

#[test]
fn emoji_sequences_use_color_quads_when_color_font_is_available() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::emoji_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let style = TextStyle {
        font: fret_core::FontId::family("Noto Color Emoji"),
        size: Px(32.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let cases = [
        ("\u{1F600}", "single emoji"),
        ("\u{2708}\u{FE0F}", "vs16 emoji presentation"),
        ("1\u{FE0F}\u{20E3}", "keycap sequence"),
        ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
        (
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "zwj family sequence",
        ),
        (
            "\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}",
            "zwj rainbow flag sequence",
        ),
    ];

    for (text_str, label) in cases {
        let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");

        let mut color_glyphs: Vec<super::GlyphKey> = Vec::new();
        for g in blob.shape.glyphs.as_ref() {
            if matches!(g.kind(), super::GlyphQuadKind::Color) {
                color_glyphs.push(g.key);
            }
        }

        assert!(
            !color_glyphs.is_empty(),
            "expected at least one color glyph quad for {label} when Noto Color Emoji is present"
        );

        let epoch = 1;
        for key in color_glyphs {
            text.ensure_glyph_in_atlas(key, epoch);
            assert!(
                text.atlas_runtime.color_atlas.get(key, epoch).is_some(),
                "expected color glyph to be present in color atlas after ensure ({label})"
            );
        }
    }
}

#[test]
fn cjk_glyphs_populate_mask_or_subpixel_atlas_when_cjk_lite_font_is_available() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::cjk_lite_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family = "Noto Sans CJK SC";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family)),
        "expected {family} to be present after loading cjk-lite fonts"
    );

    let style = TextStyle {
        font: fret_core::FontId::family(family),
        size: Px(24.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: Some(Px(360.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let cases = [
        ("你好，世界！", "basic"),
        ("这是一段用于验证换行与标点处理的文本。", "wrapping"),
        ("数字 12345 与符号（）《》“”……", "punctuation"),
    ];

    for (text_str, label) in cases {
        let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");

        let glyphs = blob.shape.glyphs.as_ref();
        assert!(
            !glyphs.is_empty(),
            "expected shaped glyphs for CJK case {label}"
        );

        let mut non_color: Vec<super::GlyphKey> = Vec::new();
        for g in glyphs {
            match g.kind() {
                super::GlyphQuadKind::Mask | super::GlyphQuadKind::Subpixel => {
                    non_color.push(g.key);
                }
                super::GlyphQuadKind::Color => {}
            }
        }

        assert!(
            !non_color.is_empty(),
            "expected at least one mask/subpixel glyph for CJK case {label}"
        );

        let epoch = 1;
        for key in non_color {
            text.ensure_glyph_in_atlas(key, epoch);
            match key.kind {
                super::GlyphQuadKind::Mask => assert!(
                    text.atlas_runtime.mask_atlas.get(key, epoch).is_some(),
                    "expected mask glyph to be present in mask atlas after ensure ({label})"
                ),
                super::GlyphQuadKind::Subpixel => assert!(
                    text.atlas_runtime.subpixel_atlas.get(key, epoch).is_some(),
                    "expected subpixel glyph to be present in subpixel atlas after ensure ({label})"
                ),
                super::GlyphQuadKind::Color => {}
            }
        }
    }
}

#[test]
fn cjk_fallback_uses_cjk_lite_font_without_explicit_family_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    reset_bundled_only_font_runtime(&mut text);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::cjk_lite_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family_inter = "Inter";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_inter)),
        "expected {family_inter} to be present after loading bootstrap fonts"
    );

    let family_cjk = "Noto Sans CJK SC";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_cjk)),
        "expected {family_cjk} to be present after loading cjk-lite fonts"
    );

    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![family_inter.to_string()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let style = TextStyle {
        font: fret_core::FontId::ui(),
        size: Px(24.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let expected_cjk_faces = {
        let explicit = TextStyle {
            font: fret_core::FontId::family(family_cjk),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("你", &explicit, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_cjk_faces.is_empty(),
        "expected at least one resolved CJK face for the explicit {family_cjk} family"
    );

    let (blob_id, _metrics) = text.prepare("你", &style, constraints);
    let glyph_keys: Vec<super::GlyphKey> = {
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape.glyphs.iter().map(|g| g.key).collect()
    };

    assert!(!glyph_keys.is_empty(), "expected shaped glyphs for CJK");

    let used_cjk_lite = glyph_keys
        .iter()
        .any(|k| expected_cjk_faces.contains(&k.font));
    assert!(
        used_cjk_lite,
        "expected cjk-lite font to be selected for CJK glyphs under the UI sans stack when system fonts are absent"
    );

    let epoch = 1;
    for key in glyph_keys {
        if !expected_cjk_faces.contains(&key.font) {
            continue;
        }

        text.ensure_glyph_in_atlas(key, epoch);
        match key.kind {
            super::GlyphQuadKind::Mask => assert!(
                text.atlas_runtime.mask_atlas.get(key, epoch).is_some(),
                "expected ensured CJK glyph to be present in the mask atlas"
            ),
            super::GlyphQuadKind::Subpixel => assert!(
                text.atlas_runtime.subpixel_atlas.get(key, epoch).is_some(),
                "expected ensured CJK glyph to be present in the subpixel atlas"
            ),
            super::GlyphQuadKind::Color => {}
        }
    }
}

#[test]
fn font_trace_records_missing_glyphs_for_named_family_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    text.begin_frame_diagnostics();

    let style = TextStyle {
        font: fret_core::FontId::family("Inter"),
        size: Px(24.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let (_blob_id, _metrics) = text.prepare("你", &style, constraints);
    let trace = text.font_trace_snapshot(fret_core::FrameId(1));
    assert!(
        !trace.entries.is_empty(),
        "expected at least one font trace entry"
    );

    let entry = trace.entries.last().expect("trace entry");
    assert!(
        entry.missing_glyphs > 0,
        "expected missing/tofu glyphs to be recorded in the trace (entry={entry:?})"
    );

    let inter_usage = entry
        .families
        .iter()
        .find(|f| f.family.to_ascii_lowercase().contains("inter"))
        .expect("expected Inter family to appear in the trace families");
    assert!(
        inter_usage.missing_glyphs > 0,
        "expected missing/tofu glyphs to be attributed to the resolved family"
    );
}

#[test]
fn cjk_fallback_uses_common_fallback_for_named_family_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::cjk_lite_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family_inter = "Inter";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_inter)),
        "expected {family_inter} to be present after loading bootstrap fonts"
    );

    let family_cjk = "Noto Sans CJK SC";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_cjk)),
        "expected {family_cjk} to be present after loading cjk-lite fonts"
    );

    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![family_inter.to_string()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let expected_cjk_faces = {
        let explicit = TextStyle {
            font: fret_core::FontId::family(family_cjk),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("你", &explicit, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_cjk_faces.is_empty(),
        "expected at least one resolved CJK face for the explicit {family_cjk} family"
    );

    let style_named = TextStyle {
        font: fret_core::FontId::family(family_inter),
        size: Px(24.0),
        ..Default::default()
    };
    let (blob_id, _metrics) = text.prepare("你", &style_named, constraints);
    let glyph_keys: Vec<super::GlyphKey> = {
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape.glyphs.iter().map(|g| g.key).collect()
    };

    assert!(!glyph_keys.is_empty(), "expected shaped glyphs for CJK");

    let used_cjk_lite = glyph_keys
        .iter()
        .any(|k| expected_cjk_faces.contains(&k.font));
    assert!(
        used_cjk_lite,
        "expected common fallback stack to select cjk-lite for CJK glyphs when an explicit named UI font is missing glyphs and system fonts are absent"
    );
}

#[test]
fn emoji_fallback_uses_bundled_color_font_without_explicit_family_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::emoji_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family_inter = "Inter";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_inter)),
        "expected {family_inter} to be present after loading bootstrap fonts"
    );

    let family_emoji = "Noto Color Emoji";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family_emoji)),
        "expected {family_emoji} to be present after loading emoji fonts"
    );

    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![family_inter.to_string()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let style = TextStyle {
        font: fret_core::FontId::ui(),
        size: Px(32.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let expected_emoji_faces = {
        let explicit = TextStyle {
            font: fret_core::FontId::family(family_emoji),
            size: Px(32.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("\u{1F600}", &explicit, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_emoji_faces.is_empty(),
        "expected at least one resolved emoji face for the explicit {family_emoji} family"
    );

    let cases = [
        ("\u{1F600}", "single emoji"),
        ("\u{2708}\u{FE0F}", "vs16 emoji presentation"),
        ("1\u{FE0F}\u{20E3}", "keycap sequence"),
        ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
        (
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "zwj family sequence",
        ),
    ];

    for (text_str, label) in cases {
        let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);

        let glyph_keys: Vec<super::GlyphKey> = {
            let blob = text.blob(blob_id).expect("text blob");
            blob.shape.glyphs.iter().map(|g| g.key).collect()
        };
        assert!(
            !glyph_keys.is_empty(),
            "expected shaped glyphs for emoji case {label}"
        );

        let emoji_keys: Vec<super::GlyphKey> = glyph_keys
            .iter()
            .copied()
            .filter(|k| expected_emoji_faces.contains(&k.font))
            .collect();
        assert!(
            !emoji_keys.is_empty(),
            "expected bundled emoji font to be selected for {label} under the UI sans stack when system fonts are absent"
        );

        let color_keys: Vec<super::GlyphKey> = emoji_keys
            .iter()
            .copied()
            .filter(|k| k.kind == super::GlyphQuadKind::Color)
            .collect();
        assert!(
            !color_keys.is_empty(),
            "expected at least one color emoji glyph quad for {label}"
        );

        let epoch = 1;
        for key in color_keys {
            text.ensure_glyph_in_atlas(key, epoch);
            assert!(
                text.atlas_runtime.color_atlas.get(key, epoch).is_some(),
                "expected ensured emoji glyph to be present in the color atlas ({label})"
            );
        }
    }
}

#[test]
fn span_fingerprints_split_shaping_and_paint() {
    let constraints = TextConstraints {
        max_width: Some(Px(200.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let base = TextStyle::default();
    let text = "hello";

    let mut spans_a = vec![TextSpan {
        len: text.len(),
        shaping: Default::default(),
        paint: Default::default(),
    }];
    spans_a[0].paint.fg = Some(Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    let mut spans_b = spans_a.clone();
    spans_b[0].paint.fg = Some(Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    });

    assert_eq!(
        spans_shaping_fingerprint(&spans_a),
        spans_shaping_fingerprint(&spans_b)
    );
    assert_ne!(
        spans_paint_fingerprint(&spans_a),
        spans_paint_fingerprint(&spans_b)
    );

    let mut spans_c = spans_a.clone();
    spans_c[0].paint.underline = Some(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });
    assert_ne!(
        spans_paint_fingerprint(&spans_a),
        spans_paint_fingerprint(&spans_c)
    );

    let mut spans_d = spans_a.clone();
    spans_d[0].shaping = spans_d[0].shaping.clone().with_axis("wght", 700.0);
    assert_ne!(
        spans_shaping_fingerprint(&spans_a),
        spans_shaping_fingerprint(&spans_d),
        "axis overrides must participate in shaping fingerprints"
    );

    let mut spans_e = spans_a.clone();
    spans_e[0].shaping = spans_e[0].shaping.clone().with_feature("liga", 0);
    assert_ne!(
        spans_shaping_fingerprint(&spans_a),
        spans_shaping_fingerprint(&spans_e),
        "feature overrides must participate in shaping fingerprints"
    );

    let mut spans_f = spans_a.clone();
    spans_f[0].shaping = spans_f[0]
        .shaping
        .clone()
        .with_feature("liga", 0)
        .with_feature("liga", 1)
        .with_feature(" lig ", 42)
        .with_feature("", 1)
        .with_feature("calt", 0);

    let mut spans_g = spans_a.clone();
    spans_g[0].shaping = spans_g[0]
        .shaping
        .clone()
        .with_feature("calt", 0)
        .with_feature("liga", 1);

    assert_eq!(
        spans_shaping_fingerprint(&spans_f),
        spans_shaping_fingerprint(&spans_g),
        "expected feature canonicalization to ignore invalid tags, coalesce duplicates, and be order-independent"
    );

    let rich_a =
        fret_core::AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans_a));
    let rich_b =
        fret_core::AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans_b));

    let k_a = TextBlobKey::new_attributed(&rich_a, &base, constraints, 7);
    let k_b = TextBlobKey::new_attributed(&rich_b, &base, constraints, 7);
    assert_ne!(k_a, k_b, "paint changes should affect blob cache keys");
    assert_eq!(
        TextShapeKey::from_blob_key(&k_a),
        TextShapeKey::from_blob_key(&k_b),
        "paint changes must not affect shape cache keys"
    );
}

#[test]
fn multispan_paint_changes_do_not_affect_shape_key() {
    let constraints = TextConstraints {
        max_width: Some(Px(200.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let base = TextStyle::default();
    let text = "let x = 1;";

    let mk_spans = |kw: Color, ident: Color| {
        vec![
            TextSpan {
                len: "let".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(kw),
                    ..Default::default()
                },
            },
            TextSpan::new(" ".len()),
            TextSpan {
                len: "x".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(ident),
                    ..Default::default()
                },
            },
            TextSpan::new(" = 1;".len()),
        ]
    };

    let spans_a = mk_spans(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    );
    let spans_b = mk_spans(
        Color {
            r: 0.2,
            g: 0.2,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 0.6,
            b: 0.2,
            a: 1.0,
        },
    );

    assert_eq!(
        spans_shaping_fingerprint(&spans_a),
        spans_shaping_fingerprint(&spans_b),
        "expected theme-only paint changes to not affect shaping fingerprints"
    );
    assert_ne!(
        spans_paint_fingerprint(&spans_a),
        spans_paint_fingerprint(&spans_b),
        "expected paint changes to affect paint fingerprints"
    );

    let rich_a =
        fret_core::AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans_a));
    let rich_b =
        fret_core::AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans_b));

    let k_a = TextBlobKey::new_attributed(&rich_a, &base, constraints, 7);
    let k_b = TextBlobKey::new_attributed(&rich_b, &base, constraints, 7);
    assert_ne!(k_a, k_b, "paint changes should affect blob cache keys");
    assert_eq!(
        TextShapeKey::from_blob_key(&k_a),
        TextShapeKey::from_blob_key(&k_b),
        "paint changes must not affect shape cache keys, even for multiple spans"
    );
}

#[test]
fn paint_only_changes_miss_blob_cache_but_hit_shape_cache() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    text.begin_frame_diagnostics();

    let constraints = TextConstraints {
        max_width: Some(Px(200.0)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };
    let base = TextStyle::default();
    let content: Arc<str> = Arc::<str>::from("let x = 1;");

    let mk_rich = |kw: Color, ident: Color| {
        let spans = vec![
            TextSpan {
                len: "let".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(kw),
                    ..Default::default()
                },
            },
            TextSpan::new(" ".len()),
            TextSpan {
                len: "x".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(ident),
                    ..Default::default()
                },
            },
            TextSpan::new(" = 1;".len()),
        ];
        fret_core::AttributedText::new(Arc::clone(&content), Arc::<[TextSpan]>::from(spans))
    };

    let rich_a = mk_rich(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    );
    let rich_b = mk_rich(
        Color {
            r: 0.2,
            g: 0.2,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 0.8,
            g: 0.8,
            b: 0.0,
            a: 1.0,
        },
    );

    let _ = text.prepare_attributed(&rich_a, &base, constraints);
    let _ = text.prepare_attributed(&rich_b, &base, constraints);

    let snap = text.diagnostics_snapshot(fret_core::FrameId(1));
    assert_eq!(
        snap.frame_blob_cache_hits, 0,
        "expected paint-only changes to miss the blob cache"
    );
    assert_eq!(
        snap.frame_blob_cache_misses, 2,
        "expected two distinct blob keys for different paint"
    );
    assert_eq!(
        snap.frame_shape_cache_misses, 1,
        "expected the first prepare to shape"
    );
    assert_eq!(
        snap.frame_shape_cache_hits, 1,
        "expected the second prepare to reuse the shaped layout"
    );
}

#[test]
fn variable_font_axis_overrides_participate_in_face_key_and_raster_output() {
    // Use a small variable-font subset as a deterministic fixture.
    const ROBOTO_FLEX_SUBSET: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/RobotoFlex-Subset.ttf"
    ));

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only the injected font.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let added = text.add_fonts([ROBOTO_FLEX_SUBSET.to_vec()]);
    assert!(added > 0, "expected variable font to load");

    let family = text
            .all_font_names()
            .into_iter()
            .find(|n| n.to_ascii_lowercase().contains("roboto flex"))
            .unwrap_or_else(|| {
                panic!(
                    "expected a Roboto Flex family name after loading the fixture font (names_head={:?})",
                    text.all_font_names().into_iter().take(8).collect::<Vec<_>>()
                )
            });
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let base_style = TextStyle {
        font: fret_core::FontId::family(family.clone()),
        size: Px(64.0),
        weight: FontWeight(400),
        ..Default::default()
    };

    let rich_with_wght = |wght: f32| {
        assert!(wght.is_finite());
        let spans = vec![TextSpan {
            len: 1,
            shaping: TextShapingStyle::default().with_axis("wght", wght),
            paint: Default::default(),
        }];
        fret_core::AttributedText::new(Arc::<str>::from("0"), Arc::<[TextSpan]>::from(spans))
    };

    let rich_light = rich_with_wght(200.0);
    let rich_heavy = rich_with_wght(900.0);
    assert_eq!(rich_light.spans.len(), 1);
    assert_eq!(rich_light.spans[0].shaping.axes.len(), 1);
    assert_eq!(rich_light.spans[0].shaping.axes[0].tag, "wght");

    let (blob_light, _) = text.prepare_attributed(&rich_light, &base_style, constraints);
    let key_light = {
        let blob = text.blob(blob_light).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    let (blob_heavy, _) = text.prepare_attributed(&rich_heavy, &base_style, constraints);
    let key_heavy = {
        let blob = text.blob(blob_heavy).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    assert!(
        key_light.font.face_index != key_heavy.font.face_index
            || key_light.font.variation_key != key_heavy.font.variation_key,
        "expected axis overrides to participate in font face identity (face_index {} vs {}, variation_key {} vs {})",
        key_light.font.face_index,
        key_heavy.font.face_index,
        key_light.font.variation_key,
        key_heavy.font.variation_key
    );

    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();
    let epoch = 1;

    text.ensure_glyph_in_atlas(key_light, epoch);
    let bytes_light = pending_upload_bytes_for_key(&text, key_light);

    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();

    text.ensure_glyph_in_atlas(key_heavy, epoch);
    let bytes_heavy = pending_upload_bytes_for_key(&text, key_heavy);

    assert_ne!(
        bytes_light, bytes_heavy,
        "expected raster output to differ across axis overrides"
    );
}

#[test]
fn variable_font_weight_changes_face_key_and_raster_output() {
    // Use a small variable-font subset as a deterministic fixture.
    const ROBOTO_FLEX_SUBSET: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/RobotoFlex-Subset.ttf"
    ));

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only the injected font.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let added = text.add_fonts([ROBOTO_FLEX_SUBSET.to_vec()]);
    assert!(added > 0, "expected variable font to load");

    let family = "Roboto Flex";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family)),
        "expected {family} to be present after loading test font"
    );

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let style_light = TextStyle {
        font: fret_core::FontId::family(family),
        size: Px(64.0),
        weight: FontWeight(200),
        ..Default::default()
    };
    let style_heavy = TextStyle {
        font: fret_core::FontId::family(family),
        size: Px(64.0),
        weight: FontWeight(900),
        ..Default::default()
    };

    let (blob_light, _) = text.prepare("0", &style_light, constraints);
    let key_light = {
        let blob = text.blob(blob_light).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    let (blob_heavy, _) = text.prepare("0", &style_heavy, constraints);
    let key_heavy = {
        let blob = text.blob(blob_heavy).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    assert_eq!(
        key_light.font.font_data_id, key_heavy.font.font_data_id,
        "expected both weights to use the same font data blob"
    );
    assert_eq!(
        key_light.font.face_index, key_heavy.font.face_index,
        "expected both weights to use the same face index"
    );
    assert_ne!(
        key_light.font.variation_key, key_heavy.font.variation_key,
        "expected variable font instance coordinates to participate in the face key"
    );

    // Ensure path must also apply instance coordinates when rasterizing on-demand.
    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();
    let epoch = 1;

    text.ensure_glyph_in_atlas(key_light, epoch);
    let bytes_light = pending_upload_bytes_for_key(&text, key_light);

    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();

    text.ensure_glyph_in_atlas(key_heavy, epoch);
    let bytes_heavy = pending_upload_bytes_for_key(&text, key_heavy);

    assert_ne!(
        bytes_light, bytes_heavy,
        "expected raster output to differ across variable font weights"
    );
}

#[test]
fn open_type_feature_overrides_can_change_shaped_glyph_output_for_known_font_fixture() {
    // Lock a "behavior visible" contract for OpenType feature overrides (e.g. `liga`/`calt`).
    // This avoids relying solely on cache-key correctness: we also want to know the shaping
    // pipeline actually applies feature overrides for fonts that support them.
    const INTER_ROMAN: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fret-fonts/assets/Inter-roman.ttf"
    ));

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only the injected font.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let added = text.add_fonts([INTER_ROMAN.to_vec()]);
    assert!(added > 0, "expected Inter fixture font to load");

    let family = text
        .all_font_names()
        .into_iter()
        .find(|n| {
            let lower = n.to_ascii_lowercase();
            lower == "inter" || lower.contains("inter ")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected an Inter family name after loading the fixture font (names_head={:?})",
                text.all_font_names()
                    .into_iter()
                    .take(8)
                    .collect::<Vec<_>>()
            )
        });

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let base_style = TextStyle {
        font: fret_core::FontId::family(family.clone()),
        size: Px(32.0),
        weight: FontWeight(400),
        ..Default::default()
    };

    let rich = |text: &str, shaping: TextShapingStyle| {
        fret_core::AttributedText::new(
            Arc::<str>::from(text),
            Arc::<[TextSpan]>::from(vec![TextSpan {
                len: text.len(),
                shaping,
                paint: Default::default(),
            }]),
        )
    };

    // Inter's OpenType "ligature-like" sequences (e.g. long arrows) are expressed via `calt`
    // rather than `liga`, so gate `calt` behavior explicitly.
    let shaping_off = TextShapingStyle::default().with_feature("calt", 0);
    let shaping_on = TextShapingStyle::default().with_feature("calt", 1);

    let candidates = [
        "->", "=>", "-->", "<--", "<=>", "==>", "--->", "<---", "<==>",
    ];
    let mut diffs: Vec<String> = Vec::new();
    let mut report: Vec<String> = Vec::new();

    for candidate in candidates {
        let rich_off = rich(candidate, shaping_off.clone());
        let rich_on = rich(candidate, shaping_on.clone());

        let (blob_off, _) = text.prepare_attributed(&rich_off, &base_style, constraints);
        let (blob_on, _) = text.prepare_attributed(&rich_on, &base_style, constraints);

        let glyph_ids_off: Vec<u32> = text
            .blob(blob_off)
            .expect("text blob")
            .shape
            .glyphs
            .iter()
            .map(|g| g.key.glyph_id)
            .collect();
        let glyph_ids_on: Vec<u32> = text
            .blob(blob_on)
            .expect("text blob")
            .shape
            .glyphs
            .iter()
            .map(|g| g.key.glyph_id)
            .collect();

        if glyph_ids_off != glyph_ids_on {
            let head_off = glyph_ids_off.iter().take(8).copied().collect::<Vec<_>>();
            let head_on = glyph_ids_on.iter().take(8).copied().collect::<Vec<_>>();
            diffs.push(format!(
                "{candidate}: off_len={} on_len={} off_head={head_off:?} on_head={head_on:?}",
                glyph_ids_off.len(),
                glyph_ids_on.len()
            ));
        }

        report.push(format!(
            "{candidate}: off={:?} on={:?}",
            glyph_ids_off, glyph_ids_on
        ));
    }

    assert!(
        !diffs.is_empty(),
        "expected at least one candidate to change shaped glyph output when toggling `calt` (family={family:?}); report={report:?}"
    );
}

#[test]
fn open_type_feature_overrides_can_change_word_wrap_breakpoints_for_known_font_fixture() {
    // Lock that OpenType feature overrides can affect layout decisions under `TextWrap::Word`,
    // not just the shaped glyph IDs. This protects against regressions where features are
    // applied for shaping but ignored by line breaking / wrapping codepaths.
    const INTER_ROMAN: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fret-fonts/assets/Inter-roman.ttf"
    ));

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only the injected font.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let added = text.add_fonts([INTER_ROMAN.to_vec()]);
    assert!(added > 0, "expected Inter fixture font to load");

    let family = text
        .all_font_names()
        .into_iter()
        .find(|n| {
            let lower = n.to_ascii_lowercase();
            lower == "inter" || lower.contains("inter ")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected an Inter family name after loading the fixture font (names_head={:?})",
                text.all_font_names()
                    .into_iter()
                    .take(8)
                    .collect::<Vec<_>>()
            )
        });

    let base_style = TextStyle {
        font: fret_core::FontId::family(family.clone()),
        size: Px(32.0),
        weight: FontWeight(400),
        ..Default::default()
    };

    let shaping_off = TextShapingStyle::default().with_feature("calt", 0);
    let shaping_on = TextShapingStyle::default().with_feature("calt", 1);

    let token_candidates = ["->", "=>", "-->", "<=>", "==>", "--->", "<==>"];

    let mut chosen: Option<(&'static str, Px)> = None;
    let mut debug: Vec<String> = Vec::new();

    for token in token_candidates {
        // Build "token " repeated so word wrap breakpoints are at deterministic whitespace.
        let token_with_space = format!("{token} ");
        let mut content = String::new();
        for _ in 0..32 {
            content.push_str(&token_with_space);
        }

        let rich = |s: &str, shaping: TextShapingStyle| {
            fret_core::AttributedText::new(
                Arc::<str>::from(s),
                Arc::<[TextSpan]>::from(vec![TextSpan {
                    len: s.len(),
                    shaping,
                    paint: Default::default(),
                }]),
            )
        };

        let single_line = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let (blob_off, _) = text.prepare_attributed(
            &rich(&content, shaping_off.clone()),
            &base_style,
            single_line,
        );
        let (blob_on, _) = text.prepare_attributed(
            &rich(&content, shaping_on.clone()),
            &base_style,
            single_line,
        );

        // Find a breakpoint width that allows N tokens under `calt=1` but only N-1 under
        // `calt=0`, then gate that wrapping produces different first-line end indices.
        let token_len = token_with_space.len();
        for n in 2..24usize {
            let idx = (token_len * n).min(content.len());
            let x_off = text
                .caret_x(blob_off, idx)
                .expect("caret_x for feature-off candidate");
            let x_on = text
                .caret_x(blob_on, idx)
                .expect("caret_x for feature-on candidate");
            if (x_off.0 - x_on.0).abs() < 0.5 {
                continue;
            }

            let next_idx = (token_len * (n + 1)).min(content.len());
            let x_off_next = text
                .caret_x(blob_off, next_idx)
                .expect("caret_x for feature-off candidate (next)");
            let x_on_next = text
                .caret_x(blob_on, next_idx)
                .expect("caret_x for feature-on candidate (next)");

            // Pick a width that fits (n+1) tokens under calt=on but not under calt=off.
            if x_on_next.0 + 0.5 < x_off_next.0 {
                let w = Px((x_on_next.0 + x_off_next.0) * 0.5);
                let wrapped_constraints = TextConstraints {
                    max_width: Some(w),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    scale_factor: 1.0,
                };

                let (wrap_off, _) = text.prepare_attributed(
                    &rich(&content, shaping_off.clone()),
                    &base_style,
                    wrapped_constraints,
                );
                let (wrap_on, _) = text.prepare_attributed(
                    &rich(&content, shaping_on.clone()),
                    &base_style,
                    wrapped_constraints,
                );

                let shape_off = text.blob(wrap_off).expect("wrapped blob off").shape.clone();
                let shape_on = text.blob(wrap_on).expect("wrapped blob on").shape.clone();

                let first_end_off = shape_off.lines.first().map(|l| l.end).unwrap_or(0);
                let first_end_on = shape_on.lines.first().map(|l| l.end).unwrap_or(0);
                let lines_off = shape_off.lines.len();
                let lines_on = shape_on.lines.len();

                debug.push(format!(
                        "{token}: n={n} off={:.2} on={:.2} off_next={:.2} on_next={:.2} w={:.2} first_end_off={first_end_off} first_end_on={first_end_on} lines_off={lines_off} lines_on={lines_on}",
                        x_off.0, x_on.0, x_off_next.0, x_on_next.0, w.0
                    ));

                text.release(wrap_off);
                text.release(wrap_on);

                if first_end_off != first_end_on || lines_off != lines_on {
                    chosen = Some((token, w));
                    break;
                }
            }
        }

        text.release(blob_off);
        text.release(blob_on);

        if chosen.is_some() {
            break;
        }
    }

    let (token, max_width) = chosen.unwrap_or_else(|| {
            panic!(
                "expected at least one Inter `calt` token to produce a wrap width that changes breakpoints; debug={debug:?}"
            )
        });

    let token_with_space = format!("{token} ");
    let mut content = String::new();
    for _ in 0..32 {
        content.push_str(&token_with_space);
    }

    let rich = |s: &str, shaping: TextShapingStyle| {
        fret_core::AttributedText::new(
            Arc::<str>::from(s),
            Arc::<[TextSpan]>::from(vec![TextSpan {
                len: s.len(),
                shaping,
                paint: Default::default(),
            }]),
        )
    };

    let wrapped_constraints = TextConstraints {
        max_width: Some(max_width),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let (blob_off, _) = text.prepare_attributed(
        &rich(&content, shaping_off),
        &base_style,
        wrapped_constraints,
    );
    let (blob_on, _) = text.prepare_attributed(
        &rich(&content, shaping_on),
        &base_style,
        wrapped_constraints,
    );

    let shape_off = text.blob(blob_off).expect("blob off").shape.clone();
    let shape_on = text.blob(blob_on).expect("blob on").shape.clone();

    let first_end_off = shape_off.lines.first().map(|l| l.end).unwrap_or(0);
    let first_end_on = shape_on.lines.first().map(|l| l.end).unwrap_or(0);
    let lines_off = shape_off.lines.len();
    let lines_on = shape_on.lines.len();

    assert_ne!(
        (first_end_off, lines_off),
        (first_end_on, lines_on),
        "expected `calt` toggles to change `TextWrap::Word` wrap output for token={token:?} max_width={max_width:?}; debug={debug:?}"
    );

    text.release(blob_off);
    text.release(blob_on);
}

#[test]
fn parley_feature_override_calt_0_disables_inter_arrow_ligature() {
    // Sanity check (upstream dependency behavior): Inter contains a `calt` ligature mapping
    // for "->" -> "arrowright". Ensure `calt=0` disables it.
    const INTER_ROMAN: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fret-fonts/assets/Inter-roman.ttf"
    ));

    let text = "->";

    fn shape_with_calt(text: &str, calt: u16) -> Vec<u32> {
        use std::borrow::Cow;
        let mut fcx = parley::FontContext {
            collection: parley::fontique::Collection::new(parley::fontique::CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            source_cache: parley::fontique::SourceCache::default(),
        };
        fcx.collection.register_fonts(
            parley::fontique::Blob::<u8>::from(INTER_ROMAN.to_vec()),
            None,
        );

        let mut lcx: parley::LayoutContext<[u8; 4]> = parley::LayoutContext::default();
        let mut builder = lcx.ranged_builder(&mut fcx, text, 1.0, true);

        builder.push_default(parley::style::StyleProperty::FontStack(
            parley::style::FontStack::Source(Cow::Borrowed("Inter")),
        ));
        builder.push_default(parley::style::StyleProperty::FontSize(32.0));
        builder.push(
            parley::style::StyleProperty::FontFeatures(parley::style::FontSettings::List(
                Cow::Owned(vec![swash::Setting {
                    tag: swash::tag_from_bytes(b"calt"),
                    value: calt,
                }]),
            )),
            0..text.len(),
        );

        let mut layout = builder.build(text);
        layout.break_all_lines(None);

        let line = layout.lines().next().expect("line");
        let item = line.items().next().expect("item");
        let glyph_run = match item {
            parley::PositionedLayoutItem::GlyphRun(glyph_run) => glyph_run,
            parley::PositionedLayoutItem::InlineBox(_) => unreachable!(),
        };
        glyph_run
            .run()
            .clusters()
            .flat_map(|c| c.glyphs().map(|g| g.id))
            .collect::<Vec<_>>()
    }

    let ids_on = shape_with_calt(text, 1);
    let ids_off = shape_with_calt(text, 0);
    assert_ne!(
        ids_on, ids_off,
        "expected `calt` override to affect glyph output for {text:?} (on={ids_on:?}, off={ids_off:?})"
    );
    assert!(
        ids_on.len() < ids_off.len(),
        "expected `calt=1` to use a ligature glyph for {text:?} (on={ids_on:?}, off={ids_off:?})"
    );
}

#[test]
fn parley_tree_builder_honors_font_features_for_inter_arrow_ligature() {
    const INTER_ROMAN: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fret-fonts/assets/Inter-roman.ttf"
    ));

    let text = "->";
    use std::borrow::Cow;

    let mut fcx = parley::FontContext {
        collection: parley::fontique::Collection::new(parley::fontique::CollectionOptions {
            shared: false,
            system_fonts: false,
        }),
        source_cache: parley::fontique::SourceCache::default(),
    };
    fcx.collection.register_fonts(
        parley::fontique::Blob::<u8>::from(INTER_ROMAN.to_vec()),
        None,
    );

    let mut lcx: parley::LayoutContext<[u8; 4]> = parley::LayoutContext::default();

    let mut shape_with_calt = |calt: u16| {
        let root = parley::style::TextStyle::default();
        let mut builder = lcx.tree_builder(&mut fcx, 1.0, true, &root);

        let base = parley::style::TextStyle {
            font_stack: parley::style::FontStack::Source(Cow::Borrowed("Inter")),
            font_size: 32.0,
            ..Default::default()
        };

        builder.push_style_span(base);

        let props = [parley::style::StyleProperty::FontFeatures(
            parley::style::FontSettings::List(Cow::Owned(vec![swash::Setting {
                tag: swash::tag_from_bytes(b"calt"),
                value: calt,
            }])),
        )];
        builder.push_style_modification_span(props.iter());

        builder.push_text(text);
        builder.pop_style_span(); // modification span
        builder.pop_style_span(); // base span

        let mut layout = parley::Layout::default();
        let _ = builder.build_into(&mut layout);
        layout.break_all_lines(None);

        let line = layout.lines().next().expect("line");
        let item = line.items().next().expect("item");
        let glyph_run = match item {
            parley::PositionedLayoutItem::GlyphRun(glyph_run) => glyph_run,
            parley::PositionedLayoutItem::InlineBox(_) => unreachable!(),
        };
        glyph_run
            .run()
            .clusters()
            .flat_map(|c| c.glyphs().map(|g| g.id))
            .collect::<Vec<_>>()
    };

    let ids_on = shape_with_calt(1);
    let ids_off = shape_with_calt(0);

    assert_ne!(
        ids_on, ids_off,
        "expected TreeBuilder + StyleProperty::FontFeatures to affect glyph output for {text:?} (on={ids_on:?}, off={ids_off:?})"
    );
    assert!(
        ids_on.len() < ids_off.len(),
        "expected `calt=1` to use a ligature glyph for {text:?} (on={ids_on:?}, off={ids_off:?})"
    );
}

#[test]
fn synthesis_skew_participates_in_face_key_and_raster_output() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only the injected font.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::cjk_lite_fonts()
        .iter()
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected cjk-lite fonts to load");

    let family = "Noto Sans CJK SC";
    assert!(
        text.all_font_names()
            .iter()
            .any(|n| n.eq_ignore_ascii_case(family)),
        "expected {family} to be present after loading test font"
    );

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let style_normal = TextStyle {
        font: fret_core::FontId::family(family),
        size: Px(96.0),
        slant: fret_core::TextSlant::Normal,
        ..Default::default()
    };
    let style_italic = TextStyle {
        font: fret_core::FontId::family(family),
        size: Px(96.0),
        slant: fret_core::TextSlant::Italic,
        ..Default::default()
    };

    let (blob_normal, _) = text.prepare("你", &style_normal, constraints);
    let key_normal = {
        let blob = text.blob(blob_normal).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    let (blob_italic, _) = text.prepare("你", &style_italic, constraints);
    let key_italic = {
        let blob = text.blob(blob_italic).expect("text blob");
        blob.shape.glyphs.first().expect("glyph").key
    };

    assert_eq!(
        key_normal.font.font_data_id, key_italic.font.font_data_id,
        "expected both styles to use the same font data blob"
    );
    assert_eq!(
        key_normal.font.face_index, key_italic.font.face_index,
        "expected both styles to use the same face index"
    );
    assert_eq!(
        key_normal.font.variation_key, key_italic.font.variation_key,
        "expected both styles to use the same variation coordinates"
    );
    assert_eq!(
        key_normal.font.synthesis_skew_degrees, 0,
        "expected the base style to require no faux skew"
    );
    assert_ne!(
        key_italic.font.synthesis_skew_degrees, 0,
        "expected italic request to trigger a faux skew when no italic face is available"
    );

    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();
    let epoch = 1;

    text.ensure_glyph_in_atlas(key_normal, epoch);
    let bytes_normal = pending_upload_bytes_for_key(&text, key_normal);

    text.atlas_runtime.mask_atlas.reset();
    text.atlas_runtime.color_atlas.reset();
    text.atlas_runtime.subpixel_atlas.reset();

    text.ensure_glyph_in_atlas(key_italic, epoch);
    let bytes_italic = pending_upload_bytes_for_key(&text, key_italic);

    assert_ne!(
        bytes_normal, bytes_italic,
        "expected raster output to differ when faux skew is applied"
    );
}

#[test]
fn common_fallback_stack_suffix_dedupes_and_preserves_order() {
    let config = vec![
        "  Noto Color Emoji  ".to_string(),
        "Noto Sans CJK SC".to_string(),
        "noto color emoji".to_string(),
        "".to_string(),
    ];
    let defaults = &["Noto Sans CJK SC", "Noto Sans Arabic", "Noto Color Emoji"];

    let suffix = fret_render_text::fallback_policy::common_fallback_stack_suffix(&config, defaults);
    assert_eq!(
        suffix,
        "Noto Color Emoji, Noto Sans CJK SC, Noto Sans Arabic"
    );
}

#[test]
fn fallback_policy_key_normalizes_family_candidates_and_stays_stable_across_case_changes() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Make the test independent from host/system fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );

    let config0 = fret_core::TextFontFamilyConfig {
        common_fallback_injection: fret_core::TextCommonFallbackInjection::CommonFallback,
        ui_sans: vec!["  Inter  ".to_string()],
        common_fallback: vec![
            " Noto Color Emoji ".to_string(),
            "Noto Sans CJK SC".to_string(),
        ],
        ..Default::default()
    };
    let _ = text.set_font_families(&config0);
    let key0 = text.font_runtime.fallback_policy.fallback_policy_key;

    let config1 = fret_core::TextFontFamilyConfig {
        common_fallback_injection: fret_core::TextCommonFallbackInjection::CommonFallback,
        ui_sans: vec!["inter".to_string()],
        common_fallback: vec![
            "noto color emoji".to_string(),
            "  noto sans cjk sc  ".to_string(),
        ],
        ..Default::default()
    };
    let _ = text.set_font_families(&config1);
    let key1 = text.font_runtime.fallback_policy.fallback_policy_key;

    assert_eq!(
        key0, key1,
        "expected fallback policy key to ignore case/whitespace changes"
    );
}

#[test]
fn fallback_policy_key_changes_when_common_fallback_injection_changes() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    let config0 = fret_core::TextFontFamilyConfig {
        common_fallback_injection: fret_core::TextCommonFallbackInjection::PlatformDefault,
        ..Default::default()
    };
    let _ = text.set_font_families(&config0);
    let snap0 = text.fallback_policy_snapshot(fret_core::FrameId(1));
    assert!(
        !snap0.prefer_common_fallback,
        "expected PlatformDefault to prefer system fallback when system fonts are enabled"
    );
    assert_eq!(
        snap0.common_fallback_stack_suffix, "",
        "expected no explicit common fallback suffix when prefer_common_fallback=false"
    );
    assert!(
        snap0.common_fallback_candidates.is_empty(),
        "expected no explicit common fallback candidates when prefer_common_fallback=false"
    );

    let config1 = fret_core::TextFontFamilyConfig {
        common_fallback_injection: fret_core::TextCommonFallbackInjection::CommonFallback,
        ..Default::default()
    };
    let changed = text.set_font_families(&config1);
    assert!(
        changed,
        "expected font families to update when common_fallback_injection changes"
    );
    let snap1 = text.fallback_policy_snapshot(fret_core::FrameId(2));
    assert!(
        snap1.prefer_common_fallback,
        "expected CommonFallback injection to prefer common fallback"
    );
    assert!(
        !snap1.common_fallback_stack_suffix.is_empty(),
        "expected a non-empty common fallback stack suffix when prefer_common_fallback=true"
    );
    assert!(
        !snap1.common_fallback_candidates.is_empty(),
        "expected non-empty common fallback candidates when prefer_common_fallback=true"
    );
    assert_ne!(
        snap0.fallback_policy_key, snap1.fallback_policy_key,
        "expected fallback_policy_key to change when the fallback injection mode changes"
    );
}

#[test]
fn mixed_script_fallback_uses_bundled_faces_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::cjk_lite_fonts().iter())
        .chain(fret_fonts::emoji_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family_inter = "Inter";
    let family_cjk = "Noto Sans CJK SC";
    let family_emoji = "Noto Color Emoji";

    for family in [family_inter, family_cjk, family_emoji] {
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family)),
            "expected {family} to be present after loading bundled fonts"
        );
    }

    // Use Inter for the UI generic, and let common fallbacks handle mixed-script coverage.
    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![family_inter.to_string()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let expected_inter_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_inter),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("m", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_inter_faces.is_empty(),
        "expected at least one resolved face for the explicit {family_inter} family"
    );

    let expected_cjk_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_cjk),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("你", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_cjk_faces.is_empty(),
        "expected at least one resolved face for the explicit {family_cjk} family"
    );

    let expected_emoji_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_emoji),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("\u{1F600}", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    assert!(
        !expected_emoji_faces.is_empty(),
        "expected at least one resolved face for the explicit {family_emoji} family"
    );

    let style = TextStyle {
        font: fret_core::FontId::ui(),
        size: Px(24.0),
        ..Default::default()
    };
    let (blob_id, _metrics) = text.prepare("m你\u{1F600}", &style, constraints);
    let blob = text.blob(blob_id).expect("text blob");

    assert_eq!(
        blob.shape.missing_glyphs, 0,
        "expected mixed-script fallback to avoid tofu when system fonts are absent"
    );

    let used_faces: std::collections::HashSet<super::FontFaceKey> =
        blob.shape.glyphs.iter().map(|g| g.key.font).collect();
    assert!(
        used_faces.iter().any(|k| expected_inter_faces.contains(k)),
        "expected the UI stack to use {family_inter} for Latin glyphs"
    );
    assert!(
        used_faces.iter().any(|k| expected_cjk_faces.contains(k)),
        "expected the UI stack to use {family_cjk} (or its subset) for CJK glyphs"
    );
    assert!(
        used_faces.iter().any(|k| expected_emoji_faces.contains(k)),
        "expected the UI stack to use {family_emoji} for emoji glyphs"
    );

    assert!(
        blob.shape
            .glyphs
            .iter()
            .any(|g| g.kind() == super::GlyphQuadKind::Color),
        "expected at least one color glyph for emoji"
    );
}

#[test]
fn mixed_script_fallback_uses_bundled_faces_for_named_family_when_system_fonts_are_absent() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut text = super::TextSystem::new(&ctx.device);

    // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
    text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
    text.font_runtime.fallback_policy = super::TextFallbackPolicyV1::new(&text.parley_shaper);
    let _ = text.parley_shaper.set_common_fallback_stack_suffix(
        text.font_runtime
            .fallback_policy
            .common_fallback_stack_suffix
            .clone(),
    );
    text.font_runtime.generic_injections.clear();
    text.font_runtime.font_db_revision = 0;
    text.font_runtime.font_stack_key = 0;

    let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
        .iter()
        .chain(fret_fonts::cjk_lite_fonts().iter())
        .chain(fret_fonts::emoji_fonts().iter())
        .map(|b| b.to_vec())
        .collect();
    let added = text.add_fonts(fonts);
    assert!(added > 0, "expected bundled fonts to load");

    let family_inter = "Inter";
    let family_cjk = "Noto Sans CJK SC";
    let family_emoji = "Noto Color Emoji";

    for family in [family_inter, family_cjk, family_emoji] {
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family)),
            "expected {family} to be present after loading bundled fonts"
        );
    }

    let config = fret_core::TextFontFamilyConfig {
        ui_sans: vec![family_inter.to_string()],
        ..Default::default()
    };
    let _ = text.set_font_families(&config);

    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: 1.0,
    };

    let expected_inter_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_inter),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("m", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    let expected_cjk_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_cjk),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("你", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };
    let expected_emoji_faces = {
        let style = TextStyle {
            font: fret_core::FontId::family(family_emoji),
            size: Px(24.0),
            ..Default::default()
        };
        let (blob_id, _metrics) = text.prepare("\u{1F600}", &style, constraints);
        let blob = text.blob(blob_id).expect("text blob");
        blob.shape
            .glyphs
            .iter()
            .map(|g| g.key.font)
            .collect::<std::collections::HashSet<super::FontFaceKey>>()
    };

    let style = TextStyle {
        font: fret_core::FontId::family(family_inter),
        size: Px(24.0),
        ..Default::default()
    };
    let (blob_id, _metrics) = text.prepare("m你\u{1F600}", &style, constraints);
    let blob = text.blob(blob_id).expect("text blob");

    assert_eq!(
        blob.shape.missing_glyphs, 0,
        "expected named-family stack to avoid tofu when system fonts are absent"
    );

    let used_faces: std::collections::HashSet<super::FontFaceKey> =
        blob.shape.glyphs.iter().map(|g| g.key.font).collect();
    assert!(
        used_faces.iter().any(|k| expected_inter_faces.contains(k)),
        "expected the stack to use {family_inter} for Latin glyphs"
    );
    assert!(
        used_faces.iter().any(|k| expected_cjk_faces.contains(k)),
        "expected the stack to use {family_cjk} (or its subset) for CJK glyphs"
    );
    assert!(
        used_faces.iter().any(|k| expected_emoji_faces.contains(k)),
        "expected the stack to use {family_emoji} for emoji glyphs"
    );
}
