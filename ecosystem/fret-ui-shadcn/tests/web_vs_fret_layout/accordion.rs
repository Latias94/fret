use super::*;

#[test]
fn web_vs_fret_layout_accordion_demo_geometry_light() {
    let web = read_web_golden("accordion-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let mut web_buttons = Vec::new();
    web_collect_tag(&theme.root, "button", &mut web_buttons);
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 accordion triggers");

    let web_items: Vec<&WebNode> = {
        let mut all = Vec::new();
        web_collect_all(&theme.root, &mut all);
        let mut items: Vec<&WebNode> = all
            .into_iter()
            .filter(|n| n.tag == "div" && class_has_token(n, "border-b"))
            .collect();
        items.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
        items
    };
    assert_eq!(web_items.len(), 3, "expected 3 accordion items");

    let web_open_content =
        web_find_by_class_tokens(&theme.root, &["pt-0", "pb-4", "flex", "flex-col", "gap-4"])
            .expect("web open accordion content wrapper");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let default_value = Some(Arc::from("item-1"));
    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_shadcn::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

        let item_1 = AccordionItem::new(
            Arc::from("item-1"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![
                decl_text::text_sm(
                    cx,
                    "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                ),
                decl_text::text_sm(
                    cx,
                    "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                ),
            ]),
        );
        let item_2 = AccordionItem::new(
            Arc::from("item-2"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 2")]),
        );
        let item_3 = AccordionItem::new(
            Arc::from("item-3"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 3")]),
        );

        let accordion = Accordion::single_uncontrolled(default_value.clone())
            .collapsible(true)
            .items([item_1, item_2, item_3])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(theme.viewport.w));
                    layout
                },
                ..Default::default()
            },
            move |_cx| vec![accordion],
        )]
    };

    for frame in 0..12 {
        app.set_frame_id(FrameId(frame));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            &render,
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let trig_1 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-1"))
        .expect("fret trigger item-1");
    let trig_2 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-2"))
        .expect("fret trigger item-2");
    let trig_3 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-3"))
        .expect("fret trigger item-3");

    assert_rect_close_px(
        "accordion-demo trigger 1",
        trig_1.bounds,
        web_buttons[0].rect,
        1.0,
    );

    let content_id = *trig_1
        .controls
        .first()
        .expect("expected controls on item-1");
    let content = snap
        .nodes
        .iter()
        .find(|n| n.id == content_id)
        .expect("fret content node (item-1)");
    assert_rect_close_px(
        "accordion-demo open content wrapper",
        content.bounds,
        web_open_content.rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 2",
        trig_2.bounds,
        web_buttons[1].rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 3",
        trig_3.bounds,
        web_buttons[2].rect,
        1.0,
    );

    let item_1_h = trig_2.bounds.origin.y.0 - trig_1.bounds.origin.y.0;
    let item_2_h = trig_3.bounds.origin.y.0 - trig_2.bounds.origin.y.0;
    assert_close_px(
        "accordion-demo item 1 height",
        Px(item_1_h),
        web_items[0].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 2 height",
        Px(item_2_h),
        web_items[1].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 3 height",
        trig_3.bounds.size.height,
        web_items[2].rect.h,
        1.0,
    );
}
#[test]
fn web_vs_fret_layout_accordion_demo_geometry_dark() {
    let web = read_web_golden("accordion-demo");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let mut web_buttons = Vec::new();
    web_collect_tag(&theme.root, "button", &mut web_buttons);
    web_buttons.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_buttons.len(), 3, "expected 3 accordion triggers");

    let web_items: Vec<&WebNode> = {
        let mut all = Vec::new();
        web_collect_all(&theme.root, &mut all);
        let mut items: Vec<&WebNode> = all
            .into_iter()
            .filter(|n| n.tag == "div" && class_has_token(n, "border-b"))
            .collect();
        items.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
        items
    };
    assert_eq!(web_items.len(), 3, "expected 3 accordion items");

    let web_open_content =
        web_find_by_class_tokens(&theme.root, &["pt-0", "pb-4", "flex", "flex-col", "gap-4"])
            .expect("web open accordion content wrapper");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let default_value = Some(Arc::from("item-1"));
    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_shadcn::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

        let item_1 = AccordionItem::new(
            Arc::from("item-1"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![
                decl_text::text_sm(
                    cx,
                    "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                ),
                decl_text::text_sm(
                    cx,
                    "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                ),
            ]),
        );
        let item_2 = AccordionItem::new(
            Arc::from("item-2"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 2")]),
        );
        let item_3 = AccordionItem::new(
            Arc::from("item-3"),
            AccordionTrigger::new(Vec::new()),
            AccordionContent::new(vec![decl_text::text_sm(cx, "Content 3")]),
        );

        let accordion = Accordion::single_uncontrolled(default_value.clone())
            .collapsible(true)
            .items([item_1, item_2, item_3])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(theme.viewport.w));
                    layout
                },
                ..Default::default()
            },
            move |_cx| vec![accordion],
        )]
    };

    for frame in 0..12 {
        app.set_frame_id(FrameId(frame));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            &render,
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let trig_1 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-1"))
        .expect("fret trigger item-1");
    let trig_2 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-2"))
        .expect("fret trigger item-2");
    let trig_3 = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("item-3"))
        .expect("fret trigger item-3");

    assert_rect_close_px(
        "accordion-demo trigger 1 (dark)",
        trig_1.bounds,
        web_buttons[0].rect,
        1.0,
    );

    let content_id = *trig_1
        .controls
        .first()
        .expect("expected controls on item-1");
    let content = snap
        .nodes
        .iter()
        .find(|n| n.id == content_id)
        .expect("fret content node (item-1)");
    assert_rect_close_px(
        "accordion-demo open content wrapper (dark)",
        content.bounds,
        web_open_content.rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 2 (dark)",
        trig_2.bounds,
        web_buttons[1].rect,
        1.0,
    );
    assert_rect_close_px(
        "accordion-demo trigger 3 (dark)",
        trig_3.bounds,
        web_buttons[2].rect,
        1.0,
    );

    let item_1_h = trig_2.bounds.origin.y.0 - trig_1.bounds.origin.y.0;
    let item_2_h = trig_3.bounds.origin.y.0 - trig_2.bounds.origin.y.0;
    assert_close_px(
        "accordion-demo item 1 height (dark)",
        Px(item_1_h),
        web_items[0].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 2 height (dark)",
        Px(item_2_h),
        web_items[1].rect.h,
        1.0,
    );
    assert_close_px(
        "accordion-demo item 3 height (dark)",
        trig_3.bounds.size.height,
        web_items[2].rect.h,
        1.0,
    );
}
