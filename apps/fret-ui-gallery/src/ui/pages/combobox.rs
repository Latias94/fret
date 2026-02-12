mod helpers;
mod models;
mod sections;

mod prelude {
    pub(super) use super::super::super::*;
}

use prelude::*;

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let models = models::get_or_init(cx);
    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

    let demo = sections::demo(cx, value, open, query);
    let custom_items_top = sections::custom_items_top(cx, &models);
    let multiple_selection = sections::multiple_selection(cx);
    let basic = sections::basic(cx, &models);
    let multiple = sections::multiple(cx);
    let clear_button = sections::clear_button(cx);
    let groups = sections::groups(cx, &models);
    let custom_items_example = sections::custom_items_example(cx);
    let invalid = sections::invalid(cx, &models, destructive);
    let disabled = sections::disabled(cx, &models);
    let auto_highlight = sections::auto_highlight(cx, &models);
    let popup = sections::popup(cx);
    let input_group = sections::input_group(cx, &models);
    let rtl = sections::rtl(cx, &models);

    let component_panel = sections::component_panel(
        cx,
        demo,
        custom_items_top,
        multiple_selection,
        basic,
        multiple,
        clear_button,
        groups,
        custom_items_example,
        invalid,
        disabled,
        auto_highlight,
        popup,
        input_group,
        rtl,
    );

    let code_panel = code_panel(cx);
    let notes_panel = notes_panel(cx);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-combobox",
        component_panel,
        code_panel,
        notes_panel,
    )
}

fn code_block(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    snippet: &'static str,
) -> AnyElement {
    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
}

fn code_panel(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .placeholder("Select a framework")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
    ])
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Style + Disabled",
                    r#"let invalid = shadcn::Combobox::new(value, open)
    .refine_style(ChromeRefinement::default().border_1())
    .disabled(true)
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Combobox::new(value, open)
        .placeholder("????")
        .into_element(cx)
})"#,
                ),
            ]
        },
    );
    helpers::shell(cx, code_stack)
}

fn notes_panel(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Current Fret `Combobox` focuses on single-select + query filtering; several Base UI recipes are tracked as explicit gaps here.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep unsupported sections visible (multiple/clear/popup) to make parity progress auditable instead of implicit.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For invalid visuals today, apply style overrides on trigger and pair with field-level error copy.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
                ),
            ]
        },
    );
    helpers::shell(cx, notes_stack)
}
