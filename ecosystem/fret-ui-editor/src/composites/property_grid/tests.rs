use std::sync::{Arc, Mutex};

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextStyle};
use fret_ui::elements::GlobalElementId;
use fret_ui::{UiTree, declarative};
use fret_ui_kit::headless::text_assist::TextAssistItem;

use super::{PropertyGrid, PropertyGridOptions};
use crate::composites::property_row::{PropertyRow, PropertyRowLayoutVariant, PropertyRowOptions};
use crate::controls::{
    DragValue, DragValueOptions, NumericInput, NumericInputOptions, TextAssistField,
    TextAssistFieldOptions, TextAssistFieldSurface, TextFieldOptions,
};
use crate::primitives::NumericPresentation;
use crate::primitives::readout::{
    editor_inline_error_text_props, editor_validation_message_text_props,
};
use crate::test_support::WrappingTextServices;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(160.0)),
    )
}

fn current_bounds(app: &mut App, window: AppWindowId, id: GlobalElementId, label: &str) -> Rect {
    fret_ui::elements::current_bounds_for_element(app, window, id)
        .unwrap_or_else(|| panic!("{label} bounds"))
}

fn lock_id(id: &Arc<Mutex<Option<GlobalElementId>>>, label: &str) -> GlobalElementId {
    id.lock().unwrap().unwrap_or_else(|| panic!("{label} id"))
}

#[test]
fn property_grid_with_single_row_returns_the_row_directly() {
    let mut app = App::new();
    let window = AppWindowId::default();

    let grid = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-grid-single-row",
        |cx| {
            PropertyGrid::new()
                .options(PropertyGridOptions {
                    label_width: Some(Px(104.0)),
                    column_gap: Some(Px(8.0)),
                    row_gap: Some(Px(4.0)),
                    ..Default::default()
                })
                .into_element(cx, |cx, rows| {
                    vec![rows.row(
                        cx,
                        |cx| rows.label_text(cx, "Exposure"),
                        |cx| cx.text("0.50"),
                    )]
                })
        },
    );

    let fret_ui::element::ElementKind::Flex(props) = &grid.kind else {
        panic!("single-row property grid should return the row itself, not a wrapper shell");
    };
    assert_eq!(
        props.direction,
        fret_core::Axis::Horizontal,
        "single-row property grid should return the horizontal property row directly"
    );
    assert_eq!(
        grid.children.len(),
        2,
        "single-row property grid should expose the row's label and value children directly"
    );
    assert!(
        matches!(
            grid.children[0].kind,
            fret_ui::element::ElementKind::Text(_)
        ),
        "first child should be the label text root from the row"
    );
    assert!(
        matches!(
            grid.children[1].kind,
            fret_ui::element::ElementKind::Text(_)
        ),
        "second child should be the value text root from the row"
    );
}

