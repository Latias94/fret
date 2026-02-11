use fret_core::{TextCommonFallbackInjection, TextFontFamilyConfig};

#[test]
fn text_font_family_config_json_roundtrips() {
    let config = TextFontFamilyConfig {
        ui_sans: vec!["Inter".to_string(), "Segoe UI".to_string()],
        ui_serif: vec!["Noto Serif".to_string()],
        ui_mono: vec!["Cascadia Mono".to_string()],
        common_fallback_injection: TextCommonFallbackInjection::CommonFallback,
        common_fallback: vec![
            "Noto Sans CJK SC".to_string(),
            "Noto Color Emoji".to_string(),
        ],
    };

    let json = serde_json::to_string(&config).expect("serialize");
    let decoded: TextFontFamilyConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, config);
}
