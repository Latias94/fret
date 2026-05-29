use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_ui::element::ElementKind;

use crate::imui::{
    GridOptions, HorizontalOptions, ScrollOptions, UiWriterImUiFacadeExt as _, VerticalOptions,
};

use super::super::{
    grid_container_element, horizontal_container_element, scroll_container_element,
    vertical_container_element,
};
use super::bounds;

#[test]
fn container_option_test_ids_land_on_outer_surface() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "horizontal.test-id", |cx| {
        let element = horizontal_container_element(
            cx,
            None,
            HorizontalOptions {
                test_id: Some(Arc::from("imui-horizontal")),
                ..Default::default()
            },
            |ui| ui.text("row"),
        );

        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui-horizontal")
        );
    });

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "vertical.test-id", |cx| {
        let element = vertical_container_element(
            cx,
            None,
            VerticalOptions {
                test_id: Some(Arc::from("imui-vertical")),
                ..Default::default()
            },
            |ui| ui.text("column"),
        );

        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui-vertical")
        );
    });

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "grid.test-id", |cx| {
        let element = grid_container_element(
            cx,
            None,
            GridOptions {
                test_id: Some(Arc::from("imui-grid")),
                ..Default::default()
            },
            |ui| {
                ui.text("A");
                ui.text("B");
            },
        );

        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui-grid")
        );
    });

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "scroll.test-id", |cx| {
        let element = scroll_container_element(
            cx,
            None,
            ScrollOptions {
                test_id: Some(Arc::from("imui-scroll")),
                ..Default::default()
            },
            |ui| ui.text("scroll"),
        );

        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui-scroll")
        );
    });
}

#[test]
fn scroll_option_viewport_test_id_lands_on_inner_scroll_root() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "scroll.viewport.test-id",
        |cx| {
            let element = scroll_container_element(
                cx,
                None,
                ScrollOptions {
                    viewport_test_id: Some(Arc::from("imui-scroll.viewport")),
                    ..Default::default()
                },
                |ui| ui.text("scroll"),
            );

            let inner = match &element.kind {
                ElementKind::Container(_) => element
                    .children
                    .first()
                    .expect("scroll helper should wrap an inner scroll root"),
                other => panic!("expected scroll helper outer container, got {other:?}"),
            };

            assert_eq!(
                inner
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui-scroll.viewport")
            );
        },
    );
}
