use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::Effect;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(240.0)),
    )
}

fn render_frame<I, F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    render: F,
) where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    app.set_frame_id(frame_id);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "extras", render);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

fn find_zone_value(snap: &fret_core::SemanticsSnapshot, label: &str) -> String {
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Group && n.label.as_deref() == Some(label))
        .and_then(|n| n.value.clone())
        .unwrap_or_else(|| panic!("missing semantics value for zone label={label:?}"))
}

fn has_request_animation_frame(effects: &[Effect], window: AppWindowId) -> bool {
    effects
        .iter()
        .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window))
}

#[test]
fn relative_time_clock_auto_update_requests_animation_frames_and_is_controllable() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let date = Date::from_calendar_date(2026, Month::February, 9).expect("date");
    let time = Time::from_hms(15, 4, 5).expect("time");
    let default_time = OffsetDateTime::new_utc(date, time);
    let controlled_time = app.models_mut().insert(default_time);

    let zones = || {
        [
            fret_ui_shadcn::extras::RelativeTimeClockZone::new("UTC", UtcOffset::UTC),
            fret_ui_shadcn::extras::RelativeTimeClockZone::new(
                "PST",
                UtcOffset::from_hms(-8, 0, 0).expect("PST offset"),
            ),
            fret_ui_shadcn::extras::RelativeTimeClockZone::new(
                "CET",
                UtcOffset::from_hms(1, 0, 0).expect("CET offset"),
            ),
        ]
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        |cx| {
            vec![
                fret_ui_shadcn::extras::RelativeTime::uncontrolled_clock(zones())
                    .default_time_utc(default_time)
                    .tick(fret_ui_shadcn::extras::RelativeTimeTick::Second)
                    .into_element(cx),
            ]
        },
    );

    let effects = app.flush_effects();
    assert!(
        has_request_animation_frame(&effects, window),
        "expected uncontrolled clock to request animation frames; effects={effects:?}"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert_eq!(
        find_zone_value(snap, "UTC"),
        "February 9, 2026 15:04:05".to_string()
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        |cx| {
            vec![
                fret_ui_shadcn::extras::RelativeTime::controlled_clock(
                    controlled_time.clone(),
                    zones(),
                )
                .tick(fret_ui_shadcn::extras::RelativeTimeTick::Second)
                .into_element(cx),
            ]
        },
    );

    let effects = app.flush_effects();
    assert!(
        !has_request_animation_frame(&effects, window),
        "did not expect controlled clock to request animation frames; effects={effects:?}"
    );

    let _ = app.models_mut().update(&controlled_time, |v| {
        *v = *v + time::Duration::seconds(1);
    });

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        |cx| {
            vec![
                fret_ui_shadcn::extras::RelativeTime::controlled_clock(
                    controlled_time.clone(),
                    zones(),
                )
                .tick(fret_ui_shadcn::extras::RelativeTimeTick::Second)
                .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .expect("semantics snapshot after model update");
    assert_eq!(
        find_zone_value(snap, "UTC"),
        "February 9, 2026 15:04:06".to_string()
    );
}
