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

#[test]
fn ai_artifact_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/artifact_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\"",
        "decl_text::text_control_readout(cx, \"Artifact closed.\")",
        "decl_text::text_section_chrome_label(cx, \"Artifact (AI Elements)\")",
        "decl_text::text_paragraph(cx, \"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "artifact_demo should route visible text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\")",
        "cx.text(\"Artifact closed.\")",
        "cx.text(\"Artifact (AI Elements)\")",
        "cx.text(\"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "artifact_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_code_block_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/code_block_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: String) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"CodeBlock (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "code_block_demo should route visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"\")",
        "cx.text(\"CodeBlock (AI Elements)\")",
        "cx.text(\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "code_block_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_sandbox_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/sandbox_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Sandbox console output (demo).\")",
        "decl_text::text_paragraph(cx, \"Apps own execution backends; this is UI-only.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox files view (demo).\")",
        "decl_text::text_paragraph(cx, \"Tabs are composable; provide your own panels.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Collapsible + tabs chrome. Apps own the sandbox backend.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "sandbox_demo should route visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Sandbox console output (demo).\")",
        "cx.text(\"Apps own execution backends; this is UI-only.\")",
        "cx.text(\"Sandbox files view (demo).\")",
        "cx.text(\"Tabs are composable; provide your own panels.\")",
        "cx.text(\"Sandbox (AI Elements)\")",
        "cx.text(\"Collapsible + tabs chrome. Apps own the sandbox backend.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "sandbox_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}
