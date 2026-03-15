pub const SOURCE: &str = include_str!("file_tree.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Point, Transform2D};
use fret_ui::Theme;
use fret_ui::element::VisualTransformProps;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rotated_lucide<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &'static str,
    rotation_deg: f32,
) -> impl IntoUiElement<H> + use<H> {
    let size = Px(16.0);
    let center = Point::new(Px(8.0), Px(8.0));
    let transform = Transform2D::rotation_about_degrees(rotation_deg, center);

    cx.visual_transform_props(
        VisualTransformProps {
            layout: {
                let theme = Theme::global(&*cx.app);
                decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .w_px(size)
                        .h_px(size)
                        .flex_shrink_0(),
                )
            },
            transform,
        },
        move |cx| {
            vec![icon::icon_with(
                cx,
                fret_icons::IconId::new_static(id),
                Some(size),
                None,
            )]
        },
    )
}

fn file_leaf<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    key: &'static str,
    label: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let row = ui::h_flex(|cx| {
        vec![
            icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.file"),
                Some(Px(16.0)),
                None,
            ),
            cx.text(label),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .justify_start()
    .items_center()
    .into_element(cx);

    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Link)
        .size(shadcn::ButtonSize::Sm)
        .refine_layout(LayoutRefinement::default().w_full())
        .children([row])
        .into_element(cx)
        .test_id(format!("ui-gallery-collapsible-tree-leaf-{key}"))
}

fn folder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    key: &'static str,
    label: &'static str,
    open_model: Model<bool>,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Collapsible::new(open_model)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                let chevron =
                    rotated_lucide(cx, "lucide.chevron-right", if is_open { 90.0 } else { 0.0 })
                        .into_element(cx);
                let icon = fret_icons::IconId::new_static(if is_open {
                    "lucide.folder-open"
                } else {
                    "lucide.folder"
                });

                let row = ui::h_flex(|cx| {
                    vec![
                        chevron,
                        icon::icon_with(cx, icon, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                })
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .justify_start()
                .items_center()
                .into_element(cx);

                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .children([row])
                    .toggle_model(open)
                    .test_id(format!("ui-gallery-collapsible-tree-trigger-{key}"))
                    .into_element(cx)
            },
            |cx| {
                shadcn::CollapsibleContent::new(vec![
                    ui::v_flex(|_cx| children)
                        .gap(Space::N1)
                        .items_stretch()
                        .layout(LayoutRefinement::default().w_full().ml(Space::N4))
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id(format!("ui-gallery-collapsible-tree-content-{key}"))
            },
        )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let tree_components_open = cx.local_model_keyed("tree_components_open", || true);
    let tree_src_open = cx.local_model_keyed("tree_src_open", || false);
    let tree_src_ui_open = cx.local_model_keyed("tree_src_ui_open", || false);

    let ui_button = file_leaf(cx, "src-ui-button", "button.rs").into_element(cx);
    let ui_dialog = file_leaf(cx, "src-ui-dialog", "dialog.rs").into_element(cx);
    let ui_folder = folder(
        cx,
        "src-ui",
        "ui",
        tree_src_ui_open.clone(),
        vec![ui_button, ui_dialog],
    )
    .into_element(cx);

    let src_main = file_leaf(cx, "src-main", "main.rs").into_element(cx);
    let src_folder = folder(
        cx,
        "src",
        "src",
        tree_src_open.clone(),
        vec![ui_folder, src_main],
    )
    .into_element(cx);

    let comp_card = file_leaf(cx, "components-card", "card.rs").into_element(cx);
    let comp_table = file_leaf(cx, "components-table", "table.rs").into_element(cx);
    let components_folder = folder(
        cx,
        "components",
        "components",
        tree_components_open.clone(),
        vec![comp_card, comp_table],
    )
    .into_element(cx);

    let cargo_toml = file_leaf(cx, "cargo-toml", "Cargo.toml").into_element(cx);
    ui::v_flex(|_cx| vec![components_folder, src_folder, cargo_toml])
        .gap(Space::N1)
        .items_stretch()
        .layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
        .into_element(cx)
        .test_id("ui-gallery-collapsible-file-tree")
}
// endregion: example
