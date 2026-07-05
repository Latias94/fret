use std::sync::Arc;

use fret::advanced::driver::UiAppBuilderAdvancedExt as _;
use fret::advanced::kernel::core::{ImageId, SvgFit, SvgId, UiServices};
use fret::advanced::kernel::ui::{
    SvgSource,
    element::{ImageProps, SvgIconProps},
};
use fret::app::AppComponentCx;
use fret::app::prelude::*;
use fret::style::{ColorRef, LayoutRefinement, Radius, Space, ThemeSnapshot};
use fret_ui_assets::{image_asset_state, svg_asset_state};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::{GlobalWatchExt as _, style as decl_style, text as decl_text};

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

fn install_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("assets-demo")
        .window("assets_demo", (720.0, 520.0))
        .view::<AssetsDemoView>()?
        .with_ui_assets_budgets(64 * 1024 * 1024, 2048, 16 * 1024 * 1024, 4096)
        .setup(install_demo_theme)
        .on_gpu_ready(|app, _context, renderer| {
            let services = renderer as &mut dyn UiServices;
            let (_key, svg, _stats) =
                svg_asset_state::use_svg_bytes_cached_with_stats(app, services, DEMO_SVG);
            app.set_global(AssetsDemoSvg { svg });
        })
        .run()?;
    Ok(())
}

struct AssetsDemoView;

impl View for AssetsDemoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        render_view(cx)
    }
}

fn render_view<'a, Cx>(cx: &mut Cx) -> Ui
where
    Cx: fret::app::AppRenderContext<'a>,
{
    let theme = cx.theme_snapshot();

    let checker_rgba = checkerboard_rgba8(96, 96, 12);
    let (image_key, image, image_status) = ui_assets::rgba8_image_state(
        cx,
        96,
        96,
        checker_rgba.as_slice(),
        ui_assets::ImageColorSpace::Srgb,
    );
    let image_stats = ui_assets::image_stats(cx);
    let svg_stats = ui_assets::svg_stats(cx);

    let cx = cx.elements();
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
    let left = left.into_element(cx);
    let right = render_svg_panel(cx, &theme, svg);
    let right = right.into_element(cx);

    let stats = ui::v_flex_build(|cx, out| {
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
                "Debug: window={:?} image_key={} status={:?}",
                cx.window,
                image_key.as_u64(),
                image_status
            ),
        ];

        let muted = theme.color_token("muted-foreground");
        out.extend(lines.into_iter().map(|line| {
            decl_text::text_control_readout(cx, Arc::<str>::from(line)).inherit_foreground(muted)
        }));
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx);

    let content = shadcn::CardContent::new([
        ui::h_flex(|_cx| [left, right])
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
        .max_w(Px(560.0))
        .into_element(cx);

    assets_page(cx, &theme, card)
}

fn assets_page<C>(cx: &mut AppComponentCx<'_>, theme: &ThemeSnapshot, card: C) -> Ui
where
    C: IntoUiElement<App>,
{
    ui::container(move |cx| {
        ui::children![
            cx;
            ui::v_flex(move |cx| ui::children![cx; card])
                .w_full()
                .h_full()
                .justify_center()
                .items_center()
        ]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx)
    .into()
}

fn render_image_panel(
    cx: &mut AppComponentCx<'_>,
    theme: &ThemeSnapshot,
    frame: u64,
    image: Option<ImageId>,
    status: image_asset_state::ImageLoadingStatus,
    error: Option<Arc<str>>,
    stats: fret_ui_assets::image_asset_cache::ImageAssetStats,
) -> impl IntoUiElement<App> + use<> {
    let title = match status {
        image_asset_state::ImageLoadingStatus::Idle => "Image (idle)",
        image_asset_state::ImageLoadingStatus::Loading => "Image (loading...)",
        image_asset_state::ImageLoadingStatus::Loaded => "Image (ready)",
        image_asset_state::ImageLoadingStatus::Error => "Image (error)",
    };

    let image_box = ui::container(|cx| {
        if let Some(image) = image {
            let mut img = ImageProps::new(image);
            img.layout = decl_style::layout_style(theme, LayoutRefinement::default().size_full());
            [cx.image_props(img)]
        } else {
            [cx.spinner()]
        }
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .rounded(Radius::Lg)
    .w_px(Px(160.0))
    .h_px(Px(160.0))
    .overflow_hidden()
    .into_element(cx);

    ui::v_flex(|cx| {
        let mut children = ui::children![cx; shadcn::Label::new(title), image_box];
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
    cx: &mut AppComponentCx<'_>,
    theme: &ThemeSnapshot,
    svg: Option<SvgId>,
) -> impl IntoUiElement<App> + use<> {
    let icon = if let Some(svg) = svg {
        let mut props = SvgIconProps::new(SvgSource::Id(svg));
        props.layout = decl_style::layout_style(
            theme,
            LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)),
        );
        props.fit = SvgFit::Contain;
        props.color = theme.color_token("foreground");
        Some(props)
    } else {
        None
    };

    let title = if icon.is_some() {
        "SVG (cached)"
    } else {
        "SVG (waiting for gpu...)"
    };

    let box_el = ui::container(|cx| match icon.as_ref() {
        Some(props) => [cx.svg_icon_props(props.clone())],
        None => [cx.spinner()],
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .rounded(Radius::Lg)
    .p(Space::N4)
    .into_element(cx);

    ui::v_flex(|cx| [shadcn::Label::new(title).into_element(cx), box_el])
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
