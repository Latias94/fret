use super::*;

use fret_core::Px;

#[test]
fn glass_effect_chain_disables_effects_when_reduced_transparency_is_true() {
    let effect = ResolvedGlassEffect {
        blur_radius_px: Px(12.0),
        blur_downsample: 2,
        saturation: 1.1,
        brightness: 1.0,
        contrast: 1.0,
    };

    let disabled = glass_effect_chain_for_environment(effect, true);
    assert!(disabled.is_empty());

    let enabled = glass_effect_chain_for_environment(effect, false);
    assert!(!enabled.is_empty());
}
