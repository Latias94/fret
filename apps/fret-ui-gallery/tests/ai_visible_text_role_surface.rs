fn canonicalize_rust_fragment(fragment: &str) -> String {
    fragment.split_whitespace().collect()
}

#[test]
fn ai_message_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/message_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, format!(\"last_action={last_action}\"))",
        "decl_text::text_paragraph",
        "\"User messages render as a bubble aligned to the right.\"",
        "decl_text::text_paragraph(cx, \"Bubble chrome is controlled by theme tokens.\")",
        "decl_text::text_section_chrome_label",
        "\"Message (AI Elements): alignment + bubble + actions + markdown response.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "message_demo should route fixed visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(format!(\"last_action={last_action}\"))",
        "cx.text(\"User messages render as a bubble aligned to the right.\")",
        "cx.text(\"Bubble chrome is controlled by theme tokens.\")",
        "cx.text(\"Message (AI Elements): alignment + bubble + actions + markdown response.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "message_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_terminal_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/terminal_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"Terminal (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Chrome-only viewer: apps own streaming + clear behavior.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "terminal_demo should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"Terminal (AI Elements)\")",
        "cx.text(\"Chrome-only viewer: apps own streaming + clear behavior.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "terminal_demo reintroduced bare visible text/state marker semantics: `{forbidden}`"
        );
    }
}
