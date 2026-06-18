use std::sync::{Arc, Mutex};

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextStyle};
use fret_ui::element::{AnyElement, ElementKind, Overflow};
use fret_ui::elements::GlobalElementId;
use fret_ui::{Theme, UiTree, declarative};

use super::{
    PROPERTY_ROW_VALUE_SLOT, PropertyRow, PropertyRowLayoutVariant, PropertyRowOptions,
    property_row_label_text,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::readout::editor_validation_message_text_props;
use crate::test_support::WrappingTextServices;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(120.0)),
    )
}

fn find_component_slot<'a>(element: &'a AnyElement, slot: &str) -> Option<&'a AnyElement> {
    if element.component_slot.as_deref() == Some(slot) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_component_slot(child, slot))
}

#[test]
fn row_value_slot_keeps_overflow_visible_for_wrapping_value_children() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let row =
        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "property-row", |cx| {
            PropertyRow::new()
                .options(PropertyRowOptions {
                    variant: PropertyRowLayoutVariant::Row,
                    test_id: Some(Arc::from("inspector.exposure")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx| property_row_label_text(cx, "Exposure"),
                    |cx| {
                        cx.text_props(editor_validation_message_text_props(
                            Arc::from(
                                "Value must stay between 0.0 and 1.0 for this render target.",
                            ),
                            Color::from_srgb_hex_rgb(0xCC_44_44),
                            TextStyle::default(),
                        ))
                    },
                    |_cx| None,
                )
        });

    let value_slot = find_component_slot(&row, PROPERTY_ROW_VALUE_SLOT)
        .expect("property row should mark its value slot for contract tests");
    let ElementKind::Container(props) = &value_slot.kind else {
        panic!(
            "property row value slot should be a container, got {:?}",
            value_slot.kind
        );
    };

    assert_eq!(
        props.layout.overflow,
        Overflow::Visible,
        "row value slot must let wrapping value children grow and paint inside their measured line boxes; fixed chrome slots may clip themselves"
    );
}

#[test]
fn row_without_trailing_slots_keeps_value_container_directly_under_root() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let row = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-row-flat-layout",
        |cx| {
            PropertyRow::new()
                .options(PropertyRowOptions {
                    variant: PropertyRowLayoutVariant::Row,
                    test_id: Some(Arc::from("inspector.exposure")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx| property_row_label_text(cx, "Exposure"),
                    |cx| cx.text("0.50"),
                    |_cx| None,
                )
        },
    );

    let ElementKind::Flex(_) = &row.kind else {
        panic!(
            "property row root should remain a flex container, got {:?}",
            row.kind
        );
    };
    assert_eq!(
        row.children.len(),
        2,
        "row without trailing slots should only keep label and value children"
    );
    assert!(
        matches!(row.children[0].kind, ElementKind::Container(_)),
        "first child should be the fixed-width label container"
    );
    assert!(
        matches!(row.children[1].kind, ElementKind::Container(_)),
        "second child should be the value container directly under the root"
    );
    assert_eq!(
        row.children[1].component_slot.as_deref(),
        Some(PROPERTY_ROW_VALUE_SLOT),
        "value container should stay the direct slot root when no trailing affordances are present"
    );
}

#[test]
fn column_without_trailing_slots_keeps_value_container_directly_under_root() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let row = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-row-flat-column-layout",
        |cx| {
            PropertyRow::new()
                .options(PropertyRowOptions {
                    variant: PropertyRowLayoutVariant::Column,
                    test_id: Some(Arc::from("inspector.exposure")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx| property_row_label_text(cx, "Exposure"),
                    |cx| cx.text("0.50"),
                    |_cx| None,
                )
        },
    );

    let ElementKind::Flex(_) = &row.kind else {
        panic!(
            "property row root should remain a flex container, got {:?}",
            row.kind
        );
    };
    assert_eq!(
        row.children.len(),
        2,
        "column row without trailing slots should only keep label and value children"
    );
    assert!(
        matches!(row.children[0].kind, ElementKind::Container(_)),
        "first child should be the fixed-width label container"
    );
    assert!(
        matches!(row.children[1].kind, ElementKind::Container(_)),
        "second child should be the value container directly under the root"
    );
    assert_eq!(
        row.children[1].component_slot.as_deref(),
        Some(PROPERTY_ROW_VALUE_SLOT),
        "value container should stay the direct slot root when no trailing affordances are present"
    );
}

