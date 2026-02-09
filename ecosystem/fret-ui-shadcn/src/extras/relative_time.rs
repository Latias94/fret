use std::sync::Arc;
use std::time::Duration;

use fret_core::Px;
use fret_core::time::Instant;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state::use_controllable_model;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};
use time::{OffsetDateTime, UtcOffset};

use crate::test_id::attach_test_id;

/// A small world-clock-like display block inspired by Kibo's "RelativeTime" shadcn block.
///
/// Notes:
/// - Display-only composition (`RelativeTime::new(...)`) remains supported.
/// - Callers provide already-formatted strings for date/time.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/relative-time`
#[derive(Debug, Clone)]
pub struct RelativeTime {
    kind: RelativeTimeKind,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone)]
enum RelativeTimeKind {
    StaticChildren(Vec<AnyElement>),
    Clock(RelativeTimeClock),
}

impl RelativeTime {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            kind: RelativeTimeKind::StaticChildren(children.into_iter().collect()),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    /// An auto-updating world clock (uncontrolled).
    ///
    /// This uses a continuous-frames lease and advances the internal time model by one tick per
    /// interval.
    pub fn uncontrolled_clock(zones: impl IntoIterator<Item = RelativeTimeClockZone>) -> Self {
        Self {
            kind: RelativeTimeKind::Clock(RelativeTimeClock::uncontrolled(zones)),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    /// An auto-updating world clock driven by a caller-provided time model (controlled).
    ///
    /// Controlled clocks do not schedule timers; callers are responsible for updating the model.
    pub fn controlled_clock(
        time: Model<OffsetDateTime>,
        zones: impl IntoIterator<Item = RelativeTimeClockZone>,
    ) -> Self {
        Self {
            kind: RelativeTimeKind::Clock(RelativeTimeClock::controlled(time, zones)),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    /// Apply an auto-update tick interval (only applies to clock variants).
    pub fn tick(mut self, tick: RelativeTimeTick) -> Self {
        if let RelativeTimeKind::Clock(clock) = &mut self.kind {
            clock.tick = tick;
        }
        self
    }

    /// Override the initial time value (only applies to uncontrolled clock variants).
    pub fn default_time_utc(mut self, time: OffsetDateTime) -> Self {
        if let RelativeTimeKind::Clock(clock) = &mut self.kind {
            clock.default_time_utc = time;
        }
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = self.layout;
        let el = match self.kind {
            RelativeTimeKind::StaticChildren(children) => stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap_y(Space::N2)
                    .layout(layout),
                |_cx| children,
            ),
            RelativeTimeKind::Clock(clock) => clock.into_element(cx, layout),
        };
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZone {
    label: Arc<str>,
    date: Arc<str>,
    time: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl RelativeTimeZone {
    pub fn new(
        label: impl Into<Arc<str>>,
        date: impl Into<Arc<str>>,
        time: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            label: label.into(),
            date: date.into(),
            time: time.into(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let label = self.label.clone();
        let date = self.date.clone();
        let time = self.time.clone();

        let left = stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_center()
                .gap_x(Space::N1p5)
                .layout(LayoutRefinement::default().min_w_0()),
            |cx| {
                vec![
                    RelativeTimeZoneLabel::new(self.label.clone()).into_element(cx),
                    RelativeTimeZoneDate::new(self.date.clone()).into_element(cx),
                ]
            },
        );
        let right = RelativeTimeZoneDisplay::new(self.time)
            .muted(true)
            .into_element(cx);

        let el = stack::hstack(
            cx,
            stack::HStackProps::default()
                .justify_between()
                .items_center()
                .gap_x(Space::N1p5)
                .layout(LayoutRefinement::default().min_w_0().merge(self.layout)),
            |_cx| vec![left, right],
        );

        let el = attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone")),
        );

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .label(label)
                .value(Arc::<str>::from(format!("{date} {time}"))),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeTick {
    Second,
    Minute,
}

impl RelativeTimeTick {
    fn std_duration(self) -> Duration {
        match self {
            Self::Second => Duration::from_secs(1),
            Self::Minute => Duration::from_secs(60),
        }
    }

    fn time_duration(self) -> time::Duration {
        match self {
            Self::Second => time::Duration::seconds(1),
            Self::Minute => time::Duration::minutes(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeClockZone {
    pub label: Arc<str>,
    pub offset: UtcOffset,
}

impl RelativeTimeClockZone {
    pub fn new(label: impl Into<Arc<str>>, offset: UtcOffset) -> Self {
        Self {
            label: label.into(),
            offset,
        }
    }
}

#[derive(Debug, Clone)]
struct RelativeTimeClock {
    zones: Vec<RelativeTimeClockZone>,
    time: Option<Model<OffsetDateTime>>,
    default_time_utc: OffsetDateTime,
    tick: RelativeTimeTick,
}

impl RelativeTimeClock {
    fn uncontrolled(zones: impl IntoIterator<Item = RelativeTimeClockZone>) -> Self {
        Self {
            zones: zones.into_iter().collect(),
            time: None,
            default_time_utc: OffsetDateTime::now_utc(),
            tick: RelativeTimeTick::Second,
        }
    }

    fn controlled(
        time: Model<OffsetDateTime>,
        zones: impl IntoIterator<Item = RelativeTimeClockZone>,
    ) -> Self {
        Self {
            zones: zones.into_iter().collect(),
            time: Some(time),
            default_time_utc: OffsetDateTime::now_utc(),
            tick: RelativeTimeTick::Second,
        }
    }

    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        layout: LayoutRefinement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();

            let out = use_controllable_model(cx, self.time, || self.default_time_utc);
            let time_model = out.model();
            let is_controlled = out.is_controlled();

            #[derive(Default)]
            struct AutoUpdateState {
                next_update_at: Option<Instant>,
            }

            let tick = self.tick;
            let want_auto_update = !is_controlled;
            scheduling::set_continuous_frames(cx, want_auto_update);
            if want_auto_update {
                let now = Instant::now();
                let due = cx.with_state_for(id, AutoUpdateState::default, |st| {
                    if let Some(next) = st.next_update_at {
                        if now >= next {
                            st.next_update_at = Some(now + tick.std_duration());
                            return true;
                        }
                        return false;
                    }

                    st.next_update_at = Some(now + tick.std_duration());
                    false
                });

                if due {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&time_model, |v| *v = *v + tick.time_duration());
                }
            } else {
                cx.with_state_for(id, AutoUpdateState::default, |st| {
                    st.next_update_at = None;
                });
            }

            let now = cx
                .watch_model(&time_model)
                .layout()
                .cloned_or_else(|| self.default_time_utc);

            let mut children = Vec::with_capacity(self.zones.len());
            for zone in self.zones {
                let dt = now.to_offset(zone.offset);
                let date = Arc::<str>::from(format_date_long(dt));
                let time = Arc::<str>::from(format_time_hms(dt));
                children
                    .push(RelativeTimeZone::new(zone.label.clone(), date, time).into_element(cx));
            }

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap_y(Space::N2)
                    .layout(layout),
                |_cx| children,
            )
        })
    }
}

fn month_name_long(m: time::Month) -> &'static str {
    match m {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    }
}

fn format_date_long(dt: OffsetDateTime) -> String {
    let d = dt.date();
    format!("{} {}, {}", month_name_long(d.month()), d.day(), d.year())
}

fn format_time_hms(dt: OffsetDateTime) -> String {
    let t = dt.time();
    format!("{:02}:{:02}:{:02}", t.hour(), t.minute(), t.second())
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneLabel {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneLabel {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let bg = theme.color_required("secondary");
        let fg = theme.color_required("secondary-foreground");

        let chrome = ChromeRefinement::default()
            .px(Space::N1p5)
            .rounded(Radius::Sm)
            .bg(ColorRef::Color(bg))
            .text_color(ColorRef::Color(fg));
        let props =
            decl_style::container_props(&theme, chrome, LayoutRefinement::default().h_px(Px(16.0)));

        let label = self.label;
        let el = cx.container(props, move |cx| {
            let layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().h_full().min_w_0(),
            );
            vec![cx.flex(
                fret_ui::element::FlexProps {
                    layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: fret_ui::element::MainAlign::Center,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |cx| vec![ui::text(cx, label.clone()).text_xs().into_element(cx)],
            )]
        });
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-label")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneDate {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneDate {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = ui::text(cx, self.text).text_xs().into_element(cx);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-date")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneDisplay {
    text: Arc<str>,
    muted: bool,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneDisplay {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            muted: true,
            test_id: None,
        }
    }

    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = if self.muted {
            theme.color_required("muted-foreground")
        } else {
            theme.color_required("foreground")
        };

        let chrome = ChromeRefinement::default().pl(Space::N8);
        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let text = self.text;

        let el = cx.container(props, move |cx| {
            vec![
                ui::text(cx, text.clone())
                    .text_xs()
                    .text_color(ColorRef::Color(fg))
                    .into_element(cx),
            ]
        });

        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-display")),
        )
    }
}
