use super::super::super::doc_layout::DocSection;
use super::super::super::*;

pub(in crate::ui) fn preview_material3_icon_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::icon_button::render(cx);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::icon_button::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_fab(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;

    fn on_activate(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, _acx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
        })
    }

    let row = {
        let last_action = last_action.clone();
        move |cx: &mut ElementContext<'_, App>,
              variant: material3::FabVariant,
              label: &'static str| {
            let last_action = last_action.clone();
            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                move |cx| {
                    vec![
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label(label)
                            .on_activate(on_activate(
                                "material3.fab.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Small")
                            .size(material3::FabSize::Small)
                            .on_activate(on_activate(
                                "material3.fab.small.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Large")
                            .size(material3::FabSize::Large)
                            .on_activate(on_activate(
                                "material3.fab.large.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Lowered")
                            .lowered(true)
                            .on_activate(on_activate(
                                "material3.fab.lowered.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Disabled")
                            .disabled(true)
                            .into_element(cx),
                    ]
                },
            )
        }
    };

    let extended = {
        let last_action = last_action.clone();
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Surface)
                        .label("Create")
                        .on_activate(on_activate(
                            "material3.extended_fab.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Primary)
                        .label("Create")
                        .on_activate(on_activate(
                            "material3.extended_fab.primary.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Surface)
                        .label("Reroute")
                        .icon(None)
                        .on_activate(on_activate(
                            "material3.extended_fab.no_icon.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                ]
            },
        )
    };

    vec![
        cx.text(
            "Material 3 FAB: token-driven variants + focus ring + state layer + bounded ripple.",
        ),
        row(cx, material3::FabVariant::Surface, "Surface"),
        row(cx, material3::FabVariant::Primary, "Primary"),
        row(cx, material3::FabVariant::Secondary, "Secondary"),
        row(cx, material3::FabVariant::Tertiary, "Tertiary"),
        extended,
    ]
}

pub(in crate::ui) fn preview_material3_checkbox(
    cx: &mut ElementContext<'_, App>,
    checked: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::checkbox::render(cx, checked);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::checkbox::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::switch::render(cx, selected);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::switch::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_slider(
    cx: &mut ElementContext<'_, App>,
    value: Model<f32>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::slider::render(cx, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::slider::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_radio(
    cx: &mut ElementContext<'_, App>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::radio::render(cx, group_value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::radio::SOURCE, "example"),
        ],
    );

    vec![page]
}
