use std::sync::Arc;

use fret::component::prelude::*;
use fret::{
    FretApp,
    advanced::prelude::*,
    assets::{self, AssetBundleId, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry},
    integration::InstallIntoApp,
    shadcn,
};
use fret_core::{ImageColorSpace, ImageId};
use fret_icons::FrozenIconRegistry;
use fret_ui::element::{ImageProps, LayoutStyle, Length, SvgIconProps};
use fret_ui_assets::ui::{ImageSourceElementContextExt as _, SvgAssetElementContextExt as _};

const TEST_ID_ROOT: &str = "cookbook.icons_and_assets_basics.root";
const TEST_ID_PANEL_ICONS: &str = "cookbook.icons_and_assets_basics.panel.icons";
const TEST_ID_PANEL_SVG: &str = "cookbook.icons_and_assets_basics.panel.svg";
const TEST_ID_PANEL_IMAGE: &str = "cookbook.icons_and_assets_basics.panel.image";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.icons_and_assets_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.icons_and_assets_basics.svg.status";

const PACKAGE_ASSET_BUNDLE_NAME: &str = "cookbook-icons-demo";
const PACKAGE_IMAGE_KEY: &str = "images/test.jpg";
const PACKAGE_SVG_KEY: &str = "icons/search.svg";
const PACKAGE_IMAGE_BYTES: &[u8] = include_bytes!("../../../assets/textures/test.jpg");
const PACKAGE_SVG_BYTES: &[u8] = include_bytes!("../../../assets/demo/icon-search.svg");
const PACKAGE_ASSET_ENTRIES: [StaticAssetEntry; 2] = [
    StaticAssetEntry::new(PACKAGE_IMAGE_KEY, AssetRevision(1), PACKAGE_IMAGE_BYTES)
        .with_media_type("image/jpeg"),
    StaticAssetEntry::new(PACKAGE_SVG_KEY, AssetRevision(1), PACKAGE_SVG_BYTES)
        .with_media_type("image/svg+xml"),
];

fn demo_package_bundle() -> AssetBundleId {
    // Model a reusable ecosystem crate by using a package-scoped bundle id rather than the host
    // app bundle id.
    AssetBundleId::package(PACKAGE_ASSET_BUNDLE_NAME)
}

struct IconsAndAssetsBundle;

impl InstallIntoApp for IconsAndAssetsBundle {
    fn install_into_app(self, app: &mut fret::app::App) {
        shadcn::app::install(app);
        fret_icons_lucide::app::install(app);
        assets::register_bundle_entries(app, demo_package_bundle(), PACKAGE_ASSET_ENTRIES);
    }
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

fn render_image_preview(
    cx: &mut UiCx<'_>,
    title: &'static str,
    image: Option<ImageId>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let theme = cx.theme_snapshot();
    let border = ColorRef::Color(theme.color_token("border"));

    let box_el = ui::container(move |cx| {
        if let Some(image) = image {
            let mut props = ImageProps::new(image);
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            props.layout = layout;
            [cx.image_props(props)]
        } else {
            [cx.spinner()]
        }
    })
    .border_1()
    .border_color(border)
    .rounded(Radius::Lg)
    .w_px(Px(160.0))
    .h_px(Px(160.0))
    .overflow_hidden();

    ui::v_flex(move |cx| ui::children![cx; shadcn::Label::new(title), box_el])
        .gap(Space::N2)
        .w_full()
}

struct IconsAndAssetsBasicsView {
    bundle_image_request: AssetRequest,
    memory_image: fret_ui_assets::ImageSource,
    svg_request: AssetRequest,
}

impl View for IconsAndAssetsBasicsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        let bundle_image_request = AssetRequest::new(AssetLocator::bundle(
            demo_package_bundle(),
            PACKAGE_IMAGE_KEY,
        ));
        let memory_image = fret_ui_assets::ImageSource::rgba8(
            128,
            128,
            checkerboard_rgba8(128, 128, 16),
            ImageColorSpace::Srgb,
        );
        let svg_request =
            AssetRequest::new(AssetLocator::bundle(demo_package_bundle(), PACKAGE_SVG_KEY));

        Self {
            bundle_image_request,
            memory_image,
            svg_request,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();

        let header = shadcn::card_header(|cx| {
            ui::children![
                cx;
                shadcn::card_title("Icons + assets basics"),
                shadcn::card_description(
                    "A reusable dependency bundle installs lucide icons plus package-owned logical assets behind one `.setup(...)` value. This is the hand-written wrapper lane to teach when a crate composes more than raw shipped bytes, so the app never replays low-level icon or asset registration manually.",
                ),
            ]
        });

        let callouts = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Badge::new("Package bundle locator: reusable crate-owned asset namespace")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("App setup bundle: composes transitive icon + asset installers")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Hand-written bundle wrapper: use when the crate also composes icons or app defaults")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Low-level registration stays internal to the dependency")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("RGBA8 source: deterministic in-memory escape hatch")
                    .variant(shadcn::BadgeVariant::Secondary)
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_center()
        .w_full();

        let icon_row = |title: &'static str, ids: [IconId; 3]| {
            let buttons = ui::h_flex(move |cx| {
                ui::children![cx;
                    shadcn::Button::new("Search")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[0].clone()),
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[1].clone()),
                    shadcn::Button::new("Copy")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[2].clone()),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .w_full();

