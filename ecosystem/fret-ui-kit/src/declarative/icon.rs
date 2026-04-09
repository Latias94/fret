use std::collections::HashMap;

use fret_core::{Color, Px};
use fret_core::{SvgId, UiServices};
use fret_icons::{
    FrozenIconRegistry, IconId, IconPresentation, IconRegistry, IconRenderMode, MISSING_ICON_SVG,
    ResolvedIconOwned, ResolvedSvgOwned,
};
use fret_ui::SvgSource;
use fret_ui::element::{AnyElement, SvgIconProps, SvgImageProps};
use fret_ui::{ElementContextAccess, Theme, UiHost};

use super::style;
use crate::{ColorRef, LayoutRefinement};

#[derive(Debug, Clone, Copy)]
pub struct PreloadedIconSvg {
    pub svg: SvgId,
    pub presentation: IconPresentation,
}

#[derive(Debug, Default)]
pub struct IconSvgRegistry {
    icons: HashMap<IconId, PreloadedIconSvg>,
    missing: Option<PreloadedIconSvg>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IconSvgPreloadDiagnostics {
    pub entries: usize,
    pub bytes_ready: u64,
    pub register_calls: u64,
}

impl IconSvgRegistry {
    pub fn resolve(&self, icon: &IconId) -> Option<PreloadedIconSvg> {
        self.icons.get(icon).copied().or(self.missing)
    }
}

fn themed_icon_color(theme: &Theme, color: Option<ColorRef>) -> (Color, bool) {
    if let Some(color) = color {
        (color.resolve(theme), false)
    } else {
        (
            theme
                .color_by_key("muted-foreground")
                .unwrap_or_else(|| theme.color_token("muted-foreground")),
            true,
        )
    }
}

fn icon_layout(theme: &Theme, size: Px) -> fret_ui::element::LayoutStyle {
    style::layout_style(
        theme,
        LayoutRefinement::default()
            .w_px(size)
            .h_px(size)
            .flex_shrink_0(),
    )
}

fn svg_source_from_resolved(svg: ResolvedSvgOwned) -> SvgSource {
    match svg {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}

fn resolve_icon_from_globals<H: UiHost>(
    app: &mut H,
    icon: &IconId,
    context: &str,
) -> ResolvedIconOwned {
    app.global::<FrozenIconRegistry>()
        .map(|frozen| frozen.resolve_icon_or_missing_owned(icon))
        .unwrap_or_else(|| {
            app.with_global_mut(IconRegistry::default, |icons, app| {
                let frozen = icons.freeze_or_default_with_context(context);
                let resolved = frozen.resolve_icon_or_missing_owned(icon);
                app.set_global(frozen);
                resolved
            })
        })
}

/// Pre-register all SVG icons in the global `IconRegistry` and store their `SvgId`s in
/// an `IconSvgRegistry` global together with their presentation metadata.
///
/// This is an optional optimization that allows `icon(...)` to return `SvgSource::Id` without
/// per-frame SVG registration.
pub fn preload_icon_svgs<H: UiHost>(app: &mut H, services: &mut dyn UiServices) {
    let resolved: Vec<(IconId, ResolvedIconOwned)> =
        app.with_global_mut(IconRegistry::default, |icons, app| {
            let frozen = icons.freeze_or_default_with_context("fret_ui_kit.preload_icon_svgs");
            app.set_global(frozen.clone());
            frozen.collect_icons_owned()
        });

    let total_icons = resolved.len();
    let mut bytes_ready = MISSING_ICON_SVG.len() as u64;
    let mut register_calls = 1_u64;
    let missing = PreloadedIconSvg {
        svg: services.svg().register_svg(MISSING_ICON_SVG),
        presentation: IconPresentation::default(),
    };

    app.with_global_mut(IconSvgRegistry::default, |registry, _app| {
        registry.icons.clear();
        registry.missing = Some(missing);

        for (icon, resolved) in resolved {
            let (id, byte_len) = match resolved.svg {
                ResolvedSvgOwned::Static(bytes) => {
                    (services.svg().register_svg(bytes), bytes.len())
                }
                ResolvedSvgOwned::Bytes(bytes) => {
                    let byte_len = bytes.len();
                    (services.svg().register_svg(bytes.as_ref()), byte_len)
                }
            };

            bytes_ready += byte_len as u64;
            register_calls += 1;
            registry.icons.insert(
                icon,
                PreloadedIconSvg {
                    svg: id,
                    presentation: resolved.presentation,
                },
            );
        }
    });

    app.set_global(IconSvgPreloadDiagnostics {
        entries: total_icons + 1,
        bytes_ready,
        register_calls,
    });
}

pub fn resolve_svg_source_from_globals<H: UiHost>(
    app: &mut H,
    icon: &IconId,
    context: &str,
) -> SvgSource {
    svg_source_from_resolved(resolve_icon_from_globals(app, icon, context).svg)
}

#[track_caller]
pub fn icon<'a, H: UiHost + 'a, Cx>(cx: &mut Cx, icon: IconId) -> fret_ui::element::AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    icon_with(cx, icon, None, None)
}

#[track_caller]
pub fn icon_with<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    icon: IconId,
    size: Option<Px>,
    color: Option<ColorRef>,
) -> fret_ui::element::AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    // Semantics:
    // - `color=None` means "use currentColor when a ForegroundScope is present; otherwise fall
    //   back to muted-foreground". This mirrors shadcn/Radix's common reliance on CSS
    //   `currentColor` for icons.
    // - `color=Some(_)` pins the icon to an explicit color and disables currentColor inheritance.
    cx.elements().scope(|cx| {
        let svg: SvgSource = if let Some(preloaded) = cx
            .app
            .global::<IconSvgRegistry>()
            .and_then(|registry| registry.resolve(&icon))
        {
            SvgSource::Id(preloaded.svg)
        } else {
            resolve_svg_source_from_globals(cx.app, &icon, "fret_ui_kit.icon_with")
        };

        let theme = Theme::global(&*cx.app);
        let size = size.unwrap_or(Px(16.0));
        let (color, inherit_color) = themed_icon_color(theme, color);
        let layout = icon_layout(theme, size);

        let mut props = SvgIconProps::new(svg);
        props.layout = layout;
        props.color = color;
        props.inherit_color = inherit_color;
        cx.svg_icon_props(props)
    })
}

