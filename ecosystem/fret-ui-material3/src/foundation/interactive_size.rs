//! Minimum interactive component sizing (touch target enforcement).
//!
//! Compose Material provides `Modifier.minimumInteractiveComponentSize()` and a tree-local
//! `LocalMinimumInteractiveComponentSize` to ensure components meet minimum touch target size
//! requirements without changing their visual chrome.
//!
//! In Fret we implement the same outcome as a small policy helper:
//! - the pressable element should have a minimum size (default: 48x48),
//! - the visual chrome should remain at the token-driven size (often 40x40) and be centered.

use std::sync::Arc;

use fret_core::Px;
use fret_ui::Theme;
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::elements::ElementContext;

use crate::foundation::test_id::chrome_part_test_id;
use crate::foundation::token_resolver::MaterialTokenResolver;

pub const DEFAULT_MINIMUM_INTERACTIVE_SIZE: Px = Px(48.0);

pub fn minimum_interactive_size(theme: &Theme) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(
        Some("md.sys.layout.minimum-touch-target.size"),
        DEFAULT_MINIMUM_INTERACTIVE_SIZE,
    )
}

pub fn enforce_minimum_interactive_size(layout: &mut LayoutStyle, theme: &Theme) {
    let min = minimum_interactive_size(theme);
    if min.0 <= 0.0 {
        return;
    }
    layout.size.min_width = Some(Length::Px(min));
    layout.size.min_height = Some(Length::Px(min));
}

pub fn centered_fill<H: UiHost>(cx: &mut ElementContext<'_, H>, child: AnyElement) -> AnyElement {
    let mut props = FlexProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    cx.flex(props, move |_cx| vec![child])
}

pub fn centered_fill_with_chrome_test_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    base_test_id: Option<&Arc<str>>,
    chrome: AnyElement,
) -> AnyElement {
    let chrome = if let Some(id) = base_test_id {
        chrome.test_id(chrome_part_test_id(id))
    } else {
        chrome
    };
    centered_fill(cx, chrome)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn minimum_interactive_size_defaults_to_material_touch_target() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(
            minimum_interactive_size(theme),
            DEFAULT_MINIMUM_INTERACTIVE_SIZE
        );
    }

    #[test]
    fn minimum_interactive_size_prefers_material_token() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.layout.minimum-touch-target.size".to_string(), 56.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(minimum_interactive_size(&theme), Px(56.0));
    }

    #[test]
    fn enforce_minimum_interactive_size_applies_square_minimum() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.layout.minimum-touch-target.size".to_string(), 40.0);
        let (_app, theme) = theme_with_patch(patch);
        let mut layout = LayoutStyle::default();

        enforce_minimum_interactive_size(&mut layout, &theme);

        assert_eq!(layout.size.min_width, Some(Length::Px(Px(40.0))));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(40.0))));
    }
}
