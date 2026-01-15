use fret_app::{App, CommandId, Model};
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

use crate::{docs::*, spec::*};

fn matches_query(query: &str, item: &NavItemSpec) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }

    let q_lower = q.to_ascii_lowercase();
    if item.label.to_ascii_lowercase().contains(&q_lower) {
        return true;
    }
    if item.origin.to_ascii_lowercase().contains(&q_lower) {
        return true;
    }
    item.tags
        .iter()
        .any(|t| t.to_ascii_lowercase().contains(&q_lower))
}

pub(crate) fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let title_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                cx.text("Fret UI Gallery"),
                shadcn::Badge::new("WIP")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        },
    );

    let query_input = shadcn::Input::new(nav_query)
        .a11y_label("Search components")
        .placeholder("Search… (id / tag)")
        .placeholder("Search (id / tag)")
        .into_element(cx);

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in NAV_GROUPS {
        let group_sections = cx.keyed(group.title, |cx| {
            let mut group_items: Vec<AnyElement> = Vec::new();
            for item in group.items {
                if !matches_query(query, item) {
                    continue;
                }

                let is_selected = selected == item.id;
                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                };

                group_items.push(cx.keyed(item.id, |cx| {
                    shadcn::Button::new(item.label)
                        .variant(variant)
                        .on_click(item.command)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                }));
            }

            if group_items.is_empty() {
                return Vec::new();
            }

            vec![
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::from(group.title),
                    style: None,
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }),
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N1),
                    |_cx| group_items,
                ),
            ]
        });

        nav_sections.extend(group_sections);
    }

    let nav_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| nav_sections,
    );
    let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
        nav_body
    } else {
        shadcn::ScrollArea::new(vec![nav_body])
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
    };

    let container = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N4),
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(280.0)))
                .h_full(),
        ),
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| vec![title_row, query_input, nav_scroll],
            )]
        },
    );

    container
}

pub(crate) fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let (title, origin, docs_md, usage_md) = page_meta(selected);

    let header = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            let left = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N1).items_start(),
                |cx| {
                    vec![
                        cx.text(title),
                        cx.text_props(TextProps {
                            layout: Default::default(),
                            text: Arc::from(origin),
                            style: None,
                            color: Some(theme.color_required("muted-foreground")),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                        }),
                    ]
                },
            );

            let theme_select = shadcn::Select::new(theme_preset, theme_preset_open)
                .placeholder("Theme preset")
                .items([
                    shadcn::SelectItem::new("zinc/light", "Zinc (light)"),
                    shadcn::SelectItem::new("zinc/dark", "Zinc (dark)"),
                    shadcn::SelectItem::new("slate/light", "Slate (light)"),
                    shadcn::SelectItem::new("slate/dark", "Slate (dark)"),
                    shadcn::SelectItem::new("neutral/light", "Neutral (light)"),
                    shadcn::SelectItem::new("neutral/dark", "Neutral (dark)"),
                ])
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(220.0))))
                .into_element(cx);

            vec![left, theme_select]
        },
    );

    let preview_panel = page_preview(
        cx,
        theme,
        selected,
        popover_open,
        dialog_open,
        alert_dialog_open,
        sheet_open,
        select_value,
        select_open,
        combobox_value,
        combobox_open,
        combobox_query,
        date_picker_open,
        date_picker_month,
        date_picker_selected,
        resizable_h_fractions,
        resizable_v_fractions,
        data_table_state,
        data_grid_selected_row,
        tabs_value,
        accordion_value,
        progress,
        checkbox,
        switch,
        text_input,
        text_area,
        dropdown_open,
        context_menu_open,
        cmdk_open,
        cmdk_query,
        last_action,
    );
    let docs_panel = if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
        cx.text(docs_md)
    } else {
        markdown::Markdown::new(Arc::from(docs_md)).into_element(cx)
    };
    let usage_panel = if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
        cx.text(usage_md)
    } else {
        markdown::Markdown::new(Arc::from(usage_md)).into_element(cx)
    };

    let tabs = if (bisect & BISECT_DISABLE_TABS) != 0 {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N6),
            |_cx| vec![preview_panel, usage_panel, docs_panel],
        )
    } else {
        shadcn::Tabs::new(content_tab)
            .refine_layout(LayoutRefinement::default().w_full())
            .list_full_width(true)
            .items([
                shadcn::TabsItem::new("preview", "Preview", vec![preview_panel]),
                shadcn::TabsItem::new("usage", "Usage", vec![usage_panel]),
                shadcn::TabsItem::new("docs", "Notes", vec![docs_panel]),
            ])
            .into_element(cx)
    };

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N6),
        |_cx| vec![header, tabs],
    );
    let content = if (bisect & BISECT_DISABLE_CONTENT_SCROLL) != 0 {
        body
    } else {
        shadcn::ScrollArea::new(vec![body])
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
    };

    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("background")))
                .p(Space::N6),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| vec![content],
    )
}

