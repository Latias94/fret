//! Minimum interactive component sizing (touch target enforcement).
//!
//! Compose Material provides `Modifier.minimumInteractiveComponentSize()` and a tree-local
//! `LocalMinimumInteractiveComponentSize` to ensure components meet minimum touch target size
//! requirements without changing their visual chrome.
//!
//! In Fret we implement the same outcome as a small policy helper:
//! - the pressable element should have a minimum size (default: 48x48),
//! - the visual chrome should remain at the token-driven size (often 40x40) and be centered.

use fret_core::Px;
use fret_ui::Theme;
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::elements::ElementContext;

pub const DEFAULT_MINIMUM_INTERACTIVE_SIZE: Px = Px(48.0);

pub fn minimum_interactive_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.sys.layout.minimum-touch-target.size")
        .unwrap_or(DEFAULT_MINIMUM_INTERACTIVE_SIZE)
}

pub fn enforce_minimum_interactive_size(layout: &mut LayoutStyle, theme: &Theme) {
    let min = minimum_interactive_size(theme);
    layout.size.min_width = Some(min);
    layout.size.min_height = Some(min);
}

pub fn centered_fill<H: UiHost>(cx: &mut ElementContext<'_, H>, child: AnyElement) -> AnyElement {
    let mut props = FlexProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    cx.flex(props, move |_cx| vec![child])
}
