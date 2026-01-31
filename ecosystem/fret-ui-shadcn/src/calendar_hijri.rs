use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y, PressableProps,
    RovingFlexProps, RovingFocusProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::calendar_solar_hijri::{SolarHijriMonth, solar_hijri_month_grid_compact};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};
use time::{Date, Weekday};

use crate::button::{ButtonSize, ButtonVariant};
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
        Weekday::Sunday => "۱ش",
        Weekday::Monday => "۲ش",
        Weekday::Tuesday => "۳ش",
        Weekday::Wednesday => "۴ش",
        Weekday::Thursday => "۵ش",
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

fn calendar_icon_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    variant: crate::button::ButtonVariant,
    size: ButtonSize,
    button_size_px: Px,
    text: Arc<str>,
    enabled: bool,
    on_activate: impl Fn(&mut dyn fret_ui::action::UiActionHost) + 'static,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let (bg, bg_hover, bg_pressed, border, fg) = crate::button::variant_colors(&theme, variant);

    let radius = theme
        .metric_by_key("component.button.radius")
        .unwrap_or_else(|| theme.metric_required("metric.radius.md"));

    control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            on_activate(host);
        }));

        let mut pressable_layout = LayoutStyle::default();
        pressable_layout.size.width = Length::Px(button_size_px);
        pressable_layout.size.height = Length::Px(button_size_px);

        let bg = if st.pressed {
            bg_pressed
        } else if st.hovered {
            bg_hover
        } else {
            bg
        };

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));

        let pressable = PressableProps {
            layout: pressable_layout,
            enabled,
            focusable: enabled,
            focus_ring: Some(decl_style::focus_ring(&theme, radius)),
            a11y: PressableA11y {
                label: Some(Arc::from(label)),
                ..Default::default()
            },
            ..Default::default()
        };

        let chrome_props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());

        let style = crate::button::button_text_style(&theme, size);
        let children = move |cx: &mut ElementContext<'_, H>| {
            let mut label = ui::label(cx, text.clone())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(fg))
                .nowrap();
            if let Some(line_height) = style.line_height {
                label = label.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = style.letter_spacing_em {
                label = label.letter_spacing_em(letter_spacing_em);
            }
            vec![label.into_element(cx)]
        };

        (pressable, chrome_props, children)
    })
}

fn hijri_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
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
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let fg = if in_month {
        theme.color_required("foreground")
    } else {
        muted_fg
    };

    let (bg, fg) = if selected {
        (
            theme.color_required("primary"),
            theme.color_required("primary-foreground"),
        )
    } else {
        (Color::TRANSPARENT, fg)
    };

    let ring_color = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"));

    let solar = fret_ui_headless::calendar_solar_hijri::solar_hijri_from_gregorian(date);
    let day_text: Arc<str> = Arc::from(to_persian_digits(solar.day));
    let date_label = solar_hijri_day_aria_label(date, selected);

    let text_sm_px = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let text_sm_line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

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

        let hover_bg = theme.color_required("accent");
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
            let mut props = TextProps::new(Arc::clone(&day_text));
            props.style = Some(TextStyle {
                font: Default::default(),
                size: text_sm_px,
                weight: FontWeight::NORMAL,
                line_height: Some(text_sm_line_height),
                ..Default::default()
            });
            props.color = Some(fg);
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
            vec![cx.text_props(props)]
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
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl CalendarHijri {
    pub fn new(month: Model<SolarHijriMonth>, selected: Model<Option<Date>>) -> Self {
        Self {
            month,
            selected,
            disable_navigation: false,
            week_start: Weekday::Saturday,
            show_outside_days: true,
            cell_size: None,
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

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

            let title = solar_hijri_month_title(month);
            let weekday_labels = hijri_weekday_labels_visual_order(week_start);

            let text_sm_px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let text_sm_line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
                .unwrap_or_else(|| theme.metric_required("font.line_height"));

            let grid_text_style = TextStyle {
                font: Default::default(),
                size: text_sm_px,
                weight: FontWeight::MEDIUM,
                line_height: Some(text_sm_line_height),
                ..Default::default()
            };

            let day_size = self.cell_size.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.calendar.day_size")
                    .unwrap_or_else(|| theme.metric_required("component.size.sm.icon_button.size"))
            });
            let week_row_gap = theme
                .metric_by_key("component.calendar.week_row_gap")
                .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
            let day_col_gap = Px(0.0);
            let day_grid_width = Px(day_size.0 * 7.0);
            let month_width = day_grid_width;

            let bg = theme.color_required("background");
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

                let header = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_px(MetricRef::Px(month_width)))
                        .items_center()
                        .justify_between(),
                    move |cx| {
                        let nav_enabled = !disable_navigation;

                        let month_model_prev = month_model.clone();
                        let prev = calendar_icon_button(
                            cx,
                            "Go to the Previous Month",
                            ButtonVariant::Ghost,
                            ButtonSize::IconSm,
                            day_size,
                            Arc::from("<"),
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
                        let month_model_next = month_model.clone();
                        let next = calendar_icon_button(
                            cx,
                            "Go to the Next Month",
                            ButtonVariant::Ghost,
                            ButtonSize::IconSm,
                            day_size,
                            Arc::from(">"),
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

                        let mut title_props = TextProps::new(title.clone());
                        title_props.style = Some(TextStyle {
                            font: Default::default(),
                            size: theme_header.metric_required("font.size"),
                            weight: FontWeight::MEDIUM,
                            line_height: Some(theme_header.metric_required("font.line_height")),
                            ..Default::default()
                        });
                        title_props.wrap = TextWrap::None;
                        title_props.overflow = TextOverflow::Clip;
                        let title_el = cx.text_props(title_props);

                        // RTL visual order: Next (left), Title (center), Previous (right).
                        vec![next, title_el, prev]
                    },
                );

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
                        gap: day_col_gap,
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        weekday_labels
                            .iter()
                            .map(|label| {
                                let mut props = TextProps::new(Arc::clone(label));
                                props.style = Some(grid_text_style.clone());
                                props.color = theme_weekdays.color_by_key("muted-foreground");
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;

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
                        gap: day_col_gap,
                        padding: fret_core::Edges::all(Px(0.0)),
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

                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N2),
                    move |_cx| vec![weekday_row, days_grid],
                );

                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N4),
                    move |_cx| vec![header, body],
                )]
            })
        })
    }
}

fn calendar_hidden_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
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
        chrome_props.layout = layout;

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

        let children = move |_cx: &mut ElementContext<'_, H>| Vec::new();
        (pressable, chrome_props, children)
    })
}
