#![cfg(feature = "web-goldens")]
// Heavy, web-golden-backed conformance. Enable via:
//   cargo nextest run -p fret-ui-shadcn --features web-goldens

use fret_app::App;
use fret_core::{
    AppWindowId, Color, Event, FrameId, KeyCode, Modifiers, MouseButton, MouseButtons, Paint,
    Point, PointerEvent, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
    Transform2D,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{GlobalElementId, bounds_for_element, with_element_cx};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

mod css_color;
use css_color::{color_to_rgba, parse_css_color};

#[path = "web_vs_fret_overlay_chrome/web.rs"]
mod web;
use web::*;

#[path = "web_vs_fret_overlay_chrome/support.rs"]
mod support;
use support::*;

#[path = "web_vs_fret_overlay_chrome/alert_dialog.rs"]
mod alert_dialog;
#[path = "web_vs_fret_overlay_chrome/button_group.rs"]
mod button_group;
#[path = "web_vs_fret_overlay_chrome/calendar.rs"]
mod calendar;
#[path = "web_vs_fret_overlay_chrome/combobox.rs"]
mod combobox;
#[path = "web_vs_fret_overlay_chrome/command_dialog.rs"]
mod command_dialog;
#[path = "web_vs_fret_overlay_chrome/context_menu.rs"]
mod context_menu;
#[path = "web_vs_fret_overlay_chrome/date_picker.rs"]
mod date_picker;
#[path = "web_vs_fret_overlay_chrome/dialog.rs"]
mod dialog;
#[path = "web_vs_fret_overlay_chrome/drawer.rs"]
mod drawer;
#[path = "web_vs_fret_overlay_chrome/dropdown_menu.rs"]
mod dropdown_menu;
#[path = "web_vs_fret_overlay_chrome/hover_card.rs"]
mod hover_card;
#[path = "web_vs_fret_overlay_chrome/menubar.rs"]
mod menubar;
#[path = "web_vs_fret_overlay_chrome/navigation_menu.rs"]
mod navigation_menu;
#[path = "web_vs_fret_overlay_chrome/popover.rs"]
mod popover;
#[path = "web_vs_fret_overlay_chrome/select.rs"]
mod select;
#[path = "web_vs_fret_overlay_chrome/sheet.rs"]
mod sheet;
#[path = "web_vs_fret_overlay_chrome/tooltip.rs"]
mod tooltip;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

fn assert_command_dialog_focused_item_chrome_matches_web(web_theme_name: &str) {
    assert_command_dialog_focused_item_chrome_matches_web_named(
        "command-dialog.focus-first",
        web_theme_name,
    );
}

fn assert_command_dialog_focused_item_chrome_matches_web_named(
    web_name: &str,
    web_theme_name: &str,
) {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, "command-item");

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
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

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| {
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

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    if let Some(text_field) = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::TextField)
        .max_by(|a, b| rect_area(a.bounds).total_cmp(&rect_area(b.bounds)))
    {
        ui.set_focus(Some(text_field.id));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames),
            true,
            |cx| {
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

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        true,
        |cx| {
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

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let option = fret_find_active_listbox_option(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        let active_owner_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.active_descendant.is_some())
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected active listbox option\n  focused_roles={focused_roles:?}\n  active_descendant_owner_roles={active_owner_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_popover_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    Popover::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open popover")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let sm_px = theme.metric_required("font.size");
            let sm_line_height = theme.metric_required("font.line_height");
            let muted_fg = theme.color_required("muted.foreground");

            // popover-demo uses `h4.leading-none.font-medium` (line height = 16px).
            let title = ui::text(cx, "Dimensions")
                .text_size_px(sm_px)
                .line_height_px(Px(16.0))
                .font_medium()
                .nowrap()
                .into_element(cx);
            // popover-demo uses `p.text-sm.text-muted-foreground` (line height = 20px).
            let description = ui::text(cx, "Set the dimensions for the layer.")
                .text_size_px(sm_px)
                .line_height_px(sm_line_height)
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx);
            let header = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| vec![title, description],
            );

            fn labeled_input_row<H: fret_ui::UiHost>(
                cx: &mut ElementContext<'_, H>,
                label: &str,
                value: &str,
            ) -> AnyElement {
                use fret_core::Px;
                use fret_ui_kit::declarative::stack;
                use fret_ui_kit::{LayoutRefinement, Space};
                use fret_ui_shadcn::{Input, Label};

                let label_el = Label::new(label).into_element(cx);
                let model = cx.app.models_mut().insert(value.to_string());
                let input_el = Input::new(model)
                    .a11y_label(label)
                    .refine_layout(LayoutRefinement::default().h_px(Px(32.0)).flex_grow(1.0))
                    .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    move |_cx| vec![label_el, input_el],
                )
            }

            let rows = vec![
                labeled_input_row(cx, "Width", "100%"),
                labeled_input_row(cx, "Max. width", "300px"),
                labeled_input_row(cx, "Height", "25px"),
                labeled_input_row(cx, "Max. height", "none"),
            ];
            let fields = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| rows,
            );

            PopoverContent::new([header, fields])
                .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                .into_element(cx)
        },
    )
}

