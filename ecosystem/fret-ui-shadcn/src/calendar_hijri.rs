use std::sync::Arc;

use crate::optional_date_model::IntoOptionalDateModel;
use crate::solar_hijri_month_model::IntoSolarHijriMonthModel;
use fret_core::{Color, FontWeight, Px, TextAlign, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y, PressableProps,
    RovingFlexProps, RovingFocusProps, TextProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_headless::calendar_solar_hijri::{SolarHijriMonth, solar_hijri_month_grid_compact};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui,
};
use time::{Date, Weekday};

use crate::rtl;
use crate::surface_slot::{ShadcnSurfaceSlot, surface_slot_in_scope};

fn persian_digit(c: char) -> char {
    match c {
        '0' => '\u{06f0}',
        '1' => '\u{06f1}',
        '2' => '\u{06f2}',
        '3' => '\u{06f3}',
        '4' => '\u{06f4}',
        '5' => '\u{06f5}',
        '6' => '\u{06f6}',
        '7' => '\u{06f7}',
        '8' => '\u{06f8}',
        '9' => '\u{06f9}',
        _ => c,
    }
}

fn to_persian_digits(s: impl ToString) -> String {
    s.to_string().chars().map(persian_digit).collect()
}

fn solar_hijri_month_name(month: u8) -> &'static str {
    match month {
        1 => "فروردین",
        2 => "اردیبهشت",
        3 => "خرداد",
        4 => "تیر",
        5 => "مرداد",
        6 => "شهریور",
        7 => "مهر",
        8 => "آبان",
        9 => "آذر",
        10 => "دی",
        11 => "بهمن",
        12 => "اسفند",
        _ => "؟",
    }
}

fn solar_hijri_weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Saturday => "شنبه",
        Weekday::Sunday => "یک\u{200c}شنبه",
        Weekday::Monday => "دوشنبه",
        Weekday::Tuesday => "سه\u{200c}شنبه",
        Weekday::Wednesday => "چهارشنبه",
        Weekday::Thursday => "پنج\u{200c}شنبه",
        Weekday::Friday => "جمعه",
    }
}

fn solar_hijri_weekday_short(weekday: Weekday) -> Arc<str> {
    Arc::from(match weekday {
        Weekday::Saturday => "ش",
        Weekday::Sunday => "ی",
        Weekday::Monday => "د",
        Weekday::Tuesday => "س",
        Weekday::Wednesday => "چ",
        Weekday::Thursday => "پ",
        Weekday::Friday => "ج",
    })
}

fn solar_hijri_month_title(month: SolarHijriMonth) -> Arc<str> {
    Arc::from(format!(
        "{} {}",
        solar_hijri_month_name(month.month),
        to_persian_digits(month.year),
    ))
}

fn solar_hijri_day_aria_label(date: Date, selected: bool) -> Arc<str> {
    let selected_suffix = if selected { ", selected" } else { "" };
    let solar = fret_ui_headless::calendar_solar_hijri::solar_hijri_from_gregorian(date);
    let year = to_persian_digits(solar.year);
    let day = to_persian_digits(solar.day);
    Arc::from(format!(
        "{} {}-ام {} {}{selected_suffix}",
        solar_hijri_weekday_name(date.weekday()),
        day,
        solar_hijri_month_name(solar.month),
        year,
    ))
}

fn hijri_weekday_labels_visual_order(week_start: Weekday) -> Arc<[Arc<str>]> {
    let week = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    let start_idx = week.iter().position(|w| *w == week_start).unwrap_or(0);

    let mut labels = Vec::with_capacity(7);
    for i in 0..7 {
        let idx = (start_idx + i) % 7;
        labels.push(solar_hijri_weekday_short(week[idx]));
    }

    // Visual RTL order: left-to-right render should start with the last day of the week.
    labels.into_iter().rev().collect::<Vec<_>>().into()
}

