macro_rules! impl_ui_patch_chrome_layout {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: ::fret_ui_kit::UiPatch) -> Self {
                self.refine_style(patch.chrome).refine_layout(patch.layout)
            }
        }

        impl ::fret_ui_kit::UiSupportsChrome for $ty {}
        impl ::fret_ui_kit::UiSupportsLayout for $ty {}

        impl ::fret_ui_kit::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

macro_rules! impl_ui_patch_layout_only {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: ::fret_ui_kit::UiPatch) -> Self {
                self.refine_layout(patch.layout)
            }
        }

        impl ::fret_ui_kit::UiSupportsLayout for $ty {}

        impl ::fret_ui_kit::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

macro_rules! impl_ui_patch_chrome_layout_patch_only {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: ::fret_ui_kit::UiPatch) -> Self {
                self.refine_style(patch.chrome).refine_layout(patch.layout)
            }
        }

        impl ::fret_ui_kit::UiSupportsChrome for $ty {}
        impl ::fret_ui_kit::UiSupportsLayout for $ty {}
    };
}

macro_rules! impl_ui_patch_layout_only_patch_only {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: ::fret_ui_kit::UiPatch) -> Self {
                self.refine_layout(patch.layout)
            }
        }

        impl ::fret_ui_kit::UiSupportsLayout for $ty {}
    };
}

macro_rules! impl_ui_patch_passthrough {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, _patch: ::fret_ui_kit::UiPatch) -> Self {
                self
            }
        }

        impl ::fret_ui_kit::UiIntoElement for $ty {
            #[track_caller]
            fn into_element<H: ::fret_ui::UiHost>(
                self,
                cx: &mut ::fret_ui::ElementContext<'_, H>,
            ) -> ::fret_ui::element::AnyElement {
                <$ty>::into_element(self, cx)
            }
        }
    };
}

macro_rules! impl_ui_patch_passthrough_patch_only {
    ($ty:ty) => {
        impl ::fret_ui_kit::UiPatchTarget for $ty {
            fn apply_ui_patch(self, _patch: ::fret_ui_kit::UiPatch) -> Self {
                self
            }
        }
    };
}
