use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{
    AnyElement, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SemanticsProps, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use crate::{IntoUiElement, collect_children};

/// A Radix-aligned `VisuallyHidden` helper.
///
/// This is a layout-only + semantics-only wrapper that keeps its subtree in the a11y tree while
/// remaining effectively invisible.
pub fn visually_hidden<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    cx.semantics(
        SemanticsProps {
            layout: LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(Px(1.0)),
                    height: Length::Px(Px(1.0)),
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx| {
            let items = f(cx);
            collect_children(cx, items)
        },
    )
}

/// Convenience helper for the common “screen-reader-only label” pattern.
pub fn visually_hidden_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            label: Some(label.into()),
            layout: LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(Px(1.0)),
                    height: Length::Px(Px(1.0)),
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    )
}
