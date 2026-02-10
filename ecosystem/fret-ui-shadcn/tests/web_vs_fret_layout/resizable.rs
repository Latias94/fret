use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutResizableRecipe {
    Demo,
    DemoWithHandle,
    Handle,
    Vertical,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutResizableCase {
    id: String,
    web_name: String,
    recipe: LayoutResizableRecipe,
}

#[test]
fn web_vs_fret_layout_resizable_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_resizable_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutResizableCase> =
        serde_json::from_str(raw).expect("layout resizable fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout resizable case={} web_name={}",
            case.id, case.web_name
        );
        match case.recipe {
            LayoutResizableRecipe::Demo => {
                assert_resizable_demo_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::DemoWithHandle => {
                assert_resizable_demo_with_handle_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::Handle => {
                assert_resizable_handle_geometry_matches_web(&case.web_name)
            }
            LayoutResizableRecipe::Vertical => {
                assert_resizable_vertical_geometry_matches_web(&case.web_name)
            }
        }
    }
}

fn assert_resizable_demo_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(&theme.root, &["max-w-md", "rounded-lg", "border"])
        .expect("web resizable group");

    let web_one = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-[200px]", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "One")
    })
    .expect("web one panel content");
    let web_two = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-full", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "Two")
    })
    .expect("web two panel content");
    let web_three = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &["flex", "h-full", "items-center", "justify-center", "p-6"],
            )
            && contains_text(n, "Three")
    })
    .expect("web three panel content");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));
    let one_label: Arc<str> = Arc::from(format!("Golden:{web_name}:one"));
    let two_label: Arc<str> = Arc::from(format!("Golden:{web_name}:two"));
    let three_label: Arc<str> = Arc::from(format!("Golden:{web_name}:three"));

    let snap = run_fret_root(bounds, |cx| {
        let model_outer: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.5, 0.5]);
        let model_inner: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        fn mk_center(
            cx: &mut fret_ui::ElementContext<'_, App>,
            theme: &Theme,
            label: Arc<str>,
            text: &'static str,
            fixed_height: Option<Px>,
        ) -> AnyElement {
            let layout = match fixed_height {
                Some(h) => LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
                None => LayoutRefinement::default().size_full(),
            };
            let layout = decl_style::layout_style(theme, layout);
            let node = cx.container(
                ContainerProps {
                    layout,
                    padding: Edges::all(Px(24.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| vec![ui::text(cx, text).font_semibold().into_element(cx)],
                    )]
                },
            );

            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(label),
                    ..Default::default()
                },
                move |_cx| vec![node],
            )
        }

        let one = mk_center(cx, &theme, one_label.clone(), "One", Some(Px(200.0)));
        let two = mk_center(cx, &theme, two_label.clone(), "Two", None);
        let three = mk_center(cx, &theme, three_label.clone(), "Three", None);

        let inner = fret_ui_shadcn::ResizablePanelGroup::new(model_inner)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![two])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![three])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let outer = fret_ui_shadcn::ResizablePanelGroup::new(model_outer)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![one])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![inner])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![outer],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret group");
    let one =
        find_semantics(&snap, SemanticsRole::Panel, Some(one_label.as_ref())).expect("fret one");
    let two =
        find_semantics(&snap, SemanticsRole::Panel, Some(two_label.as_ref())).expect("fret two");
    let three = find_semantics(&snap, SemanticsRole::Panel, Some(three_label.as_ref()))
        .expect("fret three");

    assert_rect_close_px(
        &format!("{web_name} group"),
        group.bounds,
        web_group.rect,
        2.0,
    );
    assert_rect_close_px(&format!("{web_name} one"), one.bounds, web_one.rect, 2.0);
    assert_rect_close_px(&format!("{web_name} two"), two.bounds, web_two.rect, 2.0);
    assert_rect_close_px(
        &format!("{web_name} three"),
        three.bounds,
        web_three.rect,
        2.0,
    );
}

fn assert_resizable_demo_with_handle_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(&theme.root, &["max-w-md", "rounded-lg", "border"])
        .expect("web resizable group");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));

    let snap = run_fret_root(bounds, |cx| {
        let model_outer: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.5, 0.5]);
        let model_inner: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        fn panel(
            cx: &mut fret_ui::ElementContext<'_, App>,
            theme: &Theme,
            text: &'static str,
            fixed_height: Option<Px>,
        ) -> AnyElement {
            let layout = match fixed_height {
                Some(h) => LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
                None => LayoutRefinement::default().size_full(),
            };
            let layout = decl_style::layout_style(theme, layout);
            cx.container(
                ContainerProps {
                    layout,
                    padding: Edges::all(Px(24.0)),
                    ..Default::default()
                },
                move |cx| vec![ui::text(cx, text).font_semibold().into_element(cx)],
            )
        }

        let inner = fret_ui_shadcn::ResizablePanelGroup::new(model_inner)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![panel(cx, &theme, "Two", None)]).into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![panel(cx, &theme, "Three", None)]).into(),
            ])
            .into_element(cx);

        let outer = fret_ui_shadcn::ResizablePanelGroup::new(model_outer)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![panel(
                    cx,
                    &theme,
                    "One",
                    Some(Px(200.0)),
                )])
                .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![inner]).into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![outer],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret group");

    assert_rect_close_px(
        &format!("{web_name} group"),
        group.bounds,
        web_group.rect,
        2.0,
    );
}

