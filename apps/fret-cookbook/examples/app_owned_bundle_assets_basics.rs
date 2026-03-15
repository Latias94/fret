use fret::{
    FretApp,
    advanced::prelude::*,
    assets::{AssetBundleId, AssetLocator, AssetRequest, AssetRevision, StaticAssetEntry},
    shadcn,
};
use fret_ui::element::{ImageProps, LayoutStyle, Length, SizeStyle, SvgIconProps};
use fret_ui_assets::ui::{ImageSourceElementContextExt as _, SvgAssetElementContextExt as _};

const APP_ID: &str = "cookbook-app-owned-bundle-assets-basics";
const IMAGE_KEY: &str = "textures/test.jpg";
const SVG_KEY: &str = "demo/icon-search.svg";
const TEST_ID_ROOT: &str = "cookbook.app_owned_bundle_assets_basics.root";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.app_owned_bundle_assets_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.app_owned_bundle_assets_basics.svg.status";

const APP_IMAGE_BYTES: &[u8] = include_bytes!("../../../assets/textures/test.jpg");
const APP_SVG_BYTES: &[u8] = include_bytes!("../../../assets/demo/icon-search.svg");
const APP_ASSET_ENTRIES: [StaticAssetEntry; 2] = [
    StaticAssetEntry::new(IMAGE_KEY, AssetRevision(1), APP_IMAGE_BYTES)
        .with_media_type("image/jpeg"),
    StaticAssetEntry::new(SVG_KEY, AssetRevision(1), APP_SVG_BYTES)
        .with_media_type("image/svg+xml"),
];

fn demo_app_bundle() -> AssetBundleId {
    AssetBundleId::app(APP_ID)
}

struct AppOwnedBundleAssetsBasicsView {
    image_request: AssetRequest,
    svg_request: AssetRequest,
}

impl View for AppOwnedBundleAssetsBasicsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            image_request: AssetRequest::new(AssetLocator::bundle(demo_app_bundle(), IMAGE_KEY)),
            svg_request: AssetRequest::new(AssetLocator::bundle(demo_app_bundle(), SVG_KEY)),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let image_state = cx.use_image_source_state_from_asset_request(&self.image_request);
        let svg_state = cx.svg_source_state_from_asset_request(&self.svg_request);

        let callouts = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Badge::new("Portable default: app-owned logical bundle assets")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Scaffold equivalent: `generated_assets::mount(builder)`")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Generated module is enough when the crate only publishes shipped bytes")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Lower-level builder seam: `FretApp::asset_entries(...)`")
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("`BundleAsset` is the public lookup lane; `Embedded` stays lower-level")
                    .variant(shadcn::BadgeVariant::Secondary),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_center()
        .w_full();

        let image_panel = render_image_panel(cx, &theme, image_state);
        let svg_panel = render_svg_panel(cx, &theme, svg_state);

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("App-owned bundle assets basics"),
                        shadcn::card_description(
                            "Demonstrates the compile-time portable app asset lane: register static `StaticAssetEntry` values on the builder path, then resolve image/SVG UI state through logical app bundle locators. This is the generated-module lane to teach when a crate only publishes shipped bytes.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![cx; callouts, image_panel, svg_panel]
                        })
                        .gap(Space::N4)
                        .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(960.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn render_image_panel(
    _cx: &mut UiCx<'_>,
    theme: &ThemeSnapshot,
    state: fret_ui_assets::ImageSourceState,
) -> impl IntoUiElement<KernelApp> + use<> {
    let status = match state.status {
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
    };
    let border = ColorRef::Color(theme.color_token("border"));

    let image_box = ui::container(move |cx| {
        if let Some(image) = state.image {
            let mut props = ImageProps::new(image);
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            ui::children![cx; cx.image_props(props)]
        } else {
            ui::children![cx; cx.spinner()]
        }
    })
    .border_1()
    .border_color(border)
    .rounded(Radius::Lg)
    .w_px(Px(260.0))
    .h_px(Px(180.0))
    .overflow_hidden();

    let body = ui::v_flex_build(move |cx, out| {
        out.push_ui(
            cx,
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("Image status:"),
                    shadcn::Badge::new(status)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .test_id(TEST_ID_IMAGE_STATUS),
                ]
            })
            .gap(Space::N2)
            .items_center(),
        );
        out.push_ui(
            cx,
            shadcn::Alert::build(|cx, out| {
                out.push_ui(cx, shadcn::AlertTitle::new("Compile-time app bundle"));
                out.push_ui(
                    cx,
                    shadcn::AlertDescription::new(
                        "This is the same logical bundle lane that generated `src/generated_assets.rs` modules publish; widget code only sees `AssetLocator::bundle(...)` requests.",
                    ),
                );
            }),
        );
        out.push_ui(cx, image_box);

        if let Some(message) = state.error {
            out.push_ui(
                cx,
                shadcn::Alert::build(|cx, out| {
                    out.push_ui(cx, shadcn::AlertTitle::new("Image asset failed"));
                    out.push_ui(cx, shadcn::AlertDescription::new(message));
                })
                .variant(shadcn::AlertVariant::Destructive),
            );
        }
    })
    .gap(Space::N3)
    .items_start();

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Image from app bundle locator"),
                    shadcn::card_description(
                        "Loads `textures/test.jpg` through the app bundle namespace mounted on `FretApp::asset_entries(...)`, without native-only file path assumptions.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; body]),
        ]
    })
    .ui()
    .w_full()
}

