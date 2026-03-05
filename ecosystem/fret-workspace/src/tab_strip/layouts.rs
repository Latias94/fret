use std::sync::Arc;

use fret_core::{Color, Px, TextAlign, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, TextInkOverflow, TextProps,
};
use fret_ui::{ElementContext, UiHost};

pub(super) fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

pub(super) fn tab_list_semantics_layout() -> LayoutStyle {
    // The tab strip is commonly placed into "center" regions of editor-like top bars where it
    // must be allowed to shrink; otherwise, long tab titles can push right-side controls out of
    // the window.
    //
    // This mirrors Tailwind's `w-full min-w-0 flex-shrink` rule of thumb.
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout.flex.shrink = 1.0;
    layout
}

pub(super) fn row_layout(height: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout.flex.shrink = 1.0;
    layout
}

fn scroll_content_row_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Auto;
    layout.size.height = Length::Fill;
    // Ensure the scroll content is at least as wide as the viewport so we have a stable
    // "header space" region to the right of the last tab (dockview/Zed-style).
    //
    // This is the equivalent of CSS `min-width: 100%` on the scroll content row.
    layout.size.min_width = Some(Length::Fraction(1.0));
    layout.flex.shrink = 0.0;
    layout
}

pub(super) fn tab_strip_scroll_content_layout() -> LayoutStyle {
    if std::env::var_os("FRET_DEBUG_TABSTRIP_FILL").is_some() {
        fill_layout()
    } else {
        scroll_content_row_layout()
    }
}

pub(super) fn tab_strip_scroll_row_layout(height: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Auto;
    layout.size.height = Length::Px(height);
    // Ensure the row is at least as wide as the viewport so we keep a stable "header space" band
    // to the right of the last tab (dockview/Zed-style).
    layout.size.min_width = Some(Length::Fraction(1.0));
    layout.flex.shrink = 0.0;
    layout
}

pub(super) fn fixed_square_layout(size: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.flex.shrink = 0.0;
    layout
}

pub(super) fn fill_grow_layout() -> LayoutStyle {
    let mut layout = fill_layout();
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout
}

pub(super) fn centered_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    style: TextStyle,
    color: Option<Color>,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: fill_layout(),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            ..Default::default()
        },
        |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text,
                style: Some(style),
                color,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: TextInkOverflow::None,
            })]
        },
    )
}
