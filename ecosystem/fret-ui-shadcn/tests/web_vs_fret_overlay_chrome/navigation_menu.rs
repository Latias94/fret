use super::*;

#[test]
fn web_vs_fret_navigation_menu_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_chrome_matches(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_surface_colors_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_content_surface_colors_match(
        "navigation-menu-demo",
        "navigation-menu-content",
        "open",
        "home",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-viewport",
        "open",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_surface_colors_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-viewport",
        "open",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_viewport_shadow_insets_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-viewport",
        "open",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_viewport_shadow_insets_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_viewport_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-viewport",
        "open",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_trigger_open_and_closed_surface_colors_match_web() {
    assert_navigation_menu_trigger_surface_colors_match(
        "navigation-menu-demo",
        "Home",
        "home",
        "List",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_trigger_open_and_closed_surface_colors_match_web_dark() {
    assert_navigation_menu_trigger_surface_colors_match(
        "navigation-menu-demo",
        "Home",
        "home",
        "List",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_trigger_open_and_closed_surface_colors_match_web() {
    assert_navigation_menu_trigger_surface_colors_match(
        "navigation-menu-demo.components",
        "Components",
        "components",
        "List",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_components_trigger_open_and_closed_surface_colors_match_web_dark()
 {
    assert_navigation_menu_trigger_surface_colors_match(
        "navigation-menu-demo.components",
        "Components",
        "components",
        "List",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_indicator_shadow_insets_match_web() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_indicator_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-indicator",
        "visible",
        "Home",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
#[test]
fn web_vs_fret_navigation_menu_demo_indicator_shadow_insets_match_web_dark() {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    assert_navigation_menu_indicator_shadow_insets_match(
        "navigation-menu-demo-indicator",
        "navigation-menu-indicator",
        "visible",
        "Home",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        |cx, model, root_id_out| {
            let el = NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(true)
                .items(vec![NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![cx.text("Content")],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            el
        },
    );
}