fn build_shadcn_calendar_22_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(192.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date of birth").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_shadcn_calendar_23_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_shadcn_select_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("Fruits").into(),
            SelectItem::new("apple", "Apple").into(),
            SelectItem::new("banana", "Banana").into(),
            SelectItem::new("blueberry", "Blueberry").into(),
            SelectItem::new("grapes", "Grapes").into(),
            SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a fruit")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(180.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("North America").into(),
            SelectItem::new("est", "Eastern Standard Time (EST)").into(),
            SelectItem::new("cst", "Central Standard Time (CST)").into(),
            SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
            SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
            SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
            SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Europe & Africa").into(),
            SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
            SelectItem::new("cet", "Central European Time (CET)").into(),
            SelectItem::new("eet", "Eastern European Time (EET)").into(),
            SelectItem::new("west", "Western European Summer Time (WEST)").into(),
            SelectItem::new("cat", "Central Africa Time (CAT)").into(),
            SelectItem::new("eat", "East Africa Time (EAT)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Asia").into(),
            SelectItem::new("msk", "Moscow Time (MSK)").into(),
            SelectItem::new("ist", "India Standard Time (IST)").into(),
            SelectItem::new("cst_china", "China Standard Time (CST)").into(),
            SelectItem::new("jst", "Japan Standard Time (JST)").into(),
            SelectItem::new("kst", "Korea Standard Time (KST)").into(),
            SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Australia & Pacific").into(),
            SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
            SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
            SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
            SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
            SelectItem::new("fjt", "Fiji Time (FJT)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("South America").into(),
            SelectItem::new("art", "Argentina Time (ART)").into(),
            SelectItem::new("bot", "Bolivia Time (BOT)").into(),
            SelectItem::new("brt", "Brasilia Time (BRT)").into(),
            SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a timezone")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    build_shadcn_select_scrollable_page(cx, open)
}

fn build_shadcn_combobox_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Combobox, ComboboxItem};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let items = vec![
        ComboboxItem::new("apple", "Apple"),
        ComboboxItem::new("banana", "Banana"),
        ComboboxItem::new("blueberry", "Blueberry"),
        ComboboxItem::new("grapes", "Grapes"),
        ComboboxItem::new("pineapple", "Pineapple"),
    ];

    Combobox::new(value, open.clone())
        .a11y_label("Select a fruit")
        .width(Px(200.0))
        .items(items)
        .into_element(cx)
}

fn build_shadcn_command_dialog_page(
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

fn build_shadcn_hover_card_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{
        Avatar, AvatarFallback, AvatarImage, Button, ButtonVariant, HoverCard, HoverCardContent,
    };

    let theme = Theme::global(&*cx.app).clone();
    let sm_px = theme.metric_required("font.size");
    let sm_line_height = theme.metric_required("font.line_height");
    let xs_px = theme
        .metric_by_key("component.tooltip.text_px")
        .unwrap_or(Px((sm_px.0 - 2.0).max(10.0)));
    let xs_line_height = theme
        .metric_by_key("component.tooltip.line_height")
        .unwrap_or(Px((sm_line_height.0 - 4.0).max(12.0)));
    let muted_fg = theme.color_required("muted.foreground");

    let trigger_el = Button::new("@nextjs")
        .variant(ButtonVariant::Link)
        .into_element(cx);

    let avatar = Avatar::new([
        AvatarImage::maybe(None).into_element(cx),
        AvatarFallback::new("VC").into_element(cx),
    ])
    .into_element(cx);

    let heading = ui::text(cx, "@nextjs")
        .text_size_px(sm_px)
        .line_height_px(sm_line_height)
        .font_semibold()
        .into_element(cx);
    let body = ui::text(
        cx,
        "The React Framework – created and maintained by @vercel.",
    )
    .text_size_px(sm_px)
    .line_height_px(sm_line_height)
    .into_element(cx);
    let joined = ui::text(cx, "Joined December 2021")
        .text_size_px(xs_px)
        .line_height_px(xs_line_height)
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx);

    let text_block = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .layout(LayoutRefinement::default().w_px(Px(238.0))),
        move |_cx| vec![heading, body, joined],
    );

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| vec![avatar, text_block],
    );

    let content_el = HoverCardContent::new([row])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);

    HoverCard::new(trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}

