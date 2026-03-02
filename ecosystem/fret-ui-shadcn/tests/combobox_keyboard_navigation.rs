use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press};

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "combobox-keyboard-navigation",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

#[test]
fn combobox_arrow_down_enter_selects_item_and_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: mount closed and click trigger.
    let value_frame_1 = value.clone();
    let open_frame_1 = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            let items = [
                fret_ui_shadcn::ComboboxItem::new("next", "Next.js"),
                fret_ui_shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                fret_ui_shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            ];

            vec![
                fret_ui_shadcn::Combobox::new(value_frame_1, open_frame_1)
                    .a11y_label("Combobox")
                    .trigger_test_id("combobox-trigger")
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            fret_ui_shadcn::ComboboxPart::from(
                                fret_ui_shadcn::ComboboxInput::new()
                                    .placeholder("Select a framework"),
                            ),
                            fret_ui_shadcn::ComboboxPart::from(
                                fret_ui_shadcn::ComboboxContent::new([
                                    fret_ui_shadcn::ComboboxContentPart::from(
                                        fret_ui_shadcn::ComboboxEmpty::new("No items found."),
                                    ),
                                    fret_ui_shadcn::ComboboxContentPart::from(
                                        fret_ui_shadcn::ComboboxList::new().items(items),
                                    ),
                                ]),
                            ),
                        ]
                    }),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "combobox-trigger");
    let click_pos = Point::new(
        Px(trigger.bounds.origin.x.0 + 5.0),
        Px(trigger.bounds.origin.y.0 + 5.0),
    );
    click_at(&mut ui, &mut app, &mut services, click_pos);
    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected open after click"
    );

    // Frame 2+: settle open overlays before sending keys.
    let settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let value_frame = value.clone();
        let open_frame = open.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            request_semantics,
            move |cx| {
                let items = [
                    fret_ui_shadcn::ComboboxItem::new("next", "Next.js"),
                    fret_ui_shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                    fret_ui_shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                ];

                vec![
                    fret_ui_shadcn::Combobox::new(value_frame, open_frame)
                        .a11y_label("Combobox")
                        .trigger_test_id("combobox-trigger")
                        .test_id_prefix("combobox-test")
                        .into_element_parts(cx, |_cx| {
                            vec![
                                fret_ui_shadcn::ComboboxPart::from(
                                    fret_ui_shadcn::ComboboxInput::new()
                                        .placeholder("Select a framework"),
                                ),
                                fret_ui_shadcn::ComboboxPart::from(
                                    fret_ui_shadcn::ComboboxContent::new([
                                        fret_ui_shadcn::ComboboxContentPart::from(
                                            fret_ui_shadcn::ComboboxEmpty::new("No items found."),
                                        ),
                                        fret_ui_shadcn::ComboboxContentPart::from(
                                            fret_ui_shadcn::ComboboxList::new().items(items),
                                        ),
                                    ]),
                                ),
                            ]
                        }),
                ]
            },
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);

    // Frame after selection commit.
    let value_frame_3 = value.clone();
    let open_frame_3 = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            let items = [
                fret_ui_shadcn::ComboboxItem::new("next", "Next.js"),
                fret_ui_shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                fret_ui_shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            ];

            vec![
                fret_ui_shadcn::Combobox::new(value_frame_3, open_frame_3)
                    .a11y_label("Combobox")
                    .trigger_test_id("combobox-trigger")
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            fret_ui_shadcn::ComboboxPart::from(
                                fret_ui_shadcn::ComboboxInput::new()
                                    .placeholder("Select a framework"),
                            ),
                            fret_ui_shadcn::ComboboxPart::from(
                                fret_ui_shadcn::ComboboxContent::new([
                                    fret_ui_shadcn::ComboboxContentPart::from(
                                        fret_ui_shadcn::ComboboxEmpty::new("No items found."),
                                    ),
                                    fret_ui_shadcn::ComboboxContentPart::from(
                                        fret_ui_shadcn::ComboboxList::new().items(items),
                                    ),
                                ]),
                            ),
                        ]
                    }),
            ]
        },
    );

    assert_eq!(
        app.models().get_cloned(&value).flatten().as_deref(),
        Some("svelte"),
        "expected second item to be selected via keyboard after ArrowDown"
    );
    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected closed after Enter"
    );
}
