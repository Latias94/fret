use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Radius, ShadowPreset, Space, UiBuilder,
    UiSupportsChrome,
};

/// A policy-level preset extension for common shadcn surface styling.
pub trait SurfaceUiBuilderExt {
    /// Applies the standard shadcn popover-like surface chrome (panel background, border, radius,
    /// padding, and shadow).
    fn popover_style(self) -> Self;

    /// Applies the standard shadcn dialog surface chrome (background, border, radius, padding,
    /// and shadow).
    fn dialog_style(self) -> Self;

    /// Applies the standard shadcn menu surface chrome (dropdown/context/menubar panel).
    fn menu_style(self) -> Self;

    /// Applies the standard shadcn submenu surface chrome.
    fn menu_sub_style(self) -> Self;
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

pub(crate) fn dialog_style_chrome() -> ChromeRefinement {
    ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .bg(ColorRef::Token {
            key: "background",
            fallback: ColorFallback::ThemeSurfaceBackground,
        })
        .border_color(ColorRef::Token {
            key: "border",
            fallback: ColorFallback::ThemePanelBorder,
        })
        .p(Space::N6)
        .shadow(ShadowPreset::Lg)
}

pub(crate) fn menu_style_chrome() -> ChromeRefinement {
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
        // new-york-v4: menu panels typically use `p-1`.
        .p(Space::N1)
        // new-york-v4: dropdown menu uses `shadow-md`.
        .shadow(ShadowPreset::Md)
}

pub(crate) fn menu_sub_style_chrome() -> ChromeRefinement {
    ChromeRefinement::default()
        .rounded(Radius::Sm)
        .border_1()
        .bg(ColorRef::Token {
            key: "popover.background",
            fallback: ColorFallback::ThemePanelBackground,
        })
        .border_color(ColorRef::Token {
            key: "border",
            fallback: ColorFallback::ThemePanelBorder,
        })
        .p(Space::N1)
        // new-york-v4: submenus typically use `shadow-lg`.
        .shadow(ShadowPreset::Lg)
}

impl<T> SurfaceUiBuilderExt for UiBuilder<T>
where
    T: UiSupportsChrome,
{
    fn popover_style(self) -> Self {
        self.style(popover_style_chrome())
    }

    fn dialog_style(self) -> Self {
        self.style(dialog_style_chrome())
    }

    fn menu_style(self) -> Self {
        self.style(menu_style_chrome())
    }

    fn menu_sub_style(self) -> Self {
        self.style(menu_sub_style_chrome())
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

    #[test]
    fn dialog_style_sets_expected_chrome_fields() {
        let dummy = Dummy::default().ui().dialog_style().build();

        match dummy.chrome.background {
            Some(ColorRef::Token { key, .. }) => assert_eq!(key, "background"),
            other => panic!("expected dialog background token, got {other:?}"),
        }
        match dummy.chrome.border_color {
            Some(ColorRef::Token { key, .. }) => assert_eq!(key, "border"),
            other => panic!("expected border token, got {other:?}"),
        }

        assert!(dummy.chrome.border_width.is_some());
        assert!(dummy.chrome.radius.is_some());
        assert_eq!(dummy.chrome.shadow, Some(ShadowPreset::Lg));
        assert!(dummy.chrome.padding.is_some());
    }

    #[test]
    fn menu_style_sets_expected_chrome_fields() {
        let dummy = Dummy::default().ui().menu_style().build();
        assert_eq!(dummy.chrome.shadow, Some(ShadowPreset::Md));
        assert!(dummy.chrome.padding.is_some());
    }

    #[test]
    fn menu_sub_style_sets_expected_chrome_fields() {
        let dummy = Dummy::default().ui().menu_sub_style().build();
        assert_eq!(dummy.chrome.shadow, Some(ShadowPreset::Lg));
        assert!(dummy.chrome.padding.is_some());
    }
}
