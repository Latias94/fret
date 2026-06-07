use fret_app::App;
use fret_core::Px;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

pub(super) fn diag_card(
    cx: &mut ElementContext<'_, App>,
    title: impl Into<String>,
    description: impl Into<String>,
    content: Vec<AnyElement>,
) -> AnyElement {
    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new(title.into()).into_element(cx),
            shadcn::CardDescription::new(description.into()).into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .into_element(cx)
}

pub(super) fn diag_section(
    cx: &mut ElementContext<'_, App>,
    title: impl Into<String>,
    description: impl Into<String>,
    content: Vec<AnyElement>,
) -> AnyElement {
    let theme = cx.theme_snapshot();
    let block = ui::v_stack(|cx| {
        [
            cx.text(title.into()),
            cx.text(description.into()),
            ui::v_stack(|_cx| content)
                .gap(fret_ui_kit::Space::N2)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default()
                .bg(fret_ui_kit::ColorRef::Color(theme.color_token("muted")))
                .border_1()
                .border_color(fret_ui_kit::ColorRef::Color(theme.color_token("border")))
                .px(fret_ui_kit::Space::N3)
                .py(fret_ui_kit::Space::N3),
            fret_ui_kit::LayoutRefinement::default().w_full(),
        ),
        |_cx| [block],
    )
}

pub(super) fn text_blob(cx: &mut ElementContext<'_, App>, text: String) -> AnyElement {
    let text = if text.is_empty() {
        "<empty>".to_string()
    } else {
        text
    };

    let pre = cx.text(text);
    shadcn::ScrollArea::new([pre]).into_element(cx)
}

pub(super) fn text_blob_sized(
    cx: &mut ElementContext<'_, App>,
    text: String,
    min_h: Px,
) -> AnyElement {
    let text = if text.is_empty() {
        "<empty>".to_string()
    } else {
        text
    };

    let pre = cx.text(text);
    shadcn::ScrollArea::new([pre])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(min_h),
        )
        .into_element(cx)
}
