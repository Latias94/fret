//! Small ImGui-porting layout conveniences.

use fret_core::{Px, Size};
use fret_ui::element::{AnyElement, Length, SemanticsDecoration, SpacerProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::style::MetricFallback;

use super::containers::{horizontal_container_element, vertical_container_element};
use super::{
    DummyOptions, HorizontalOptions, ImUiFacade, IndentOptions, ItemFlowOptions, SameLineOptions,
    SpacingOptions, VerticalOptions,
};

pub(super) fn items_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: ItemFlowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    vertical_container_element(
        cx,
        build_focus,
        VerticalOptions {
            layout: options.layout,
            gap: options.gap,
            justify: options.justify,
            items: options.items,
            wrap: options.wrap,
            test_id: options.test_id,
        },
        f,
    )
}

pub(super) fn same_line_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: SameLineOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    horizontal_container_element(
        cx,
        build_focus,
        HorizontalOptions {
            layout: options.layout,
            gap: options.gap,
            justify: options.justify,
            items: options.items,
            wrap: options.wrap,
            test_id: options.test_id,
        },
        f,
    )
}

pub(super) fn dummy_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: Size,
    options: DummyOptions,
) -> AnyElement {
    let mut props = SpacerProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
    props.layout.size.min_width = Some(Length::Px(size.width));
    props.layout.size.min_height = Some(Length::Px(size.height));
    props.layout.flex.grow = 0.0;
    props.layout.flex.shrink = 0.0;
    props.layout.flex.basis = Length::Px(size.height);
    props.min = size.height;

    let mut element = cx.spacer(props);
    if let Some(test_id) = options.test_id {
        element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }
    element
}

pub(super) fn spacing_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: SpacingOptions,
) -> AnyElement {
    let size = options.size.unwrap_or_else(|| {
        let theme = Theme::global(&*cx.app);
        Size::new(
            Px(0.0),
            crate::MetricRef::Token {
                key: "component.imui.item_spacing_y_px",
                fallback: MetricFallback::Px(Px(4.0)),
            }
            .resolve(&*theme),
        )
    });
    dummy_element(
        cx,
        size,
        DummyOptions {
            test_id: options.test_id,
        },
    )
}

pub(super) fn indent_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: IndentOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let indent_width = options.width.resolve(&*theme);
    let spacer = dummy_element(
        cx,
        Size::new(indent_width, Px(0.0)),
        DummyOptions::default(),
    );
    let content = items_element(
        cx,
        build_focus,
        ItemFlowOptions {
            test_id: options.content_test_id,
            ..Default::default()
        },
        f,
    );

    horizontal_container_element(
        cx,
        None,
        HorizontalOptions {
            gap: Px(0.0).into(),
            items: crate::Items::Start,
            test_id: options.test_id,
            ..Default::default()
        },
        move |ui| {
            ui.add(spacer);
            ui.add(content);
        },
    )
}
