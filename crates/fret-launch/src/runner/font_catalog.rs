use fret_runtime::{FontCatalogEntry, FontCatalogUpdate, FontFamilyDefaultsPolicy, GlobalsHost};

pub(super) fn apply_renderer_font_catalog_update(
    app: &mut impl GlobalsHost,
    renderer: &mut fret_render::Renderer,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let entries = renderer
        .all_font_catalog_entries()
        .into_iter()
        .map(|e| FontCatalogEntry {
            family: e.family,
            has_variable_axes: e.has_variable_axes,
            known_variable_axes: e.known_variable_axes,
            variable_axes: e
                .variable_axes
                .into_iter()
                .map(|a| fret_runtime::FontVariableAxisInfo {
                    tag: a.tag,
                    min_bits: a.min_bits,
                    max_bits: a.max_bits,
                    default_bits: a.default_bits,
                })
                .collect(),
            is_monospace_candidate: e.is_monospace_candidate,
        })
        .collect::<Vec<_>>();

    let update = fret_runtime::apply_font_catalog_update_with_metadata(app, entries, policy);
    let _ = renderer.set_text_font_families(&update.config);
    app.set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
        renderer.text_font_stack_key(),
    ));

    update
}
