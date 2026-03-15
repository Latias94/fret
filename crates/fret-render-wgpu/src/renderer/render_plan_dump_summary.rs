use super::render_plan::{PlanTarget, RenderPlanPass};

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpCustomEffectSummary {
    effect: String,
    abi: &'static str,
    pass_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user0_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user0_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user1_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user1_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_requested: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_distinct: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_aliased: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_requested: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_applied_levels_ge2: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_degraded_to_one: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_levels_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_levels_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_levels_sum: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_build_scissor_some: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpCustomEffectV3SourceDegradations {
    raw_requested: u64,
    raw_distinct: u64,
    raw_aliased_to_src: u64,
    pyramid_requested: u64,
    pyramid_applied_levels_ge2: u64,
    pyramid_degraded_to_one_budget_zero: u64,
    pyramid_degraded_to_one_budget_insufficient: u64,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpBackdropSourceGroupDegradations {
    requested: u64,
    applied_raw: u64,
    raw_degraded_budget_zero: u64,
    raw_degraded_budget_insufficient: u64,
    raw_degraded_target_exhausted: u64,
    pyramid_requested: u64,
    pyramid_applied_levels_ge2: u64,
    pyramid_degraded_to_one_budget_zero: u64,
    pyramid_degraded_to_one_budget_insufficient: u64,
    pyramid_skipped_raw_unavailable: u64,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpCustomEffectV3DiagnosticsSummary {
    custom_effect_v3_sources: JsonDumpCustomEffectV3SourceDegradations,
    backdrop_source_groups: JsonDumpBackdropSourceGroupDegradations,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpTargetUsage {
    target: &'static str,
    max_size: [u32; 2],
    src_uses: usize,
    dst_uses: usize,
    mask_uses: usize,
}

pub(super) fn summarize_custom_effects(
    passes: &[RenderPlanPass],
) -> Vec<JsonDumpCustomEffectSummary> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Abi {
        V1,
        V2,
        V3,
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct Acc {
        pass_count: usize,
        input_image_some: usize,
        input_image_none: usize,
        user0_image_some: usize,
        user0_image_none: usize,
        user1_image_some: usize,
        user1_image_none: usize,
        raw_requested: usize,
        raw_distinct: usize,
        raw_aliased: usize,
        pyramid_requested: usize,
        pyramid_applied_levels_ge2: usize,
        pyramid_degraded_to_one: usize,
        pyramid_levels_min: u32,
        pyramid_levels_max: u32,
        pyramid_levels_sum: u64,
        pyramid_build_scissor_some: usize,
    }

    let mut by_effect: std::collections::HashMap<(fret_core::EffectId, Abi), Acc> =
        std::collections::HashMap::new();
    for p in passes {
        match p {
            RenderPlanPass::CustomEffect(p) => {
                let acc = by_effect.entry((p.common.effect, Abi::V1)).or_default();
                acc.pass_count += 1;
            }
            RenderPlanPass::CustomEffectV2(p) => {
                let acc = by_effect.entry((p.common.effect, Abi::V2)).or_default();
                acc.pass_count += 1;
                if p.input_image.is_some() {
                    acc.input_image_some += 1;
                } else {
                    acc.input_image_none += 1;
                }
            }
            RenderPlanPass::CustomEffectV3(p) => {
                let acc = by_effect.entry((p.common.effect, Abi::V3)).or_default();
                acc.pass_count += 1;
                if p.user0_image.is_some() {
                    acc.user0_image_some += 1;
                } else {
                    acc.user0_image_none += 1;
                }
                if p.user1_image.is_some() {
                    acc.user1_image_some += 1;
                } else {
                    acc.user1_image_none += 1;
                }
                if p.raw_wanted {
                    acc.raw_requested += 1;
                    if p.src_raw == p.common.src {
                        acc.raw_aliased += 1;
                    } else {
                        acc.raw_distinct += 1;
                    }
                }
                if p.pyramid_wanted {
                    acc.pyramid_requested += 1;
                    if p.pyramid_levels <= 1 {
                        acc.pyramid_degraded_to_one += 1;
                    } else {
                        acc.pyramid_applied_levels_ge2 += 1;
                    }

                    let levels = p.pyramid_levels.max(1);
                    acc.pyramid_levels_sum = acc.pyramid_levels_sum.saturating_add(levels as u64);
                    acc.pyramid_levels_max = acc.pyramid_levels_max.max(levels);
                    if acc.pyramid_levels_min == 0 {
                        acc.pyramid_levels_min = levels;
                    } else {
                        acc.pyramid_levels_min = acc.pyramid_levels_min.min(levels);
                    }
                    if p.pyramid_build_scissor.is_some() {
                        acc.pyramid_build_scissor_some += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let mut out: Vec<_> = by_effect
        .into_iter()
        .map(|((effect, abi), acc)| JsonDumpCustomEffectSummary {
            effect: format!("{effect:?}"),
            abi: match abi {
                Abi::V1 => "custom_v1.params_only",
                Abi::V2 => "custom_v2.user_image",
                Abi::V3 => "custom_v3.renderer_sources",
            },
            pass_count: acc.pass_count,
            input_image_some: (abi == Abi::V2).then_some(acc.input_image_some),
            input_image_none: (abi == Abi::V2).then_some(acc.input_image_none),
            user0_image_some: (abi == Abi::V3).then_some(acc.user0_image_some),
            user0_image_none: (abi == Abi::V3).then_some(acc.user0_image_none),
            user1_image_some: (abi == Abi::V3).then_some(acc.user1_image_some),
            user1_image_none: (abi == Abi::V3).then_some(acc.user1_image_none),
            raw_requested: (abi == Abi::V3).then_some(acc.raw_requested),
            raw_distinct: (abi == Abi::V3).then_some(acc.raw_distinct),
            raw_aliased: (abi == Abi::V3).then_some(acc.raw_aliased),
            pyramid_requested: (abi == Abi::V3).then_some(acc.pyramid_requested),
            pyramid_applied_levels_ge2: (abi == Abi::V3).then_some(acc.pyramid_applied_levels_ge2),
            pyramid_degraded_to_one: (abi == Abi::V3).then_some(acc.pyramid_degraded_to_one),
            pyramid_levels_min: (abi == Abi::V3 && acc.pyramid_requested > 0)
                .then_some(acc.pyramid_levels_min),
            pyramid_levels_max: (abi == Abi::V3 && acc.pyramid_requested > 0)
                .then_some(acc.pyramid_levels_max),
            pyramid_levels_sum: (abi == Abi::V3 && acc.pyramid_requested > 0)
                .then_some(acc.pyramid_levels_sum),
            pyramid_build_scissor_some: (abi == Abi::V3).then_some(acc.pyramid_build_scissor_some),
        })
        .collect();

    out.sort_by(|a, b| (a.abi, &a.effect).cmp(&(b.abi, &b.effect)));
    out
}

fn plan_target_index(t: PlanTarget) -> usize {
    match t {
        PlanTarget::Output => 0,
        PlanTarget::Intermediate0 => 1,
        PlanTarget::Intermediate1 => 2,
        PlanTarget::Intermediate2 => 3,
        PlanTarget::Intermediate3 => 4,
        PlanTarget::Mask0 => 5,
        PlanTarget::Mask1 => 6,
        PlanTarget::Mask2 => 7,
    }
}

fn bump_usage(
    usage: &mut [Option<JsonDumpTargetUsage>; 8],
    target: PlanTarget,
    kind: &str,
    size: (u32, u32),
) {
    let slot = &mut usage[plan_target_index(target)];
    let entry = slot.get_or_insert_with(|| JsonDumpTargetUsage {
        target: super::render_plan_dump_encode::plan_target_name(target),
        max_size: [0, 0],
        src_uses: 0,
        dst_uses: 0,
        mask_uses: 0,
    });

    entry.max_size[0] = entry.max_size[0].max(size.0.max(1));
    entry.max_size[1] = entry.max_size[1].max(size.1.max(1));
    match kind {
        "src" => entry.src_uses += 1,
        "dst" => entry.dst_uses += 1,
        "mask" => entry.mask_uses += 1,
        _ => {}
    }
}

pub(super) fn summarize_target_usage(passes: &[RenderPlanPass]) -> Vec<JsonDumpTargetUsage> {
    let mut usage: [Option<JsonDumpTargetUsage>; 8] = std::array::from_fn(|_| None);

    for p in passes {
        match p {
            RenderPlanPass::SceneDrawRange(pass) => {
                bump_usage(&mut usage, pass.target, "dst", pass.target_size);
            }
            RenderPlanPass::PathMsaaBatch(pass) => {
                bump_usage(&mut usage, pass.target, "dst", pass.target_size);
            }
            RenderPlanPass::PathClipMask(pass) => {
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::FullscreenBlit(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::CompositePremul(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ScaleNearest(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Blur(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::BackdropWarp(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ColorAdjust(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ColorMatrix(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::AlphaThreshold(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Dither(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Noise(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::DropShadow(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffect(pass) => {
                bump_usage(&mut usage, pass.common.src, "src", pass.common.src_size);
                bump_usage(&mut usage, pass.common.dst, "dst", pass.common.dst_size);
                if let Some(mask) = pass.common.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffectV2(pass) => {
                bump_usage(&mut usage, pass.common.src, "src", pass.common.src_size);
                bump_usage(&mut usage, pass.common.dst, "dst", pass.common.dst_size);
                if let Some(mask) = pass.common.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffectV3(pass) => {
                bump_usage(&mut usage, pass.common.src, "src", pass.common.src_size);
                bump_usage(&mut usage, pass.src_raw, "src", pass.common.src_size);
                bump_usage(&mut usage, pass.src_pyramid, "src", pass.common.src_size);
                bump_usage(&mut usage, pass.common.dst, "dst", pass.common.dst_size);
                if let Some(mask) = pass.common.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ClipMask(pass) => {
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::ReleaseTarget(_) => {}
        }
    }

    let mut out: Vec<_> = usage.into_iter().flatten().collect();
    out.sort_by(|a, b| a.target.cmp(b.target));
    out
}

pub(super) fn encode_custom_effect_v3_diagnostics_summary(
    snapshot: super::EffectDegradationSnapshot,
) -> JsonDumpCustomEffectV3DiagnosticsSummary {
    JsonDumpCustomEffectV3DiagnosticsSummary {
        custom_effect_v3_sources: JsonDumpCustomEffectV3SourceDegradations {
            raw_requested: snapshot.custom_effect_v3_sources.raw_requested,
            raw_distinct: snapshot.custom_effect_v3_sources.raw_distinct,
            raw_aliased_to_src: snapshot.custom_effect_v3_sources.raw_aliased_to_src,
            pyramid_requested: snapshot.custom_effect_v3_sources.pyramid_requested,
            pyramid_applied_levels_ge2: snapshot
                .custom_effect_v3_sources
                .pyramid_applied_levels_ge2,
            pyramid_degraded_to_one_budget_zero: snapshot
                .custom_effect_v3_sources
                .pyramid_degraded_to_one_budget_zero,
            pyramid_degraded_to_one_budget_insufficient: snapshot
                .custom_effect_v3_sources
                .pyramid_degraded_to_one_budget_insufficient,
        },
        backdrop_source_groups: JsonDumpBackdropSourceGroupDegradations {
            requested: snapshot.backdrop_source_groups.requested,
            applied_raw: snapshot.backdrop_source_groups.applied_raw,
            raw_degraded_budget_zero: snapshot.backdrop_source_groups.raw_degraded_budget_zero,
            raw_degraded_budget_insufficient: snapshot
                .backdrop_source_groups
                .raw_degraded_budget_insufficient,
            raw_degraded_target_exhausted: snapshot
                .backdrop_source_groups
                .raw_degraded_target_exhausted,
            pyramid_requested: snapshot.backdrop_source_groups.pyramid_requested,
            pyramid_applied_levels_ge2: snapshot.backdrop_source_groups.pyramid_applied_levels_ge2,
            pyramid_degraded_to_one_budget_zero: snapshot
                .backdrop_source_groups
                .pyramid_degraded_to_one_budget_zero,
            pyramid_degraded_to_one_budget_insufficient: snapshot
                .backdrop_source_groups
                .pyramid_degraded_to_one_budget_insufficient,
            pyramid_skipped_raw_unavailable: snapshot
                .backdrop_source_groups
                .pyramid_skipped_raw_unavailable,
        },
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::renderer::ScissorRect;
    use crate::renderer::render_plan::{
        CustomEffectPass, CustomEffectPassCommon, CustomEffectV2Pass, CustomEffectV3Pass,
        LocalScissorRect,
    };
    use slotmap::SlotMap;

    #[test]
    fn custom_effect_summaries_include_abi_and_input_counts() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect_v1 = effects.insert(());
        let effect_v2 = effects.insert(());

        let mut images: SlotMap<fret_core::ImageId, ()> = SlotMap::with_key();
        let image = images.insert(());

        let passes = vec![
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common: CustomEffectPassCommon {
                    src: PlanTarget::Output,
                    dst: PlanTarget::Intermediate0,
                    src_size: (10, 11),
                    dst_size: (12, 13),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect: effect_v1,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
            }),
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common: CustomEffectPassCommon {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Intermediate1,
                    src_size: (12, 13),
                    dst_size: (20, 21),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect: effect_v2,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
                input_image: Some(image),
                input_uv: fret_core::scene::UvRect::FULL,
                input_sampling: fret_core::scene::ImageSamplingHint::Linear,
            }),
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common: CustomEffectPassCommon {
                    src: PlanTarget::Intermediate1,
                    dst: PlanTarget::Output,
                    src_size: (20, 21),
                    dst_size: (30, 31),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect: effect_v2,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
                input_image: None,
                input_uv: fret_core::scene::UvRect::FULL,
                input_sampling: fret_core::scene::ImageSamplingHint::Nearest,
            }),
        ];

        let summary = summarize_custom_effects(&passes);
        assert_eq!(summary.len(), 2);

        let v1 = summary
            .iter()
            .find(|s| s.abi == "custom_v1.params_only")
            .expect("v1 summary");
        assert_eq!(v1.pass_count, 1);
        assert_eq!(v1.input_image_some, None);
        assert_eq!(v1.input_image_none, None);

        let v2 = summary
            .iter()
            .find(|s| s.abi == "custom_v2.user_image")
            .expect("v2 summary");
        assert_eq!(v2.pass_count, 2);
        assert_eq!(v2.input_image_some, Some(1));
        assert_eq!(v2.input_image_none, Some(1));
    }

    #[test]
    fn custom_effect_v3_summary_tracks_pyramid_levels_min_max_sum() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect_v3 = effects.insert(());

        let passes = vec![
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                src_raw: PlanTarget::Intermediate1,
                src_pyramid: PlanTarget::Intermediate2,
                pyramid_levels: 1,
                pyramid_build_scissor: None,
                raw_wanted: true,
                pyramid_wanted: true,
                common: CustomEffectPassCommon {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Intermediate0,
                    src_size: (100, 100),
                    dst_size: (100, 100),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect: effect_v3,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
                user0_image: None,
                user0_uv: fret_core::scene::UvRect::FULL,
                user0_sampling: fret_core::scene::ImageSamplingHint::Default,
                user1_image: None,
                user1_uv: fret_core::scene::UvRect::FULL,
                user1_sampling: fret_core::scene::ImageSamplingHint::Default,
            }),
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                src_raw: PlanTarget::Intermediate1,
                src_pyramid: PlanTarget::Intermediate2,
                pyramid_levels: 3,
                pyramid_build_scissor: Some(LocalScissorRect(ScissorRect {
                    x: 10,
                    y: 20,
                    w: 30,
                    h: 40,
                })),
                raw_wanted: true,
                pyramid_wanted: true,
                common: CustomEffectPassCommon {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Intermediate0,
                    src_size: (100, 100),
                    dst_size: (100, 100),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect: effect_v3,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
                user0_image: None,
                user0_uv: fret_core::scene::UvRect::FULL,
                user0_sampling: fret_core::scene::ImageSamplingHint::Default,
                user1_image: None,
                user1_uv: fret_core::scene::UvRect::FULL,
                user1_sampling: fret_core::scene::ImageSamplingHint::Default,
            }),
        ];

        let summary = summarize_custom_effects(&passes);
        assert_eq!(summary.len(), 1);

        let v3 = summary
            .iter()
            .find(|s| s.abi == "custom_v3.renderer_sources")
            .expect("v3 summary");
        assert_eq!(v3.pass_count, 2);
        assert_eq!(v3.pyramid_requested, Some(2));
        assert_eq!(v3.pyramid_applied_levels_ge2, Some(1));
        assert_eq!(v3.pyramid_degraded_to_one, Some(1));
        assert_eq!(v3.pyramid_levels_min, Some(1));
        assert_eq!(v3.pyramid_levels_max, Some(3));
        assert_eq!(v3.pyramid_levels_sum, Some(4));
        assert_eq!(v3.pyramid_build_scissor_some, Some(1));
    }

    #[test]
    fn target_usage_tracks_max_size() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect = effects.insert(());

        let passes = vec![
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common: CustomEffectPassCommon {
                    src: PlanTarget::Output,
                    dst: PlanTarget::Intermediate0,
                    src_size: (100, 100),
                    dst_size: (10, 11),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
            }),
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common: CustomEffectPassCommon {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Intermediate0,
                    src_size: (20, 21),
                    dst_size: (22, 23),
                    dst_scissor: None,
                    mask_uniform_index: None,
                    mask: None,
                    effect,
                    params: fret_core::EffectParamsV1::ZERO,
                    load: wgpu::LoadOp::Load,
                },
            }),
        ];

        let usage = summarize_target_usage(&passes);
        let i0 = usage
            .iter()
            .find(|u| u.target == "Intermediate0")
            .expect("Intermediate0 usage");
        assert_eq!(i0.max_size, [22, 23]);
        assert!(i0.src_uses >= 1);
        assert!(i0.dst_uses >= 1);
    }
}