pub(crate) fn page_meta(
    selected: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match selected {
        PAGE_LAYOUT => (
            "Layout / Stacks & Constraints",
            "fret-ui + fret-ui-kit",
            DOC_LAYOUT,
            USAGE_LAYOUT,
        ),
        PAGE_BUTTON => ("Button", "fret-ui-shadcn", DOC_BUTTON, USAGE_BUTTON),
        PAGE_FORMS => ("Forms", "fret-ui-shadcn", DOC_FORMS, USAGE_FORMS),
        PAGE_SELECT => ("Select", "fret-ui-shadcn", DOC_SELECT, USAGE_SELECT),
        PAGE_COMBOBOX => ("Combobox", "fret-ui-shadcn", DOC_COMBOBOX, USAGE_COMBOBOX),
        PAGE_DATE_PICKER => (
            "Date Picker",
            "fret-ui-shadcn",
            DOC_DATE_PICKER,
            USAGE_DATE_PICKER,
        ),
        PAGE_RESIZABLE => (
            "Resizable",
            "fret-ui-shadcn",
            DOC_RESIZABLE,
            USAGE_RESIZABLE,
        ),
        PAGE_DATA_TABLE => (
            "DataTable",
            "fret-ui-shadcn + fret-ui-headless",
            DOC_DATA_TABLE,
            USAGE_DATA_TABLE,
        ),
        PAGE_DATA_GRID => ("DataGrid", "fret-ui-shadcn", DOC_DATA_GRID, USAGE_DATA_GRID),
        PAGE_TABS => ("Tabs", "fret-ui-shadcn", DOC_TABS, USAGE_TABS),
        PAGE_ACCORDION => (
            "Accordion",
            "fret-ui-shadcn",
            DOC_ACCORDION,
            USAGE_ACCORDION,
        ),
        PAGE_TABLE => ("Table", "fret-ui-shadcn", DOC_TABLE, USAGE_TABLE),
        PAGE_PROGRESS => ("Progress", "fret-ui-shadcn", DOC_PROGRESS, USAGE_PROGRESS),
        PAGE_MENUS => ("Menus", "fret-ui-shadcn", DOC_MENUS, USAGE_MENUS),
        PAGE_COMMAND => (
            "Command Palette",
            "fret-ui-shadcn",
            DOC_COMMAND,
            USAGE_COMMAND,
        ),
        PAGE_TOAST => ("Toast", "fret-ui-shadcn", DOC_TOAST, USAGE_TOAST),
        PAGE_OVERLAY => (
            "Overlay / Popover & Dialog",
            "fret-ui-shadcn (Radix-shaped primitives)",
            DOC_OVERLAY,
            USAGE_OVERLAY,
        ),
        _ => ("Introduction", "Core contracts", DOC_INTRO, USAGE_INTRO),
    }
}

fn page_preview(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_BUTTON => preview_button(cx),
        PAGE_OVERLAY => {
            preview_overlay(cx, popover_open, dialog_open, alert_dialog_open, sheet_open)
        }
        PAGE_FORMS => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_SELECT => preview_select(cx, select_value, select_open),
        PAGE_COMBOBOX => preview_combobox(cx, combobox_value, combobox_open, combobox_query),
        PAGE_DATE_PICKER => preview_date_picker(
            cx,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
        ),
        PAGE_RESIZABLE => {
            preview_resizable(cx, theme, resizable_h_fractions, resizable_v_fractions)
        }
        PAGE_DATA_TABLE => preview_data_table(cx, data_table_state),
        PAGE_DATA_GRID => preview_data_grid(cx, data_grid_selected_row),
        PAGE_TABS => preview_tabs(cx, tabs_value),
        PAGE_ACCORDION => preview_accordion(cx, accordion_value),
        PAGE_TABLE => preview_table(cx),
        PAGE_PROGRESS => preview_progress(cx, progress),
        PAGE_MENUS => preview_menus(cx, dropdown_open, context_menu_open, last_action.clone()),
        PAGE_COMMAND => preview_command_palette(cx, cmdk_open, cmdk_query, last_action.clone()),
        PAGE_TOAST => preview_toast(cx, last_action.clone()),
        _ => preview_intro(cx, theme),
    };

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Preview").into_element(cx),
            shadcn::CardDescription::new("Interactive preview for validating behaviors.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(body).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn preview_intro(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(desc)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full(),
        );
        cx.container(props, |cx| {
            vec![cx.text("Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.")]
        })
    };

    vec![grid, note]
}

