fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

fn assert_custom_effect_overlay_text_roles(source: &str, label: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fncustom_effect_label_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(srgb(255,255,255,0.92))",
        "lettitle=custom_effect_label_text(cx,label.clone());",
    ] {
        assert!(
            source.contains(needle),
            "{label} should keep fixed overlay labels on shared chrome text roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{layout:Default::default(),text:label.clone()",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "{label} should not render overlay labels with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_custom_effect_v2_web_overlay_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnoverlay_label_text<H:UiHost>(",
        "fnoverlay_readout_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(Self::srgb(255,255,255,0.92))",
        "decl_text::text_control_readout(cx,text).inherit_foreground(foreground)",
        "Self::overlay_readout_text(cx,\"CustomV2unsupportedonthisadapter/backend\",theme.color_token(\"muted_foreground\"),)",
        "letbadge_text=Self::overlay_label_text(cx,\"CustomEffectV2(WebGPU)\");",
        "Self::overlay_readout_text(cx,\"PressVtotogglethedemosurface.PressRtoresetcontrols.\",Self::with_alpha(theme.color_token(\"foreground\"),0.55),)",
    ] {
        assert!(
            source.contains(needle),
            "custom_effect_v2_web_demo should keep fixed overlay/readout text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "custom_effect_v2_web_demo should not render overlay/readout text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

#[test]
fn custom_effect_v1_v2_overlay_labels_use_shared_chrome_role() {
    assert_custom_effect_overlay_text_roles(
        include_str!("../src/custom_effect_v1_demo.rs"),
        "custom_effect_v1_demo",
    );
    assert_custom_effect_overlay_text_roles(
        include_str!("../src/custom_effect_v2_demo.rs"),
        "custom_effect_v2_demo",
    );
}

#[test]
fn custom_effect_v2_web_overlay_readouts_use_shared_roles() {
    assert_custom_effect_v2_web_overlay_text_roles(include_str!(
        "../src/custom_effect_v2_web_demo.rs"
    ));
}
