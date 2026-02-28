use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SheetSideCase {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MiscOverlaysRecipe {
    TooltipDemoOverlayPlacement,
    HoverCardDemoOverlayPlacement,
    DialogDemoOverlayCenter,
    Sidebar13DialogOverlayCenter,
    CommandDialogOverlayCenter,
    CommandDialogInputHeight,
    CommandDialogListboxHeight,
    CommandDialogListboxOptionHeight,
    CommandDialogListboxOptionInsets,
    AlertDialogDemoOverlayCenter,
    SheetDemoOverlayInsets,
    SheetSideOverlayInsets,
    DrawerDemoOverlayInsets,
    DrawerDialogDesktopOverlayCenter,
    DrawerDialogMobileOverlayInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct MiscOverlaysCase {
    id: String,
    web_name: String,
    recipe: MiscOverlaysRecipe,
    #[serde(default)]
    side: Option<SheetSideCase>,
}

fn build_dialog_demo_overlay(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    Dialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open Dialog")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            DialogContent::new(vec![cx.text("Edit profile")])
                .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                .into_element(cx)
        },
    )
}

fn build_sidebar_13_dialog_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonSize, Dialog, DialogContent};

    Dialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open Dialog")
                .size(ButtonSize::Sm)
                .into_element(cx)
        },
        |cx| {
            DialogContent::new(Vec::new())
                .refine_style(fret_ui_kit::ChromeRefinement::default().p(Space::N0))
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .max_w(fret_ui_kit::MetricRef::Px(Px(800.0)))
                        .max_h(fret_ui_kit::MetricRef::Px(Px(500.0))),
                )
                .into_element(cx)
        },
    )
}

fn build_command_dialog_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    #[derive(Default)]
    struct Models {
        query: Option<Model<String>>,
    }

    let existing = cx.with_state(Models::default, |st| st.query.clone());
    let query = if let Some(existing) = existing {
        existing
    } else {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.query = Some(model.clone()));
        model
    };

    let items = vec![
        CommandItem::new("Calendar"),
        CommandItem::new("Search Emoji"),
        CommandItem::new("Calculator"),
    ];

    CommandDialog::new(open.clone(), query, items)
        .into_element(cx, |cx| Button::new("Open").into_element(cx))
}

fn build_alert_dialog_demo_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    AlertDialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Show Dialog")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")]).into_element(cx),
    )
}

fn build_sheet_demo_overlay(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    Sheet::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
    )
}

fn build_sheet_side_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    side: fret_ui_shadcn::SheetSide,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    let label = match side {
        fret_ui_shadcn::SheetSide::Top => "top",
        fret_ui_shadcn::SheetSide::Right => "right",
        fret_ui_shadcn::SheetSide::Bottom => "bottom",
        fret_ui_shadcn::SheetSide::Left => "left",
    };

    Sheet::new(open.clone()).side(side).into_element(
        cx,
        |cx| {
            Button::new(label)
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
    )
}

fn build_drawer_demo_overlay(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    Drawer::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open Drawer")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
    )
}

fn build_drawer_dialog_desktop_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    Dialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Edit Profile")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            DialogContent::new(vec![cx.text("Edit profile")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .max_w(fret_ui_kit::MetricRef::Px(Px(425.0))),
                )
                .into_element(cx)
        },
    )
}

fn build_drawer_dialog_mobile_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle,
    };

    Drawer::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Edit Profile")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            DrawerContent::new(vec![
                DrawerHeader::new(vec![
                    DrawerTitle::new("Edit profile").into_element(cx),
                    DrawerDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    )
}

#[test]
fn web_vs_fret_misc_overlays_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_misc_overlays_cases_v1.json"
    ));
    let suite: FixtureSuite<MiscOverlaysCase> =
        serde_json::from_str(raw).expect("misc-overlays fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("misc-overlays case={}", case.id);
        match case.recipe {
            MiscOverlaysRecipe::TooltipDemoOverlayPlacement => {
                assert_tooltip_demo_overlay_placement_matches(&case.web_name);
            }
            MiscOverlaysRecipe::HoverCardDemoOverlayPlacement => {
                assert_hover_card_demo_overlay_placement_matches(&case.web_name);
            }
            MiscOverlaysRecipe::DialogDemoOverlayCenter => {
                assert_centered_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_dialog_demo_overlay,
                );
            }
            MiscOverlaysRecipe::Sidebar13DialogOverlayCenter => {
                assert_centered_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_sidebar_13_dialog_overlay,
                );
            }
            MiscOverlaysRecipe::CommandDialogOverlayCenter => {
                assert_centered_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_command_dialog_overlay,
                );
            }
            MiscOverlaysRecipe::CommandDialogInputHeight => {
                assert_command_dialog_input_height_matches(&case.web_name);
            }
            MiscOverlaysRecipe::CommandDialogListboxHeight => {
                assert_command_dialog_listbox_height_matches(&case.web_name);
            }
            MiscOverlaysRecipe::CommandDialogListboxOptionHeight => {
                assert_command_dialog_listbox_option_height_matches(&case.web_name);
            }
            MiscOverlaysRecipe::CommandDialogListboxOptionInsets => {
                assert_command_dialog_listbox_option_insets_match(&case.web_name);
            }
            MiscOverlaysRecipe::AlertDialogDemoOverlayCenter => {
                assert_centered_overlay_placement_matches(
                    &case.web_name,
                    "alertdialog",
                    SemanticsRole::AlertDialog,
                    build_alert_dialog_demo_overlay,
                );
            }
            MiscOverlaysRecipe::SheetDemoOverlayInsets => {
                assert_viewport_anchored_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_sheet_demo_overlay,
                );
            }
            MiscOverlaysRecipe::SheetSideOverlayInsets => {
                let side = case.side.unwrap_or_else(|| {
                    panic!(
                        "missing side for recipe sheet_side_overlay_insets: {}",
                        case.id
                    )
                });
                let side = match side {
                    SheetSideCase::Top => fret_ui_shadcn::SheetSide::Top,
                    SheetSideCase::Right => fret_ui_shadcn::SheetSide::Right,
                    SheetSideCase::Bottom => fret_ui_shadcn::SheetSide::Bottom,
                    SheetSideCase::Left => fret_ui_shadcn::SheetSide::Left,
                };
                let build = move |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
                    build_sheet_side_overlay(cx, open, side)
                };
                assert_viewport_anchored_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build,
                );
            }
            MiscOverlaysRecipe::DrawerDemoOverlayInsets => {
                assert_viewport_anchored_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_drawer_demo_overlay,
                );
            }
            MiscOverlaysRecipe::DrawerDialogDesktopOverlayCenter => {
                assert_centered_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_drawer_dialog_desktop_overlay,
                );
            }
            MiscOverlaysRecipe::DrawerDialogMobileOverlayInsets => {
                assert_viewport_anchored_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_drawer_dialog_mobile_overlay,
                );
            }
        }
    }
}
