use super::draw_scope::DrawScopeStack;
use super::target_budget::can_allocate_intermediate_bytes;
use crate::renderer::PlanTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TargetSelection {
    pub(super) target: Option<PlanTarget>,
    pub(super) had_free_target: bool,
}

impl TargetSelection {
    pub(super) fn none() -> Self {
        Self {
            target: None,
            had_free_target: false,
        }
    }
}

pub(super) fn choose_free_intermediate_target(
    draw_scopes: &DrawScopeStack,
    reserved_targets: &[PlanTarget],
) -> TargetSelection {
    choose_free_intermediate_target_except(draw_scopes, reserved_targets, None)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IntermediateAllocationBudget {
    pub(super) intermediate_budget_bytes: u64,
    pub(super) required_bytes: u64,
    pub(super) extra_in_use_bytes: u64,
    pub(super) format: wgpu::TextureFormat,
}

pub(super) fn choose_budgeted_intermediate_target(
    draw_scopes: &DrawScopeStack,
    reserved_targets: &[PlanTarget],
    budget: IntermediateAllocationBudget,
) -> TargetSelection {
    budget_filter_intermediate_target(
        choose_free_intermediate_target(draw_scopes, reserved_targets),
        draw_scopes,
        budget,
    )
}

pub(super) fn budget_filter_intermediate_target(
    mut selection: TargetSelection,
    draw_scopes: &DrawScopeStack,
    budget: IntermediateAllocationBudget,
) -> TargetSelection {
    if selection.target.is_some()
        && !can_allocate_intermediate_bytes(
            budget.intermediate_budget_bytes,
            draw_scopes,
            budget.required_bytes,
            budget.extra_in_use_bytes,
            budget.format,
        )
    {
        selection.target = None;
    }
    selection
}

pub(super) fn choose_free_intermediate_target_except(
    draw_scopes: &DrawScopeStack,
    reserved_targets: &[PlanTarget],
    excluded: Option<PlanTarget>,
) -> TargetSelection {
    for target in [
        PlanTarget::Intermediate0,
        PlanTarget::Intermediate1,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate3,
    ] {
        if excluded == Some(target)
            || draw_scopes.contains_target(target)
            || reserved_targets.contains(&target)
        {
            continue;
        }
        return TargetSelection {
            target: Some(target),
            had_free_target: true,
        };
    }

    TargetSelection {
        target: None,
        had_free_target: false,
    }
}

pub(super) fn has_free_intermediate_target_except(
    draw_scopes: &DrawScopeStack,
    reserved_targets: &[PlanTarget],
    excluded: PlanTarget,
) -> bool {
    choose_free_intermediate_target_except(draw_scopes, reserved_targets, Some(excluded))
        .had_free_target
}

pub(super) fn choose_free_clip_path_mask_target(
    mut is_target_in_use: impl FnMut(PlanTarget) -> bool,
) -> TargetSelection {
    for target in [PlanTarget::Mask0, PlanTarget::Mask1, PlanTarget::Mask2] {
        if is_target_in_use(target) {
            continue;
        }
        return TargetSelection {
            target: Some(target),
            had_free_target: true,
        };
    }

    TargetSelection {
        target: None,
        had_free_target: false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw_scope::DrawScope;
    use super::*;

    fn scope(target: PlanTarget) -> DrawScope {
        DrawScope {
            target,
            origin: (0, 0),
            size: (1, 1),
            needs_clear: false,
            clear_color: wgpu::Color::TRANSPARENT,
        }
    }

    #[test]
    fn intermediate_target_selection_preserves_order_and_exclusions() {
        let draw_scopes = DrawScopeStack::new(scope(PlanTarget::Intermediate0));
        let reserved_targets = [PlanTarget::Intermediate2];

        let selection = choose_free_intermediate_target(&draw_scopes, &reserved_targets);

        assert_eq!(selection.target, Some(PlanTarget::Intermediate1));
        assert!(selection.had_free_target);
    }

    #[test]
    fn intermediate_target_selection_respects_explicit_exclusion() {
        let draw_scopes = DrawScopeStack::new(scope(PlanTarget::Intermediate0));
        let reserved_targets = [PlanTarget::Intermediate2];

        let selection = choose_free_intermediate_target_except(
            &draw_scopes,
            &reserved_targets,
            Some(PlanTarget::Intermediate1),
        );

        assert_eq!(selection.target, Some(PlanTarget::Intermediate3));
        assert!(selection.had_free_target);
    }

    #[test]
    fn budgeted_intermediate_target_preserves_free_target_evidence_when_budget_blocks() {
        let draw_scopes = DrawScopeStack::new(scope(PlanTarget::Intermediate0));
        let selection = choose_budgeted_intermediate_target(
            &draw_scopes,
            &[],
            IntermediateAllocationBudget {
                intermediate_budget_bytes: 0,
                required_bytes: 1,
                extra_in_use_bytes: 0,
                format: wgpu::TextureFormat::Rgba8Unorm,
            },
        );

        assert_eq!(selection.target, None);
        assert!(
            selection.had_free_target,
            "budget filtering must not turn budget pressure into target exhaustion"
        );
    }

    #[test]
    fn clip_path_mask_target_selection_uses_separate_mask_pool_order() {
        let in_use = [PlanTarget::Mask0, PlanTarget::Mask1];

        let selection = choose_free_clip_path_mask_target(|target| in_use.contains(&target));

        assert_eq!(selection.target, Some(PlanTarget::Mask2));
        assert!(selection.had_free_target);
    }

    #[test]
    fn clip_path_mask_target_selection_reports_exhaustion() {
        let in_use = [PlanTarget::Mask0, PlanTarget::Mask1, PlanTarget::Mask2];

        let selection = choose_free_clip_path_mask_target(|target| in_use.contains(&target));

        assert_eq!(selection.target, None);
        assert!(!selection.had_free_target);
    }
}
