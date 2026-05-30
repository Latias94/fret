use fret_app::App;
use fret_core::{AppWindowId, Px};
use fret_ui::element::{ElementKind, Length};

use crate::LayoutRefinement;
use crate::imui::{
    GridOptions, HorizontalOptions, ScrollOptions, UiWriterImUiFacadeExt as _, VerticalOptions,
};

use super::super::{
    grid_container_element, horizontal_container_element, scroll_container_element,
    vertical_container_element,
};
use super::bounds;

#[test]
fn horizontal_and_vertical_container_options_forward_layout_to_outer_box() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "horizontal", |cx| {
        let element = horizontal_container_element(
            cx,
            None,
            HorizontalOptions {
                layout: LayoutRefinement::default().w_px(Px(180.0)),
                ..Default::default()
            },
            |ui| ui.text("row"),
        );

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected horizontal helper outer container");
        };
        assert_eq!(props.layout.size.width, Length::Px(Px(180.0)));
    });

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "vertical", |cx| {
        let element = vertical_container_element(
            cx,
            None,
            VerticalOptions {
                layout: LayoutRefinement::default().h_px(Px(120.0)),
                ..Default::default()
            },
            |ui| ui.text("column"),
        );

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected vertical helper outer container");
        };
        assert_eq!(props.layout.size.height, Length::Px(Px(120.0)));
    });
}

#[test]
fn grid_and_scroll_container_options_forward_layout_to_outer_box() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "grid", |cx| {
        let element = grid_container_element(
            cx,
            None,
            GridOptions {
                layout: LayoutRefinement::default().w_px(Px(200.0)),
                columns: 2,
                ..Default::default()
            },
            |ui| {
                ui.text("A");
                ui.text("B");
            },
        );

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected grid helper outer container");
        };
        assert_eq!(props.layout.size.width, Length::Px(Px(200.0)));
    });

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "scroll", |cx| {
        let element = scroll_container_element(
            cx,
            None,
            ScrollOptions {
                layout: LayoutRefinement::default().h_px(Px(96.0)),
                ..Default::default()
            },
            |ui| ui.text("scroll"),
        );

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected scroll helper outer container");
        };
        assert_eq!(props.layout.size.height, Length::Px(Px(96.0)));
    });
}
