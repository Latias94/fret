use crate::ui::style::NodeGraphBackgroundPattern;

pub(super) fn validate_pattern_size(
    pattern: NodeGraphBackgroundPattern,
    dot_size: f32,
    cross_size: f32,
) -> bool {
    if matches!(pattern, NodeGraphBackgroundPattern::Dots)
        && !(dot_size.is_finite() && dot_size > 0.0)
    {
        return false;
    }
    if matches!(pattern, NodeGraphBackgroundPattern::Cross)
        && !(cross_size.is_finite() && cross_size > 0.0)
    {
        return false;
    }
    true
}
