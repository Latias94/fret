pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_app::App;
use fret_core::{FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{LayoutStyle, Length, TextProps};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    #[derive(Default, Clone)]
    struct NavigationMenuModels {
        rtl_value: Option<Model<Option<Arc<str>>>>,
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
            layout: {
                let mut layout = LayoutStyle::default();
                // Upstream `line-clamp-2` outcome.
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
            .into_element(cx)
    };

    let icon_row = |cx: &mut ElementContext<'_, App>,
                    model: Model<Option<Arc<str>>>,
                    icon: &'static str,
                    label: &'static str,
                    test_id: &'static str,
                    command: &'static str| {
        let icon_el = fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon));
        let label_el = cx.text(label);
        let row = ui::h_row(move |_cx| [icon_el, label_el])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);
        shadcn::NavigationMenuLink::new(model, [row])
            .label(label)
            .test_id(test_id)
            .action(command)
            .into_element(cx)
    };

    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
        let getting_started = shadcn::NavigationMenuItem::new(
            "getting_started",
            "البدء",
            [ui::v_stack(|cx| {
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
                })
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(384.0)).min_w_0()).into_element(cx)],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-getting-started");

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
                "مجموعة من أقسام المحتوى المتعددة الطبقات—المعروفة بألواح التبويب—التي يتم عرضها واحدة في كل مرة.",
                "ui-gallery-navigation-menu-rtl-link-tabs",
                CMD_APP_SAVE,
            ),
            (
                "تلميح",
                "نافذة منبثقة تعرض معلومات متعلقة بعنصر عندما يتلقى العنصر التركيز على لوحة المفاتيح أو عند تحويم الماوس فوقه.",
                "ui-gallery-navigation-menu-rtl-link-tooltip",
                CMD_APP_SAVE,
            ),
        ];

        let components_content = if md_breakpoint {
            let mut col_left = Vec::new();
            let mut col_right = Vec::new();
            for (idx, (title, desc, test_id, command)) in components_specs.iter().enumerate() {
                let el = list_item(cx, rtl_value.clone(), title, desc, test_id, command);
                if idx % 2 == 0 {
                    col_left.push(el);
                } else {
                    col_right.push(el);
                }
            }

            ui::h_row(move |cx| {
                    let left = ui::v_stack(move |_cx| col_left).gap(Space::N2).items_start().into_element(cx);
                    let right = ui::v_stack(move |_cx| col_right).gap(Space::N2).items_start().into_element(cx);
                    vec![left, right]
                })
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0()).into_element(cx)
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
                        })
                        .collect::<Vec<_>>()
                })
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0()).into_element(cx)
        };

        let components =
            shadcn::NavigationMenuItem::new("components", "المكونات", [components_content])
                .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-components");

        let with_icon = shadcn::NavigationMenuItem::new(
            "with_icon",
            "مع أيقونة",
            [ui::v_stack(|cx| {
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
                })
                    .gap(Space::N0)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()).into_element(cx)],
        )
        .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-with-icon");

        let docs = shadcn::NavigationMenuItem::new("docs", "الوثائق", std::iter::empty())
            .trigger_test_id("ui-gallery-navigation-menu-rtl-trigger-docs")
            .action(CMD_APP_OPEN);

        shadcn::NavigationMenu::new(rtl_value.clone())
            .list(shadcn::NavigationMenuList::new(vec![
                getting_started,
                components,
                with_icon,
                docs,
            ]))
            .viewport_test_id("ui-gallery-navigation-menu-rtl-viewport")
            .into_element(cx)
            .test_id("ui-gallery-navigation-menu-rtl")
    })
}
// endregion: example
