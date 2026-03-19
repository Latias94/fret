pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let hard_disks = cx.local_model_keyed("hard_disks", || true);
    let external_disks = cx.local_model_keyed("external_disks", || false);
    let cds_dvds = cx.local_model_keyed("cds_dvds", || false);
    let connected_servers = cx.local_model_keyed("connected_servers", || false);
    let sync_desktop = cx.local_model_keyed("sync_desktop", || true);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldGroup::new([
        shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Show these items on the desktop")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Select the items you want to show on the desktop.")
                .into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::Checkbox::new(hard_disks)
                        .control_id("ui-gallery-field-checkbox-hard-disks")
                        .a11y_label("Hard disks")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Hard disks")
                        .for_control("ui-gallery-field-checkbox-hard-disks")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(external_disks)
                        .control_id("ui-gallery-field-checkbox-external-disks")
                        .a11y_label("External disks")
                        .into_element(cx),
                    shadcn::FieldLabel::new("External disks")
                        .for_control("ui-gallery-field-checkbox-external-disks")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(cds_dvds)
                        .control_id("ui-gallery-field-checkbox-cds-dvds")
                        .a11y_label("CDs, DVDs, and iPods")
                        .into_element(cx),
                    shadcn::FieldLabel::new("CDs, DVDs, and iPods")
                        .for_control("ui-gallery-field-checkbox-cds-dvds")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(connected_servers)
                        .control_id("ui-gallery-field-checkbox-connected-servers")
                        .a11y_label("Connected servers")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Connected servers")
                        .for_control("ui-gallery-field-checkbox-connected-servers")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ])
            .checkbox_group()
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::FieldSeparator::new().into_element(cx),
        shadcn::Field::new([
            shadcn::Checkbox::new(sync_desktop)
                .control_id("ui-gallery-field-checkbox-sync-desktop")
                .a11y_label("Sync Desktop and Documents folders")
                .into_element(cx),
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("Sync Desktop & Documents folders")
                    .for_control("ui-gallery-field-checkbox-sync-desktop")
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "Your Desktop & Documents folders are synced with iCloud Drive and available on your other devices.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-checkbox")
}
// endregion: example
