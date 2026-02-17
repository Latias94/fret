use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let icon_item = |cx: &mut ElementContext<'_, App>, value: &'static str, label: &'static str| {
        shadcn::ToggleGroupItem::new(
            value,
            [shadcn::icon::icon(
                cx,
                fret_icons::IconId::new_static(match value {
                    "bold" => "lucide.bold",
                    "italic" => "lucide.italic",
                    _ => "lucide.underline",
                }),
            )],
        )
        .a11y_label(label)
    };

    let text_item = |cx: &mut ElementContext<'_, App>, value: &'static str, label: &'static str| {
        shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
    };

    let demo = shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-demo");

    let outline = shadcn::ToggleGroup::multiple_uncontrolled(["left"])
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            text_item(cx, "left", "Left"),
            text_item(cx, "center", "Center"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-outline");

    let size = {
        let sm = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);
        let default = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Default)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);
        let lg = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Lg)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| {
                vec![
                    shadcn::typography::muted(cx, "Sm / Default / Lg"),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![sm, default, lg],
                    ),
                ]
            },
        )
        .test_id("ui-gallery-toggle-group-size")
    };

    let spacing = shadcn::ToggleGroup::single_uncontrolled(Some("top"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            text_item(cx, "top", "Top"),
            text_item(cx, "bottom", "Bottom"),
            text_item(cx, "left", "Left"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing");

    let vertical = shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
        .orientation(fret_ui_kit::primitives::toggle_group::ToggleGroupOrientation::Vertical)
        .spacing(Space::N1)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-vertical");

    let disabled = shadcn::ToggleGroup::multiple_uncontrolled(["bold"])
        .disabled(true)
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-disabled");

    let rtl = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::ToggleGroup::single_uncontrolled(Some("list"))
                .variant(shadcn::ToggleVariant::Outline)
                .items([
                    text_item(cx, "list", "List"),
                    text_item(cx, "grid", "Grid"),
                    text_item(cx, "cards", "Cards"),
                ])
                .into_element(cx)
        },
    )
    .test_id("ui-gallery-toggle-group-rtl");
    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/toggle_group.rs` and `ecosystem/fret-ui-shadcn/src/toggle.rs`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use Single mode for mutually-exclusive options (alignment, list/grid/cards).",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use Multiple mode for formatting toggles where users may combine states.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`spacing` is useful when each item needs stronger visual separation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For icon-only groups, keep explicit `a11y_label` for assistive technologies.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Toggle Group docs order: Demo, Outline, Size, Spacing, Vertical, Disabled, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Multiple selection with icon-only items.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
    .variant(shadcn::ToggleVariant::Outline)
    .items([
        shadcn::ToggleGroupItem::new("bold", [shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bold"))]),
        shadcn::ToggleGroupItem::new("italic", [shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic"))]),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Outline", outline)
                .description("Text items with outline chrome.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::multiple_uncontrolled(["left"])
    .variant(shadcn::ToggleVariant::Outline)
    .items([
        shadcn::ToggleGroupItem::new("left", [cx.text("Left")]).a11y_label("Toggle Left"),
        shadcn::ToggleGroupItem::new("center", [cx.text("Center")]).a11y_label("Toggle Center"),
        shadcn::ToggleGroupItem::new("right", [cx.text("Right")]).a11y_label("Toggle Right"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .description("Size presets for toolbar density.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::single_uncontrolled(Some("left"))
    .variant(shadcn::ToggleVariant::Outline)
    .size(shadcn::ToggleSize::Sm)
    .items([
        shadcn::ToggleGroupItem::new("left", [cx.text("Left")]).a11y_label("Toggle Left"),
        shadcn::ToggleGroupItem::new("center", [cx.text("Center")]).a11y_label("Toggle Center"),
        shadcn::ToggleGroupItem::new("right", [cx.text("Right")]).a11y_label("Toggle Right"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Spacing", spacing)
                .description("Explicit spacing between items to reduce mis-clicks.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::single_uncontrolled(Some("top"))
    .variant(shadcn::ToggleVariant::Outline)
    .size(shadcn::ToggleSize::Sm)
    .spacing(Space::N2)
    .items([
        shadcn::ToggleGroupItem::new("top", [cx.text("Top")]).a11y_label("Toggle Top"),
        shadcn::ToggleGroupItem::new("bottom", [cx.text("Bottom")]).a11y_label("Toggle Bottom"),
        shadcn::ToggleGroupItem::new("left", [cx.text("Left")]).a11y_label("Toggle Left"),
        shadcn::ToggleGroupItem::new("right", [cx.text("Right")]).a11y_label("Toggle Right"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation for side panels / inspectors.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
    .orientation(fret_ui_kit::primitives::toggle_group::ToggleGroupOrientation::Vertical)
    .spacing(Space::N1)
    .items([
        shadcn::ToggleGroupItem::new("bold", [cx.text("Bold")]).a11y_label("Toggle Bold"),
        shadcn::ToggleGroupItem::new("italic", [cx.text("Italic")]).a11y_label("Toggle Italic"),
        shadcn::ToggleGroupItem::new("underline", [cx.text("Underline")]).a11y_label("Toggle Underline"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled)
                .description("Disabled groups keep layout but block interaction.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"shadcn::ToggleGroup::multiple_uncontrolled(["bold"])
    .disabled(true)
    .variant(shadcn::ToggleVariant::Outline)
    .items([
        shadcn::ToggleGroupItem::new("bold", [cx.text("Bold")]).a11y_label("Toggle Bold"),
        shadcn::ToggleGroupItem::new("italic", [cx.text("Italic")]).a11y_label("Toggle Italic"),
        shadcn::ToggleGroupItem::new("underline", [cx.text("Underline")]).a11y_label("Toggle Underline"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Item ordering and pressed visuals under RTL.")
                .max_w(Px(560.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::ToggleGroup::single_uncontrolled(Some("list"))
            .variant(shadcn::ToggleVariant::Outline)
            .items([
                shadcn::ToggleGroupItem::new("list", [cx.text("List")]).a11y_label("Toggle List"),
                shadcn::ToggleGroupItem::new("grid", [cx.text("Grid")]).a11y_label("Toggle Grid"),
                shadcn::ToggleGroupItem::new("cards", [cx.text("Cards")]).a11y_label("Toggle Cards"),
            ])
            .into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-toggle-group")]
}