fn build_shadcn_menubar_demo(cx: &mut ElementContext<'_, App>) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    #[derive(Default)]
    struct Models {
        view_bookmarks_bar: Option<Model<bool>>,
        view_full_urls: Option<Model<bool>>,
        profile_value: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| {
        match (
            st.view_bookmarks_bar.as_ref(),
            st.view_full_urls.as_ref(),
            st.profile_value.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
            _ => None,
        }
    });

    let (view_bookmarks_bar, view_full_urls, profile_value) = if let Some(existing) = existing {
        existing
    } else {
        let view_bookmarks_bar = cx.app.models_mut().insert(false);
        let view_full_urls = cx.app.models_mut().insert(true);
        let profile_value = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.view_bookmarks_bar = Some(view_bookmarks_bar.clone());
            st.view_full_urls = Some(view_full_urls.clone());
            st.profile_value = Some(profile_value.clone());
        });

        (view_bookmarks_bar, view_full_urls, profile_value)
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab")
                    .test_id("menubar.file.new_tab")
                    .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(
                MenubarItem::new("Share")
                    .test_id("menubar.file.share")
                    .submenu(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ]),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]),
        MenubarMenu::new("Edit").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Search the web")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]),
        MenubarMenu::new("View").entries(vec![
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar,
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls,
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Toggle Fullscreen").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Hide Sidebar").inset(true)),
        ]),
        MenubarMenu::new("Profiles").entries(vec![
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile_value)
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("Luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]),
    ])
    .into_element(cx)
}

fn render_shadcn_menubar_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_menubar_demo(cx)],
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    ui.set_focus(Some(file_trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 201);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .unwrap_or_else(|| {
            let focused_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.flags.focused)
                .filter_map(|n| n.label.as_deref())
                .collect();
            let menu_item_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::MenuItem)
                .filter_map(|n| n.label.as_deref())
                .collect();
            panic!(
                "{web_name} {web_theme_name}: expected menubar menu item 'New Tab'\n  focused_labels={focused_labels:?}\n  menu_item_labels={menu_item_labels:?}",
            )
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: focused menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_menubar_file_menu_destructive(
    cx: &mut ElementContext<'_, App>,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> AnyElement {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu, MenubarShortcut};

    let mut new_tab =
        MenubarItem::new("New Tab").trailing(MenubarShortcut::new("⌘T").into_element(cx));
    if new_tab_destructive {
        new_tab = new_tab.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    let mut new_window =
        MenubarItem::new("New Window").trailing(MenubarShortcut::new("⌘N").into_element(cx));
    if new_window_destructive {
        new_window = new_window.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    Menubar::new(vec![MenubarMenu::new("File").entries(vec![
        MenubarEntry::Item(new_tab),
        MenubarEntry::Item(new_window),
        MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
        MenubarEntry::Separator,
        MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
            MenubarEntry::Item(MenubarItem::new("Email link")),
            MenubarEntry::Item(MenubarItem::new("Messages")),
            MenubarEntry::Item(MenubarItem::new("Notes")),
        ])),
        MenubarEntry::Separator,
        MenubarEntry::Item(
            MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
        ),
    ])])
    .into_element(cx)
}

