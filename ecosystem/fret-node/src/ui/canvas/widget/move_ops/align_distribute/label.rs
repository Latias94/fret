use crate::ui::canvas::widget::move_ops::*;

pub(super) fn label_for_mode(mode: AlignDistributeMode) -> &'static str {
    match mode {
        AlignDistributeMode::AlignLeft => "Align Left",
        AlignDistributeMode::AlignRight => "Align Right",
        AlignDistributeMode::AlignTop => "Align Top",
        AlignDistributeMode::AlignBottom => "Align Bottom",
        AlignDistributeMode::AlignCenterX => "Align Center X",
        AlignDistributeMode::AlignCenterY => "Align Center Y",
        AlignDistributeMode::DistributeX => "Distribute X",
        AlignDistributeMode::DistributeY => "Distribute Y",
    }
}