fn calendar_nav_icon_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    button_size_px: Px,
    icon: fret_icons::IconId,
    enabled: bool,
    on_activate: impl Fn(&mut dyn fret_ui::action::UiActionHost) + 'static,
) -> AnyElement {
    let icon = decl_icon::icon_with(
        cx,
        icon,
        None,
        Some(ColorRef::Token {
            key: "foreground",
            fallback: ColorFallback::ThemeTextPrimary,
        }),
    );

    crate::button::Button::new(label)
        .variant(crate::button::ButtonVariant::Ghost)
        .size(crate::button::ButtonSize::IconSm)
        .children([icon])
        .disabled(!enabled)
        // Nav buttons use `size-(--cell-size)` and `p-0`.
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(button_size_px))
                .h_px(MetricRef::Px(button_size_px)),
        )
        .on_activate(Arc::new(move |host, _acx, _reason| on_activate(host)))
        .into_element(cx)
}

fn hijri_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    date: Date,
    in_month: bool,
    selected: bool,
    disabled: bool,
    size: Px,
    week_row_gap: Px,
    selected_model: &Model<Option<Date>>,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);

    let muted_fg = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let fg = if in_month {
        theme.color_token("foreground")
    } else {
        muted_fg
    };

    let (bg, fg) = if selected {
        (
            theme.color_token("primary"),
            theme.color_token("primary-foreground"),
        )
    } else {
        (Color::TRANSPARENT, fg)
    };

    let ring_color = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));

    let solar = fret_ui_headless::calendar_solar_hijri::solar_hijri_from_gregorian(date);
    let day_text: Arc<str> = Arc::from(to_persian_digits(solar.day));
    let date_label = solar_hijri_day_aria_label(date, selected);

    let text_sm_px = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let text_sm_line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
        let selected_model = selected_model.clone();

        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            if disabled {
                return;
            }
            let _ = host
                .models_mut()
                .update(&selected_model, |v| *v = Some(date));
        }));

        let hover_bg = theme.color_token("accent");
        let pressed_bg = {
            let mut c = hover_bg;
            c.a *= 0.85;
            c
        };

        let bg = if selected {
            bg
        } else if st.pressed {
            pressed_bg
        } else if st.hovered {
            hover_bg
        } else {
            Color::TRANSPARENT
        };

        let border = if st.focused {
            ring_color
        } else {
            Color::TRANSPARENT
        };

        let mut chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));
        if !in_month {
            chrome = chrome.text_color(ColorRef::Color(muted_fg));
        }

        let focus_ring = Some(decl_style::focus_ring(theme, Px(6.0)));

        let pressable = PressableProps {
            layout,
            enabled: !disabled,
            focusable: !disabled,
            focus_ring,
            a11y: PressableA11y {
                label: Some(Arc::clone(&date_label)),
                ..Default::default()
            },
            ..Default::default()
        };

        let chrome_props = decl_style::container_props(theme, chrome, LayoutRefinement::default());

        let children = move |cx: &mut ElementContext<'_, H>| {
            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: fret_ui::element::Length::Fill,
                            height: fret_ui::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: fret_core::Axis::Vertical,
                    gap: Px(0.0).into(),
                    padding: fret_core::Edges::all(Px(0.0)).into(),
                    justify: fret_ui::element::MainAlign::Center,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let mut props = TextProps::new(Arc::clone(&day_text));
                    let mut style = typography::fixed_line_box_style(
                        fret_core::FontId::ui(),
                        text_sm_px,
                        text_sm_line_height,
                    );
                    style.weight = FontWeight::NORMAL;
                    props.style = Some(style);
                    props.color = Some(fg);
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    props.align = TextAlign::Center;
                    vec![cx.text_props(props)]
                },
            )]
        };

        (pressable, chrome_props, children)
    })
}

