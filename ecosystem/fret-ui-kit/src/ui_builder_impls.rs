use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::aspect_ratio::AspectRatio;
use crate::primitives::label::Label;
use crate::primitives::separator::Separator;
use crate::{IntoUiElement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout};

impl UiPatchTarget for Label {
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost> IntoUiElement<H> for Label {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Label::into_element(self, cx)
    }
}

impl UiPatchTarget for Separator {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl UiSupportsLayout for Separator {}

impl<H: UiHost> IntoUiElement<H> for Separator {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Separator::into_element(self, cx)
    }
}

impl UiPatchTarget for AspectRatio {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl UiSupportsChrome for AspectRatio {}
impl UiSupportsLayout for AspectRatio {}

impl<H: UiHost> IntoUiElement<H> for AspectRatio {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AspectRatio::into_element(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutRefinement;
    use crate::UiExt as _;

    #[test]
    fn ui_builder_supports_selected_primitives() {
        let _ = Label::new("x").ui().build();
        let _ = Separator::new()
            .ui()
            .layout(LayoutRefinement::default())
            .build();
        let _ = Separator::new().ui().w_full().build();

        // Compile-only smoke: `AspectRatio` stays compatible with the UI patch/builder surface.
        fn assert_aspect_ratio_builds(ar: AspectRatio) {
            let _ = ar.ui().p(crate::Space::N4).w_full().build();
        }
        let _ = assert_aspect_ratio_builds;
    }
}
