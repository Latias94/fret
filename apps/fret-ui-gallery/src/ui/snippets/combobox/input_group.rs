pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::{ElementContextThemeExt, style as decl_style};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn state_row(
    cx: &mut UiCx<'_>,
    text: Arc<str>,
    test_id: Arc<str>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default().w_full().min_w_0(),
        )
    });
    cx.container(props, move |cx| {
        [shadcn::raw::typography::muted(text).into_element(cx)]
    })
    .test_id(test_id)
}

fn state_rows(
    cx: &mut UiCx<'_>,
    value: &Model<Option<Arc<str>>>,
    query: &Model<String>,
    test_id_prefix: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let selected: Arc<str> = cx
        .get_model_cloned(value, Invalidation::Paint)
        .unwrap_or_default()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query_text = cx
        .get_model_cloned(query, Invalidation::Paint)
        .unwrap_or_default();

    let selected_row_text: Arc<str> = Arc::from(format!("Selected: {selected}"));
    let query_row_text: Arc<str> = Arc::from(format!("Query: {query_text}"));
    let selected_row_test_id: Arc<str> = Arc::from(format!("{test_id_prefix}-selected"));
    let query_row_test_id: Arc<str> = Arc::from(format!("{test_id_prefix}-query"));

    ui::v_flex(move |cx| {
        vec![
            state_row(cx, selected_row_text.clone(), selected_row_test_id.clone()),
            state_row(cx, query_row_text.clone(), query_row_test_id.clone()),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox input group")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-input-group")
        .items([
            shadcn::ComboboxItem::new("new-file", "New File"),
            shadcn::ComboboxItem::new("open-file", "Open File"),
            shadcn::ComboboxItem::new("save-all", "Save All"),
        ])
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(shadcn::ComboboxTrigger::new().width_px(Px(220.0))),
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new().placeholder("Search command"),
                ),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            ui::h_row(|cx| {
                vec![
                    {
                        let props = cx.with_theme(|theme| {
                            decl_style::container_props(
                                theme,
                                ChromeRefinement::default()
                                    .border_1()
                                    .rounded(Radius::Sm)
                                    .px(Space::N2)
                                    .py(Space::N1),
                                LayoutRefinement::default(),
                            )
                        });
                        cx.container(props, |cx| {
                            vec![shadcn::raw::typography::muted("Cmd").into_element(cx)]
                        })
                    },
                    combo,
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            state_rows(cx, &value, &query, "ui-gallery-combobox-input-group").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
    .into_element(cx)
}
// endregion: example