fn assert_resizable_handle_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &["min-h-[200px]", "max-w-md", "rounded-lg", "border"],
    )
    .expect("web resizable group");

    let web_left = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Sidebar")
    })
    .expect("web left panel");
    let web_right = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Content")
    })
    .expect("web right panel");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));
    let left_label: Arc<str> = Arc::from(format!("Golden:{web_name}:left"));
    let right_label: Arc<str> = Arc::from(format!("Golden:{web_name}:right"));

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let fill_layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

        let left_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Sidebar").font_semibold().into_element(cx)],
        );
        let left = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(left_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![left_box],
        );

        let right_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Content").font_semibold().into_element(cx)],
        );
        let right = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(right_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![right_box],
        );

        let group = fret_ui_shadcn::ResizablePanelGroup::new(model)
            .axis(fret_core::Axis::Horizontal)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![left])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![right])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret group");

    assert_rect_close_px(
        &format!("{web_name} group"),
        group.bounds,
        web_group.rect,
        2.0,
    );

    let left =
        find_semantics(&snap, SemanticsRole::Panel, Some(left_label.as_ref())).expect("fret left");
    let right = find_semantics(&snap, SemanticsRole::Panel, Some(right_label.as_ref()))
        .expect("fret right");

    assert_close_px(
        &format!("{web_name} left x"),
        left.bounds.origin.x,
        web_left.rect.x,
        2.0,
    );
    assert_close_px(
        &format!("{web_name} right x"),
        right.bounds.origin.x,
        web_right.rect.x,
        2.0,
    );
}

fn assert_resizable_vertical_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_group = web_find_by_class_tokens(
        &theme.root,
        &["min-h-[200px]", "max-w-md", "rounded-lg", "border"],
    )
    .expect("web resizable group");

    let web_header = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Header")
    })
    .expect("web header panel");
    let web_content = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "p-6") && contains_text(n, "Content")
    })
    .expect("web content panel");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let group_label: Arc<str> = Arc::from(format!("Golden:{web_name}:group"));
    let top_label: Arc<str> = Arc::from(format!("Golden:{web_name}:top"));
    let bottom_label: Arc<str> = Arc::from(format!("Golden:{web_name}:bottom"));

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![0.25, 0.75]);
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let fill_layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

        let top_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Header").font_semibold().into_element(cx)],
        );
        let top = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(top_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![top_box],
        );

        let bottom_box = cx.container(
            ContainerProps {
                layout: fill_layout,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |cx| vec![ui::text(cx, "Content").font_semibold().into_element(cx)],
        );
        let bottom = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(bottom_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![bottom_box],
        );

        let group = fret_ui_shadcn::ResizablePanelGroup::new(model)
            .axis(fret_core::Axis::Vertical)
            .entries(vec![
                fret_ui_shadcn::ResizablePanel::new(vec![top])
                    .min_px(Px(0.0))
                    .into(),
                fret_ui_shadcn::ResizableHandle::new().into(),
                fret_ui_shadcn::ResizablePanel::new(vec![bottom])
                    .min_px(Px(0.0))
                    .into(),
            ])
            .into_element(cx);

        let frame = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(web_group.rect.w)))
                        .h_px(MetricRef::Px(Px(web_group.rect.h))),
                ),
                border: Edges::all(Px(1.0)),
                border_color: Some(border),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(group_label.clone()),
                ..Default::default()
            },
            move |_cx| vec![frame],
        )]
    });

    let group = find_semantics(&snap, SemanticsRole::Panel, Some(group_label.as_ref()))
        .expect("fret group");
    assert_rect_close_px(
        &format!("{web_name} group"),
        group.bounds,
        web_group.rect,
        2.0,
    );

    let top =
        find_semantics(&snap, SemanticsRole::Panel, Some(top_label.as_ref())).expect("fret top");
    let bottom = find_semantics(&snap, SemanticsRole::Panel, Some(bottom_label.as_ref()))
        .expect("fret bottom");

    assert_close_px(
        &format!("{web_name} top y"),
        top.bounds.origin.y,
        web_header.rect.y,
        2.0,
    );
    assert_close_px(
        &format!("{web_name} bottom y"),
        bottom.bounds.origin.y,
        web_content.rect.y,
        2.0,
    );
}
