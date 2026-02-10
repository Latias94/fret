use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DropdownMenuRadioGroupRecipe {
    OverlayPlacement,
    RadioIndicatorSlotInset,
    MenuContentInsets,
    MenuItemHeight,
}

#[derive(Debug, Clone, Deserialize)]
struct DropdownMenuRadioGroupCase {
    id: String,
    web_name: String,
    recipe: DropdownMenuRadioGroupRecipe,
}

fn build_dropdown_menu_radio_group_demo_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    #[derive(Default)]
    struct Models {
        position: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());

    let position = if let Some(existing) = existing {
        existing
    } else {
        let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
        cx.with_state(Models::default, |st| st.position = Some(position.clone()));
        position
    };

    build_dropdown_menu_radio_group_demo(cx, open, position)
}

fn assert_dropdown_menu_radio_group_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let position: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("bottom")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_dropdown_menu_radio_group_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-radio-item"]);
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web dropdown-menu-radio-item height for {web_name}")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let position: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("bottom")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_dropdown_menu_radio_group_cases_v1.json"
    ));
    let suite: FixtureSuite<DropdownMenuRadioGroupCase> =
        serde_json::from_str(raw).expect("dropdown-menu-radio-group fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("dropdown-menu-radio-group case={}", case.id);
        match case.recipe {
            DropdownMenuRadioGroupRecipe::OverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("menu"),
                    |cx, open| build_dropdown_menu_radio_group_demo_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Open"),
                    SemanticsRole::Menu,
                );
            }
            DropdownMenuRadioGroupRecipe::RadioIndicatorSlotInset => {
                assert_dropdown_menu_radio_group_indicator_slot_inset_matches_web_impl(
                    &case.web_name,
                );
            }
            DropdownMenuRadioGroupRecipe::MenuContentInsets => {
                assert_dropdown_menu_radio_group_menu_content_insets_match(&case.web_name);
            }
            DropdownMenuRadioGroupRecipe::MenuItemHeight => {
                assert_dropdown_menu_radio_group_menu_item_height_matches(&case.web_name);
            }
        }
    }
}