#[test]
fn row_label_slot_keeps_fixed_line_box_when_label_text_wraps_under_narrow_layout() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let mut services = WrappingTextServices;
    let row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let expected_row_height = Arc::new(Mutex::new(None::<Px>));

    let row_id_for_render = Arc::clone(&row_id);
    let expected_row_height_for_render = Arc::clone(&expected_row_height);
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "property-row-wrapping-label-layout",
        move |cx| {
            let metrics = InspectorLayoutMetrics::resolve(Theme::global(&*cx.app));
            *expected_row_height_for_render.lock().unwrap() = Some(metrics.density.row_height);

            let row = PropertyRow::new()
                .options(PropertyRowOptions {
                    variant: PropertyRowLayoutVariant::Row,
                    label_width: Some(Px(48.0)),
                    gap: Some(Px(8.0)),
                    trailing_gap: Some(Px(0.0)),
                    value_max_width: Some(Px(1024.0)),
                    test_id: Some(Arc::from("inspector.long-label")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx| cx.text("Very long property label that would normally wrap under resize"),
                    |cx| cx.text("0.50"),
                    |_cx| None,
                );
            *row_id_for_render.lock().unwrap() = Some(row.id);
            vec![row]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    let row_id = row_id.lock().unwrap().expect("row id");
    let expected_row_height = expected_row_height
        .lock()
        .unwrap()
        .expect("expected row height");
    let row_bounds = fret_ui::elements::current_bounds_for_element(&mut app, window, row_id)
        .expect("row bounds");

    assert!(
        row_bounds.size.height.0 <= expected_row_height.0 + 0.5,
        "property-row label chrome must not grow fixed-height rows when bare/default text wraps under resize: row={row_bounds:?} expected_row_height={expected_row_height:?}"
    );
}

#[test]
fn row_value_slot_grows_to_wrapping_value_text_under_narrow_layout() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(120.0)),
    );
    let mut services = WrappingTextServices;
    let row_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));
    let value_slot_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));
    let validation_text_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));

    let row_id_for_render = Arc::clone(&row_id);
    let value_slot_id_for_render = Arc::clone(&value_slot_id);
    let validation_text_id_for_render = Arc::clone(&validation_text_id);
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "property-row-wrapping-value-layout",
        move |cx| {
            let row = PropertyRow::new()
                .options(PropertyRowOptions {
                    variant: PropertyRowLayoutVariant::Row,
                    label_width: Some(Px(104.0)),
                    gap: Some(Px(8.0)),
                    trailing_gap: Some(Px(0.0)),
                    value_max_width: Some(Px(1024.0)),
                    test_id: Some(Arc::from("inspector.exposure")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx| property_row_label_text(cx, "Exposure"),
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

            let value_slot = find_component_slot(&row, PROPERTY_ROW_VALUE_SLOT)
                .expect("property row should mark its value slot for layout tests");
            *row_id_for_render.lock().unwrap() = Some(row.id);
            *value_slot_id_for_render.lock().unwrap() = Some(value_slot.id);

            vec![row]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let row_id = row_id.lock().unwrap().expect("row id");
    let value_slot_id = value_slot_id.lock().unwrap().expect("value slot id");
    let validation_text_id = validation_text_id
        .lock()
        .unwrap()
        .expect("validation text id");

    let row_bounds = fret_ui::elements::current_bounds_for_element(&mut app, window, row_id)
        .expect("row bounds");
    let value_bounds =
        fret_ui::elements::current_bounds_for_element(&mut app, window, value_slot_id)
            .expect("value slot bounds");
    let text_bounds =
        fret_ui::elements::current_bounds_for_element(&mut app, window, validation_text_id)
            .expect("validation text bounds");

    assert!(
        text_bounds.size.height.0 > 28.0,
        "validation text should wrap to multiple measured lines under narrow layout: {text_bounds:?}"
    );
    assert!(
        value_bounds.size.height.0 + 0.5 >= text_bounds.size.height.0,
        "value slot should grow to contain wrapping validation text: value={value_bounds:?} text={text_bounds:?}"
    );
    assert!(
        row_bounds.size.height.0 + 0.5 >= value_bounds.size.height.0,
        "property row should grow to contain its value slot: row={row_bounds:?} value={value_bounds:?}"
    );
    assert!(
        text_bounds.origin.y.0 + text_bounds.size.height.0
            <= value_bounds.origin.y.0 + value_bounds.size.height.0 + 0.5,
        "validation text bottom should stay inside value slot bottom: value={value_bounds:?} text={text_bounds:?}"
    );
}
