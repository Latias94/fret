pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{LayoutStyle, Length, TextProps};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

fn list_item(
    cx: &mut AppComponentCx<'_>,
    model: Model<Option<Arc<str>>>,
    title: &'static str,
    description: &'static str,
    test_id: &'static str,
    command: &'static str,
) -> shadcn::NavigationMenuLink {
    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
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
        layout: {
            let mut layout = LayoutStyle::default();
            layout.size.max_height = Some(Length::Px(Px(40.0)));
            layout
        },
        text: Arc::from(description),
        style: Some(TextStyle {
            font: FontId::default(),
            size: Px(14.0),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(Px(20.0)),
            letter_spacing_em: None,
            ..Default::default()
        }),
        color: Some(muted_foreground),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Ellipsis,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    });

    let body = ui::v_stack(move |_cx| [title_el, description_el])
        .gap(Space::N1)
        .items_start()
        .into_element(cx);

    shadcn::NavigationMenuLink::new(model, [body])
        .label(title)
        .test_id(test_id)
        .action(command)
}

fn icon_row(
    cx: &mut AppComponentCx<'_>,
    model: Model<Option<Arc<str>>>,
    icon: &'static str,
    label: &'static str,
    test_id: &'static str,
    command: &'static str,
) -> shadcn::NavigationMenuLink {
    let icon_el = icon::icon(cx, fret_icons::IconId::new_static(icon));
    let label_el = cx.text(label);
    let row = ui::h_row(move |_cx| [icon_el, label_el])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    shadcn::NavigationMenuLink::new(model, [row])
        .label(label)
        .test_id(test_id)
        .action(command)
}

