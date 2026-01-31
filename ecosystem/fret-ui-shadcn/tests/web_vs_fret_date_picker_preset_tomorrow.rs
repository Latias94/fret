use std::sync::Arc;

use fret_app::App;
use fret_core::{
    AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point,
    Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_runtime::{FrameId, Model};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use time::Month;

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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_core::SemanticsSnapshot {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "web-vs-fret-date-picker-preset-tomorrow",
        f,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

#[test]
fn web_vs_fret_date_picker_with_presets_preset_tomorrow_trigger_text_and_selected_day_match_web() {
    // Source of truth: `repo-ref/ui/.../date-picker-with-presets.tsx` uses:
    // `format(date, "PPP")` + `setDate(addDays(new Date(), parseInt(value)))`.
    //
    // With `freezeDate=2026-01-15` and selecting `Tomorrow`, the expected label is deterministic.
    let expected_trigger_text: Arc<str> = Arc::from("January 16th, 2026");
    let expected_selected_day_marker = "January 16th, 2026";

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1440.0), Px(900.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    use fret_ui_headless::calendar::CalendarMonth;
    use time::Date;

    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected: Model<Option<Date>> = app.models_mut().insert(Some(
        Date::from_calendar_date(2026, Month::January, 16).expect("date"),
    ));
    let preset_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("1")));

    let build = |cx: &mut fret_ui::ElementContext<'_, App>| {
        vec![
            fret_ui_shadcn::DatePickerWithPresets::new(
                open.clone(),
                month.clone(),
                selected.clone(),
            )
            .today(Date::from_calendar_date(2026, Month::January, 15).expect("today"))
            .preset_value_model(preset_value.clone())
            .into_element(cx),
        ]
    };

    // Render twice: popover positioning/presence uses previous layout state.
    let _ = render_frame(&mut ui, &mut app, &mut services, window, bounds, build);
    let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, build);

    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::Button
                && n.label.as_deref() == Some(expected_trigger_text.as_ref())
        }),
        "expected trigger button label to match web (PPP)"
    );

    assert!(
        snap.nodes.iter().any(|n| {
            n.label
                .as_deref()
                .is_some_and(|l| l.contains(expected_selected_day_marker) && l.contains("selected"))
        }),
        "expected selected day aria label to include 'selected' marker"
    );
}
