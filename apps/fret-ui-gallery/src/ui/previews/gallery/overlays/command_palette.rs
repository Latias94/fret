use super::super::super::super::*;

pub(in crate::ui) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_command_palette(cx, open, query, last_action)
}

pub(super) fn sonner_position_key(position: shadcn::ToastPosition) -> &'static str {
    match position {
        shadcn::ToastPosition::TopLeft => "top-left",
        shadcn::ToastPosition::TopCenter => "top-center",
        shadcn::ToastPosition::TopRight => "top-right",
        shadcn::ToastPosition::BottomLeft => "bottom-left",
        shadcn::ToastPosition::BottomCenter => "bottom-center",
        shadcn::ToastPosition::BottomRight => "bottom-right",
    }
}
