use super::super::*;

pub(super) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let trunc_layout = LayoutRefinement::default().max_w(Px(112.0));

    let demo_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-demo");
    let demo = section_card(cx, "Demo", demo_content);

    let basic_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-basic");
    let basic = section_card(cx, "Basic", basic_content);

    let custom_separator_content = shadcn::Breadcrumb::new()
        .separator(shadcn::BreadcrumbSeparator::Text(Arc::from("?")))
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-separator");
    let custom_separator = section_card(cx, "Custom Separator", custom_separator_content);

    let dropdown_content = shadcn::Breadcrumb::new()
        .separator(shadcn::BreadcrumbSeparator::Text(Arc::from("?")))
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components ?"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-dropdown");
    let dropdown = section_card(cx, "Dropdown", dropdown_content);

    let collapsed_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Documentation"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-collapsed");
    let collapsed = section_card(cx, "Collapsed", collapsed_content);

    let link_component_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home (router link)")
                .truncate(true)
                .refine_layout(trunc_layout.clone()),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-link");
    let link_component = section_card(cx, "Link Component", link_component_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Breadcrumb::new()
                .separator(shadcn::BreadcrumbSeparator::Text(Arc::from("?")))
                .items([
                    shadcn::BreadcrumbItem::new("Home"),
                    shadcn::BreadcrumbItem::new("Components"),
                    shadcn::BreadcrumbItem::new("Breadcrumb"),
                ])
                .into_element(cx)
        },
    )
    .test_id("ui-gallery-breadcrumb-rtl");
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Breadcrumb docs order for quick lookup and side-by-side behavior checks.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                basic,
                custom_separator,
                dropdown,
                collapsed,
                link_component,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).attach_semantics(
        SemanticsDecoration::default().test_id("ui-gallery-breadcrumb-component"),
    );

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

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
                    r#"Breadcrumb::new().items([
    BreadcrumbItem::new("Home"),
    BreadcrumbItem::new("Components"),
    BreadcrumbItem::new("Breadcrumb"),
])"#,
                ),
                code_block(
                    cx,
                    "Custom Separator + Collapsed",
                    r#"Breadcrumb::new()
    .separator(BreadcrumbSeparator::Text(Arc::from("?")))
    .items([BreadcrumbItem::new("Home"), BreadcrumbItem::ellipsis(), ...])"#,
                ),
                code_block(
                    cx,
                    "Link + RTL",
                    r#"BreadcrumbItem::new("Home (router link)").truncate(true)
with_direction_provider(LayoutDirection::Rtl, |cx| Breadcrumb::new().items([...]).into_element(cx))"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

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
                    "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current dropdown and router-link samples are visual approximations; full `asChild` composition can be added in a follow-up primitive demo.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Validate RTL with long labels to ensure truncation and separator spacing remain stable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-breadcrumb",
        component_panel,
        code_panel,
        notes_panel,
    )
}