#[derive(Clone)]
pub struct CalendarHijri {
    month: Model<SolarHijriMonth>,
    selected: Model<Option<Date>>,
    disable_navigation: bool,
    week_start: Weekday,
    show_outside_days: bool,
    cell_size: Option<Px>,
    test_id_prefix: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl CalendarHijri {
    pub fn new(month: impl IntoSolarHijriMonthModel, selected: impl IntoOptionalDateModel) -> Self {
        Self {
            month: month.into_solar_hijri_month_model(),
            selected: selected.into_optional_date_model(),
            disable_navigation: false,
            week_start: Weekday::Saturday,
            show_outside_days: true,
            cell_size: None,
            test_id_prefix: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disable_navigation(mut self, disable: bool) -> Self {
        self.disable_navigation = disable;
        self
    }

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    pub fn cell_size(mut self, size: Px) -> Self {
        self.cell_size = Some(size);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let month_model = self.month;
            let selected_model = self.selected;
            let month = cx
                .watch_model(&month_model)
                .copied()
                .unwrap_or(SolarHijriMonth::new(1400, 1));
            let selected = cx.watch_model(&selected_model).copied().flatten();

            let disable_navigation = self.disable_navigation;
            let week_start = self.week_start;
            let show_outside_days = self.show_outside_days;
            let test_id_prefix = self.test_id_prefix.clone();

            let title = solar_hijri_month_title(month);
            let weekday_labels = hijri_weekday_labels_visual_order(week_start);

            let weekday_text_px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_PX)
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let weekday_text_line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
                .unwrap_or_else(|| theme.metric_token("font.line_height"));

            let mut weekday_text_style = typography::fixed_line_box_style(
                fret_core::FontId::ui(),
                weekday_text_px,
                weekday_text_line_height,
            );
            weekday_text_style.weight = FontWeight::NORMAL;

            let day_size = self.cell_size.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.calendar.day_size")
                    .unwrap_or_else(|| theme.metric_token("component.size.sm.icon_button.size"))
            });
            let week_row_gap = theme
                .metric_by_key("component.calendar.week_row_gap")
                .unwrap_or_else(|| theme.metric_token("metric.padding.sm"));
            let day_col_gap = Px(0.0);
            let day_grid_width = Px(day_size.0 * 7.0);
            let month_width = day_grid_width;

            let bg = theme.color_token("background");
            let mut chrome = ChromeRefinement::default()
                .bg(ColorRef::Color(bg))
                .p(Space::N3);
            if matches!(
                surface_slot_in_scope(cx),
                Some(ShadcnSurfaceSlot::PopoverContent | ShadcnSurfaceSlot::CardContent)
            ) {
                chrome = chrome.bg(ColorRef::Color(Color::TRANSPARENT));
            }
            let chrome = chrome.merge(self.chrome);
            let root = LayoutRefinement::default().merge(self.layout);
            let container_props = decl_style::container_props(&theme, chrome, root);

            cx.container(container_props, move |cx| {
                let theme_header = theme.clone();
                let theme_weekdays = theme.clone();
                let theme_days = theme.clone();

                let header = ui::h_row(move |cx| {
                    let nav_enabled = !disable_navigation;
                    let direction = crate::direction::use_direction(cx, None);
                    let prev_icon = rtl::chevron_inline_start(direction);
                    let next_icon = rtl::chevron_inline_end(direction);

                    let month_model_prev = month_model.clone();
                    let prev = calendar_nav_icon_button(
                        cx,
                        "Go to the Previous Month",
                        day_size,
                        prev_icon,
                        nav_enabled,
                        move |host| {
                            if disable_navigation {
                                return;
                            }
                            let _ = host.models_mut().update(&month_model_prev, |m| {
                                *m = m.prev_month();
                            });
                        },
                    );
                    let prev = if let Some(prefix) = test_id_prefix.as_ref() {
                        prev.test_id(Arc::<str>::from(format!("{prefix}.nav-prev")))
                    } else {
                        prev
                    };
                    let month_model_next = month_model.clone();
                    let next = calendar_nav_icon_button(
                        cx,
                        "Go to the Next Month",
                        day_size,
                        next_icon,
                        nav_enabled,
                        move |host| {
                            if disable_navigation {
                                return;
                            }
                            let _ = host.models_mut().update(&month_model_next, |m| {
                                *m = m.next_month();
                            });
                        },
                    );
                    let next = if let Some(prefix) = test_id_prefix.as_ref() {
                        next.test_id(Arc::<str>::from(format!("{prefix}.nav-next")))
                    } else {
                        next
                    };

                    let mut title_props = TextProps::new(title.clone());
                    let size = theme_header.metric_token("font.size");
                    let line_height = theme_header.metric_token("font.line_height");
                    let mut style = typography::fixed_line_box_style(
                        fret_core::FontId::ui(),
                        size,
                        line_height,
                    );
                    style.weight = FontWeight::MEDIUM;
                    title_props.style = Some(style);
                    title_props.wrap = TextWrap::None;
                    title_props.overflow = TextOverflow::Clip;
                    title_props.align = TextAlign::Center;
                    // Keep the caption centered between nav buttons.
                    title_props.layout.flex.grow = 1.0;
                    title_props.layout.flex.shrink = 1.0;
                    title_props.layout.flex.basis = Length::Px(Px(0.0));
                    let title_el = cx.text_props(title_props);

                    let (prev, next) = crate::rtl::inline_start_end_pair(direction, prev, next);
                    vec![prev, title_el, next]
                })
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(MetricRef::Px(month_width)))
                .items_center()
                .justify_start()
                .into_element(cx);

                let weekday_row = cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(month_width),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: day_col_gap.into(),
                        padding: fret_core::Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        weekday_labels
                            .iter()
                            .map(|label| {
                                let mut props = TextProps::new(Arc::clone(label));
                                props.style = Some(weekday_text_style.clone());
                                props.color = theme_weekdays.color_by_key("muted-foreground");
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                props.align = TextAlign::Center;

                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(day_size);
                                layout.size.height = Length::Auto;
                                props.layout = layout;
                                cx.text_props(props)
                            })
                            .collect::<Vec<_>>()
                    },
                );

