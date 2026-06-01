use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::composites::PropertyRow;
use crate::composites::property_row::{PropertyRowOptions, property_row_label_text};
use crate::controls::{
    ColorEdit, ColorEditOptions, DragValue, DragValueOptions, IconButton, IconButtonOptions,
    OnIconButtonActivate,
};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::{EditorDensity, NumericPresentation};

use super::GradientStopBinding;

pub(super) fn gradient_stop_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled: bool,
    stops_test_id: Option<Arc<str>>,
    stop: GradientStopBinding,
    row_options: PropertyRowOptions,
) -> AnyElement {
    let remove = stop.remove.clone();
    let stop_id = stop.id;
    let row_test_id = stops_test_id
        .as_ref()
        .map(|base| Arc::<str>::from(format!("{}.stop.{}", base.as_ref(), stop_id)));
    let position_test_id = derived_test_id(row_test_id.as_ref(), "position");
    let color_test_id = derived_test_id(row_test_id.as_ref(), "color");
    let remove_test_id = derived_test_id(row_test_id.as_ref(), "remove");

    let mut row_options = row_options;
    row_options.test_id = row_test_id.clone();
    row_options.reset_slot_width = Some(Px(0.0));
    row_options.status_slot_width = Some(density.affordance_extent());

    PropertyRow::new().options(row_options).into_element(
        cx,
        |cx| property_row_label_text(cx, "Stop"),
        move |cx| {
            cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(Px(6.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let pos = DragValue::from_presentation(
                        stop.position.clone(),
                        NumericPresentation::<f64>::percent_0_1(0),
                    )
                    .options(DragValueOptions {
                        test_id: position_test_id.clone(),
                        ..Default::default()
                    })
                    .into_element(cx);
                    let color = ColorEdit::new(stop.color.clone())
                        .options(ColorEditOptions {
                            test_id: color_test_id.clone(),
                            ..Default::default()
                        })
                        .into_element(cx);
                    vec![pos, color]
                },
            )
        },
        move |cx| {
            let remove = remove.clone()?;
            let on_activate: OnIconButtonActivate = Arc::new(move |host, action_cx| {
                remove(host, action_cx, stop_id);
            });
            Some(
                IconButton::new(fret_icons::ids::ui::CLOSE, on_activate)
                    .options(IconButtonOptions {
                        enabled,
                        focusable: false,
                        icon_size: Some(Px(12.0)),
                        a11y_label: Some(Arc::from("Remove stop")),
                        test_id: remove_test_id.clone(),
                        ..Default::default()
                    })
                    .into_element(cx),
            )
        },
    )
}
