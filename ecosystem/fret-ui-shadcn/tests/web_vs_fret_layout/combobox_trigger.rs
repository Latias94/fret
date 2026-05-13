use super::*;
use fret_ui_shadcn::facade as shadcn;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LayoutComboboxTriggerRecipe {
    DemoTrigger,
    ResponsiveTrigger,
    PopoverTrigger,
    DemoLongSelectedTrigger,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LayoutComboboxTriggerIconExpectation {
    Present,
    Absent,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutComboboxTriggerCase {
    id: String,
    web_name: String,
    recipe: LayoutComboboxTriggerRecipe,
    label: String,
    icon: LayoutComboboxTriggerIconExpectation,
    #[serde(default)]
    icon_svg_contains: Vec<String>,
}

#[test]
fn web_vs_fret_layout_combobox_trigger_slots_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_combobox_trigger_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutComboboxTriggerCase> =
        serde_json::from_str(raw).expect("layout combobox trigger fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout combobox trigger case={}", case.id);

        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);
        let web_trigger = web_combobox_trigger(theme, case.recipe);
        let web_icon = find_first(web_trigger, &|n| n.tag == "svg");

        match case.icon {
            LayoutComboboxTriggerIconExpectation::Present => {
                assert!(
                    web_icon.is_some(),
                    "{} web trigger should include an icon slot",
                    case.id
                );
            }
            LayoutComboboxTriggerIconExpectation::Absent => {
                assert!(
                    web_icon.is_none(),
                    "{} web trigger should not include an icon slot",
                    case.id
                );
            }
        }

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );
        let (snap, scene, services) = render_fret_combobox_trigger(bounds, case.recipe);

        let trigger = find_by_test_id(&snap, "combobox-trigger-fixture-trigger");
        assert_close_px(
            &format!("{} trigger w", case.id),
            trigger.bounds.size.width,
            web_trigger.rect.w,
            1.0,
        );
        assert_close_px(
            &format!("{} trigger h", case.id),
            trigger.bounds.size.height,
            web_trigger.rect.h,
            1.0,
        );

        let label = find_by_test_id(&snap, "combobox-trigger-fixture-trigger-label");
        let expected_font_weight = web_css_u16(web_trigger, "fontWeight");
        let prepared_label = services
            .prepared
            .iter()
            .find(|record| record.text == case.label)
            .unwrap_or_else(|| {
                panic!(
                    "{} missing prepared label text {:?}; prepared={:?}",
                    case.id, case.label, services.prepared
                )
            });
        assert_eq!(
            prepared_label.style.weight.0, expected_font_weight,
            "{} label font weight should match web trigger",
            case.id
        );
        assert!(
            fret_rect_contains(trigger.bounds, label.bounds),
            "{} label should stay inside trigger: trigger={:?} label={:?}",
            case.id,
            trigger.bounds,
            label.bounds
        );

        let icon = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("combobox-trigger-fixture-trigger-icon"));

        match (case.icon, icon, web_icon) {
            (LayoutComboboxTriggerIconExpectation::Absent, None, None) => {
                let content_width = web_trigger.rect.w
                    - web_css_px(web_trigger, "paddingLeft").0
                    - web_css_px(web_trigger, "paddingRight").0;
                assert!(
                    label.bounds.size.width.0 >= content_width - 1.0,
                    "{} text-only trigger label should own the content lane: label={:?} content_width={content_width}",
                    case.id,
                    label.bounds
                );
            }
            (LayoutComboboxTriggerIconExpectation::Absent, Some(icon), None) => {
                panic!(
                    "{} Fret trigger should not render an icon slot: icon={:?}",
                    case.id, icon.bounds
                );
            }
            (LayoutComboboxTriggerIconExpectation::Present, Some(icon), Some(web_icon)) => {
                assert_close_px(
                    &format!("{} icon w", case.id),
                    icon.bounds.size.width,
                    web_icon.rect.w,
                    1.0,
                );
                assert_close_px(
                    &format!("{} icon h", case.id),
                    icon.bounds.size.height,
                    web_icon.rect.h,
                    1.0,
                );

                let fret_icon_right = icon.bounds.origin.x.0 + icon.bounds.size.width.0;
                let fret_trigger_right = trigger.bounds.origin.x.0 + trigger.bounds.size.width.0;
                let web_icon_right = web_icon.rect.x + web_icon.rect.w;
                let web_trigger_right = web_trigger.rect.x + web_trigger.rect.w;
                assert_close_px(
                    &format!("{} icon right inset", case.id),
                    Px(fret_trigger_right - fret_icon_right),
                    web_trigger_right - web_icon_right,
                    1.0,
                );

                let fret_icon_center_y = icon.bounds.origin.y.0 + icon.bounds.size.height.0 * 0.5;
                let fret_trigger_center_y =
                    trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5;
                let web_icon_center_y = web_icon.rect.y + web_icon.rect.h * 0.5;
                let web_trigger_center_y = web_trigger.rect.y + web_trigger.rect.h * 0.5;
                assert_close_px(
                    &format!("{} icon center-y offset", case.id),
                    Px(fret_icon_center_y - fret_trigger_center_y),
                    web_icon_center_y - web_trigger_center_y,
                    1.0,
                );

                let label_right = label.bounds.origin.x.0 + label.bounds.size.width.0;
                assert!(
                    label_right <= icon.bounds.origin.x.0 + 1.0,
                    "{} label should not overlap icon: label={:?} icon={:?}",
                    case.id,
                    label.bounds,
                    icon.bounds
                );

                if !case.icon_svg_contains.is_empty() {
                    let (_rect, svg) = find_scene_svg_with_rect_close(&scene, icon.bounds, 1.0)
                        .unwrap_or_else(|| {
                            panic!(
                                "{} missing painted SVG for trigger icon bounds {:?}",
                                case.id, icon.bounds
                            )
                        });
                    let bytes = services.svg_bytes(svg).unwrap_or_else(|| {
                        panic!("{} missing registered SVG bytes for {svg:?}", case.id)
                    });
                    let svg_text = std::str::from_utf8(bytes)
                        .unwrap_or_else(|err| panic!("{} SVG bytes not utf8: {err}", case.id));
                    for expected in &case.icon_svg_contains {
                        assert!(
                            svg_text.contains(expected),
                            "{} trigger icon SVG should contain {:?}; svg={svg_text}",
                            case.id,
                            expected
                        );
                    }
                }
            }
            (LayoutComboboxTriggerIconExpectation::Present, None, Some(_)) => {
                panic!("{} Fret trigger should render an icon slot", case.id);
            }
            _ => panic!("{} inconsistent web/Fret icon expectation", case.id),
        }
    }
}

