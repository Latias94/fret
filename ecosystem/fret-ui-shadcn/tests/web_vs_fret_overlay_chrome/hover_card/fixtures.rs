use super::*;
use fret_ui_shadcn::facade as shadcn;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebThemeName {
    Light,
    Dark,
}

impl WebThemeName {
    fn as_str(&self) -> &'static str {
        match self {
            WebThemeName::Light => "light",
            WebThemeName::Dark => "dark",
        }
    }

    fn scheme(&self) -> shadcn::themes::ShadcnColorScheme {
        match self {
            WebThemeName::Light => shadcn::themes::ShadcnColorScheme::Light,
            WebThemeName::Dark => shadcn::themes::ShadcnColorScheme::Dark,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HoverCardOverlayChromeRecipe {
    PanelChrome,
    DemoPanelSize,
    SurfaceColors,
}

#[derive(Debug, Clone, Deserialize)]
struct HoverCardOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: HoverCardOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_hover_card(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    let trigger_el = shadcn::Button::new("@nextjs")
        .variant(shadcn::ButtonVariant::Link)
        .into_element(cx);
    let content_el = shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

    shadcn::HoverCard::new(cx, trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}

fn assert_hover_card_demo_panel_size_matches_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: shadcn::themes::ShadcnColorScheme,
    settle_frames: u64,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);
    let probe = HoverCardDemoProbe::default();

    let build_probe = probe.clone();
    let build_open = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        move |cx| {
            vec![build_shadcn_hover_card_demo_page_with_probe(
                cx,
                &build_open,
                Some(&build_probe),
            )]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        let build_probe = probe.clone();
        let build_open = open.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames.max(1),
            move |cx| {
                vec![build_shadcn_hover_card_demo_page_with_probe(
                    cx,
                    &build_open,
                    Some(&build_probe),
                )]
            },
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let content_bounds = bounds_for_element(
        &mut app,
        window,
        probe.content.get().expect("hover-card content id"),
    )
    .expect("hover-card content bounds");
    let row_bounds = bounds_for_element(
        &mut app,
        window,
        probe.row.get().expect("hover-card row id"),
    )
    .expect("hover-card row bounds");
    let text_block_bounds = bounds_for_element(
        &mut app,
        window,
        probe.text_block.get().expect("hover-card text block id"),
    )
    .expect("hover-card text block bounds");
    let body_bounds = bounds_for_element(
        &mut app,
        window,
        probe.body.get().expect("hover-card body id"),
    )
    .expect("hover-card body bounds");

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} content.h"),
        content_bounds.size.height.0,
        web_h,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} row.h"),
        row_bounds.size.height.0,
        84.0,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} text_block.h"),
        text_block_bounds.size.height.0,
        84.0,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} body.h"),
        body_bounds.size.height.0,
        40.0,
        1.0,
    );
}

#[test]
fn web_vs_fret_hover_card_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_hover_card_cases_v1.json"
    ));
    let suite: FixtureSuite<HoverCardOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome hover-card fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome hover-card case={}", case.id);
        match case.recipe {
            HoverCardOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches_by_portal_slot(
                    &case.web_name,
                    "hover-card-content",
                    build_hover_card,
                );
            }
            HoverCardOverlayChromeRecipe::DemoPanelSize => {
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_hover_card_demo_panel_size_matches_theme(
                    &case.web_name,
                    "hover-card-content",
                    theme.as_str(),
                    theme.scheme(),
                    crate::shadcn_motion::ticks_100() + 2,
                );
            }
            HoverCardOverlayChromeRecipe::SurfaceColors => {
                let theme = case.theme.as_ref().expect("surface_colors requires theme");
                assert_overlay_chrome_matches_by_portal_slot_theme(
                    &case.web_name,
                    "hover-card-content",
                    theme.as_str(),
                    theme.scheme(),
                    crate::shadcn_motion::ticks_100() + 2,
                    build_hover_card,
                );
            }
        }
    }
}
