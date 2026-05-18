pub(super) fn startup_parley_shaper() -> fret_render_text::ParleyShaper {
    #[cfg(target_arch = "wasm32")]
    {
        // Web/WASM has no truthful system-font capability today, so bootstrap must be bundled-only.
        fret_render_text::ParleyShaper::new_without_system_fonts()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        fret_render_text::ParleyShaper::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_parley_shaper_matches_platform_contract() {
        let shaper = super::startup_parley_shaper();

        #[cfg(target_arch = "wasm32")]
        assert!(
            !shaper.system_fonts_enabled(),
            "wasm bootstrap must stay bundled-only"
        );

        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            shaper.system_fonts_enabled(),
            fret_render_text::ParleyShaper::new().system_fonts_enabled(),
            "native bootstrap should continue to follow the existing default constructor contract"
        );
    }
}
