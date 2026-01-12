use super::*;
use crate::widget::MeasureCx;

#[derive(Default)]
struct ReentrantMeasureWidget;

impl<H: UiHost> Widget<H> for ReentrantMeasureWidget {
    fn measure(&mut self, cx: &mut MeasureCx<'_, H>) -> Size {
        cx.measure_in(cx.node, cx.constraints)
    }
}

#[test]
#[should_panic(expected = "measure_in re-entered")]
fn measure_in_reentrancy_panics_in_debug_builds() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(ReentrantMeasureWidget);
    ui.set_root(node);

    let mut services = FakeUiServices;
    let constraints = crate::layout_constraints::LayoutConstraints::new(
        crate::layout_constraints::LayoutSize::new(None, None),
        crate::layout_constraints::LayoutSize::new(
            crate::layout_constraints::AvailableSpace::MinContent,
            crate::layout_constraints::AvailableSpace::MinContent,
        ),
    );

    let _ = ui.measure_in(&mut app, &mut services, node, constraints, 1.0);
}

#[test]
fn measure_reentrancy_diagnostics_rate_limits_by_frame() {
    let mut diagnostics = MeasureReentrancyDiagnostics::default();

    assert_eq!(diagnostics.record(FrameId(0)), Some(0));
    assert_eq!(diagnostics.record(FrameId(0)), None);
    assert_eq!(diagnostics.record(FrameId(119)), None);
    assert_eq!(diagnostics.record(FrameId(120)), Some(2));
}
