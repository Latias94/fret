use fret::{
    FretApp,
    advanced::prelude::*,
    assets::{self, AssetBundleId, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry},
    shadcn,
};
use fret_core::{ImageColorSpace, ImageId};
use fret_icons::FrozenIconRegistry;
use fret_ui::element::{ImageProps, LayoutStyle, SvgIconProps};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

const TEST_ID_ROOT: &str = "cookbook.icons_and_assets_basics.root";
const TEST_ID_PANEL_ICONS: &str = "cookbook.icons_and_assets_basics.panel.icons";
const TEST_ID_PANEL_SVG: &str = "cookbook.icons_and_assets_basics.panel.svg";
const TEST_ID_PANEL_IMAGE: &str = "cookbook.icons_and_assets_basics.panel.image";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.icons_and_assets_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.icons_and_assets_basics.svg.status";

const COOKBOOK_IMAGE_KEY: &str = "images/test.jpg";
const COOKBOOK_SVG_KEY: &str = "icons/search.svg";
const COOKBOOK_IMAGE_BYTES: &[u8] = include_bytes!("../../../assets/textures/test.jpg");
const COOKBOOK_SVG_BYTES: &[u8] = include_bytes!("../../../assets/demo/icon-search.svg");

fn cookbook_asset_bundle() -> AssetBundleId {
    AssetBundleId::app("fret-cookbook")
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
    let theme = Theme::global(&*cx.app).snapshot();
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

fn install_demo_asset_resolver(app: &mut KernelApp) {
    assets::register_bundle_entries(
        app,
        cookbook_asset_bundle(),
        [
            StaticAssetEntry::new(COOKBOOK_IMAGE_KEY, AssetRevision(1), COOKBOOK_IMAGE_BYTES)
                .with_media_type("image/jpeg"),
            StaticAssetEntry::new(COOKBOOK_SVG_KEY, AssetRevision(1), COOKBOOK_SVG_BYTES)
                .with_media_type("image/svg+xml"),
        ],
    );
}

struct IconsAndAssetsBasicsView {
    bundle_image: fret_ui_assets::ImageSource,
    memory_image: fret_ui_assets::ImageSource,
    svg_request: AssetRequest,
}

impl View for IconsAndAssetsBasicsView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        // Ensure the UI assets caches exist and set budgets explicitly (optional).
        fret_ui_assets::UiAssets::configure(
            app,
            fret_ui_assets::UiAssetsBudgets {
                image_budget_bytes: 32 * 1024 * 1024,
                image_max_ready_entries: 1024,
                svg_budget_bytes: 8 * 1024 * 1024,
                svg_max_ready_entries: 2048,
            },
        );

        install_demo_asset_resolver(app);

        let bundle_image = fret_ui_assets::resolve_image_source_from_host_locator(
            app,
            AssetLocator::bundle(cookbook_asset_bundle(), COOKBOOK_IMAGE_KEY),
        )
        .expect("cookbook bundle image should resolve");
        let memory_image = fret_ui_assets::ImageSource::rgba8(
            128,
            128,
            checkerboard_rgba8(128, 128, 16),
            ImageColorSpace::Srgb,
        );
        let svg_request = AssetRequest::new(AssetLocator::bundle(
            cookbook_asset_bundle(),
            COOKBOOK_SVG_KEY,
        ));

        Self {
            bundle_image,
            memory_image,
            svg_request,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();

        let header = shadcn::card_header(|cx| {
            ui::children![
                cx;
                shadcn::card_title("Icons + assets basics"),
                shadcn::card_description(
                    "Icon packs (lucide), semantic ui.* aliases, and logical bundle assets resolved through the `fret::assets` facade.",
                ),
            ]
        });

        let callouts = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Badge::new("Bundle locator: no repo-relative filesystem assumption")
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

        let bundle_image_state = cx.use_image_source_state(&self.bundle_image);
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
                            "Bundle-based decode is async; in-memory RGBA8 is immediate and useful for deterministic demos.",
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
                                            "From bundle locator: `cookbook.demo_assets/images/test.jpg`",
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

        let svg_source = fret_ui_assets::resolve_svg_source_from_host(&*cx.app, &self.svg_request);
        let svg_status = if svg_source.is_ok() { "ready" } else { "error" };
        let svg_foreground = theme.color_token("foreground");
        let svg_error_color = ColorRef::Color(theme.color_token("destructive"));

        let svg_box = ui::container(move |cx| match svg_source.clone() {
            Ok(source) => {
                let mut props = SvgIconProps::new(source);
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(160.0));
                layout.size.height = Length::Px(Px(160.0));
                props.layout = layout;
                props.fit = fret_core::SvgFit::Contain;
                props.color = svg_foreground;
                ui::children![cx; cx.svg_icon_props(props)]
            }
            Err(err) => {
                ui::children![cx;
                    ui::text(format!("Failed to resolve SVG asset: {err}"))
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
                            "Resolves an SVG byte asset through the host asset resolver instead of reading from a raw filesystem path.",
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
        .window("cookbook-icons-and-assets-basics", (960.0, 860.0))
        .setup((
            fret_cookbook::install_cookbook_defaults,
            fret_icons_lucide::app::install,
        ))
        .view::<IconsAndAssetsBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