#[test]
fn property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = WrappingTextServices;
    let grid_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let first_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let wrapping_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let validation_text_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let trailing_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));

    let grid_id_for_render = Arc::clone(&grid_id);
    let first_row_id_for_render = Arc::clone(&first_row_id);
    let wrapping_row_id_for_render = Arc::clone(&wrapping_row_id);
    let validation_text_id_for_render = Arc::clone(&validation_text_id);
    let trailing_row_id_for_render = Arc::clone(&trailing_row_id);
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "property-grid-wrapping-layout",
        move |cx| {
            let grid = PropertyGrid::new()
                .options(PropertyGridOptions {
                    label_width: Some(Px(104.0)),
                    column_gap: Some(Px(8.0)),
                    row_gap: Some(Px(4.0)),
                    test_id: Some(Arc::from("inspector.grid")),
                    ..Default::default()
                })
                .into_element(cx, |cx, rows| {
                    let first = rows.row_with(
                        cx,
                        PropertyRow::new().options(PropertyRowOptions {
                            variant: PropertyRowLayoutVariant::Row,
                            test_id: Some(Arc::from("inspector.grid.exposure")),
                            ..Default::default()
                        }),
                        |cx| rows.label_text(cx, "Exposure"),
                        |cx| {
                            cx.text_props(editor_inline_error_text_props(
                                Arc::from("0.50"),
                                Color::from_srgb_hex_rgb(0xCC_CC_CC),
                                Px(20.0),
                            ))
                        },
                        |_cx| None,
                    );
                    *first_row_id_for_render.lock().unwrap() = Some(first.id);

                    let wrapping = rows.row_with(
                        cx,
                        PropertyRow::new().options(PropertyRowOptions {
                            variant: PropertyRowLayoutVariant::Row,
                            test_id: Some(Arc::from("inspector.grid.validation")),
                            ..Default::default()
                        }),
                        |cx| rows.label_text(cx, "Validation"),
                        |cx| {
                            let text = cx.text_props(editor_validation_message_text_props(
                                Arc::from(
                                    "Value must stay between 0.0 and 1.0 for this render target.",
                                ),
                                Color::from_srgb_hex_rgb(0xCC_44_44),
                                TextStyle::default(),
                            ));
                            *validation_text_id_for_render.lock().unwrap() = Some(text.id);
                            text
                        },
                        |_cx| None,
                    );
                    *wrapping_row_id_for_render.lock().unwrap() = Some(wrapping.id);

                    let trailing = rows.row_with(
                        cx,
                        PropertyRow::new().options(PropertyRowOptions {
                            variant: PropertyRowLayoutVariant::Row,
                            test_id: Some(Arc::from("inspector.grid.roughness")),
                            ..Default::default()
                        }),
                        |cx| rows.label_text(cx, "Roughness"),
                        |cx| {
                            cx.text_props(editor_inline_error_text_props(
                                Arc::from("0.25"),
                                Color::from_srgb_hex_rgb(0xCC_CC_CC),
                                Px(20.0),
                            ))
                        },
                        |_cx| None,
                    );
                    *trailing_row_id_for_render.lock().unwrap() = Some(trailing.id);

                    vec![first, wrapping, trailing]
                });
            *grid_id_for_render.lock().unwrap() = Some(grid.id);
            vec![grid]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    let grid_bounds = current_bounds(&mut app, window, lock_id(&grid_id, "grid"), "grid");
    let first_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&first_row_id, "first row"),
        "first row",
    );
    let wrapping_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&wrapping_row_id, "wrapping row"),
        "wrapping row",
    );
    let validation_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&validation_text_id, "validation text"),
        "validation text",
    );
    let trailing_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&trailing_row_id, "trailing row"),
        "trailing row",
    );

    assert!(
        validation_bounds.size.height.0 > 28.0,
        "validation text should wrap to multiple measured lines under narrow grid layout: {validation_bounds:?}"
    );
    assert!(
        wrapping_bounds.size.height.0 + 0.5 >= validation_bounds.size.height.0,
        "wrapping property row should grow to contain validation text: row={wrapping_bounds:?} text={validation_bounds:?}"
    );
    assert!(
        first_bounds.origin.y.0 + first_bounds.size.height.0 <= wrapping_bounds.origin.y.0 + 0.5,
        "first row should not overlap wrapping row: first={first_bounds:?} wrapping={wrapping_bounds:?}"
    );
    assert!(
        wrapping_bounds.origin.y.0 + wrapping_bounds.size.height.0
            <= trailing_bounds.origin.y.0 + 0.5,
        "wrapping row should push the following row down: wrapping={wrapping_bounds:?} trailing={trailing_bounds:?}"
    );
    assert!(
        trailing_bounds.origin.y.0 + trailing_bounds.size.height.0
            <= grid_bounds.origin.y.0 + grid_bounds.size.height.0 + 0.5,
        "property grid should contain rows after a wrapping value row: grid={grid_bounds:?} trailing={trailing_bounds:?}"
    );
}

#[test]
fn property_grid_row_context_defaults_to_row_variant() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let mut captured = None;
    let _grid = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-grid-default-row-layout",
        |cx| {
            let grid = PropertyGrid::new()
                .options(PropertyGridOptions {
                    label_width: Some(Px(104.0)),
                    column_gap: Some(Px(8.0)),
                    row_gap: Some(Px(4.0)),
                    test_id: Some(Arc::from("inspector.grid.default")),
                    ..Default::default()
                })
                .into_element(cx, |_cx, rows| {
                    captured = Some(rows.row_options().variant);
                    Vec::new()
                });
            vec![grid]
        },
    );

    assert_eq!(captured, Some(PropertyRowLayoutVariant::Row));
}

