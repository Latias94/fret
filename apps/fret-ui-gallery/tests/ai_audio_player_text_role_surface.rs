fn canonicalize_rust_fragment(fragment: &str) -> String {
    fragment.split_whitespace().collect()
}

#[test]
fn audio_player_state_markers_use_non_text_spacers() {
    for (name, source) in [
        (
            "audio_player_demo",
            include_str!("../src/ui/snippets/ai/audio_player_demo.rs"),
        ),
        (
            "audio_player_remote_demo",
            include_str!("../src/ui/snippets/ai/audio_player_remote_demo.rs"),
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: String) -> AnyElement",
            "role: fret_core::SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            "width: Length::Px(fret_core::Px(0.0))",
            "height: Length::Px(fret_core::Px(0.0))",
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should expose state-only diagnostics anchors as non-text spacers; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: fret_core::SemanticsRole::Text",
            "vec![cx.text(\"\")]",
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced empty text state markers: `{forbidden}`"
            );
        }
    }
}
