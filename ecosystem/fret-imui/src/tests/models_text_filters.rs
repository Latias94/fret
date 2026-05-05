use super::*;
use fret_ui_kit::imui::InputTextCustomFilter;
use fret_ui_kit::imui::InputTextFilters;
use fret_ui_kit::imui::InputTextOptions;

#[test]
fn input_text_named_filters_transform_and_reject_text_input() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let changed = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>, changed_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            changed_out.set(
                ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        filters: InputTextFilters::uppercase().with_no_blank(),
                        test_id: Some(Arc::from("imui-input-text-filter-uppercase")),
                        ..Default::default()
                    },
                )
                .changed(),
            );
        })
    };

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-filter-uppercase",
        |cx| render(cx, &changed_out),
    );
    assert!(!changed.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "ab c\tD1");
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("ABCD1"));

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-filter-uppercase",
        |cx| render(cx, &changed_out),
    );
    assert!(changed.get());
}

#[test]
fn input_text_numeric_filters_match_imgui_named_sets() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let decimal = app.models_mut().insert(String::new());
    let scientific = app.models_mut().insert(String::new());
    let hexadecimal = app.models_mut().insert(String::new());

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.input_text_model_with_options(
                    &decimal,
                    InputTextOptions {
                        filters: InputTextFilters::decimal(),
                        test_id: Some(Arc::from("imui-input-text-filter-decimal")),
                        ..Default::default()
                    },
                );
                let _ = ui.input_text_model_with_options(
                    &scientific,
                    InputTextOptions {
                        filters: InputTextFilters::scientific(),
                        test_id: Some(Arc::from("imui-input-text-filter-scientific")),
                        ..Default::default()
                    },
                );
                let _ = ui.input_text_model_with_options(
                    &hexadecimal,
                    InputTextOptions {
                        filters: InputTextFilters::hexadecimal().with_uppercase(),
                        test_id: Some(Arc::from("imui-input-text-filter-hexadecimal")),
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-filter-numeric",
        render,
    );

    let decimal_at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-filter-decimal",
    );
    click_at(&mut ui, &mut app, &mut services, decimal_at);
    text_input_event(&mut ui, &mut app, &mut services, "a1e+2.3*/-");
    assert_eq!(
        app.models().get_cloned(&decimal).as_deref(),
        Some("1+2.3*/-")
    );

    let scientific_at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-filter-scientific",
    );
    click_at(&mut ui, &mut app, &mut services, scientific_at);
    text_input_event(&mut ui, &mut app, &mut services, "a1e+2.3*/-");
    assert_eq!(
        app.models().get_cloned(&scientific).as_deref(),
        Some("1e+2.3*/-")
    );

    let hexadecimal_at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-filter-hexadecimal",
    );
    click_at(&mut ui, &mut app, &mut services, hexadecimal_at);
    text_input_event(&mut ui, &mut app, &mut services, "gG19aFz-");
    assert_eq!(
        app.models().get_cloned(&hexadecimal).as_deref(),
        Some("19AF")
    );
}

#[test]
fn input_text_custom_filter_runs_after_named_filters() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-custom-filter",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        filters: InputTextFilters::uppercase().with_no_blank(),
                        custom_filter: Some(InputTextCustomFilter::new(|text| {
                            text.chars().filter(|c| *c != 'B').collect()
                        })),
                        test_id: Some(Arc::from("imui-input-text-custom-filter")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "ab c\tD1");

    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("ACD1"));
}