                let grid = solar_hijri_month_grid_compact(month, week_start);
                let disabled_flags: Arc<[bool]> = vec![false; grid.len()].into();

                let roving_props = RovingFlexProps {
                    flex: FlexProps {
                        layout: LayoutStyle {
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(day_grid_width),
                                ..Default::default()
                            },
                            overflow: Overflow::Visible,
                            ..Default::default()
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: day_col_gap.into(),
                        padding: fret_core::Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Start,
                        wrap: true,
                    },
                    roving: RovingFocusProps {
                        enabled: true,
                        wrap: false,
                        disabled: Arc::clone(&disabled_flags),
                    },
                };

                let days_grid = cx.roving_flex(roving_props, move |cx| {
                    grid.chunks(7)
                        .flat_map(|week| week.iter().rev())
                        .map(|day| {
                            let is_hidden = !day.in_month && !show_outside_days;
                            if is_hidden {
                                return calendar_hidden_cell(
                                    cx,
                                    &theme_days,
                                    day_size,
                                    week_row_gap,
                                );
                            }

                            let is_selected = selected.is_some_and(|d| d == day.date);
                            hijri_day_cell(
                                cx,
                                &theme_days,
                                day.date,
                                day.in_month,
                                is_selected,
                                false,
                                day_size,
                                week_row_gap,
                                &selected_model,
                            )
                        })
                        .collect::<Vec<_>>()
                });

                let body = ui::v_stack(move |_cx| vec![weekday_row, days_grid])
                    .gap(Space::N2)
                    .into_element(cx);

                vec![
                    ui::v_stack(move |_cx| vec![header, body])
                        .gap(Space::N4)
                        .into_element(cx),
                ]
            })
        })
    }
}

fn calendar_hidden_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    size: Px,
    week_row_gap: Px,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);

    control_chrome_pressable_with_id_props(cx, move |_cx, _st, _id| {
        let mut chrome_props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        // Keep margins on the pressable node so row gaps don't inflate the chrome/background quad.
        chrome_props.layout.margin = Default::default();

        let pressable = PressableProps {
            layout,
            enabled: false,
            focusable: false,
            focus_ring: None,
            a11y: PressableA11y {
                hidden: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let children = move |_cx: &mut ElementContext<'_, H>| Vec::<AnyElement>::new();
        (pressable, chrome_props, children)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekday_short_uses_single_letter_abbreviations() {
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Saturday), "ش");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Sunday), "ی");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Monday), "د");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Tuesday), "س");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Wednesday), "چ");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Thursday), "پ");
        assert_eq!(&*solar_hijri_weekday_short(Weekday::Friday), "ج");
    }

    #[test]
    fn weekday_labels_match_rtl_visual_order_for_saturday_start() {
        let labels = hijri_weekday_labels_visual_order(Weekday::Saturday);
        let as_str = labels.iter().map(|s| &**s).collect::<Vec<_>>();
        // Visual RTL order: left-to-right render begins with the last weekday (Friday).
        assert_eq!(as_str, vec!["ج", "پ", "چ", "س", "د", "ی", "ش"]);
    }
}
