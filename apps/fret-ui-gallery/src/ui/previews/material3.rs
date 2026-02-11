use super::super::*;

pub(in crate::ui) fn material3_scoped_page<I, F>(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive));

    let body = if enabled {
        material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            content,
        )
    } else {
        content(cx)
    };

    out.extend(body);
    out
}

pub(in crate::ui) fn material3_variant_toggle_row(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
) -> AnyElement {
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Switch::new(material3_expressive.clone())
                    .a11y_label("Enable Material 3 Expressive variant")
                    .into_element(cx),
                ui::label(
                    cx,
                    if enabled {
                        "Variant: Expressive"
                    } else {
                        "Variant: Standard"
                    },
                )
                .into_element(cx),
            ]
        },
    )
}

mod buttons;
mod controls;
mod forms;
mod gallery;
mod nav;
mod navigation;
mod overlays;

pub(in crate::ui) use buttons::*;
pub(in crate::ui) use controls::*;
pub(in crate::ui) use forms::*;
pub(in crate::ui) use gallery::*;
pub(in crate::ui) use nav::*;
pub(in crate::ui) use navigation::*;
pub(in crate::ui) use overlays::*;
