pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let hard_disks = cx.local_model_keyed("group_hard_disks", || true);
    let external_disks = cx.local_model_keyed("group_external_disks", || true);
    let cds_dvds_ipods = cx.local_model_keyed("group_cds_dvds_ipods", || false);
    let connected_servers = cx.local_model_keyed("group_connected_servers", || false);

    let group_item =
        |cx: &mut UiCx<'_>, label: &'static str, value: Model<bool>, test_id: &'static str| {
            shadcn::Field::new([
                shadcn::Checkbox::new(value)
                    // Required for label click -> focus/toggle forwarding.
                    .control_id(test_id)
                    .a11y_label(label)
                    .test_id(test_id)
                    .into_element(cx),
                shadcn::FieldLabel::new(label)
                    .for_control(test_id)
                    .test_id(format!("{test_id}.label"))
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
        };

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Show these items on the desktop:")
                .variant(shadcn::FieldLegendVariant::Label),
            shadcn::FieldDescription::new("Select the items you want to show on the desktop."),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    group_item(
                        cx,
                        "Hard disks",
                        hard_disks,
                        "ui-gallery-checkbox-group-hard-disks",
                    ),
                    group_item(
                        cx,
                        "External disks",
                        external_disks,
                        "ui-gallery-checkbox-group-external-disks",
                    ),
                    group_item(
                        cx,
                        "CDs, DVDs, and iPods",
                        cds_dvds_ipods,
                        "ui-gallery-checkbox-group-cds-dvds-ipods",
                    ),
                    group_item(
                        cx,
                        "Connected servers",
                        connected_servers,
                        "ui-gallery-checkbox-group-connected-servers",
                    ),
                ]
            })
            .checkbox_group(),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-group")
}
// endregion: example