fn web_combobox_trigger(theme: &WebGoldenTheme, recipe: LayoutComboboxTriggerRecipe) -> &WebNode {
    match recipe {
        LayoutComboboxTriggerRecipe::DemoTrigger
        | LayoutComboboxTriggerRecipe::DemoLongSelectedTrigger => find_first(&theme.root, &|n| {
            n.tag == "button" && n.attrs.get("role").is_some_and(|role| role == "combobox")
        })
        .expect("web combobox-demo trigger"),
        LayoutComboboxTriggerRecipe::ResponsiveTrigger
        | LayoutComboboxTriggerRecipe::PopoverTrigger => find_first(&theme.root, &|n| {
            n.tag == "button" && class_has_token(n, "w-[150px]")
        })
        .expect("web combobox text-only Button trigger"),
    }
}

fn render_fret_combobox_trigger(
    bounds: Rect,
    recipe: LayoutComboboxTriggerRecipe,
) -> (fret_core::SemanticsSnapshot, Scene, StyleAwareServices) {
    let mut services = StyleAwareServices::default();
    let (snap, scene) = render_and_paint_in_bounds_with_services(bounds, &mut services, |cx| {
        use std::sync::Arc;

        let model = match recipe {
            LayoutComboboxTriggerRecipe::DemoLongSelectedTrigger => cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("enterprise-observability-platform"))),
            LayoutComboboxTriggerRecipe::DemoTrigger
            | LayoutComboboxTriggerRecipe::ResponsiveTrigger
            | LayoutComboboxTriggerRecipe::PopoverTrigger => {
                cx.app.models_mut().insert(None::<Arc<str>>)
            }
        };
        let open = cx.app.models_mut().insert(false);
        let query = cx.app.models_mut().insert(String::new());

        let mut combobox = shadcn::Combobox::new(model, open)
            .query_model(query)
            .a11y_label("Combobox trigger fixture")
            .test_id_prefix("combobox-trigger-fixture");

        combobox = match recipe {
            LayoutComboboxTriggerRecipe::DemoTrigger => combobox
                .items(framework_items())
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(200.0)),
                )
                .input(
                    shadcn::ComboboxInput::new()
                        .placeholder("Select framework...")
                        .show_trigger(true),
                ),
            LayoutComboboxTriggerRecipe::ResponsiveTrigger => combobox
                .device_shell_responsive(true)
                .items(status_items())
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(150.0)),
                )
                .input(shadcn::ComboboxInput::new().placeholder("+ Set status")),
            LayoutComboboxTriggerRecipe::PopoverTrigger => combobox
                .items(status_items())
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(150.0)),
                )
                .input(shadcn::ComboboxInput::new().placeholder("+ Set status")),
            LayoutComboboxTriggerRecipe::DemoLongSelectedTrigger => combobox
                .items([shadcn::ComboboxItem::new(
                    "enterprise-observability-platform",
                    "Enterprise Observability Platform With Extremely Long Label",
                )])
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(200.0)),
                )
                .input(
                    shadcn::ComboboxInput::new()
                        .placeholder("Select framework...")
                        .show_trigger(true),
                ),
        };

        vec![combobox.into_element(cx)]
    });

    (snap, scene, services)
}

fn framework_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("next.js", "Next.js"),
        shadcn::ComboboxItem::new("sveltekit", "SvelteKit"),
        shadcn::ComboboxItem::new("nuxt.js", "Nuxt.js"),
        shadcn::ComboboxItem::new("remix", "Remix"),
        shadcn::ComboboxItem::new("astro", "Astro"),
    ]
}

fn status_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("backlog", "Backlog"),
        shadcn::ComboboxItem::new("todo", "Todo"),
        shadcn::ComboboxItem::new("in progress", "In Progress"),
        shadcn::ComboboxItem::new("done", "Done"),
        shadcn::ComboboxItem::new("canceled", "Canceled"),
    ]
}
