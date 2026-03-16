use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

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
        "combobox-test-id-prefix-semantics",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_has_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) {
    let found = snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id));
    assert!(found, "missing semantics node with test_id={id:?}");
}

#[test]
fn combobox_prefix_test_ids_survive_open_close_open() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: mount closed.
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
                shadcn::ComboboxItem::new("Next.js", "Next.js"),
                shadcn::ComboboxItem::new("SvelteKit", "SvelteKit"),
                shadcn::ComboboxItem::new("Nuxt.js", "Nuxt.js"),
            ];

            vec![
                shadcn::Combobox::new(value_frame_1, open_frame_1)
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            shadcn::ComboboxPart::from(
                                shadcn::ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                                shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                    "No items found.",
                                )),
                                shadcn::ComboboxContentPart::from(
                                    shadcn::ComboboxList::new().items(items),
                                ),
                            ])),
                        ]
                    }),
            ]
        },
    );

    // Frame 2: open.
    let _ = app.models_mut().update(&open, |v| *v = true);
    let value_frame_2 = value.clone();
    let open_frame_2 = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            let items = [
                shadcn::ComboboxItem::new("Next.js", "Next.js"),
                shadcn::ComboboxItem::new("SvelteKit", "SvelteKit"),
                shadcn::ComboboxItem::new("Nuxt.js", "Nuxt.js"),
            ];

            vec![
                shadcn::Combobox::new(value_frame_2, open_frame_2)
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            shadcn::ComboboxPart::from(
                                shadcn::ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                                shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                    "No items found.",
                                )),
                                shadcn::ComboboxContentPart::from(
                                    shadcn::ComboboxList::new().items(items),
                                ),
                            ])),
                        ]
                    }),
            ]
        },
    );
    {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert_has_test_id(&snap, "combobox-test-input");
        assert_has_test_id(&snap, "combobox-test-listbox");
        assert_has_test_id(&snap, "combobox-test-item-next-js");
    }

    // Frame 3: close.
    let _ = app.models_mut().update(&open, |v| *v = false);
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
                shadcn::ComboboxItem::new("Next.js", "Next.js"),
                shadcn::ComboboxItem::new("SvelteKit", "SvelteKit"),
                shadcn::ComboboxItem::new("Nuxt.js", "Nuxt.js"),
            ];

            vec![
                shadcn::Combobox::new(value_frame_3, open_frame_3)
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            shadcn::ComboboxPart::from(
                                shadcn::ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                                shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                    "No items found.",
                                )),
                                shadcn::ComboboxContentPart::from(
                                    shadcn::ComboboxList::new().items(items),
                                ),
                            ])),
                        ]
                    }),
            ]
        },
    );

    // Frame 4: open again, ids still present.
    let _ = app.models_mut().update(&open, |v| *v = true);
    let value_frame_4 = value;
    let open_frame_4 = open;
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            let items = [
                shadcn::ComboboxItem::new("Next.js", "Next.js"),
                shadcn::ComboboxItem::new("SvelteKit", "SvelteKit"),
                shadcn::ComboboxItem::new("Nuxt.js", "Nuxt.js"),
            ];

            vec![
                shadcn::Combobox::new(value_frame_4, open_frame_4)
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            shadcn::ComboboxPart::from(
                                shadcn::ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                                shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                    "No items found.",
                                )),
                                shadcn::ComboboxContentPart::from(
                                    shadcn::ComboboxList::new().items(items),
                                ),
                            ])),
                        ]
                    }),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert_has_test_id(&snap, "combobox-test-input");
    assert_has_test_id(&snap, "combobox-test-listbox");
    assert_has_test_id(&snap, "combobox-test-item-next-js");
}