#[track_caller]
pub fn icon_authored<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    icon: IconId,
) -> fret_ui::element::AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    icon_authored_with(cx, icon, None)
}

#[track_caller]
pub fn icon_authored_with<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    icon: IconId,
    size: Option<Px>,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    cx.elements().scope(|cx| {
        let (svg, presentation) = if let Some(preloaded) = cx
            .app
            .global::<IconSvgRegistry>()
            .and_then(|registry| registry.resolve(&icon))
        {
            (SvgSource::Id(preloaded.svg), preloaded.presentation)
        } else {
            let resolved = resolve_icon_from_globals(cx.app, &icon, "fret_ui_kit.icon_authored");
            (
                svg_source_from_resolved(resolved.svg),
                resolved.presentation,
            )
        };

        let theme = Theme::global(&*cx.app);
        let size = size.unwrap_or(Px(16.0));
        let layout = icon_layout(theme, size);

        match presentation.render_mode {
            IconRenderMode::Mask => {
                let (color, inherit_color) = themed_icon_color(theme, None);
                let mut props = SvgIconProps::new(svg);
                props.layout = layout;
                props.color = color;
                props.inherit_color = inherit_color;
                cx.svg_icon_props(props)
            }
            IconRenderMode::OriginalColors => {
                let mut props = SvgImageProps::new(svg);
                props.layout = layout;
                cx.svg_image_props(props)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use fret_core::{
        PathId, Rect, Size, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_ui::ElementContext;
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

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
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

    #[test]
    fn icon_defaults_to_inherit_color() {
        let icon_id = IconId::new_static("ui.close");

        let mut app = fret_app::App::new();
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

        assert!(
            props.inherit_color,
            "expected icon(...) to opt into late-bound foreground inheritance by default"
        );
    }

    #[test]
    fn icon_keeps_themed_svg_icon_posture_for_original_color_registry_icons() {
        let icon_id = IconId::new_static("brand.logo");
        let svg_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        let mut app = fret_app::App::new();
        app.with_global_mut(IconRegistry::default, |icons, _app| {
            let _ = icons.register_svg_static_with_presentation(
                icon_id.clone(),
                svg_bytes,
                IconPresentation {
                    render_mode: IconRenderMode::OriginalColors,
                },
            );
        });

        let mut runtime = ElementRuntime::default();
        let window = fret_core::AppWindowId::default();
        let bounds = Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        let mut cx =
            ElementContext::new_for_root_name(&mut app, &mut runtime, window, bounds, "test");

        let el = icon(&mut cx, icon_id);
        let fret_ui::element::ElementKind::SvgIcon(_) = el.kind else {
            panic!("expected icon(...) to remain on the themed SvgIcon posture");
        };
    }

    #[test]
    fn icon_authored_uses_svg_image_for_original_color_icons() {
        let icon_id = IconId::new_static("brand.logo");
        let svg_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        let mut app = fret_app::App::new();
        app.with_global_mut(IconRegistry::default, |icons, _app| {
            let _ = icons.register_svg_static_with_presentation(
                icon_id.clone(),
                svg_bytes,
                IconPresentation {
                    render_mode: IconRenderMode::OriginalColors,
                },
            );
        });

        let mut services = FakeUiServices::default();
        preload_icon_svgs(&mut app, &mut services);

        let mut runtime = ElementRuntime::default();
        let window = fret_core::AppWindowId::default();
        let bounds = Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        let mut cx =
            ElementContext::new_for_root_name(&mut app, &mut runtime, window, bounds, "test");

        let el = icon_authored(&mut cx, icon_id);
        let fret_ui::element::ElementKind::SvgImage(props) = el.kind else {
            panic!("expected icon_authored(...) to honor OriginalColors with SvgImage");
        };

        let SvgSource::Id(_) = props.svg else {
            panic!("expected preloaded icon_authored(...) to reuse SvgSource::Id");
        };
    }

    #[test]
    fn icon_authored_uses_svg_icon_for_mask_icons() {
        let icon_id = IconId::new_static("ui.close");
        let svg_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        let mut app = fret_app::App::new();
        app.with_global_mut(IconRegistry::default, |icons, _app| {
            let _ = icons.register_svg_static_with_presentation(
                icon_id.clone(),
                svg_bytes,
                IconPresentation {
                    render_mode: IconRenderMode::Mask,
                },
            );
        });

        let mut runtime = ElementRuntime::default();
        let window = fret_core::AppWindowId::default();
        let bounds = Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        let mut cx =
            ElementContext::new_for_root_name(&mut app, &mut runtime, window, bounds, "test");

        let el = icon_authored(&mut cx, icon_id);
        let fret_ui::element::ElementKind::SvgIcon(props) = el.kind else {
            panic!("expected icon_authored(...) to keep mask icons on SvgIcon");
        };

        assert!(
            props.inherit_color,
            "expected mask-mode authored icons to keep the late-bound currentColor posture"
        );
    }
}