            ui::v_flex(move |cx| ui::children![cx; shadcn::Label::new(title), buttons])
                .gap(Space::N2)
                .w_full()
        };

        let icons_panel = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Icons"),
                        shadcn::card_description(
                            "IconId is renderer-agnostic. Packs register data; components consume semantic ids (ui.*).",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx: &mut UiCx<'_>| {
                            let frozen = cx.app.global::<FrozenIconRegistry>().cloned();
                            let preload = cx
                                .app
                                .global::<icon::IconSvgPreloadDiagnostics>()
                                .copied();
                            let frozen_len = frozen.as_ref().map(|v| v.len()).unwrap_or(0);
                            let preload_entries = preload.map(|v| v.entries).unwrap_or(0);
                            let preload_bytes = preload.map(|v| v.bytes_ready).unwrap_or(0);

                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Badge::new(format!("frozen icons: {frozen_len}"))
                                            .variant(shadcn::BadgeVariant::Secondary),
                                        shadcn::Badge::new(format!(
                                            "preloaded: {preload_entries} ({} KB)",
                                            preload_bytes / 1024
                                        ))
                                        .variant(shadcn::BadgeVariant::Secondary),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                                icon_row(
                                    "Semantic ids (ui.*)",
                                    [
                                        IconId::new_static("ui.search"),
                                        IconId::new_static("ui.close"),
                                        IconId::new_static("ui.copy"),
                                    ],
                                ),
                                icon_row(
                                    "Vendor ids (lucide.*)",
                                    [
                                        IconId::new_static("lucide.search"),
                                        IconId::new_static("lucide.x"),
                                        IconId::new_static("lucide.copy"),
                                    ],
                                ),
                                icon_row(
                                    "Vendor ids (lucide.*)",
                                    [
                                        IconId::new_static("lucide.search"),
                                        IconId::new_static("lucide.x"),
                                        IconId::new_static("lucide.copy"),
                                    ],
                                ),
                            ]
                        })
                        .gap(Space::N4)
                        .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_ICONS);

        let bundle_image_state =
            cx.use_image_source_state_from_asset_request(&self.bundle_image_request);
        let memory_image_state = cx.use_image_source_state(&self.memory_image);

        let image_status = match bundle_image_state.status {
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
        };

        let image_panel = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Images"),
                        shadcn::card_description(
                            "Bundle-based decode is async; the UI helper keeps the app/widget story on logical requests while in-memory RGBA8 remains the immediate deterministic escape hatch.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Label::new("Bundle image status:"),
                                        shadcn::Badge::new(image_status)
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .test_id(TEST_ID_IMAGE_STATUS),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                        ui::h_flex(|cx| {
                            ui::children![
                                cx;
                                render_image_preview(
                                    cx,
                                    "From package bundle locator: `pkg:cookbook-icons-demo/images/test.jpg`",
                                    bundle_image_state.image,
                                ),
                                render_image_preview(
                                    cx,
                                    "From RGBA8 buffer",
                                            memory_image_state.image,
                                        ),
                                    ]
                                })
                                .gap(Space::N4)
                                .items_center()
                                .w_full(),
                            ]
                        })
                        .gap(Space::N3)
                        .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_IMAGE);

        let svg_state = cx.svg_source_state_from_asset_request(&self.svg_request);
        let svg_status = if svg_state.source.is_some() {
            "ready"
        } else {
            "error"
        };
        let svg_foreground = theme.color_token("foreground");
        let svg_error_color = ColorRef::Color(theme.color_token("destructive"));

        let svg_box = ui::container(move |cx| match svg_state.source.clone() {
            Some(source) => {
                let mut props = SvgIconProps::new(source);
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(160.0));
                layout.size.height = Length::Px(Px(160.0));
                props.layout = layout;
                props.fit = fret_core::SvgFit::Contain;
                props.color = svg_foreground;
                ui::children![cx; cx.svg_icon_props(props)]
            }
            None => {
                let error = svg_state
                    .error
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("unknown SVG asset error"));
                ui::children![cx;
                    ui::text(format!("Failed to resolve SVG asset: {error}"))
                        .text_color(svg_error_color)
                ]
            }
        })
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Lg)
        .p(Space::N4);

        let svg_panel = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("SVG icon from bundle locator"),
                        shadcn::card_description(
                            "Resolves an SVG asset through the package bundle registration and lets the shared UI helper pick the right native/web handoff instead of reading a raw filesystem path.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Label::new("SVG status:"),
                                        shadcn::Badge::new(svg_status)
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .test_id(TEST_ID_SVG_STATUS),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                                svg_box,
                            ]
                        })
                        .gap(Space::N3)
                        .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_SVG);

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                header,
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(
                            |cx| ui::children![cx; callouts, icons_panel, svg_panel, image_panel],
                        )
                        .gap(Space::N5)
                        .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(900.0));

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-icons-and-assets-basics")
        .ui_assets_budgets(32 * 1024 * 1024, 1024, 8 * 1024 * 1024, 2048)
        .window("cookbook-icons-and-assets-basics", (960.0, 860.0))
        .setup((
            IconsAndAssetsBundle,
            fret_cookbook::install_cookbook_defaults,
        ))
        .view::<IconsAndAssetsBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
