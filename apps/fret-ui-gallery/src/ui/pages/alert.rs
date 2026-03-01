use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let build_alert = |cx: &mut ElementContext<'_, App>,
                       test_id: &'static str,
                       variant: shadcn::AlertVariant,
                       icon_name: &'static str,
                       title: &'static str,
                       description: &'static str| {
        shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon_name)),
            shadcn::AlertTitle::new(title).into_element(cx),
            shadcn::AlertDescription::new(description).into_element(cx),
        ])
        .variant(variant)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .test_id(test_id)
    };

    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            ui::children![
                cx;
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-success",
                    shadcn::AlertVariant::Default,
                    "lucide.circle-check",
                    "Payment successful",
                    "Your payment of $29.99 has been processed and a receipt has been emailed.",
                ),
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-info",
                    shadcn::AlertVariant::Default,
                    "lucide.info",
                    "New feature available",
                    "Dark mode support is now available in account settings.",
                ),
            ]
        },
    )
    .test_id("ui-gallery-alert-demo");

    let basic = build_alert(
        cx,
        "ui-gallery-alert-basic",
        shadcn::AlertVariant::Default,
        "lucide.circle-check",
        "Account updated successfully",
        "Your profile information has been saved and applied immediately.",
    );

    let destructive = build_alert(
        cx,
        "ui-gallery-alert-destructive",
        shadcn::AlertVariant::Destructive,
        "lucide.triangle-alert",
        "Payment failed",
        "Please verify card details, billing address, and available funds.",
    );

    let action = {
        shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.moon")),
            shadcn::AlertTitle::new("Dark mode is now available").into_element(cx),
            shadcn::AlertDescription::new(
                "Enable it in profile settings to reduce eye strain during long sessions.",
            )
            .into_element(cx),
            shadcn::AlertAction::new([shadcn::Button::new("Enable")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)
                .test_id("ui-gallery-alert-action-enable")])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .test_id("ui-gallery-alert-action")
    };

    let warn_bg = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFF_FA_EB));
    let warn_border = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFA_D9_73));
    let custom_colors = shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
        shadcn::AlertTitle::new("Your subscription expires in 3 days").into_element(cx),
        shadcn::AlertDescription::new(
            "Renew now to avoid service interruption or upgrade to a paid plan.",
        )
        .into_element(cx),
    ])
    .refine_style(
        ChromeRefinement::default()
            .bg(warn_bg.clone())
            .border_color(warn_border.clone()),
    )
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-colors");

    let rtl = doc_layout::rtl(cx, |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_start(),
            |cx| {
                ui::children![cx; build_alert(
                    cx,
                    "ui-gallery-alert-rtl",
                    shadcn::AlertVariant::Default,
                    "lucide.info",
                    "RTL alert sample",
                    "This alert validates right-to-left layout and text alignment.",
                )]
            },
        )
    });

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/alert.rs` and `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`.",
            "Keep alert copy concise and action-oriented; reserve longer guidance for Dialog or Sheet.",
            "Use `Destructive` only for high-risk or blocking failures to preserve visual hierarchy.",
            "Validate RTL + narrow layout so icon/title/description remain readable in editor sidebars.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Alert docs order: Demo, Basic, Destructive, Action, Custom Colors, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A small set of inline alerts for different message tones.")
                .max_w(Px(720.0))
                .test_id_prefix("ui-gallery-alert")
                .code(
                    "rust",
                    r#"shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-check")),
    shadcn::AlertTitle::new("Payment successful").into_element(cx),
    shadcn::AlertDescription::new("...").into_element(cx),
])
.variant(shadcn::AlertVariant::Default)
.into_element(cx);"#,
                ),
            DocSection::new("Basic", basic)
                .description("Default variant for neutral info.")
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-check")),
    shadcn::AlertTitle::new("Account updated successfully").into_element(cx),
    shadcn::AlertDescription::new("...").into_element(cx),
])
.variant(shadcn::AlertVariant::Default)
.into_element(cx);"#,
                ),
            DocSection::new("Destructive", destructive)
                .description("Destructive variant for critical errors.")
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
    shadcn::AlertTitle::new("Payment failed").into_element(cx),
    shadcn::AlertDescription::new("...").into_element(cx),
])
.variant(shadcn::AlertVariant::Destructive)
.into_element(cx);"#,
                ),
            DocSection::new("Action", action)
                .description("Use `AlertAction` to pin a top-right action inside the alert.")
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.moon")),
    shadcn::AlertTitle::new("Dark mode is now available").into_element(cx),
    shadcn::AlertDescription::new("...").into_element(cx),
    shadcn::AlertAction::new([
        shadcn::Button::new("Enable")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx),
    ])
    .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Custom Colors", custom_colors)
                .description("Custom chrome override for special emphasis.")
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
    shadcn::AlertTitle::new("Your subscription expires soon").into_element(cx),
    shadcn::AlertDescription::new("...").into_element(cx),
])
.refine_style(
    ChromeRefinement::default()
        .bg(ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFF_FA_EB)))
        .border_color(ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFA_D9_73))),
)
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Alert layout under an RTL direction provider.")
                .max_w(Px(720.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info")),
            shadcn::AlertTitle::new("RTL alert sample").into_element(cx),
            shadcn::AlertDescription::new("...").into_element(cx),
        ])
        .into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and caveats.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-alert-component")]
}
