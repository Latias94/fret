use super::*;

#[test]
fn web_vs_fret_layout_aspect_ratio_demo_geometry_matches() {
    let web = read_web_golden("aspect-ratio-demo");
    let theme = web_theme(&web);

    let web_img = find_first(&theme.root, &|n| n.tag == "img").expect("web img node");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, _snap, root) = run_fret_root_with_ui(bounds, |cx| {
        let child = cx.container(ContainerProps::default(), |_cx| Vec::new());
        vec![fret_ui_shadcn::AspectRatio::new(16.0 / 9.0, child).into_element(cx)]
    });

    let (_node, fret_bounds) = find_node_with_bounds_close(&ui, root, web_img.rect, 2.0)
        .expect("fret aspect ratio bounds close to web image rect");
    assert_rect_close_px("aspect-ratio-demo", fret_bounds, web_img.rect, 2.0);
}

#[test]
fn web_vs_fret_layout_checkbox_demo_control_size() {
    let web = read_web_golden("checkbox-demo");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Checkbox::new(model)
                .a11y_label("Checkbox")
                .into_element(cx),
        ]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Checkbox"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox semantics node");

    assert_close_px(
        "checkbox width",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox height",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_label_demo_geometry() {
    let web = read_web_golden("label-demo");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Accept terms and conditions")
        .expect("web label");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        let checkbox = fret_ui_shadcn::Checkbox::new(model)
            .a11y_label("Terms")
            .into_element(cx);
        let label = fret_ui_shadcn::Label::new("Accept terms and conditions").into_element(cx);
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:label-demo:label")),
                ..Default::default()
            },
            move |_cx| vec![label],
        );

        let row = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(8.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![checkbox, label],
        );

        vec![row]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Terms"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox node");
    let label = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:label-demo:label"))
        .expect("fret label node");

    assert_close_px(
        "label-demo checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "label-demo checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );

    assert_close_px(
        "label-demo label x",
        label.bounds.origin.x,
        web_label.rect.x,
        1.0,
    );
    assert_close_px(
        "label-demo label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "label-demo label h",
        label.bounds.size.height,
        web_label.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_checkbox_with_text_geometry() {
    let web = read_web_golden("checkbox-with-text");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
    })
    .expect("web checkbox");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Accept terms and conditions")
        .expect("web label");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Terms of Service").expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let model: Model<bool> = cx.app.models_mut().insert(false);

        let checkbox = fret_ui_shadcn::Checkbox::new(model)
            .a11y_label("Terms")
            .into_element(cx);

        let label = fret_ui_shadcn::Label::new("Accept terms and conditions").into_element(cx);
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:checkbox-with-text:label")),
                ..Default::default()
            },
            move |_cx| vec![label],
        );

        let desc = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from("You agree to our Terms of Service and Privacy Policy."),
            style: None,
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });
        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:checkbox-with-text:desc")),
                ..Default::default()
            },
            move |_cx| vec![desc],
        );

        let content = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Vertical,
                gap: Px(6.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![label, desc],
        );

        let row = cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(8.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![checkbox, content],
        );

        vec![row]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Terms"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox node");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:checkbox-with-text:label"),
    )
    .expect("fret label node");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:checkbox-with-text:desc"),
    )
    .expect("fret desc node");

    assert_close_px(
        "checkbox-with-text checkbox w",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox-with-text checkbox h",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );

    assert_close_px(
        "checkbox-with-text label x",
        label.bounds.origin.x,
        web_label.rect.x,
        1.0,
    );
    assert_close_px(
        "checkbox-with-text label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );

    assert_close_px(
        "checkbox-with-text desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_slider_demo_geometry() {
    let web = read_web_golden("slider-demo");
    let theme = web_theme(&web);
    let web_thumb = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|r| r == "slider")
    })
    .expect("web slider thumb");

    let thumb_center_y = web_thumb.rect.y + web_thumb.rect.h * 0.5;
    let web_track = web_find_best_by(
        &theme.root,
        &|n| {
            n.tag == "span"
                && n.attrs
                    .get("data-orientation")
                    .is_some_and(|v| v == "horizontal")
                && class_has_token(n, "bg-muted")
                && class_has_token(n, "rounded-full")
                && (n.rect.h - 6.0).abs() <= 0.1
        },
        &|n| ((n.rect.y + n.rect.h * 0.5) - thumb_center_y).abs(),
    )
    .expect("web slider track");

    let web_range = web_find_best_by(
        &theme.root,
        &|n| {
            n.tag == "span"
                && n.attrs
                    .get("data-orientation")
                    .is_some_and(|v| v == "horizontal")
                && class_has_token(n, "bg-primary")
                && class_has_token(n, "absolute")
                && (n.rect.h - 6.0).abs() <= 0.1
        },
        &|n| ((n.rect.y + n.rect.h * 0.5) - thumb_center_y).abs(),
    )
    .expect("web slider range");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let t = (web_thumb.rect.x + web_thumb.rect.w * 0.5) / web_track.rect.w.max(1.0);
    let initial_value = 100.0 * t.clamp(0.0, 1.0);

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![initial_value]);
        let slider = fret_ui_shadcn::Slider::new(model)
            .range(0.0, 100.0)
            .a11y_label("Slider")
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_track.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![slider],
        )]
    });

    let thumb = find_semantics(&snap, SemanticsRole::Slider, Some("Slider"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Slider, None))
        .expect("fret slider thumb semantics");
    let slider = thumb
        .parent
        .and_then(|parent| snap.nodes.iter().find(|n| n.id == parent))
        .unwrap_or(thumb);

    assert_close_px(
        "slider layout width",
        slider.bounds.size.width,
        web_track.rect.w,
        1.0,
    );
    assert_close_px(
        "slider layout height",
        slider.bounds.size.height,
        web_track.rect.h,
        1.0,
    );

    let mut stack = vec![slider.id];
    let mut rects: Vec<(NodeId, Rect)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(bounds) = ui.debug_node_bounds(node) {
            rects.push((node, bounds));
        }
        for child in ui.children(node).into_iter().rev() {
            stack.push(child);
        }
    }

    let pick_best = |label: &str, expected: WebRect, rects: &[(NodeId, Rect)]| -> Rect {
        let mut best: Option<Rect> = None;
        let mut best_score = f32::INFINITY;
        for (_, rect) in rects {
            let score = (rect.origin.x.0 - expected.x).abs()
                + (rect.origin.y.0 - expected.y).abs()
                + (rect.size.width.0 - expected.w).abs()
                + (rect.size.height.0 - expected.h).abs();
            if score < best_score {
                best_score = score;
                best = Some(*rect);
            }
        }
        best.unwrap_or_else(|| panic!("missing {label} match"))
    };

    let fret_track = pick_best("track", web_track.rect, &rects);
    let fret_range = pick_best("range", web_range.rect, &rects);
    let fret_thumb = pick_best("thumb", web_thumb.rect, &rects);

    assert_close_px("track x", fret_track.origin.x, web_track.rect.x, 1.0);
    assert_close_px("track y", fret_track.origin.y, web_track.rect.y, 1.0);
    assert_close_px("track w", fret_track.size.width, web_track.rect.w, 1.0);
    assert_close_px("track h", fret_track.size.height, web_track.rect.h, 1.0);

    assert_close_px("range x", fret_range.origin.x, web_range.rect.x, 1.0);
    assert_close_px("range y", fret_range.origin.y, web_range.rect.y, 1.0);
    assert_close_px("range w", fret_range.size.width, web_range.rect.w, 1.0);
    assert_close_px("range h", fret_range.size.height, web_range.rect.h, 1.0);

    assert_close_px("thumb x", fret_thumb.origin.x, web_thumb.rect.x, 1.0);
    assert_close_px("thumb y", fret_thumb.origin.y, web_thumb.rect.y, 1.0);
    assert_close_px("thumb w", fret_thumb.size.width, web_thumb.rect.w, 1.0);
    assert_close_px("thumb h", fret_thumb.size.height, web_thumb.rect.h, 1.0);
}
#[test]
fn web_vs_fret_layout_checkbox_disabled_control_size_matches_web() {
    let web = read_web_golden("checkbox-disabled");
    let theme = web_theme(&web);
    let web_checkbox = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-checked").is_some_and(|v| v == "false")
            && n.attrs.contains_key("data-disabled")
    })
    .expect("web checkbox");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<bool> = cx.app.models_mut().insert(false);
        vec![
            fret_ui_shadcn::Checkbox::new(model)
                .a11y_label("Checkbox")
                .disabled(true)
                .into_element(cx),
        ]
    });

    let checkbox = find_semantics(&snap, SemanticsRole::Checkbox, Some("Checkbox"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret checkbox semantics node");

    assert_close_px(
        "checkbox-disabled width",
        checkbox.bounds.size.width,
        web_checkbox.rect.w,
        1.0,
    );
    assert_close_px(
        "checkbox-disabled height",
        checkbox.bounds.size.height,
        web_checkbox.rect.h,
        1.0,
    );
}
