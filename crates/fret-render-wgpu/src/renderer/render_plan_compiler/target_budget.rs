use super::super::render_plan_effects as effects;
use super::draw_scope::DrawScope;
use crate::renderer::{PlanTarget, estimate_texture_bytes};

pub(super) fn choose_backdrop_source_group_pyramid_choice(
    req: fret_core::scene::CustomEffectPyramidRequestV1,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    intermediate_budget_bytes: u64,
    in_use_intermediate_bytes: u64,
    clip_path_mask_in_use_bytes: u64,
    backdrop_source_group_in_use_bytes: u64,
    raw_bytes: u64,
) -> effects::CustomV3PyramidChoice {
    // `choose_custom_v3_pyramid_choice_for_request` expects:
    // - `budget_bytes`: total available headroom
    // - `base_required_bytes`: bytes that are already reserved within that headroom
    //
    // Here we pre-subtract all reservations (including `raw_bytes`) to produce a pure headroom
    // value, so `base_required_bytes` must be 0 to avoid double-counting.
    let headroom_after_raw = intermediate_budget_bytes.saturating_sub(
        in_use_intermediate_bytes
            .saturating_add(clip_path_mask_in_use_bytes)
            .saturating_add(backdrop_source_group_in_use_bytes)
            .saturating_add(raw_bytes),
    );

    effects::choose_custom_v3_pyramid_choice_for_request(
        req,
        viewport_size,
        format,
        headroom_after_raw,
        0,
    )
}

fn is_intermediate_target(t: PlanTarget) -> bool {
    matches!(
        t,
        PlanTarget::Intermediate0
            | PlanTarget::Intermediate1
            | PlanTarget::Intermediate2
            | PlanTarget::Intermediate3
    )
}

pub(super) fn estimate_in_use_intermediate_bytes(
    draw_scopes: &[DrawScope],
    format: wgpu::TextureFormat,
) -> u64 {
    draw_scopes
        .iter()
        .filter(|s| is_intermediate_target(s.target))
        .map(|s| estimate_texture_bytes(s.size, format, 1))
        .sum()
}

fn estimate_target_bytes_in_scopes(
    draw_scopes: &[DrawScope],
    target: PlanTarget,
    format: wgpu::TextureFormat,
) -> Option<u64> {
    draw_scopes
        .iter()
        .find(|s| s.target == target)
        .map(|s| estimate_texture_bytes(s.size, format, 1))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IntermediateBudgetBreakdown {
    pub(super) effective_budget_bytes: u64,
    pub(super) other_live_bytes: u64,
}

pub(super) fn intermediate_budget_breakdown_for_chain(
    intermediate_budget_bytes: u64,
    draw_scopes: &[DrawScope],
    srcdst: PlanTarget,
    format: wgpu::TextureFormat,
    extra_in_use_bytes: u64,
) -> IntermediateBudgetBreakdown {
    let in_use_intermediate_bytes = estimate_in_use_intermediate_bytes(draw_scopes, format);
    let srcdst_bytes = if is_intermediate_target(srcdst) {
        estimate_target_bytes_in_scopes(draw_scopes, srcdst, format).unwrap_or(0)
    } else {
        0
    };
    let other_live_bytes = in_use_intermediate_bytes
        .saturating_sub(srcdst_bytes)
        .saturating_add(extra_in_use_bytes);
    IntermediateBudgetBreakdown {
        effective_budget_bytes: intermediate_budget_bytes.saturating_sub(other_live_bytes),
        other_live_bytes,
    }
}

pub(super) fn can_allocate_intermediate_bytes(
    intermediate_budget_bytes: u64,
    draw_scopes: &[DrawScope],
    required_bytes: u64,
    extra_in_use_bytes: u64,
    format: wgpu::TextureFormat,
) -> bool {
    let in_use_bytes = estimate_in_use_intermediate_bytes(draw_scopes, format);
    in_use_bytes
        .saturating_add(extra_in_use_bytes)
        .saturating_add(required_bytes)
        <= intermediate_budget_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_source_group_pyramid_choice_does_not_double_count_raw_bytes() {
        let viewport_size = (64u32, 64u32);
        let format = wgpu::TextureFormat::Rgba8Unorm;

        let req = fret_core::scene::CustomEffectPyramidRequestV1 {
            max_levels: 7,
            max_radius_px: fret_core::geometry::Px(16.0),
        };

        let raw_bytes = estimate_texture_bytes(viewport_size, format, 1);
        let pyramid_bytes = effects::estimate_custom_v3_pyramid_bytes(viewport_size, format, 2);
        let intermediate_budget_bytes = raw_bytes.saturating_add(pyramid_bytes);

        let choice = choose_backdrop_source_group_pyramid_choice(
            req,
            viewport_size,
            format,
            intermediate_budget_bytes,
            0,
            0,
            0,
            raw_bytes,
        );

        assert_eq!(
            choice.levels, 2,
            "expected enough headroom for a 2-level pyramid after reserving raw bytes"
        );
        assert_eq!(
            choice.degraded_to_one, None,
            "expected no degradation when budget exactly fits the requested 2-level pyramid"
        );
    }

    #[test]
    fn effective_intermediate_budget_excludes_srcdst_bytes() {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let intermediate_budget_bytes = 1000u64;

        let draw_scopes = [
            DrawScope {
                target: PlanTarget::Intermediate0,
                origin: (0, 0),
                size: (4, 4),
                needs_clear: false,
                clear_color: wgpu::Color::TRANSPARENT,
            },
            DrawScope {
                target: PlanTarget::Intermediate1,
                origin: (0, 0),
                size: (2, 2),
                needs_clear: false,
                clear_color: wgpu::Color::TRANSPARENT,
            },
        ];

        let srcdst_bytes = estimate_texture_bytes((4, 4), format, 1);
        let in_use_bytes = estimate_texture_bytes((4, 4), format, 1)
            .saturating_add(estimate_texture_bytes((2, 2), format, 1));
        let extra_in_use_bytes = 7u64;

        let expected = intermediate_budget_bytes.saturating_sub(
            in_use_bytes
                .saturating_sub(srcdst_bytes)
                .saturating_add(extra_in_use_bytes),
        );

        let got = intermediate_budget_breakdown_for_chain(
            intermediate_budget_bytes,
            &draw_scopes,
            PlanTarget::Intermediate0,
            format,
            extra_in_use_bytes,
        )
        .effective_budget_bytes;
        assert_eq!(got, expected);
    }
}
