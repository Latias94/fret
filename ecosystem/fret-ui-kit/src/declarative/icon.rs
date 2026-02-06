use std::collections::HashMap;

use fret_core::{Color, Px};
use fret_core::{SvgId, UiServices};
use fret_icons::{IconId, IconRegistry, ResolvedSvgOwned, MISSING_ICON_SVG};
use fret_ui::element::SvgIconProps;
use fret_ui::SvgSource;
use fret_ui::{ElementContext, Theme, UiHost};

use super::style;
use crate::{ColorRef, LayoutRefinement};

#[derive(Debug, Default)]
pub struct IconSvgRegistry {
    icons: HashMap<IconId, SvgId>,
    missing: Option<SvgId>,
}

impl IconSvgRegistry {
    pub fn resolve(&self, icon: &IconId) -> Option<SvgId> {
        self.icons.get(icon).copied().or(self.missing)
    }

    pub fn clear(&mut self) {
        self.icons.clear();
        self.missing = None;
    }

    pub fn set_missing(&mut self, missing: SvgId) {
        self.missing = Some(missing);
    }

    pub fn insert(&mut self, icon: IconId, svg: SvgId) {
        self.icons.insert(icon, svg);
    }
}

/// Pre-register all SVG icons in the global `IconRegistry` and store their `SvgId`s in
/// an `IconSvgRegistry` global.
///
/// This is an optional optimization that allows `icon(...)` to return `SvgSource::Id` without
/// per-frame SVG registration.
pub fn preload_icon_svgs<H: UiHost>(app: &mut H, services: &mut dyn UiServices) {
    let resolved: Vec<(IconId, ResolvedSvgOwned)> = app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons.collect_resolved_owned()
        });

    let missing = services.svg().register_svg(MISSING_ICON_SVG);

    app.with_global_mut(IconSvgRegistry::default, |registry, _app| {
        registry.icons.clear();
        registry.missing = Some(missing);

        for (icon, svg) in resolved {
            let id = match svg {
                ResolvedSvgOwned::Static(bytes) => services.svg().register_svg(bytes),
                ResolvedSvgOwned::Bytes(bytes) => services.svg().register_svg(bytes.as_ref()),
            };
            registry.icons.insert(icon, id);
        }
    });
}

#[track_caller]
pub fn icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: IconId,
) -> fret_ui::element::AnyElement {
    icon_with(cx, icon, None, None)
}

#[track_caller]
pub fn icon_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: IconId,
    size: Option<Px>,
    color: Option<ColorRef>,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        let svg: SvgSource = if let Some(svg) = cx
            .app
            .global::<IconSvgRegistry>()
            .and_then(|registry| registry.resolve(&icon))
        {
            SvgSource::Id(svg)
        } else {
            let resolved = cx
                .app
                .with_global_mut(IconRegistry::default, |icons, _app| {
                    icons.resolve_or_missing_owned(&icon)
                });

            match resolved {
                ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
                ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
            }
        };

        let theme = Theme::global(&*cx.app);
        let size = size.unwrap_or(Px(16.0));
        let color: Color = color
            .map(|c| c.resolve(theme))
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or_else(|| theme.color_required("muted-foreground"));

        let layout = style::layout_style(
            theme,
            LayoutRefinement::default()
                .w_px(size)
                .h_px(size)
                .flex_shrink_0(),
        );

        let mut props = SvgIconProps::new(svg);
        props.layout = layout;
        props.color = color;
        cx.svg_icon_props(props)
    })
}

#[cfg(test)]
mod tests {
    use fret_core::{
        PathId, Rect, Size, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_ui::elements::ElementRuntime;

    use super::*;

    #[derive(Default)]
    struct FakeUiServices {
        registered: Vec<Vec<u8>>,
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, bytes: &[u8]) -> SvgId {
            self.registered.push(bytes.to_vec());
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (PathId, fret_core::PathMetrics) {
            (PathId::default(), fret_core::PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::default(),
                    baseline: fret_core::Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    #[test]
    fn icon_prefers_preloaded_svg_ids() {
        let icon_id = IconId::new_static("ui.close");
        let svg_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        let mut app = fret_app::App::new();
        app.with_global_mut(IconRegistry::default, |icons, _app| {
            let _ = icons.register_svg_static(icon_id.clone(), svg_bytes);
        });

        let mut services = FakeUiServices::default();
        preload_icon_svgs(&mut app, &mut services);
        assert!(services.registered.len() >= 2); // missing + our icon

        let mut runtime = ElementRuntime::default();
        let window = fret_core::AppWindowId::default();
        let bounds = Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        let mut cx =
            ElementContext::new_for_root_name(&mut app, &mut runtime, window, bounds, "test");

        let el = icon(&mut cx, icon_id);
        let fret_ui::element::ElementKind::SvgIcon(props) = el.kind else {
            panic!("expected SvgIcon element");
        };

        let SvgSource::Id(_) = props.svg else {
            panic!("expected SvgSource::Id after preload");
        };

        // Ensure resolving does not re-register SVG bytes.
        let mut panicking_services = FakeUiServices::default();
        let _ = props.svg.resolve(&mut panicking_services);
        assert!(panicking_services.registered.is_empty());
    }
}
