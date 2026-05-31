//! Shared Material field chrome helpers.

use std::sync::Arc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, LayoutDirection, Point, Px, Rect, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, PointerRegionProps, PositionStyle, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};

use crate::foundation::floating_label;
use crate::foundation::logical_edges::{
    set_inset_inline_end, set_inset_inline_start, set_margin_inline_end, set_margin_inline_start,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialFieldVariant {
    Filled,
    Outlined,
}

pub(crate) struct MaterialFieldFloatingLabelProps {
    pub variant: MaterialFieldVariant,
    pub text: Arc<str>,
    pub progress: f32,
    pub style: Option<TextStyle>,
    pub color: Color,
    pub input_bg: Color,
    pub outline_width: Px,
    pub test_id: Option<Arc<str>>,
    pub leading_icon_size: Option<Px>,
    pub layout_direction: LayoutDirection,
    pub focus_target: Option<GlobalElementId>,
}

pub(crate) struct MaterialFieldSupportingTextProps {
    pub text: Arc<str>,
    pub style: Option<TextStyle>,
    pub color: Color,
    pub test_id: Option<Arc<str>>,
    pub leading_icon_size: Option<Px>,
    pub layout_direction: LayoutDirection,
}

pub(crate) fn material_field_active_indicator_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    height: Px,
    color: Color,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0)).into();
    props.layout.inset.right = Some(Px(0.0)).into();
    props.layout.inset.bottom = Some(Px(0.0)).into();
    props.layout.inset.left = Some(Px(0.0)).into();

    let mut indicator = cx.canvas(props, move |p| {
        if height.0 <= 0.0 || color.a <= 0.0 {
            return;
        }

        let bounds = p.bounds();
        let y = Px(bounds.origin.y.0 + bounds.size.height.0 - height.0);
        let rect = Rect::new(
            Point::new(bounds.origin.x, y),
            Size::new(bounds.size.width, height),
        );
        p.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: fret_core::Paint::Solid(color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    });

    if let Some(test_id) = test_id {
        indicator = indicator.test_id(test_id);
    }

    indicator
}

pub(crate) fn material_field_text_start_inset_x(default: Px, leading_icon_size: Option<Px>) -> Px {
    // Material field layouts use a 12px leading slot inset plus a 16px icon-content gap.
    leading_icon_size
        .map(|icon_size| Px(12.0 + icon_size.0 + 16.0))
        .unwrap_or(default)
}

pub(crate) fn material_field_icon_adjusted_padding(
    mut padding: Edges,
    layout_direction: LayoutDirection,
    leading_hit_width: Px,
    leading_icon_size: Option<Px>,
    trailing_hit_width: Px,
) -> Edges {
    let leading_padding = (leading_hit_width.0 > 0.0)
        .then(|| material_field_text_start_inset_x(leading_hit_width, leading_icon_size));
    let trailing_padding = (trailing_hit_width.0 > 0.0).then_some(trailing_hit_width);

    match layout_direction {
        LayoutDirection::Ltr => {
            if let Some(value) = leading_padding {
                padding.left = Px(padding.left.0.max(value.0));
            }
            if let Some(value) = trailing_padding {
                padding.right = Px(padding.right.0.max(value.0));
            }
        }
        LayoutDirection::Rtl => {
            if let Some(value) = leading_padding {
                padding.right = Px(padding.right.0.max(value.0));
            }
            if let Some(value) = trailing_padding {
                padding.left = Px(padding.left.0.max(value.0));
            }
        }
    }

    padding
}

pub(crate) fn material_field_floating_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: MaterialFieldFloatingLabelProps,
) -> AnyElement {
    let MaterialFieldFloatingLabelProps {
        variant,
        text,
        progress,
        style,
        color,
        input_bg,
        outline_width,
        test_id,
        leading_icon_size,
        layout_direction,
        focus_target,
    } = props;

    let (x, y) = floating_label::material_floating_label_offsets(progress);
    let x = material_field_text_start_inset_x(x, leading_icon_size);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y).into();
    set_inset_inline_start(&mut layout, layout_direction, x);
    set_inset_inline_end(&mut layout, layout_direction, Px(16.0));
    layout.overflow = fret_ui::element::Overflow::Visible;

    let floated = floating_label::is_floated(progress);

    let mut patch = ContainerProps::default();
    if variant == MaterialFieldVariant::Outlined {
        let patch_padding_x = Px(4.0);
        let patch_padding_y = Px((outline_width.0 + 1.0).max(0.0));
        patch.padding = (if floated {
            Edges {
                top: patch_padding_y,
                right: patch_padding_x,
                bottom: patch_padding_y,
                left: patch_padding_x,
            }
        } else {
            Edges::all(Px(0.0))
        })
        .into();
        patch.background = floated.then_some(input_bg);
    }

    let mut label = if let Some(target) = focus_target {
        cx.pointer_region(
            PointerRegionProps {
                layout,
                enabled: true,
                ..Default::default()
            },
            move |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                    host.request_focus(target);
                    true
                }));
                vec![cx.container(patch, move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        text: text.clone(),
                        style,
                        color: Some(color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    })]
                })]
            },
        )
    } else {
        patch.layout = layout;
        cx.container(patch, move |cx| {
            vec![cx.text_props(TextProps {
                layout: fret_ui::element::LayoutStyle::default(),
                text: text.clone(),
                style,
                color: Some(color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })]
        })
    };

    if let Some(test_id) = test_id {
        label = label.test_id(test_id);
    }

    label
}

pub(crate) fn material_field_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: MaterialFieldSupportingTextProps,
) -> AnyElement {
    let MaterialFieldSupportingTextProps {
        text,
        style,
        color,
        test_id,
        leading_icon_size,
        layout_direction,
    } = props;

    let mut layout = fret_ui::element::LayoutStyle::default();
    set_margin_inline_start(
        &mut layout,
        layout_direction,
        material_field_text_start_inset_x(Px(16.0), leading_icon_size),
    );
    set_margin_inline_end(&mut layout, layout_direction, Px(16.0));

    let mut supporting_text = cx.text_props(TextProps {
        layout,
        text,
        style,
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: Default::default(),
    });
    if let Some(test_id) = test_id {
        supporting_text = supporting_text.test_id(test_id);
    }
    supporting_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_adjusted_padding_maps_slots_to_logical_sides() {
        let base = Edges {
            top: Px(18.0),
            right: Px(16.0),
            bottom: Px(14.0),
            left: Px(16.0),
        };

        let ltr = material_field_icon_adjusted_padding(
            base,
            LayoutDirection::Ltr,
            Px(48.0),
            Some(Px(24.0)),
            Px(48.0),
        );
        assert_eq!(ltr.left, Px(52.0));
        assert_eq!(ltr.right, Px(48.0));

        let rtl = material_field_icon_adjusted_padding(
            base,
            LayoutDirection::Rtl,
            Px(48.0),
            Some(Px(24.0)),
            Px(48.0),
        );
        assert_eq!(rtl.left, Px(48.0));
        assert_eq!(rtl.right, Px(52.0));
    }
}