fn render_svg_panel(
    _cx: &mut UiCx<'_>,
    theme: &ThemeSnapshot,
    state: fret_ui_assets::ui::SvgAssetSourceState,
) -> impl IntoUiElement<KernelApp> + use<> {
    let status = if state.error.is_some() {
        "error"
    } else if state.source.is_some() {
        "ready"
    } else {
        "missing"
    };
    let border = ColorRef::Color(theme.color_token("border"));
    let destructive = ColorRef::Color(theme.color_token("destructive"));
    let foreground = theme.color_token("foreground");

    let svg_box = ui::container(move |cx| {
        if let Some(source) = state.source.clone() {
            let mut props = SvgIconProps::new(source);
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            props.fit = fret_core::SvgFit::Contain;
            props.color = foreground;
            ui::children![cx; cx.svg_icon_props(props)]
        } else if let Some(error) = state.error.clone() {
            ui::children![cx;
                ui::text(format!("Failed to resolve SVG asset: {error}"))
                    .text_color(destructive)
            ]
        } else {
            ui::children![cx; cx.spinner()]
        }
    })
    .border_1()
    .border_color(border)
    .rounded(Radius::Lg)
    .p(Space::N4)
    .w_px(Px(260.0))
    .h_px(Px(180.0));

    let body = ui::v_flex_build(move |cx, out| {
        out.push_ui(
            cx,
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("SVG status:"),
                    shadcn::Badge::new(status)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .test_id(TEST_ID_SVG_STATUS),
                ]
            })
            .gap(Space::N2)
            .items_center(),
        );
        out.push_ui(
            cx,
            shadcn::Alert::build(|cx, out| {
                let bundle = demo_app_bundle();
                out.push_ui(cx, shadcn::AlertTitle::new("App-owned bundle id"));
                out.push_ui(
                    cx,
                    shadcn::AlertDescription::new(format!(
                        "The request targets `{}` and remains portable across desktop, web, and future mobile packaging.",
                        bundle.as_str()
                    )),
                );
            }),
        );
        out.push_ui(cx, svg_box);
    })
    .gap(Space::N3)
    .items_start();

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("SVG from app bundle locator"),
                    shadcn::card_description(
                        "Loads `demo/icon-search.svg` from the same app-owned logical bundle and lets the shared UI helper choose the correct native/web bridge.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; body]),
        ]
    })
    .ui()
    .w_full()
}

fn main() -> anyhow::Result<()> {
    FretApp::new(APP_ID)
        .window(APP_ID, (960.0, 780.0))
        .ui_assets_budgets(32 * 1024 * 1024, 1024, 8 * 1024 * 1024, 2048)
        .asset_entries(APP_ASSET_ENTRIES)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<AppOwnedBundleAssetsBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