fn featured_home_link(
    cx: &mut AppComponentCx<'_>,
    muted_background: Color,
    muted_foreground: Color,
    model: Model<Option<Arc<str>>>,
) -> shadcn::NavigationMenuLink {
    let title_el = cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::from("shadcn/ui"),
        style: Some(TextStyle {
            font: FontId::default(),
            size: Px(18.0),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(Px(28.0)),
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
        text: Arc::from("مكونات مصممة بعناية ومبنية باستخدام Tailwind CSS."),
        style: Some(TextStyle {
            font: FontId::default(),
            size: Px(14.0),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(Px(20.0)),
            letter_spacing_em: None,
            ..Default::default()
        }),
        color: Some(muted_foreground),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Ellipsis,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    });

    let body = ui::v_stack(move |_cx| [title_el, description_el])
        .gap(Space::N2)
        .items_start()
        .into_element(cx);

    shadcn::NavigationMenuLink::new(model, [body])
        .label("shadcn/ui")
        .test_id("ui-gallery-navigation-menu-rtl-link-home")
        .action(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .refine_style(
            ChromeRefinement::default()
                .bg(ColorRef::Color(muted_background))
                .rounded(Radius::Md)
                .p(Space::N4),
        )
}

fn text_link(
    cx: &mut AppComponentCx<'_>,
    model: Model<Option<Arc<str>>>,
    label: &'static str,
    test_id: &'static str,
    command: &'static str,
) -> shadcn::NavigationMenuLink {
    shadcn::NavigationMenuLink::new(model, [cx.text(label)])
        .label(label)
        .test_id(test_id)
        .action(command)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let muted_background = cx.with_theme(|theme| theme.color_token("muted"));
    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
    let rtl_value = cx.local_model(|| None::<Arc<str>>);

    let md_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_tailwind::MD,
        fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
    );
    let lg_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_tailwind::LG,
        fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
    );

    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
        let home_width = if lg_breakpoint {
            Px(500.0)
        } else if md_breakpoint {
            Px(400.0)
        } else {
            Px(360.0)
        };

        let home_content = if lg_breakpoint {
            ui::h_row(|cx| {
                let hero =
                    featured_home_link(cx, muted_background, muted_foreground, rtl_value.clone())
                        .refine_layout(LayoutRefinement::default().w_px(Px(188.0)).min_w_0())
                        .into_element(cx);
                let links = ui::v_stack(|cx| {
                    vec![
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "المقدمة",
                            "مكونات قابلة لإعادة الاستخدام مبنية باستخدام Radix UI و Tailwind CSS.",
                            "ui-gallery-navigation-menu-rtl-link-introduction",
                            CMD_APP_OPEN,
                        )
                        .into_element(cx),
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "التثبيت",
                            "كيفية تثبيت التبعيات وبنية تطبيقك.",
                            "ui-gallery-navigation-menu-rtl-link-installation",
                            CMD_APP_OPEN,
                        )
                        .into_element(cx),
                        list_item(
                            cx,
                            rtl_value.clone(),
                            "الطباعة",
                            "أنماط للعناوين والفقرات والقوائم...",
                            "ui-gallery-navigation-menu-rtl-link-typography",
                            CMD_APP_OPEN,
                        )
                        .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(288.0)).min_w_0())
                .into_element(cx);

                vec![hero, links]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(home_width).min_w_0())
            .into_element(cx)
        } else {
            ui::v_stack(|cx| {
                vec![
                    featured_home_link(cx, muted_background, muted_foreground, rtl_value.clone())
                        .into_element(cx),
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "المقدمة",
                        "مكونات قابلة لإعادة الاستخدام مبنية باستخدام Radix UI و Tailwind CSS.",
                        "ui-gallery-navigation-menu-rtl-link-introduction",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "التثبيت",
                        "كيفية تثبيت التبعيات وبنية تطبيقك.",
                        "ui-gallery-navigation-menu-rtl-link-installation",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "الطباعة",
                        "أنماط للعناوين والفقرات والقوائم...",
                        "ui-gallery-navigation-menu-rtl-link-typography",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(home_width).min_w_0())
            .into_element(cx)
        };

        let home = shadcn::NavigationMenuItem::new("home", "الرئيسية", [home_content])
            .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-home");

        let components_w_px = if lg_breakpoint {
            Px(600.0)
        } else if md_breakpoint {
            Px(500.0)
        } else {
            Px(400.0)
        };

        let components_specs = [
            (
                "حوار تنبيه",
                "حوار نمطي يقاطع المستخدم بمحتوى مهم ويتوقع استجابة.",
                "ui-gallery-navigation-menu-rtl-link-alert-dialog",
                CMD_APP_OPEN,
            ),
            (
                "بطاقة تمرير",
                "للمستخدمين المبصرين لمعاينة المحتوى المتاح خلف رابط.",
                "ui-gallery-navigation-menu-rtl-link-hover-card",
                CMD_APP_OPEN,
            ),
            (
                "التقدم",
                "يعرض مؤشرًا يوضح تقدم إتمام المهمة، عادةً يتم عرضه كشريط تقدم.",
                "ui-gallery-navigation-menu-rtl-link-progress",
                CMD_APP_OPEN,
            ),
            (
                "منطقة التمرير",
                "يفصل المحتوى بصريًا أو دلاليًا.",
                "ui-gallery-navigation-menu-rtl-link-scroll-area",
                CMD_APP_SAVE,
            ),
            (
                "التبويبات",
                "مجموعة من أقسام المحتوى المتعددة الطبقات التي يتم عرضها واحدة في كل مرة.",
                "ui-gallery-navigation-menu-rtl-link-tabs",
                CMD_APP_SAVE,
            ),
            (
                "تلميح",
                "نافذة منبثقة تعرض معلومات مرتبطة بعنصر عند التركيز أو التحويم.",
                "ui-gallery-navigation-menu-rtl-link-tooltip",
                CMD_APP_SAVE,
            ),
        ];

        let components_content = if md_breakpoint {
            let mut col_left = Vec::new();
            let mut col_right = Vec::new();
            for (idx, (title, desc, test_id, command)) in components_specs.iter().enumerate() {
                let el = list_item(cx, rtl_value.clone(), title, desc, test_id, command)
                    .into_element(cx);
                if idx % 2 == 0 {
                    col_left.push(el);
                } else {
                    col_right.push(el);
                }
            }

            ui::h_row(move |cx| {
                let left = ui::v_stack(move |_cx| col_left)
                    .gap(Space::N2)
                    .items_start()
                    .into_element(cx);
                let right = ui::v_stack(move |_cx| col_right)
                    .gap(Space::N2)
                    .items_start()
                    .into_element(cx);
                vec![left, right]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0())
            .into_element(cx)
        } else {
            let rtl_value_for_components = rtl_value.clone();
            ui::v_stack(move |cx| {
                components_specs
                    .into_iter()
                    .map(|(title, desc, test_id, command)| {
                        list_item(
                            cx,
                            rtl_value_for_components.clone(),
                            title,
                            desc,
                            test_id,
                            command,
                        )
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0())
            .into_element(cx)
        };

        let components =
            shadcn::NavigationMenuItem::new("components", "المكونات", [components_content])
                .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-components");

        let docs = shadcn::NavigationMenuItem::new("docs", "الوثائق", std::iter::empty())
            .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-docs")
            .action(CMD_APP_OPEN);

        let list = shadcn::NavigationMenuItem::new(
            "list",
            "القائمة",
            [ui::v_stack(|cx| {
                vec![
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "المكونات",
                        "استعرض كل المكونات في المكتبة.",
                        "ui-gallery-navigation-menu-rtl-link-list-components",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "الوثائق",
                        "تعلّم كيفية استخدام المكتبة.",
                        "ui-gallery-navigation-menu-rtl-link-list-documentation",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    list_item(
                        cx,
                        rtl_value.clone(),
                        "المدونة",
                        "اقرأ أحدث التدوينات.",
                        "ui-gallery-navigation-menu-rtl-link-list-blog",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(Px(300.0)).min_w_0())
            .into_element(cx)],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-list");

        let simple = shadcn::NavigationMenuItem::new(
            "simple",
            "بسيط",
            [ui::v_stack(|cx| {
                vec![
                    text_link(
                        cx,
                        rtl_value.clone(),
                        "المكونات",
                        "ui-gallery-navigation-menu-rtl-link-simple-components",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    text_link(
                        cx,
                        rtl_value.clone(),
                        "الوثائق",
                        "ui-gallery-navigation-menu-rtl-link-simple-documentation",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    text_link(
                        cx,
                        rtl_value.clone(),
                        "الكتل",
                        "ui-gallery-navigation-menu-rtl-link-simple-blocks",
                        CMD_APP_SAVE,
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0())
            .into_element(cx)],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-simple");

        let with_icon = shadcn::NavigationMenuItem::new(
            "with_icon",
            "مع أيقونة",
            [ui::v_stack(|cx| {
                vec![
                    icon_row(
                        cx,
                        rtl_value.clone(),
                        "lucide.circle-question-mark",
                        "قائمة الانتظار",
                        "ui-gallery-navigation-menu-rtl-link-backlog",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    icon_row(
                        cx,
                        rtl_value.clone(),
                        "lucide.circle",
                        "المهام",
                        "ui-gallery-navigation-menu-rtl-link-to-do",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                    icon_row(
                        cx,
                        rtl_value.clone(),
                        "lucide.circle-check",
                        "منجز",
                        "ui-gallery-navigation-menu-rtl-link-done",
                        CMD_APP_OPEN,
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N0)
            .items_start()
            .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0())
            .into_element(cx)],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-with-icon");

        let mut items = vec![home, components, docs];
        if md_breakpoint {
            items.push(list);
            items.push(simple);
            items.push(with_icon);
        }

        shadcn::navigation_menu(cx, rtl_value.clone(), |_cx| items)
            .viewport_test_id("ui-gallery-navigation-menu-rtl-viewport")
            .into_element(cx)
            .test_id("ui-gallery-navigation-menu-rtl")
    })
}
// endregion: example
