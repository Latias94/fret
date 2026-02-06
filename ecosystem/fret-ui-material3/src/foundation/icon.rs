use fret_icons::{FrozenIconRegistry, IconId, IconRegistry, ResolvedSvgOwned};
use fret_ui::SvgSource;
use fret_ui::{ElementContext, UiHost};

pub(crate) fn svg_source_for_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
) -> SvgSource {
    let resolved = cx
        .app
        .global::<FrozenIconRegistry>()
        .map(|frozen| frozen.resolve_or_missing_owned(icon))
        .unwrap_or_else(|| {
            cx.app.with_global_mut(IconRegistry::default, |icons, app| {
                let frozen = icons.freeze().unwrap_or_default();
                let resolved = frozen.resolve_or_missing_owned(icon);
                app.set_global(frozen);
                resolved
            })
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
