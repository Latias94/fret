use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Radius, ShadowPreset, Space, UiBuilder,
    UiSupportsChrome,
};

/// A policy-level preset extension for common shadcn surface styling.
pub trait SurfaceUiBuilderExt {
    /// Applies the standard shadcn popover-like surface chrome (panel background, border, radius,
    /// padding, and shadow).
    fn popover_style(self) -> Self;
}

pub(crate) fn popover_style_chrome() -> ChromeRefinement {
    ChromeRefinement::default()
        .rounded(Radius::Md)
        .border_1()
        .bg(ColorRef::Token {
            key: "popover.background",
            fallback: ColorFallback::ThemePanelBackground,
        })
        .border_color(ColorRef::Token {
            key: "border",
            fallback: ColorFallback::ThemePanelBorder,
        })
        .p(Space::N4)
        .shadow(ShadowPreset::Md)
}

impl<T> SurfaceUiBuilderExt for UiBuilder<T>
where
    T: UiSupportsChrome,
{
    fn popover_style(self) -> Self {
        self.style(popover_style_chrome())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui_kit::{LayoutRefinement, UiExt as _, UiPatch, UiPatchTarget, UiSupportsLayout};

    #[derive(Debug, Default, Clone)]
    struct Dummy {
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl UiPatchTarget for Dummy {
        fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
            self.chrome = self.chrome.merge(patch.chrome);
            self.layout = self.layout.merge(patch.layout);
            self
        }
    }

    impl UiSupportsChrome for Dummy {}
    impl UiSupportsLayout for Dummy {}

    #[test]
    fn popover_style_sets_expected_chrome_fields() {
        let dummy = Dummy::default().ui().popover_style().build();

        match dummy.chrome.background {
            Some(ColorRef::Token { key, .. }) => assert_eq!(key, "popover.background"),
            other => panic!("expected popover background token, got {other:?}"),
        }
        match dummy.chrome.border_color {
            Some(ColorRef::Token { key, .. }) => assert_eq!(key, "border"),
            other => panic!("expected border token, got {other:?}"),
        }

        assert!(dummy.chrome.border_width.is_some());
        assert!(dummy.chrome.radius.is_some());
        assert_eq!(dummy.chrome.shadow, Some(ShadowPreset::Md));
        assert!(dummy.chrome.padding.is_some());
    }
}