fn render_shadcn_menubar_file_menu_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_menubar_file_menu_destructive(
                    cx,
                    new_tab_destructive,
                    new_window_destructive,
                )]
            },
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-idle");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "menubar-item",
        "destructive",
        "New Window ⌘N",
    );
    assert!(
        expected.bg.a < 0.02,
        "menubar-demo.destructive-idle {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, false, true)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        false,
        true,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        false,
        true,
    );

    let new_window = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Window"))
        .expect("menubar New Window item semantics node");
    assert!(
        !new_window.flags.focused,
        "menubar-demo.destructive-idle {web_theme_name}: expected New Window to be idle (not focused)"
    );

    let text = find_best_text_color_near(
        &scene,
        new_window.bounds,
        leftish_text_probe_point(new_window.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-idle {web_theme_name}: destructive idle menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-idle {web_theme_name} destructive idle menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-focus-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, true, false)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        true,
        false,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        true,
        false,
    );

    let focused_fallback = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(focused_fallback);
    assert!(
        focused.role == SemanticsRole::MenuItem && focused.label.as_deref() == Some("New Tab"),
        "menubar-demo.destructive-focus-first {web_theme_name}: expected focused menu item to be New Tab, got role={:?} label={:?}",
        focused.role,
        focused.label
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");

    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(new_tab.bounds),
    );

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_submenu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(375.0), Px(240.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let share = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("menubar submenu trigger (Share) semantics node");
    ui.set_focus(Some(share.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let (snap3, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);
    let email_link = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node");
    ui.set_focus(Some(email_link.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(350),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let email_link = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node after focus");

    let quad = find_best_solid_quad_within_matching_bg(&scene, email_link.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted submenu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        email_link.bounds,
        leftish_text_probe_point(email_link.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted submenu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_navigation_menu_trigger_surface_colors_match(
    web_name: &str,
    open_label: &str,
    open_value: &str,
    closed_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_open_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "open",
        open_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web open trigger: slot=navigation-menu-trigger state=open text={open_label:?}"
        )
    });
    let web_closed_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "closed",
        closed_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web closed trigger: slot=navigation-menu-trigger state=closed text={closed_label:?}"
        )
    });

    let web_open_bg = web_open_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_open_text = web_open_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let web_closed_bg = web_closed_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_closed_text = web_closed_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![
                        NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                        NavigationMenuItem::new(
                            "components",
                            "Components",
                            vec![cx.text("Components content")],
                        ),
                        NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                    ])
                    .into_element(cx),
            ]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| panic!("missing fret trigger semantics node: Button {open_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(open_trigger.bounds),
    );

    let _ = app
        .models_mut()
        .update(&model, |v| *v = Some(Arc::from(open_value)));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                vec![
                    NavigationMenu::new(model.clone())
                        .viewport(false)
                        .indicator(false)
                        .items(vec![
                            NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                            NavigationMenuItem::new(
                                "components",
                                "Components",
                                vec![cx.text("Components content")],
                            ),
                            NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                        ])
                        .into_element(cx),
                ]
            },
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {open_label:?}")
        });
    let closed_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(closed_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {closed_label:?}")
        });

    let open_quad = find_best_chrome_quad(&scene, open_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (open)");
    let closed_quad = find_best_chrome_quad(&scene, closed_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (closed)");

    if let Some(web_open_bg) = web_open_bg
        && web_open_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(open_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.r"),
            fret_bg.r,
            web_open_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.g"),
            fret_bg.g,
            web_open_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.b"),
            fret_bg.b,
            web_open_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.a"),
            fret_bg.a,
            web_open_bg.a,
            0.02,
        );
    }

    if let Some(web_closed_bg) = web_closed_bg
        && web_closed_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(closed_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.r"),
            fret_bg.r,
            web_closed_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.g"),
            fret_bg.g,
            web_closed_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.b"),
            fret_bg.b,
            web_closed_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.a"),
            fret_bg.a,
            web_closed_bg.a,
            0.02,
        );
    }

    if let Some(web_open_text) = web_open_text
        && web_open_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            open_trigger.bounds,
            bounds_center(open_trigger.bounds),
        )
        .expect("open trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.r"),
            text.r,
            web_open_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.g"),
            text.g,
            web_open_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.b"),
            text.b,
            web_open_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.a"),
            text.a,
            web_open_text.a,
            0.05,
        );
    }

    if let Some(web_closed_text) = web_closed_text
        && web_closed_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            closed_trigger.bounds,
            bounds_center(closed_trigger.bounds),
        )
        .expect("closed trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.r"),
            text.r,
            web_closed_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.g"),
            text.g,
            web_closed_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.b"),
            text.b,
            web_closed_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.a"),
            text.a,
            web_closed_text.a,
            0.05,
        );
    }
}
