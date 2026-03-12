use fret_app::App;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_shadcn::facade as shadcn;

use crate::ui;

pub(super) fn toaster_view(
    cx: &mut ElementContext<'_, App>,
    models: &ui::UiGalleryModels,
    disabled: bool,
) -> AnyElement {
    if disabled {
        return cx.text("");
    }

    let position = cx
        .get_model_copied(&models.sonner_position, Invalidation::Layout)
        .unwrap_or(shadcn::ToastPosition::TopCenter);
    shadcn::Toaster::new()
        .position(position)
        .shadcn_lucide_icons()
        .into_element(cx)
}