#[test]
fn property_grid_keeps_common_editor_controls_on_same_row_height() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = WrappingTextServices;

    let exposure = app.models_mut().insert(1.25f64);
    let roughness = app.models_mut().insert(0.42f64);
    let query = app.models_mut().insert(String::from("ca"));
    let dismissed_query = app.models_mut().insert(String::new());
    let active_item = app.models_mut().insert(None::<Arc<str>>);
    let items: Arc<[TextAssistItem]> = vec![
        TextAssistItem::new("camera", "Camera"),
        TextAssistItem::new("canvas", "Canvas"),
    ]
    .into();

    let numeric_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let drag_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let assist_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));

    let numeric_row_id_for_render = Arc::clone(&numeric_row_id);
    let drag_row_id_for_render = Arc::clone(&drag_row_id);
    let assist_row_id_for_render = Arc::clone(&assist_row_id);
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(240.0)),
        ),
        "property-grid-editor-control-row-heights",
        move |cx| {
            let exposure_presentation =
                NumericPresentation::<f64>::fixed_decimals(2).with_chrome_suffix(" EV");
            let roughness_presentation = NumericPresentation::<f64>::percent_0_1(0);
            let items = items.clone();

            let grid = PropertyGrid::new()
                .options(PropertyGridOptions {
                    label_width: Some(Px(120.0)),
                    column_gap: Some(Px(6.0)),
                    row_gap: Some(Px(4.0)),
                    ..Default::default()
                })
                .into_element(cx, move |cx, rows| {
                    let numeric = rows.row(
                        cx,
                        |cx| rows.label_text(cx, "Exposure"),
                        |cx| {
                            NumericInput::from_presentation(exposure.clone(), exposure_presentation)
                                .options(NumericInputOptions {
                                    id_source: Some(Arc::from("test.exposure")),
                                    ..Default::default()
                                })
                                .into_element(cx)
                        },
                    );
                    *numeric_row_id_for_render.lock().unwrap() = Some(numeric.id);

                    let drag = rows.row(
                        cx,
                        |cx| rows.label_text(cx, "Roughness"),
                        |cx| {
                            DragValue::from_presentation(roughness.clone(), roughness_presentation)
                                .options(DragValueOptions {
                                    id_source: Some(Arc::from("test.roughness")),
                                    ..Default::default()
                                })
                                .into_element(cx)
                        },
                    );
                    *drag_row_id_for_render.lock().unwrap() = Some(drag.id);

                    let assist = rows.row(
                        cx,
                        |cx| rows.label_text(cx, "Asset"),
                        |cx| {
                            TextAssistField::new(
                                query.clone(),
                                dismissed_query.clone(),
                                active_item.clone(),
                                items.clone(),
                            )
                            .options(TextAssistFieldOptions {
                                field: TextFieldOptions {
                                    id_source: Some(Arc::from("test.asset")),
                                    ..Default::default()
                                },
                                surface: TextAssistFieldSurface::AnchoredOverlay,
                                ..Default::default()
                            })
                            .into_element(cx)
                        },
                    );
                    *assist_row_id_for_render.lock().unwrap() = Some(assist.id);

                    vec![numeric, drag, assist]
                });
            vec![grid]
        },
    );
    ui.set_root(root);
    ui.layout_all(
        &mut app,
        &mut services,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(240.0)),
        ),
        1.0,
    );

    let numeric_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&numeric_row_id, "numeric row"),
        "numeric row",
    );
    let drag_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&drag_row_id, "drag row"),
        "drag row",
    );
    let assist_bounds = current_bounds(
        &mut app,
        window,
        lock_id(&assist_row_id, "assist row"),
        "assist row",
    );

    assert!(
        numeric_bounds.size.height.0 >= 24.0,
        "numeric input should keep a stable editor shell height: numeric={numeric_bounds:?}"
    );
    assert!(
        drag_bounds.size.height.0 >= 24.0,
        "drag value should keep a stable editor shell height: drag={drag_bounds:?}"
    );
    assert!(
        (numeric_bounds.size.height.0 - assist_bounds.size.height.0).abs() <= 0.5,
        "numeric and assist rows should share row height: numeric={numeric_bounds:?} assist={assist_bounds:?}"
    );
}
