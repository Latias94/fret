pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use crate::spec::{CMD_APP_OPEN, CMD_APP_SAVE};
use crate::ui::doc_layout;
use fret_app::App;
use fret_core::{FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::TextProps;
use fret_ui_kit::declarative::{ElementContextThemeExt as _, ModelWatchExt as _};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    #[derive(Default, Clone)]
    struct NavigationMenuModels {
        rtl_value: Option<Model<Option<Arc<str>>>>,
        md_breakpoint_query_uses_container: Option<Model<bool>>,
    }

    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));

    let state = cx.with_state(NavigationMenuModels::default, |st| st.clone());
    let rtl_value = match state.rtl_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(NavigationMenuModels::default, |st| {
                st.rtl_value = Some(model.clone())
            });
            model
        }
    };

    let md_breakpoint_query_uses_container = match state.md_breakpoint_query_uses_container {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(NavigationMenuModels::default, |st| {
                st.md_breakpoint_query_uses_container = Some(model.clone())
            });
            model
        }
    };

    let md_breakpoint_query = if cx
        .watch_model(&md_breakpoint_query_uses_container)
        .cloned()
        .unwrap_or(false)
    {
        shadcn::navigation_menu::NavigationMenuMdBreakpointQuery::Container
    } else {
        shadcn::navigation_menu::NavigationMenuMdBreakpointQuery::Viewport
    };

    let list_item = |cx: &mut ElementContext<'_, App>,
                     model: Model<Option<Arc<str>>>,
                     title: &'static str,
                     description: &'static str,
                     test_id: &'static str,
                     command: &'static str| {
        let title_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(title),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: None,
                letter_spacing_em: None,
                ..Default::default()
            }),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });
        let description_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(description),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: None,
                letter_spacing_em: None,
                ..Default::default()
            }),
            color: Some(muted_foreground),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            move |_cx| [title_el, description_el],
        );

        shadcn::NavigationMenuLink::new(model, [body])
            .label(title)
            .test_id(test_id)
            .on_click(command)
            .into_element(cx)
    };

    let icon_row = |cx: &mut ElementContext<'_, App>,
                    model: Model<Option<Arc<str>>>,
                    icon: &'static str,
                    label: &'static str,
                    test_id: &'static str,
                    command: &'static str| {
        let icon_el = shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon));
        let label_el = cx.text(label);
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |_cx| [icon_el, label_el],
        );
        shadcn::NavigationMenuLink::new(model, [row])
            .label(label)
            .test_id(test_id)
            .on_click(command)
            .into_element(cx)
    };

    doc_layout::rtl(cx, |cx| {
        let region_props = fret_ui::element::LayoutQueryRegionProps {
            layout: cx.with_theme(|theme| {
                fret_ui_kit::declarative::style::layout_style(
                    theme,
                    LayoutRefinement::default().w_px(Px(640.0)).min_w_0(),
                )
            }),
            name: None,
        };

        let getting_started = shadcn::NavigationMenuItem::new(
            "getting_started",
            "البدء",
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(384.0)).min_w_0()),
                |cx| {
                    vec![
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "المقدمة",
                            "مكونات قابلة لإعادة الاستخدام مبنية باستخدام Tailwind CSS.",
                            "ui-gallery-navigation-menu-rtl-link-introduction",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "التثبيت",
                            "كيفية تثبيت التبعيات وبنية تطبيقك.",
                            "ui-gallery-navigation-menu-rtl-link-installation",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "الطباعة",
                            "أنماط للعناوين والفقرات والقوائم...",
                            "ui-gallery-navigation-menu-rtl-link-typography",
                            CMD_APP_OPEN,
                        ),
                    ]
                },
            )],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-getting-started");

        let components = shadcn::NavigationMenuItem::new(
            "components",
            "المكونات",
            [stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(600.0)).min_w_0()),
                |cx| {
                    let left = stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "حوار تنبيه",
                                    "حوار نمطي يقاطع المستخدم بمحتوى مهم ويتوقع استجابة.",
                                    "ui-gallery-navigation-menu-rtl-link-alert-dialog",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "بطاقة تمرير",
                                    "للمستخدمين المبصرين لمعاينة المحتوى المتاح خلف رابط.",
                                    "ui-gallery-navigation-menu-rtl-link-hover-card",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "التقدم",
                                    "يعرض مؤشرًا يوضح تقدم إتمام المهمة، عادةً يتم عرضه كشريط تقدم.",
                                    "ui-gallery-navigation-menu-rtl-link-progress",
                                    CMD_APP_OPEN,
                                ),
                            ]
                        },
                    );

                    let right = stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "منطقة التمرير",
                                    "يفصل المحتوى بصريًا أو دلاليًا.",
                                    "ui-gallery-navigation-menu-rtl-link-scroll-area",
                                    CMD_APP_SAVE,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "التبويبات",
                                    "مجموعة من أقسام المحتوى المتعددة الطبقات—المعروفة بألواح التبويب—التي يتم عرضها واحدة في كل مرة.",
                                    "ui-gallery-navigation-menu-rtl-link-tabs",
                                    CMD_APP_SAVE,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "تلميح",
                                    "نافذة منبثقة تعرض معلومات متعلقة بعنصر عندما يتلقى العنصر التركيز على لوحة المفاتيح أو عند تحويم الماوس فوقه.",
                                    "ui-gallery-navigation-menu-rtl-link-tooltip",
                                    CMD_APP_SAVE,
                                ),
                            ]
                        },
                    );

                    [left, right]
                },
            )],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-components");

        let with_icon = shadcn::NavigationMenuItem::new(
            "with_icon",
            "مع أيقونة",
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()),
                |cx| {
                    vec![
                        icon_row(
                            cx,
                            rtl_value.clone(),
                            "lucide.circle-alert",
                            "قائمة الانتظار",
                            "ui-gallery-navigation-menu-rtl-link-backlog",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            rtl_value.clone(),
                            "lucide.circle-dashed",
                            "المهام",
                            "ui-gallery-navigation-menu-rtl-link-to-do",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            rtl_value.clone(),
                            "lucide.circle-check",
                            "منجز",
                            "ui-gallery-navigation-menu-rtl-link-done",
                            CMD_APP_OPEN,
                        ),
                    ]
                },
            )],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-with-icon");

        let docs = shadcn::NavigationMenuItem::new("docs", "الوثائق", std::iter::empty())
            .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-docs");

        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "ui-gallery.navigation_menu.rtl",
            region_props,
            move |cx, region_id| {
                vec![
                    shadcn::NavigationMenu::new(rtl_value.clone())
                        .md_breakpoint_query(md_breakpoint_query)
                        .container_query_region(region_id)
                        .list(shadcn::NavigationMenuList::new([
                            getting_started,
                            components,
                            with_icon,
                            docs,
                        ]))
                        .viewport_test_id("ui-gallery-navigation-menu-rtl-viewport")
                        .into_element(cx),
                ]
            },
        )
    })
}
// endregion: example
