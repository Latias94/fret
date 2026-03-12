pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret::UiCx;
use fret_core::Edges;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    dropdown_open: Option<Model<bool>>,
}

fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement {
    fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let dropdown_open = cx
        .with_state(Models::default, |st| st.dropdown_open.clone())
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.dropdown_open = Some(model.clone()));
            model
        });

    let people = [
        ("shadcn", "S", "shadcn@vercel.com"),
        ("maxleiter", "M", "maxleiter@vercel.com"),
        ("evilrabbit", "E", "evilrabbit@vercel.com"),
    ];

    let menu = shadcn::DropdownMenu::new(dropdown_open.clone())
        .align(shadcn::DropdownMenuAlign::End)
        .min_width(Px(288.0))
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Select")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(dropdown_open.clone())
                    .children([
                        ui::text("Select").text_sm().into_element(cx),
                        icon(cx, "lucide.chevron-down"),
                    ])
                    .test_id("ui-gallery-item-dropdown-trigger")
                    .into_element(cx)
            },
            |cx| {
                people
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(idx, (username, initials, email))| {
                        let avatar = shadcn::Avatar::new([
                            shadcn::AvatarFallback::new(initials).into_element(cx)
                        ])
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .into_element(cx);
                        let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
                        let content = shadcn::ItemContent::new([
                            shadcn::ItemTitle::new(username).into_element(cx),
                            shadcn::ItemDescription::new(email).into_element(cx),
                        ])
                        .gap(Px(6.0))
                        .into_element(cx);

                        let item = shadcn::Item::new([media, content])
                            .size(shadcn::ItemSize::Sm)
                            .variant(shadcn::ItemVariant::Outline)
                            .refine_style(ChromeRefinement::default().px(Space::N2).py(Space::N2))
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                            .test_id(format!("ui-gallery-item-dropdown-item-{idx}"));

                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new(username)
                                .padding(Edges::all(Px(0.0)))
                                .content(item),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

    ui::v_flex(|_cx| vec![menu])
        .gap(Space::N6)
        .items_center()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .min_h(Px(256.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-item-dropdown")
}
// endregion: example
