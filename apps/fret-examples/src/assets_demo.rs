use std::sync::Arc;

use fret_kit::fret::core::{ImageColorSpace, SvgId};
use fret_kit::fret::ui::element::{ImageProps, SvgIconProps};
use fret_kit::prelude::*;
use fret_ui_assets::{UiAssets, image_asset_state, svg_asset_state};
use fret_ui_kit::declarative::GlobalWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ColorRef, LayoutRefinement, Radius, Space};

static DEMO_SVG: &[u8] = br##"
<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <rect x="8" y="8" width="112" height="112" rx="16" fill="#0ea5e9"/>
  <path d="M40 64c0-13.255 10.745-24 24-24s24 10.745 24 24-10.745 24-24 24-24-10.745-24-24Z" fill="#ffffff"/>
  <path d="M52 64h24" stroke="#0ea5e9" stroke-width="8" stroke-linecap="round"/>
</svg>
"##;

#[derive(Clone, Copy)]
struct AssetsDemoSvg {
    svg: SvgId,
}

#[derive(Debug, Default, Clone, Copy)]
struct AssetsDemoImageEvents {
    registered: u64,
    failed: u64,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("assets-demo", init_window, view, |d| d.on_event(on_event))?
        .with_main_window("assets_demo", (720.0, 520.0))
        .with_ui_assets_budgets(64 * 1024 * 1024, 2048, 16 * 1024 * 1024, 4096)
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .on_gpu_ready(|app, _context, renderer| {
            let services = renderer as &mut dyn UiServices;
            let (_key, svg, _stats) =
                svg_asset_state::use_svg_bytes_cached_with_stats(app, services, DEMO_SVG);
            app.set_global(AssetsDemoSvg { svg });
        })
        .run()?;
    Ok(())
}

fn init_window(_app: &mut App, _window: AppWindowId) -> () {}

fn on_event(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    _state: &mut (),
    event: &Event,
) {
    match event {
        Event::ImageRegistered { .. } => {
            let log_events = std::env::var_os("FRET_ASSETS_DEMO_LOG_IMAGE_EVENTS")
                .is_some_and(|v| !v.is_empty());
            let log_stats =
                std::env::var_os("FRET_ASSETS_DEMO_LOG_STATS").is_some_and(|v| !v.is_empty());

            if log_events {
                eprintln!("[assets_demo] ImageRegistered window={window:?}");
            }
            if log_stats {
                let images = UiAssets::image_stats(app);
                let svgs = UiAssets::svg_stats(app);
                eprintln!(
                    "[assets_demo] stats images(ready={} pending={} failed={} bytes={} / {}) svgs(ready={} bytes={} / {})",
                    images.ready_count,
                    images.pending_count,
                    images.failed_count,
                    images.bytes_ready,
                    images.bytes_budget,
                    svgs.ready_count,
                    svgs.bytes_ready,
                    svgs.bytes_budget
                );
            }
            app.with_global_mut(AssetsDemoImageEvents::default, |c, app| {
                c.registered = c.registered.saturating_add(1);
                app.request_redraw(window);
            });
        }
        Event::ImageRegisterFailed { .. } => {
            let log_events = std::env::var_os("FRET_ASSETS_DEMO_LOG_IMAGE_EVENTS")
                .is_some_and(|v| !v.is_empty());
            let log_stats =
                std::env::var_os("FRET_ASSETS_DEMO_LOG_STATS").is_some_and(|v| !v.is_empty());

            if log_events {
                eprintln!("[assets_demo] ImageRegisterFailed window={window:?}");
            }
            if log_stats {
                let images = UiAssets::image_stats(app);
                let svgs = UiAssets::svg_stats(app);
                eprintln!(
                    "[assets_demo] stats images(ready={} pending={} failed={} bytes={} / {}) svgs(ready={} bytes={} / {})",
                    images.ready_count,
                    images.pending_count,
                    images.failed_count,
                    images.bytes_ready,
                    images.bytes_budget,
                    svgs.ready_count,
                    svgs.bytes_ready,
                    svgs.bytes_budget
                );
            }
            app.with_global_mut(AssetsDemoImageEvents::default, |c, app| {
                c.failed = c.failed.saturating_add(1);
                app.request_redraw(window);
            });
        }
        _ => {}
    }
}

