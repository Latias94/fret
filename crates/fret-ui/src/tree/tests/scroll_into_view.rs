use super::*;

#[test]
fn focus_traversal_scrolls_focused_descendant_into_view() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let mut first: Option<GlobalElementId> = None;
    let mut second: Option<GlobalElementId> = None;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            vec![cx.keyed(1, |cx| {
                cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Fill;
                            layout.size.height = crate::element::Length::Px(Px(40.0));
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        scroll_handle: Some(scroll_handle.clone()),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            crate::element::FlexProps {
                                layout: crate::element::LayoutStyle::default(),
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: crate::element::MainAlign::Start,
                                align: crate::element::CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                vec![
                                    cx.keyed(10, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Fill;
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(20.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                first = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                    cx.container(
                                        crate::element::ContainerProps {
                                            layout: {
                                                let mut layout =
                                                    crate::element::LayoutStyle::default();
                                                layout.size.width = crate::element::Length::Fill;
                                                layout.size.height =
                                                    crate::element::Length::Px(Px(200.0));
                                                layout
                                            },
                                            ..Default::default()
                                        },
                                        |_| Vec::new(),
                                    ),
                                    cx.keyed(11, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Fill;
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(20.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                second = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                ]
                            },
                        )]
                    },
                )
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first = first.expect("first pressable id");
    let second = second.expect("second pressable id");

    let scroll_node = ui
        .nodes
        .get(root)
        .and_then(|n| n.children.first().copied())
        .expect("scroll node");
    let first_node = crate::elements::node_for_element(&mut app, window, first).expect("first");
    let second_node = crate::elements::node_for_element(&mut app, window, second).expect("second");

    ui.set_focus(Some(first_node));

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(second_node));
    assert!(scroll_handle.offset().y.0 > 0.0);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_bounds = ui
        .debug_node_visual_bounds(scroll_node)
        .expect("scroll visual bounds");
    let second_bounds = ui
        .debug_node_visual_bounds(second_node)
        .expect("second visual bounds");

    assert!(
        UiTree::<crate::test_host::TestHost>::rects_intersect(scroll_bounds, second_bounds),
        "expected focused node to be in view after scroll-into-view"
    );
}
