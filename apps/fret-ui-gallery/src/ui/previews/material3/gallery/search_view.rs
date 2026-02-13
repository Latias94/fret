use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_search_view(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use fret_icons::ids::ui as ui_icons;

    #[derive(Default)]
    struct SearchViewPageModels {
        open: Option<Model<bool>>,
        query: Option<Model<String>>,
        selected: Option<Model<Arc<str>>>,
    }

    let open = cx.with_state(SearchViewPageModels::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SearchViewPageModels::default, |st| {
                st.open = Some(model.clone())
            });
            model
        }
    };

    let query = cx.with_state(SearchViewPageModels::default, |st| st.query.clone());
    let query = match query {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(SearchViewPageModels::default, |st| {
                st.query = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(SearchViewPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("alpha"));
            cx.with_state(SearchViewPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let suggestions = material3::List::new(selected)
        .a11y_label("Suggestions")
        .test_id("ui-gallery-material3-search-view-suggestions")
        .items(vec![
            material3::ListItem::new("alpha", "Alpha")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-alpha"),
            material3::ListItem::new("bravo", "Bravo")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-bravo"),
            material3::ListItem::new("charlie", "Charlie")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-charlie"),
        ])
        .into_element(cx);

    let view = material3::SearchView::new(open, query)
        .leading_icon(ui_icons::SEARCH)
        .trailing_icon(ui_icons::CLOSE)
        .placeholder("Search")
        .a11y_label("Search")
        .test_id("ui-gallery-material3-search-view")
        .overlay_test_id("ui-gallery-material3-search-view-panel")
        .into_element(cx, |_cx| vec![suggestions]);

    vec![view]
}
