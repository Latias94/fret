use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::shadcn_themes;
use fret_ui_shadcn::{
    Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxChipsPart, ComboboxContent,
    ComboboxContentPart, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPart,
    ComboboxValue,
};
use std::sync::Arc;

struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                baseline: fret_core::Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
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
        "combobox-v4-parts-semantic-gate",
        root,
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_semantics_has_combobox_and_listbox(ui: &UiTree<App>) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let _ = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .expect("missing combobox semantics");
    let _ = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("missing listbox semantics (open)");
}

#[test]
fn combobox_v4_parts_produce_listbox_semantics_when_open() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        CoreSize::new(fret_core::Px(640.0), fret_core::Px(480.0)),
    );

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    // Frame 1: mount closed so element ids are stable before opening overlays.
    let value_frame_1 = value.clone();
    let open_frame_1 = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        move |cx| {
            let items = [
                ComboboxItem::new("next", "Next.js"),
                ComboboxItem::new("svelte", "SvelteKit"),
                ComboboxItem::new("nuxt", "Nuxt.js"),
            ];

            vec![
                Combobox::new(value_frame_1.clone(), open_frame_1.clone()).into_element_parts(
                    cx,
                    |_cx| {
                        vec![
                            ComboboxPart::from(
                                ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            ComboboxPart::from(ComboboxContent::new([
                                ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                                ComboboxContentPart::from(ComboboxList::new().items(items)),
                            ])),
                        ]
                    },
                ),
            ]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2: open + semantics snapshot.
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
                ComboboxItem::new("next", "Next.js"),
                ComboboxItem::new("svelte", "SvelteKit"),
                ComboboxItem::new("nuxt", "Nuxt.js"),
            ];

            vec![
                Combobox::new(value_frame_2.clone(), open_frame_2.clone()).into_element_parts(
                    cx,
                    |_cx| {
                        vec![
                            ComboboxPart::from(
                                ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            ComboboxPart::from(ComboboxContent::new([
                                ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                                ComboboxContentPart::from(ComboboxList::new().items(items)),
                            ])),
                        ]
                    },
                ),
            ]
        },
    );

    assert_semantics_has_combobox_and_listbox(&ui);
}

#[test]
fn combobox_chips_v4_parts_produce_listbox_semantics_when_open() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        CoreSize::new(fret_core::Px(640.0), fret_core::Px(480.0)),
    );

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let values: Model<Vec<Arc<str>>> = app.models_mut().insert(Vec::new());
    let open: Model<bool> = app.models_mut().insert(false);

    let values_frame_1 = values.clone();
    let open_frame_1 = open.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        move |cx| {
            let items = [
                ComboboxItem::new("next", "Next.js"),
                ComboboxItem::new("svelte", "SvelteKit"),
            ];

            vec![
                ComboboxChips::new(values_frame_1.clone(), open_frame_1.clone())
                    .into_element_parts(cx, |_cx| {
                        vec![
                            ComboboxChipsPart::from(ComboboxValue::new([ComboboxChip::new(
                                "next",
                            )])),
                            ComboboxChipsPart::from(
                                ComboboxChipsInput::new().placeholder("Add framework"),
                            ),
                            ComboboxChipsPart::from(ComboboxContent::new([
                                ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                                ComboboxContentPart::from(ComboboxList::new().items(items)),
                            ])),
                        ]
                    }),
            ]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    let values_frame_2 = values.clone();
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
                ComboboxItem::new("next", "Next.js"),
                ComboboxItem::new("svelte", "SvelteKit"),
            ];

            vec![
                ComboboxChips::new(values_frame_2.clone(), open_frame_2.clone())
                    .into_element_parts(cx, |_cx| {
                        vec![
                            ComboboxChipsPart::from(ComboboxValue::new([ComboboxChip::new(
                                "next",
                            )])),
                            ComboboxChipsPart::from(
                                ComboboxChipsInput::new().placeholder("Add framework"),
                            ),
                            ComboboxChipsPart::from(ComboboxContent::new([
                                ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                                ComboboxContentPart::from(ComboboxList::new().items(items)),
                            ])),
                        ]
                    }),
            ]
        },
    );

    assert_semantics_has_combobox_and_listbox(&ui);
}
