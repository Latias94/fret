use super::draw_scope::DrawScope;
use crate::renderer::PlanTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TargetSelection {
    pub(super) target: Option<PlanTarget>,
    pub(super) had_free_target: bool,
}

pub(super) fn choose_free_intermediate_target(
    draw_scopes: &[DrawScope],
    reserved_targets: &[PlanTarget],
) -> TargetSelection {
    choose_free_intermediate_target_except(draw_scopes, reserved_targets, None)
}

pub(super) fn choose_free_intermediate_target_except(
    draw_scopes: &[DrawScope],
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
            || draw_scopes.iter().any(|scope| scope.target == target)
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
    draw_scopes: &[DrawScope],
    reserved_targets: &[PlanTarget],
    excluded: PlanTarget,
) -> bool {
    choose_free_intermediate_target_except(draw_scopes, reserved_targets, Some(excluded))
        .had_free_target
}