fn view(cx: &mut ElementContext<'_, App>, _st: &mut ()) -> fret_kit::ViewElements {
    let theme = cx.theme().clone();

    let checker_rgba = checkerboard_rgba8(96, 96, 12);
    let (image_key, image, image_status) = image_asset_state::use_rgba8_image_state(
        cx.app,
        cx.window,
        96,
        96,
        checker_rgba.as_slice(),
        ImageColorSpace::Srgb,
    );
    let image_error = match image_status {
        image_asset_state::ImageLoadingStatus::Error => {
            use fret_ui_assets::image_asset_cache::ImageAssetCacheHostExt as _;
            cx.app.with_image_asset_cache(|cache, _app| {
                cache
                    .error(image_key)
                    .map(|s| Arc::<str>::from(s.to_string()))
            })
        }
        _ => None,
    };

    let svg = cx.watch_global::<AssetsDemoSvg>().layout().map(|v| v.svg);

    let image_events = cx
        .watch_global::<AssetsDemoImageEvents>()
        .layout()
        .copied()
        .unwrap_or_default();

    let image_stats = UiAssets::image_stats(cx.app);
    let svg_stats = UiAssets::svg_stats(cx.app);

    let header = shadcn::CardHeader::new([
        shadcn::CardTitle::new("UI Assets (Golden Path)").into_element(cx),
        shadcn::CardDescription::new("ImageAssetCache + SvgAssetCache wired by UiAppDriver.")
            .into_element(cx),
    ])
    .into_element(cx);

    let frame = cx.app.frame_id().0;
    let left = render_image_panel(
        cx,
        &theme,
        frame,
        image,
        image_status,
        image_error,
        image_stats,
    );
    let right = render_svg_panel(cx, &theme, svg);

    let stats = ui::v_flex_build(cx, |cx, out| {
        let lines = [
            format!(
                "Images: ready={} pending={} failed={} bytes={} / {}",
                image_stats.ready_count,
                image_stats.pending_count,
                image_stats.failed_count,
                image_stats.bytes_ready,
                image_stats.bytes_budget
            ),
            format!(
                "SVGs: ready={} bytes={} / {}",
                svg_stats.ready_count, svg_stats.bytes_ready, svg_stats.bytes_budget
            ),
            format!(
                "Debug: window={:?} image_key={} events(registered={}, failed={})",
                cx.window,
                image_key.as_u64(),
                image_events.registered,
                image_events.failed
            ),
        ];

        out.extend(lines.into_iter().map(|line| {
            cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(line),
                style: None,
                color: Some(theme.color_required("muted-foreground")),
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            })
        }));
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx);

    let content = shadcn::CardContent::new([
        ui::h_flex(cx, |_cx| [left, right])
            .w_full()
            .gap(Space::N4)
            .items_start()
            .into_element(cx),
        stats,
    ])
    .into_element(cx);

    let card = shadcn::Card::new([header, content])
        .ui()
        .w_full()
        .max_w(fret_core::Px(560.0))
        .into_element(cx);

    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_required("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    page.into()
}

fn render_image_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    frame: u64,
    image: Option<fret_core::ImageId>,
    status: image_asset_state::ImageLoadingStatus,
    error: Option<Arc<str>>,
    stats: fret_ui_assets::image_asset_cache::ImageAssetStats,
) -> AnyElement {
    let title = match status {
        image_asset_state::ImageLoadingStatus::Idle => "Image (idle)",
        image_asset_state::ImageLoadingStatus::Loading => "Image (loading...)",
        image_asset_state::ImageLoadingStatus::Loaded => "Image (ready)",
        image_asset_state::ImageLoadingStatus::Error => "Image (error)",
    };

    let image_box = ui::container(cx, |cx| {
        if let Some(image) = image {
            let mut img = ImageProps::new(image);
            img.layout = decl_style::layout_style(theme, LayoutRefinement::default().size_full());
            [cx.image_props(img)]
        } else {
            [cx.spinner()]
        }
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_required("border")))
    .rounded(Radius::Lg)
    .w_px(fret_core::Px(160.0))
    .h_px(fret_core::Px(160.0))
    .overflow_hidden()
    .into_element(cx);

    ui::v_flex(cx, |cx| {
        let mut children = vec![shadcn::Label::new(title).into_element(cx), image_box];
            if let Some(msg) = error {
                children.push(
                    shadcn::Alert::new([
                        shadcn::AlertTitle::new("Image upload failed").into_element(cx),
                        shadcn::AlertDescription::new(msg).into_element(cx),
                    ])
                    .variant(shadcn::AlertVariant::Destructive)
                    .into_element(cx),
                );
            }
            if matches!(status, image_asset_state::ImageLoadingStatus::Loading)
                && stats.ready_count == 0
                && stats.pending_count > 0
                && frame > 5
            {
                children.push(
                    shadcn::Alert::new([
                        shadcn::AlertTitle::new("Still loading?").into_element(cx),
                        shadcn::AlertDescription::new(Arc::<str>::from(
                            "This demo does not fetch from the network. If loading never finishes, it usually means `ImageRegistered` events are not reaching `ImageAssetCache` (check that you are running the latest binary via `cargo run -p fret-demo --bin assets_demo`, and that `fret-bootstrap` is built with the `ui-assets` feature).",
                        ))
                        .into_element(cx),
                    ])
                    .into_element(cx),
                );
            }
            children
    })
    .flex_1()
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

fn render_svg_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    svg: Option<fret_core::SvgId>,
) -> AnyElement {
    let icon = if let Some(svg) = svg {
        let mut props = SvgIconProps::new(fret_ui::SvgSource::Id(svg));
        props.layout = decl_style::layout_style(
            theme,
            LayoutRefinement::default()
                .w_px(fret_core::Px(160.0))
                .h_px(fret_core::Px(160.0)),
        );
        props.fit = fret_core::SvgFit::Contain;
        props.color = theme.color_required("foreground");
        Some(props)
    } else {
        None
    };

    let title = if icon.is_some() {
        "SVG (cached)"
    } else {
        "SVG (waiting for gpu...)"
    };

    let box_el = ui::container(cx, |cx| match icon.as_ref() {
        Some(props) => [cx.svg_icon_props(props.clone())],
        None => [cx.spinner()],
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_required("border")))
    .rounded(Radius::Lg)
    .p(Space::N4)
    .into_element(cx);

    ui::v_flex(cx, |cx| {
        [shadcn::Label::new(title).into_element(cx), box_el]
    })
    .flex_1()
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

fn checkerboard_rgba8(width: u32, height: u32, cell: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let on = ((x / cell) + (y / cell)) % 2 == 0;
            let (r, g, b) = if on {
                (14u8, 165u8, 233u8)
            } else {
                (241u8, 245u8, 249u8)
            };
            let i = ((y * width + x) * 4) as usize;
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
            out[i + 3] = 255;
        }
    }
    out
}