fn preview_layout(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                LayoutRefinement::default().w_full(),
            ),
            |cx| vec![cx.text(label)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        cx.text("Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing)."),
        row,
    ]
}

fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let variants = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default").into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .into_element(cx),
            ]
        },
    );

    let sizes = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::Button::new("Default")
                    .size(shadcn::ButtonSize::Default)
                    .into_element(cx),
                shadcn::Button::new("Large")
                    .size(shadcn::ButtonSize::Lg)
                    .into_element(cx),
            ]
        },
    );

    vec![variants, sizes]
}

fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let input = shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .into_element(cx);

    let toggles = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checkbox)
                                .a11y_label("Accept terms")
                                .into_element(cx),
                            cx.text("Accept terms"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(switch)
                                .a11y_label("Enable feature")
                                .into_element(cx),
                            cx.text("Enable feature"),
                        ]
                    },
                ),
            ]
        },
    );

    vec![
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |_cx| vec![input, textarea, toggles],
        ),
        cx.text(
            "Tip: these are model-bound controls; values persist while you stay in the window.",
        ),
    ]
}

fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .placeholder("Pick a fruit")
        .items([
            shadcn::SelectItem::new("apple", "Apple"),
            shadcn::SelectItem::new("banana", "Banana"),
            shadcn::SelectItem::new("orange", "Orange"),
        ])
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![select, cx.text(format!("Selected: {selected}"))]
}

fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let combo = shadcn::Combobox::new(value.clone(), open)
        .a11y_label("Combobox")
        .width(Px(240.0))
        .placeholder("Pick a fruit")
        .query_model(query.clone())
        .items([
            shadcn::ComboboxItem::new("apple", "Apple"),
            shadcn::ComboboxItem::new("banana", "Banana"),
            shadcn::ComboboxItem::new("orange", "Orange"),
            shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query = cx.app.models().get_cloned(&query).unwrap_or_default();

    vec![
        combo,
        cx.text(format!("Selected: {selected}")),
        cx.text(format!("Query: {query}")),
    ]
}

fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let picker = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .into_element(cx);

    let selected_text: Arc<str> = cx
        .app
        .models()
        .read(&selected, |v| v.map(|d| Arc::<str>::from(d.to_string())))
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![picker, cx.text(format!("Selected: {selected_text}"))]
}

fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, title: &str, color_key: &str| -> AnyElement {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required(color_key)))
                .rounded(Radius::Md)
                .p(Space::N3),
            LayoutRefinement::default().w_full().h_full(),
        );
        cx.container(props, move |cx| vec![cx.text(title)])
    };

    let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
        .axis(fret_core::Axis::Vertical)
        .entries(vec![
            shadcn::ResizablePanel::new(vec![boxy(cx, "Viewport", "muted")])
                .min_px(Px(120.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new(vec![boxy(cx, "Console", "card")])
                .min_px(Px(80.0))
                .into(),
        ])
        .into_element(cx);

    let root = shadcn::ResizablePanelGroup::new(h_fractions)
        .axis(fret_core::Axis::Horizontal)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(320.0))),
        )
        .entries(vec![
            shadcn::ResizablePanel::new(vec![boxy(cx, "Explorer", "accent")])
                .min_px(Px(140.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new(vec![nested_vertical])
                .min_px(Px(240.0))
                .into(),
        ])
        .into_element(cx);

    vec![cx.text("Drag the handles to resize panels."), root]
}

#[derive(Debug, Clone)]
struct DemoProcessRow {
    id: u64,
    name: Arc<str>,
    status: Arc<str>,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone)]
struct DemoProcessTableAssets {
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]>,
}

fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let assets = cx.with_state(
        || {
            let data: Arc<[DemoProcessRow]> = Arc::from(vec![
                DemoProcessRow {
                    id: 1,
                    name: Arc::from("Renderer"),
                    status: Arc::from("Running"),
                    cpu: 12,
                    mem_mb: 420,
                },
                DemoProcessRow {
                    id: 2,
                    name: Arc::from("Asset Cache"),
                    status: Arc::from("Idle"),
                    cpu: 0,
                    mem_mb: 128,
                },
                DemoProcessRow {
                    id: 3,
                    name: Arc::from("Indexer"),
                    status: Arc::from("Running"),
                    cpu: 38,
                    mem_mb: 860,
                },
                DemoProcessRow {
                    id: 4,
                    name: Arc::from("Spellcheck"),
                    status: Arc::from("Disabled"),
                    cpu: 0,
                    mem_mb: 0,
                },
                DemoProcessRow {
                    id: 5,
                    name: Arc::from("Language Server"),
                    status: Arc::from("Running"),
                    cpu: 7,
                    mem_mb: 512,
                },
            ]);

            let columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]> =
                Arc::from(vec![
                    fret_ui_headless::table::ColumnDef::new("name")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.mem_mb.cmp(&b.mem_mb))
                        .size(110.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let selected_count = cx
        .app
        .models()
        .read(&state, |st| st.row_selection.len())
        .ok()
        .unwrap_or(0);
    let sorting = cx
        .app
        .models()
        .read(&state, |st| {
            st.sorting.first().map(|s| (s.column.clone(), s.desc))
        })
        .ok()
        .flatten();

    let sorting_text: Arc<str> = sorting
        .map(|(col, desc)| {
            Arc::<str>::from(format!(
                "Sorting: {} {}",
                col,
                if desc { "desc" } else { "asc" }
            ))
        })
        .unwrap_or_else(|| Arc::<str>::from("Sorting: <none>"));

    let table = shadcn::DataTable::new()
        .row_height(Px(36.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(280.0))),
        )
        .into_element(
            cx,
            assets.data.clone(),
            1,
            state,
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            |cx, col, row| match col.id.as_ref() {
                "name" => cx.text(row.name.as_ref()),
                "status" => cx.text(row.status.as_ref()),
                "cpu%" => cx.text(format!("{}%", row.cpu)),
                "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                _ => cx.text("?"),
            },
        );

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        table,
    ]
}

fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx.app.models().get_cloned(&selected_row).flatten();

    let grid = shadcn::DataGridElement::new(["PID", "Name", "State", "CPU%"], 200)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(320.0))),
        )
        .into_element(
            cx,
            1,
            1,
            |row| row as u64,
            move |row| {
                let is_selected = selected == Some(row as u64);
                let cmd = CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"));
                shadcn::DataGridRowState {
                    selected: is_selected,
                    enabled: row % 17 != 0,
                    on_click: Some(cmd),
                }
            },
            |cx, row, col| {
                let pid = 1000 + row as u64;
                match col {
                    0 => cx.text(pid.to_string()),
                    1 => cx.text(format!("Process {row}")),
                    2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                    _ => cx.text(((row * 7) % 100).to_string()),
                }
            },
        );

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}

fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let selected = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let tabs = shadcn::Tabs::new(value)
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(false)
        .items([
            shadcn::TabsItem::new(
                "overview",
                "Overview",
                vec![cx.text("Tabs are stateful and roving-focus friendly.")],
            ),
            shadcn::TabsItem::new(
                "details",
                "Details",
                vec![cx.text("Use Tabs for sections within a single page.")],
            ),
            shadcn::TabsItem::new(
                "notes",
                "Notes",
                vec![cx.text(
                    "In this gallery, the outer shell also uses Tabs (Preview/Usage/Notes).",
                )],
            ),
        ])
        .into_element(cx);

    vec![tabs, cx.text(format!("Selected: {selected}"))]
}

fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let open = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let accordion = shadcn::Accordion::single(value)
        .collapsible(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 1")]),
                shadcn::AccordionContent::new(vec![cx.text("This section is collapsible.")]),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 2")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Keyboard navigation uses roving focus."),
                ]),
            ),
            shadcn::AccordionItem::new(
                "item-3",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 3")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Content lives in normal layout flow (no portal)."),
                ]),
            ),
        ])
        .into_element(cx);

    vec![accordion, cx.text(format!("Open item: {open}"))]
}

fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let header = shadcn::TableHeader::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableHead::new("Crate").into_element(cx),
                shadcn::TableHead::new("Layer").into_element(cx),
                shadcn::TableHead::new("Notes").into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx),
    ])
    .into_element(cx);

    let body = shadcn::TableBody::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui")).into_element(cx),
                shadcn::TableCell::new(cx.text("mechanisms")).into_element(cx),
                shadcn::TableCell::new(cx.text("Element tree + layout")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-kit")).into_element(cx),
                shadcn::TableCell::new(cx.text("policies")).into_element(cx),
                shadcn::TableCell::new(cx.text("Dismiss / focus / menu / overlays"))
                    .into_element(cx),
            ],
        )
        .selected(true)
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-shadcn")).into_element(cx),
                shadcn::TableCell::new(cx.text("recipes")).into_element(cx),
                shadcn::TableCell::new(cx.text("skinned components + defaults")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let caption = shadcn::TableCaption::new("Tip: TableRow has hover + selected styling parity.")
        .into_element(cx);

    let table = shadcn::Table::new(vec![header, body, caption])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    vec![table]
}

fn preview_progress(cx: &mut ElementContext<'_, App>, progress: Model<f32>) -> Vec<AnyElement> {
    let bar = shadcn::Progress::new(progress).into_element(cx);

    let controls = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("-10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_DEC)
                    .into_element(cx),
                shadcn::Button::new("+10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_INC)
                    .into_element(cx),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_click(CMD_PROGRESS_RESET)
                    .into_element(cx),
            ]
        },
    );

    vec![bar, controls]
}

fn preview_menus(
    cx: &mut ElementContext<'_, App>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dropdown-trigger")
                .toggle_model(dropdown_open.clone())
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-dropdown-item-apple")
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-context-trigger")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-context-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![dropdown, context_menu],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let cmdk = shadcn::CommandDialog::new_with_host_commands(cx, open.clone(), query)
        .a11y_label("Command palette")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Command Palette")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open)
                .into_element(cx)
        });

    vec![
        cx.text("Tip: Ctrl/Cmd+P triggers the command palette command."),
        cx.text(format!("last action: {last}")),
        cmdk,
    ]
}

fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default")
                    .on_click(CMD_TOAST_DEFAULT)
                    .into_element(cx),
                shadcn::Button::new("Success")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SUCCESS)
                    .into_element(cx),
                shadcn::Button::new("Error")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_ERROR)
                    .into_element(cx),
                shadcn::Button::new("Action + Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SHOW_ACTION_CANCEL)
                    .into_element(cx),
            ]
        },
    );

    vec![buttons, cx.text(format!("last action: {last}"))]
}

fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
) -> Vec<AnyElement> {
    let tooltip = shadcn::Tooltip::new(
        shadcn::Button::new("Tooltip (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
            cx,
            "Tooltip: hover intent + placement",
        )])
        .into_element(cx),
    )
    .arrow(true)
    .open_delay_frames(10)
    .close_delay_frames(10)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let hover_card = shadcn::HoverCard::new(
        shadcn::Button::new("HoverCard (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
        shadcn::HoverCardContent::new(vec![
            cx.text("HoverCard content (overlay-root)"),
            cx.text("Move pointer from trigger to content."),
        ])
        .into_element(cx),
    )
    .close_delay_frames(10)
    .into_element(cx);

    let popover = shadcn::Popover::new(popover_open.clone())
        .auto_focus(true)
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(popover_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new(vec![
                    cx.text("Popover content"),
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .toggle_model(popover_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx)
            },
        );

    let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-trigger")
                .toggle_model(dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog").into_element(cx),
                    shadcn::DialogDescription::new("Escape / overlay click closes")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .test_id("ui-gallery-dialog-close")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("AlertDialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(alert_dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::AlertDialogContent::new(vec![
                shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::AlertDialogDescription::new("This is non-closable by overlay click.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    let sheet = shadcn::Sheet::new(sheet_open.clone())
        .side(shadcn::SheetSide::Right)
        .size(Px(360.0))
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(sheet_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new(vec![
                    shadcn::SheetHeader::new(vec![
                        shadcn::SheetTitle::new("Sheet").into_element(cx),
                        shadcn::SheetDescription::new("A modal side panel.").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::SheetFooter::new(vec![
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .toggle_model(sheet_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
            },
        );

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![tooltip, hover_card],
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![popover, dialog],
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![alert_dialog, sheet],
        ),
    ]
}
