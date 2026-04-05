use super::*;

#[test]
fn scroll_intrinsic_viewport_mode_does_not_measure_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-intrinsic-viewport-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Viewport;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll),
        0,
        "expected viewport-mode scroll intrinsic measurement to avoid measuring children"
    );
}

#[test]
fn scroll_intrinsic_content_mode_measures_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-intrinsic-content-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert!(
        ui.debug_measure_child_calls_for_parent(scroll) > 0,
        "expected content-mode scroll intrinsic measurement to measure children"
    );
}

#[test]
fn scroll_probe_unbounded_treats_zero_placeholder_cross_axis_width_as_unknown() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-zero-placeholder-cross-axis-width",
        |cx| {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Fill;
            scroll.layout.size.height = Length::Auto;
            scroll.axis = crate::element::ScrollAxis::Y;
            scroll.probe_unbounded = true;
            scroll.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;

            let mut child = crate::element::ContainerProps::default();
            child.layout.size.width = Length::Px(Px(80.0));
            child.layout.size.height = Length::Px(Px(24.0));

            vec![cx.scroll(scroll, |cx| vec![cx.container(child, |_cx| vec![])])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];
    let constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(0.0)),
            AvailableSpace::MaxContent,
        ),
    );
    let size = ui.measure_in(&mut app, &mut text, scroll, constraints, 1.0);

    assert!(
        size.width.0 >= 79.5,
        "expected cross-axis placeholder width to be treated as unknown during scroll probing; size={size:?}"
    );
    assert!(
        size.height.0 >= 23.5,
        "expected scroll probe height to preserve child height under zero cross-axis placeholder width; size={size:?}"
    );
}

#[test]
fn scroll_probe_unbounded_treats_zero_placeholder_cross_axis_height_as_unknown() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-zero-placeholder-cross-axis-height",
        |cx| {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Auto;
            scroll.layout.size.height = Length::Fill;
            scroll.axis = crate::element::ScrollAxis::X;
            scroll.probe_unbounded = true;
            scroll.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;

            let mut child = crate::element::ContainerProps::default();
            child.layout.size.width = Length::Px(Px(24.0));
            child.layout.size.height = Length::Px(Px(80.0));

            vec![cx.scroll(scroll, |cx| vec![cx.container(child, |_cx| vec![])])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];
    let constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::MaxContent,
            AvailableSpace::Definite(Px(0.0)),
        ),
    );
    let size = ui.measure_in(&mut app, &mut text, scroll, constraints, 1.0);

    assert!(
        size.width.0 >= 23.5,
        "expected scroll probe width to preserve child width under zero cross-axis placeholder height; size={size:?}"
    );
    assert!(
        size.height.0 >= 79.5,
        "expected cross-axis placeholder height to be treated as unknown during scroll probing; size={size:?}"
    );
}

#[test]
fn scroll_probe_unbounded_cache_respects_cross_axis_width_in_same_frame() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    struct WidthSensitiveMeasure;

    impl<H: UiHost> Widget<H> for WidthSensitiveMeasure {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            match cx.constraints.available.width.definite() {
                Some(width) if width.0 > 0.0 => Size::new(width, Px(24.0)),
                _ => Size::new(Px(160.0), Px(96.0)),
            }
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-cache-cross-axis-width-same-frame",
        |cx| {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Auto;
            scroll.layout.size.height = Length::Auto;
            scroll.axis = crate::element::ScrollAxis::Y;
            scroll.probe_unbounded = true;
            scroll.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;
            vec![cx.scroll(scroll, |_cx| Vec::new())]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];
    let child = ui.create_node(WidthSensitiveMeasure);
    ui.set_children(scroll, vec![child]);

    let placeholder_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(0.0)),
            AvailableSpace::MaxContent,
        ),
    );
    let placeholder = ui.measure_in(&mut app, &mut text, scroll, placeholder_constraints, 1.0);
    assert!(
        placeholder.height.0 >= 95.5,
        "expected placeholder probe to measure the unbounded width variant first; size={placeholder:?}"
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(80.0)),
            AvailableSpace::MaxContent,
        ),
    );
    let definite = ui.measure_in(&mut app, &mut text, scroll, definite_constraints, 1.0);
    assert!(
        (definite.height.0 - 24.0).abs() <= 0.5,
        "expected same-frame remeasure to respect the definite cross-axis width instead of reusing the placeholder probe: placeholder={placeholder:?} definite={definite:?}"
    );
}

#[test]
fn scroll_probe_unbounded_cache_respects_cross_axis_height_in_same_frame() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    struct HeightSensitiveMeasure;

    impl<H: UiHost> Widget<H> for HeightSensitiveMeasure {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            match cx.constraints.available.height.definite() {
                Some(height) if height.0 > 0.0 => Size::new(Px(24.0), height),
                _ => Size::new(Px(96.0), Px(160.0)),
            }
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-cache-cross-axis-height-same-frame",
        |cx| {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Auto;
            scroll.layout.size.height = Length::Auto;
            scroll.axis = crate::element::ScrollAxis::X;
            scroll.probe_unbounded = true;
            scroll.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;
            vec![cx.scroll(scroll, |_cx| Vec::new())]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];
    let child = ui.create_node(HeightSensitiveMeasure);
    ui.set_children(scroll, vec![child]);

    let placeholder_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::MaxContent,
            AvailableSpace::Definite(Px(0.0)),
        ),
    );
    let placeholder = ui.measure_in(&mut app, &mut text, scroll, placeholder_constraints, 1.0);
    assert!(
        placeholder.width.0 >= 95.5,
        "expected placeholder probe to measure the unbounded cross-axis height variant first; size={placeholder:?}"
    );

    let definite_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::MaxContent,
            AvailableSpace::Definite(Px(80.0)),
        ),
    );
    let definite = ui.measure_in(&mut app, &mut text, scroll, definite_constraints, 1.0);
    assert!(
        (definite.width.0 - 24.0).abs() <= 0.5,
        "expected same-frame remeasure to respect the definite cross-axis height instead of reusing the placeholder probe: placeholder={placeholder:?} definite={definite:?}"
    );
}

#[test]
fn scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    struct FixedMeasureLayoutNode {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedMeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.size
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut cfg = crate::runtime_config::ui_runtime_config().clone();
    cfg.scroll_defer_unbounded_probe_on_invalidation = true;
    cfg.scroll_defer_unbounded_probe_stable_frames = 2;
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    scroll_handle.set_viewport_size_internal(Size::new(Px(120.0), Px(40.0)));
    scroll_handle.set_content_size_internal(Size::new(Px(120.0), Px(160.0)));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-deferred-invalidation-intrinsic-seed",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );
    ui.set_root(root);

    let scroll_node = ui.children(root)[0];
    let child = ui.create_node(FixedMeasureLayoutNode {
        size: Size::new(Px(120.0), Px(160.0)),
    });
    ui.set_children(scroll_node, vec![child]);
    ui.test_set_layout_invalidation(child, true);

    let retained_child_size = ui.debug_node_measured_size(child);
    assert!(
        retained_child_size.is_none() || retained_child_size == Some(Size::default()),
        "expected the retained child measured size to be absent or default-sized before the deferred frame, got {retained_child_size:?}"
    );

    let scroll_element = ui
        .node_element(scroll_node)
        .expect("expected scroll host node to carry an element id");
    let child_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(120.0)),
            AvailableSpace::MaxContent,
        ),
    );
    let intrinsic_key = crate::element::ScrollIntrinsicMeasureCacheKey {
        avail_w: child_constraints
            .available
            .width
            .definite()
            .unwrap()
            .0
            .to_bits() as u64,
        avail_h: 2u64 << 62,
        axis: 1,
        probe_unbounded: true,
        scale_bits: 1.0f32.to_bits(),
    };
    crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| {
            state.intrinsic_measure_cache = Some(crate::element::ScrollIntrinsicMeasureCache {
                key: intrinsic_key,
                max_child: Size::new(Px(120.0), Px(160.0)),
            });
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node),
        0,
        "expected invalidation defer to consume the retained intrinsic seed instead of remeasuring the child subtree"
    );
    assert!(
        (scroll_handle.content_size().height.0 - 160.0).abs() <= 0.5,
        "expected deferred frame to preserve the seeded content extent without collapsing: content={:?}",
        scroll_handle.content_size()
    );
    assert!(
        (scroll_handle.max_offset().y.0 - 120.0).abs() <= 0.5,
        "expected deferred frame to preserve the seeded scroll range: max_offset={:?}",
        scroll_handle.max_offset()
    );
}

#[test]
fn scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    struct FixedMeasureLayoutNode {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedMeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.size
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut cfg = crate::runtime_config::ui_runtime_config().clone();
    cfg.scroll_defer_unbounded_probe_on_invalidation = true;
    cfg.scroll_defer_unbounded_probe_stable_frames = 2;
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    scroll_handle.set_viewport_size_internal(Size::new(Px(120.0), Px(40.0)));
    scroll_handle.set_content_size_internal(Size::new(Px(120.0), Px(160.0)));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-authoritative-observation-clears-deferred-invalidation-pending",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );
    ui.set_root(root);

    let scroll_node = ui.children(root)[0];
    let child = ui.create_node(FixedMeasureLayoutNode {
        size: Size::new(Px(120.0), Px(160.0)),
    });
    ui.set_children(scroll_node, vec![child]);
    ui.test_set_layout_invalidation(child, true);

    let scroll_element = ui
        .node_element(scroll_node)
        .expect("expected scroll host node to carry an element id");
    let child_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(
            AvailableSpace::Definite(Px(120.0)),
            AvailableSpace::MaxContent,
        ),
    );
    let intrinsic_key = crate::element::ScrollIntrinsicMeasureCacheKey {
        avail_w: child_constraints
            .available
            .width
            .definite()
            .unwrap()
            .0
            .to_bits() as u64,
        avail_h: 2u64 << 62,
        axis: 1,
        probe_unbounded: true,
        scale_bits: 1.0f32.to_bits(),
    };
    crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| {
            state.intrinsic_measure_cache = Some(crate::element::ScrollIntrinsicMeasureCache {
                key: intrinsic_key,
                max_child: Size::new(Px(120.0), Px(160.0)),
            });
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node),
        0,
        "expected the deferred invalidation frame to consume the intrinsic seed without measuring"
    );

    ui.test_set_layout_invalidation(child, false);
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), scroll_handle.max_offset().y));

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node),
        0,
        "expected an unchanged authoritative post-layout observation to clear deferred invalidation state so the next at-edge frame does not force an extra probe"
    );
    assert!(
        (scroll_handle.content_size().height.0 - 160.0).abs() <= 0.5,
        "expected the follow-up frame to preserve the authoritative content extent: content={:?}",
        scroll_handle.content_size()
    );
    assert!(
        (scroll_handle.max_offset().y.0 - 120.0).abs() <= 0.5,
        "expected the follow-up frame to preserve the scroll range without an extra probe: max_offset={:?}",
        scroll_handle.max_offset()
    );
}

#[test]
fn scroll_authoritative_observation_same_extent_clears_resize_deferred_state() {
    struct FixedMeasureLayoutNode {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedMeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.size
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut cfg = crate::runtime_config::ui_runtime_config().clone();
    cfg.scroll_defer_unbounded_probe_on_resize = true;
    cfg.scroll_defer_unbounded_probe_stable_frames = 2;
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let roomy_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let compact_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        roomy_bounds,
        "scroll-authoritative-observation-clears-resize-deferred-state",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Both,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );
    ui.set_root(root);

    let scroll_node = ui.children(root)[0];
    let child = ui.create_node(FixedMeasureLayoutNode {
        size: Size::new(Px(160.0), Px(160.0)),
    });
    ui.set_children(scroll_node, vec![child]);

    layout_frame(&mut ui, &mut app, &mut text, roomy_bounds);
    ui.take_pending_barrier_relayouts();
    let _ = app.flush_effects();

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, compact_bounds);

    assert!(
        ui.interactive_resize_active(),
        "expected changed bounds to enter interactive-resize mode"
    );
    assert!(
        ui.take_pending_barrier_relayouts().is_empty(),
        "expected the active resize frame to avoid scheduling a follow-up barrier relayout while the viewport is still changing"
    );
    let compact_effects = app.flush_effects();
    assert!(
        !compact_effects
            .iter()
            .any(|effect| matches!(effect, Effect::Redraw(w) if *w == window)),
        "expected no redraw carry-over from the authoritative resize frame; effects={compact_effects:?}"
    );

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, compact_bounds);

    assert!(
        ui.take_pending_barrier_relayouts().is_empty(),
        "expected authoritative same-extent observation during the resize frame to clear resize-deferred probe state instead of scheduling a later follow-up barrier relayout"
    );
    let settled_effects = app.flush_effects();
    assert!(
        !settled_effects
            .iter()
            .any(|effect| matches!(effect, Effect::Redraw(w) if *w == window)),
        "expected no follow-up redraw once authoritative observation has already closed the resize-deferred probe state; effects={settled_effects:?}"
    );
}

#[test]
fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected scroll content subtree nodes to remain registered in the layout engine"
    );

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_wheel_bubbles_to_ancestor_when_axis_mismatch() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(140.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-wheel-bubbles-to-ancestor-axis-mismatch",
        |cx| {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();

            let mut outer_layout = crate::element::LayoutStyle::default();
            outer_layout.size.width = crate::element::Length::Fill;
            outer_layout.size.height = crate::element::Length::Px(Px(80.0));
            outer_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: outer_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(outer_handle),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            let mut inner_layout = crate::element::LayoutStyle::default();
                            inner_layout.size.width = crate::element::Length::Fill;
                            inner_layout.size.height = crate::element::Length::Px(Px(20.0));
                            inner_layout.overflow = crate::element::Overflow::Clip;

                            let inner = cx.scroll(
                                crate::element::ScrollProps {
                                    layout: inner_layout,
                                    axis: crate::element::ScrollAxis::X,
                                    scroll_handle: Some(inner_handle),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.row(crate::element::RowProps::default(), |cx| {
                                        (0..6)
                                            .map(|i| {
                                                cx.container(
                                                    crate::element::ContainerProps {
                                                        layout: crate::element::LayoutStyle {
                                                            size: crate::element::SizeStyle {
                                                                width: crate::element::Length::Px(
                                                                    Px(60.0),
                                                                ),
                                                                height: crate::element::Length::Px(
                                                                    Px(20.0),
                                                                ),
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    move |cx| vec![cx.text(format!("x{i}"))],
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    })]
                                },
                            );

                            let mut children = Vec::new();
                            for i in 0..16 {
                                children.push(cx.text(format!("row{i}")));
                            }
                            // Keep the nested scroll visible inside the outer viewport so hit
                            // testing (which respects overflow clips) can route the wheel event to
                            // the nested scroll first.
                            children.insert(2, inner);
                            children
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        outer_handle.offset().y.0.abs() <= 0.01,
        "expected initial outer scroll offset to be 0, got={:?}",
        outer_handle.offset().y
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected initial inner scroll offset to be 0, got={:?}",
        inner_handle.offset().x
    );

    assert!(
        outer_handle.max_offset().y.0 > 0.01,
        "expected outer scroll to be scrollable; max_offset={:?} viewport={:?} content={:?}",
        outer_handle.max_offset(),
        outer_handle.viewport_size(),
        outer_handle.content_size()
    );

    let outer_scroll_node = ui.children(root)[0];
    let outer_column_node = ui.children(outer_scroll_node)[0];
    let inner_scroll_node = ui.children(outer_column_node)[2];
    let inner_bounds = ui
        .debug_node_bounds(inner_scroll_node)
        .expect("inner bounds");
    let wheel_pos = fret_core::Point::new(
        Px(inner_bounds.origin.x.0 + 5.0),
        Px(inner_bounds.origin.y.0 + 5.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        outer_handle.offset().y.0 > 0.01,
        "expected vertical wheel on inner horizontal scroll to bubble to ancestor Y scroll; outer_offset={:?} inner_offset={:?}",
        outer_handle.offset(),
        inner_handle.offset()
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected vertical wheel on inner horizontal scroll to not move inner X offset; got={:?}",
        inner_handle.offset()
    );
}

#[test]
fn scroll_touch_pan_bubbles_to_ancestor_when_axis_mismatch() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(140.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-touch-pan-bubbles-to-ancestor-axis-mismatch",
        |cx| {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();

            let mut outer_layout = crate::element::LayoutStyle::default();
            outer_layout.size.width = crate::element::Length::Fill;
            outer_layout.size.height = crate::element::Length::Px(Px(80.0));
            outer_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: outer_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(outer_handle),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            let mut inner_layout = crate::element::LayoutStyle::default();
                            inner_layout.size.width = crate::element::Length::Fill;
                            inner_layout.size.height = crate::element::Length::Px(Px(20.0));
                            inner_layout.overflow = crate::element::Overflow::Clip;

                            let inner = cx.scroll(
                                crate::element::ScrollProps {
                                    layout: inner_layout,
                                    axis: crate::element::ScrollAxis::X,
                                    scroll_handle: Some(inner_handle),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.row(crate::element::RowProps::default(), |cx| {
                                        (0..6)
                                            .map(|i| {
                                                cx.container(
                                                    crate::element::ContainerProps {
                                                        layout: crate::element::LayoutStyle {
                                                            size: crate::element::SizeStyle {
                                                                width: crate::element::Length::Px(
                                                                    Px(60.0),
                                                                ),
                                                                height: crate::element::Length::Px(
                                                                    Px(20.0),
                                                                ),
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    move |cx| vec![cx.text(format!("x{i}"))],
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    })]
                                },
                            );

                            let mut children = Vec::new();
                            for i in 0..16 {
                                children.push(cx.text(format!("row{i}")));
                            }
                            children.insert(2, inner);
                            children
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        outer_handle.offset().y.0.abs() <= 0.01,
        "expected initial outer scroll offset to be 0, got={:?}",
        outer_handle.offset().y
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected initial inner scroll offset to be 0, got={:?}",
        inner_handle.offset().x
    );
    assert!(
        outer_handle.max_offset().y.0 > 0.01,
        "expected outer scroll to be scrollable; max_offset={:?} viewport={:?} content={:?}",
        outer_handle.max_offset(),
        outer_handle.viewport_size(),
        outer_handle.content_size()
    );

    let outer_scroll_node = ui.children(root)[0];
    let outer_column_node = ui.children(outer_scroll_node)[0];
    let inner_scroll_node = ui.children(outer_column_node)[2];
    let inner_bounds = ui
        .debug_node_bounds(inner_scroll_node)
        .expect("inner bounds");
    let start = fret_core::Point::new(
        Px(inner_bounds.origin.x.0 + 5.0),
        Px(inner_bounds.origin.y.0 + 5.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: fret_core::Point::new(start.x, Px(start.y.0 - 40.0)),
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert!(
        outer_handle.offset().y.0 > 0.01,
        "expected vertical touch pan on inner horizontal scroll to bubble to ancestor Y scroll; outer_offset={:?} inner_offset={:?}",
        outer_handle.offset(),
        inner_handle.offset()
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected vertical touch pan on inner horizontal scroll to not move inner X offset; got={:?}",
        inner_handle.offset()
    );
}

#[test]
fn scroll_touch_pan_bubbles_past_pressable_capture_when_axis_mismatch() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(140.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-touch-pan-bubbles-past-pressable-capture-axis-mismatch",
        |cx| {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();

            let mut pressable_layout = crate::element::LayoutStyle::default();
            pressable_layout.size.width = crate::element::Length::Fill;
            pressable_layout.size.height = crate::element::Length::Fill;

            let mut outer_layout = crate::element::LayoutStyle::default();
            outer_layout.size.width = crate::element::Length::Fill;
            outer_layout.size.height = crate::element::Length::Px(Px(80.0));
            outer_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: pressable_layout,
                    ..Default::default()
                },
                move |cx, _state| {
                    vec![cx.scroll(
                        crate::element::ScrollProps {
                            layout: outer_layout,
                            axis: crate::element::ScrollAxis::Y,
                            scroll_handle: Some(outer_handle),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0).into(),
                                    ..Default::default()
                                },
                                move |cx| {
                                    let mut inner_layout = crate::element::LayoutStyle::default();
                                    inner_layout.size.width = crate::element::Length::Fill;
                                    inner_layout.size.height =
                                        crate::element::Length::Px(Px(20.0));
                                    inner_layout.overflow = crate::element::Overflow::Clip;

                                    let inner = cx.scroll(
                                        crate::element::ScrollProps {
                                            layout: inner_layout,
                                            axis: crate::element::ScrollAxis::X,
                                            scroll_handle: Some(inner_handle),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.row(
                                                crate::element::RowProps::default(),
                                                |cx| {
                                                    (0..6)
                                                        .map(|i| {
                                                            cx.container(
                                                                crate::element::ContainerProps {
                                                                    layout:
                                                                        crate::element::LayoutStyle {
                                                                            size: crate::element::SizeStyle {
                                                                                width: crate::element::Length::Px(Px(60.0)),
                                                                                height: crate::element::Length::Px(Px(20.0)),
                                                                                ..Default::default()
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                    ..Default::default()
                                                                },
                                                                move |cx| {
                                                                    vec![cx.text(format!("x{i}"))]
                                                                },
                                                            )
                                                        })
                                                        .collect::<Vec<_>>()
                                                },
                                            )]
                                        },
                                    );

                                    let mut children = Vec::new();
                                    for i in 0..16 {
                                        children.push(cx.text(format!("row{i}")));
                                    }
                                    children.insert(2, inner);
                                    children
                                },
                            )]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        outer_handle.offset().y.0.abs() <= 0.01,
        "expected initial outer scroll offset to be 0, got={:?}",
        outer_handle.offset().y
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected initial inner scroll offset to be 0, got={:?}",
        inner_handle.offset().x
    );
    assert!(
        outer_handle.max_offset().y.0 > 0.01,
        "expected outer scroll to be scrollable; max_offset={:?} viewport={:?} content={:?}",
        outer_handle.max_offset(),
        outer_handle.viewport_size(),
        outer_handle.content_size()
    );

    let pressable_node = ui.children(root)[0];
    let outer_scroll_node = ui.children(pressable_node)[0];
    let outer_column_node = ui.children(outer_scroll_node)[0];
    let inner_scroll_node = ui.children(outer_column_node)[2];
    let inner_bounds = ui
        .debug_node_bounds(inner_scroll_node)
        .expect("inner bounds");
    let start = fret_core::Point::new(
        Px(inner_bounds.origin.x.0 + 5.0),
        Px(inner_bounds.origin.y.0 + 5.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: fret_core::Point::new(start.x, Px(start.y.0 - 40.0)),
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert!(
        outer_handle.offset().y.0 > 0.01,
        "expected vertical touch pan to bubble past the pressable capture into the ancestor Y scroll; outer_offset={:?} inner_offset={:?}",
        outer_handle.offset(),
        inner_handle.offset()
    );
    assert!(
        inner_handle.offset().x.0.abs() <= 0.01,
        "expected the inner X scroll to ignore the dominant vertical touch pan under pressable capture; got={:?}",
        inner_handle.offset()
    );
}

#[test]
fn scroll_translation_does_not_force_layout_engine_solves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel-solve-stats",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "expected scroll translation to avoid triggering layout engine solves"
    );
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected translation-only scroll to keep engine nodes alive (stable identity)"
    );

    // Even when the tree is fully clean (no invalidation, no translation), the request/build phase
    // must keep barrier-mounted subtrees registered so identity remains stable across frames.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected steady-state frames to keep scroll content nodes registered in the engine"
    );
}

#[test]
fn scroll_axis_both_probe_unbounded_keeps_content_at_least_viewport_width() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-axis-both-min-content-width",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Fill;
            p.axis = crate::element::ScrollAxis::Both;
            p.probe_unbounded = true;

            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let column_bounds = ui.debug_node_bounds(column_node).expect("column bounds");

    assert_eq!(
        column_bounds.size.width, bounds.size.width,
        "expected scroll content bounds to be at least the viewport width; got={:?} want={:?}",
        column_bounds.size.width, bounds.size.width
    );
}

#[test]
fn scroll_thumb_drag_updates_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar has enough room for Radix-style padding + 18px minimum thumb.
            // With very small tracks, Radix clamps the thumb to the available space and dragging
            // cannot change the scroll offset.
            stack_layout.size.height = crate::element::Length::Fill;

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0).into(),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        cx.text("a"),
                                        cx.text("b"),
                                        cx.text("c"),
                                        cx.text("d"),
                                        cx.text("e"),
                                        cx.text("f"),
                                    ]
                                },
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)).into(),
                            right: Some(Px(0.0)).into(),
                            bottom: Some(Px(0.0)).into(),
                            left: None.into(),
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let thumb = crate::declarative::paint_helpers::scrollbar_thumb_rect(
        scrollbar_bounds,
        scroll_handle.viewport_size().height,
        scroll_handle.content_size().height,
        scroll_handle.offset().y,
        crate::element::ScrollbarStyle::default().track_padding,
    )
    .expect("thumb rect");
    let down_pos = fret_core::Point::new(Px(thumb.origin.x.0 + 1.0), Px(thumb.origin.y.0 + 1.0));
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured(),
        Some(scrollbar_node),
        "expected thumb down to capture the pointer on the scrollbar node"
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().y
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after drag");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after thumb drag: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_thumb_drag_uses_baseline_metrics_when_content_grows_mid_drag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag-baseline",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            stack_layout.size.height = crate::element::Length::Fill;

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0).into(),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        cx.text("a"),
                                        cx.text("b"),
                                        cx.text("c"),
                                        cx.text("d"),
                                        cx.text("e"),
                                        cx.text("f"),
                                    ]
                                },
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)).into(),
                            right: Some(Px(0.0)).into(),
                            bottom: Some(Px(0.0)).into(),
                            left: None.into(),
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scrollbar_node = ui.children(stack_node)[1];

    let baseline_viewport = scroll_handle.viewport_size().height;
    let baseline_content = scroll_handle.content_size().height;

    // Start a thumb drag at offset=0.
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let thumb = crate::declarative::paint_helpers::scrollbar_thumb_rect(
        scrollbar_bounds,
        baseline_viewport,
        baseline_content,
        scroll_handle.offset().y,
        crate::element::ScrollbarStyle::default().track_padding,
    )
    .expect("thumb rect");
    let down_pos = fret_core::Point::new(Px(thumb.origin.x.0 + 1.0), Px(thumb.origin.y.0 + 1.0));
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured(),
        Some(scrollbar_node),
        "expected thumb down to capture the pointer on the scrollbar node"
    );

    // Simulate content growth while dragging (e.g. measurement updates in a vlist-like surface).
    scroll_handle.set_content_size(Size::new(
        scroll_handle.content_size().width,
        Px(baseline_content.0 + 200.0),
    ));

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let pad = crate::declarative::paint_helpers::scrollbar_track_padding_px(
        scrollbar_bounds.size.height.0,
        crate::element::ScrollbarStyle::default().track_padding,
    );
    let inner = (scrollbar_bounds.size.height.0 - pad * 2.0).max(0.0);
    let max_thumb_y = (inner - thumb.size.height.0).max(0.0);
    assert!(max_thumb_y > 0.0, "expected a draggable thumb track");

    let baseline_max_offset = Px((baseline_content.0 - baseline_viewport.0).max(0.0));
    let delta_y = move_pos.y.0 - down_pos.y.0;
    let scale = baseline_max_offset.0 / max_thumb_y;
    let expected = Px((delta_y * scale).max(0.0).min(baseline_max_offset.0));
    let actual = scroll_handle.offset().y;

    assert!(
        (actual.0 - expected.0).abs() <= 0.5,
        "expected baseline-locked drag delta: actual={:?} expected={:?} baseline_max_offset={:?}",
        actual,
        expected,
        baseline_max_offset
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
}

#[test]
fn scroll_handle_set_offset_triggers_visual_scroll_without_manual_invalidate() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.text("a"),
                            cx.text("b"),
                            cx.text("c"),
                            cx.text("d"),
                            cx.text("e"),
                            cx.text("f"),
                        ]
                    },
                )]
            },
        )]
    }

    // Frame 0: establish viewport and content extent.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: stable mount (no intentional invalidations).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Outside the UI runtime, programmatically update the handle.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(20.0)));
    app.advance_frame();

    // Frame 2: the scroll change should invalidate bound nodes implicitly via handle bindings.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after offset");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after programmatic scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_content_extent_updates_immediately_when_growing_at_scroll_end() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&show_more, Invalidation::Layout);
                let expanded = cx.app.models().get_copied(&show_more).unwrap_or(false);
                let rows = if expanded { 24 } else { 6 };

                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..rows)
                            .map(|i| cx.text(format!("row {i}")))
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    }

    // Frame 0: establish content extent and scroll to the end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = app.models_mut().update(&show_more, |v| *v = true);
    assert!(
        app.models().get_copied(&show_more).unwrap_or(false),
        "expected show_more model update to commit before the next frame"
    );
    app.advance_frame();

    // Frame 1: content grows while we're at the previous max offset.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let column_node = ui.children(scroll_node)[0];
    assert!(
        ui.node_needs_layout(column_node),
        "expected the scroll content subtree to be marked dirty when its children change"
    );
    assert!(
        scroll_handle.offset().y.0 + 0.5 >= scroll_handle.max_offset().y.0,
        "expected the scroll handle to remain at the previous max offset before relayout: offset={:?} max={:?}",
        scroll_handle.offset().y,
        scroll_handle.max_offset().y
    );

    {
        use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
        let max_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
        );
        let measured = ui.measure_in(&mut app, &mut text, column_node, max_constraints, 1.0);
        assert!(
            measured.height.0 > 60.0,
            "expected measuring the expanded column to observe increased height, got {measured:?}"
        );
    }

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root1)[0];
    let column_node = ui.children(scroll_node)[0];
    assert_eq!(
        ui.children(column_node).len(),
        24,
        "expected render to mount the expanded column children before layout"
    );

    let max1 = scroll_handle.max_offset().y;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected scroll extent to grow immediately when content expands at the end: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_at_end_reuses_cached_extent_when_clean() {
    let mut app = TestHost::new();

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..12)
                            .map(|i| cx.text(format!("row {i}")))
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    }

    // Frame 0: populate intrinsic measurement caches and scroll to the end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-at-end-reuses-caches",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    app.advance_frame();

    // Frame 1: no content changes while at the scroll end; layout should avoid measuring children.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-at-end-reuses-caches",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root1)[0];
    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node),
        0,
        "expected scroll layout to reuse intrinsic caches when at the extent edge and clean"
    );
    assert!(
        (scroll_handle.max_offset().y.0 - max0.0).abs() <= 0.5,
        "expected max offset to remain stable: before={max0:?} after={:?}",
        scroll_handle.max_offset().y
    );
}

#[test]
fn scroll_clamped_edge_wheel_does_not_probe_clean_docs_like_tree() {
    let mut app = TestHost::new();

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(56.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn paragraph(cx: &mut ElementContext<'_, TestHost>, text: impl Into<String>) -> AnyElement {
        let mut props = crate::element::ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.min_width = Some(Length::Px(Px(0.0)));
        let text = text.into();
        cx.container(props, move |cx| vec![cx.text(text.clone())])
    }

    fn section(cx: &mut ElementContext<'_, TestHost>, index: usize) -> AnyElement {
        let mut section_layout = crate::element::LayoutStyle::default();
        section_layout.size.width = Length::Fill;
        section_layout.size.min_width = Some(Length::Px(Px(0.0)));

        let mut shell_layout = crate::element::LayoutStyle::default();
        shell_layout.size.width = Length::Fill;
        shell_layout.size.min_width = Some(Length::Px(Px(0.0)));

        let preview_shell = cx.container(
            crate::element::ContainerProps {
                layout: shell_layout,
                ..Default::default()
            },
            move |cx| {
                let mut list_layout = crate::element::LayoutStyle::default();
                list_layout.size.width = Length::Fill;
                list_layout.size.min_width = Some(Length::Px(Px(0.0)));
                vec![cx.column(
                    crate::element::ColumnProps {
                        layout: list_layout,
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..4)
                            .map(|line| {
                                cx.text(format!(
                                    "section {index} line {line} - alert docs preview content"
                                ))
                            })
                            .collect::<Vec<_>>()
                    },
                )]
            },
        );

        cx.column(
            crate::element::ColumnProps {
                layout: section_layout,
                gap: Px(6.0).into(),
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.text(format!("Section {index}")),
                    paragraph(
                        cx,
                        format!(
                            "Wrapper-heavy doc section {index} with title, description and preview shell"
                        ),
                    ),
                    preview_shell,
                ]
            },
        )
    }

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                let mut wrapper_layout = crate::element::LayoutStyle::default();
                wrapper_layout.size.width = Length::Fill;
                wrapper_layout.size.min_width = Some(Length::Px(Px(0.0)));

                let mut body_layout = crate::element::LayoutStyle::default();
                body_layout.size.width = Length::Fill;
                body_layout.size.min_width = Some(Length::Px(Px(0.0)));

                vec![cx.container(
                    crate::element::ContainerProps {
                        layout: wrapper_layout,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.container(
                            crate::element::ContainerProps {
                                layout: wrapper_layout,
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.column(
                                    crate::element::ColumnProps {
                                        layout: body_layout,
                                        gap: Px(12.0).into(),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        let mut out = Vec::with_capacity(10);
                                        out.push(paragraph(
                                            cx,
                                            "Alert docs intro paragraph mirroring the gallery's centered doc body.",
                                        ));
                                        out.extend((0..8).map(|index| section(cx, index)));
                                        out.push(paragraph(
                                            cx,
                                            "Notes: keep alert copy concise, support rich title/description children, and avoid scroll drift at the end.",
                                        ));
                                        out
                                    },
                                )]
                            },
                        )]
                    },
                )]
            },
        )]
    }

    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-edge-wheel-clean-docs-like-tree",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    assert!(max0.0 > 0.0, "expected a non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-edge-wheel-clean-docs-like-tree",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let content0 = scroll_handle.content_size().height;
    let scroll_node0 = ui.children(root1)[0];
    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node0),
        0,
        "expected clean edge layout to reuse cached extents before the extra wheel"
    );

    let wheel_pos = fret_core::Point::new(Px(8.0), Px(8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-48.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    app.advance_frame();

    let root2 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-edge-wheel-clean-docs-like-tree",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root2);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node1 = ui.children(root2)[0];
    let content1 = scroll_handle.content_size().height;
    let max1 = scroll_handle.max_offset().y;
    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node1),
        0,
        "expected an extra wheel at a clean extent edge not to trigger a new child probe"
    );
    assert!(
        (content1.0 - content0.0).abs() <= 0.5,
        "expected content extent to remain stable after an extra edge wheel: before={content0:?} after={content1:?}"
    );
    assert!(
        (max1.0 - max0.0).abs() <= 0.5,
        "expected max offset to remain stable after an extra edge wheel: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_offset_clamps_when_content_shrinks_below_end() {
    let mut app = TestHost::new();
    let expanded = app.models_mut().insert(true);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn row(cx: &mut ElementContext<'_, TestHost>, i: usize) -> AnyElement {
        let mut props = crate::element::ContainerProps::default();
        props.layout.size.height = crate::element::Length::Px(Px(10.0));
        props.layout.size.width = crate::element::Length::Fill;
        cx.container(props, move |cx| vec![cx.text(format!("row {i}"))])
    }

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        expanded: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&expanded, Invalidation::Layout);
                let expanded = cx.app.models().get_copied(&expanded).unwrap_or(false);
                let rows = if expanded { 30 } else { 6 };

                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| (0..rows).map(|i| row(cx, i)).collect::<Vec<_>>(),
                )]
            },
        )]
    }

    // Frame 0: scroll to the end with the expanded content.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-shrink-clamps-offset",
        |cx| build_root(cx, scroll_handle.clone(), expanded.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = app.models_mut().update(&expanded, |v| *v = false);
    app.advance_frame();

    // Frame 1: content shrinks while we're beyond the new max; offset must clamp.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-shrink-clamps-offset",
        |cx| build_root(cx, scroll_handle.clone(), expanded.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        max1.0 + 0.5 < max0.0,
        "expected shrink to reduce max offset: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp to the new max: offset={off1:?} max={max1:?}"
    );
}

#[test]
fn scroll_axis_both_updates_extent_for_axis_growing_at_end() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&wide, Invalidation::Layout);
                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                let mut content = crate::element::ContainerProps::default();
                content.layout.size.width =
                    crate::element::Length::Px(if wide { Px(260.0) } else { Px(140.0) });
                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                vec![cx.container(content, |_cx| Vec::new())]
            },
        )]
    }

    // Frame 0: establish max offset and scroll to x end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-at-x-end",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().x;
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = true);
    app.advance_frame();

    // Frame 1: content grows in x while we're at the previous x max.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-at-x-end",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max1 = scroll_handle.max_offset().x;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected x extent to grow immediately when content expands at x end: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_observed_extent_does_not_double_count_scroll_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..96)
                            .map(|i| cx.text(format!("row {i}")))
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    }

    // Frame 0: establish a stable content extent.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-observed-extent-offset-regression",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        content0.height.0 > bounds.size.height.0 + 1.0,
        "expected scroll content to exceed viewport: content={:?} viewport={:?}",
        content0,
        bounds.size
    );

    // Frame 1: apply a non-zero scroll offset and ensure the content extent remains stable.
    assert!(max0.0 > 0.0, "expected a non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px((max0.0 * 0.6).max(1.0))));
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-observed-extent-offset-regression",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        scroll_handle.offset().y.0 > 0.5,
        "expected scroll offset to be non-zero after set_offset: offset={:?}",
        scroll_handle.offset()
    );
    assert!(
        (max1.0 - max0.0).abs() <= 0.5,
        "expected max offset to remain stable under scroll offset: before={max0:?} after={max1:?}",
    );
    assert!(
        (content1.height.0 - content0.height.0).abs() <= 0.5,
        "expected content extent to remain stable under scroll offset: before={:?} after={:?}",
        content0,
        content1
    );
}

#[test]
fn scroll_extent_updates_under_view_cache_reconciliation_when_growing_at_end() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        let mut cache = crate::element::ViewCacheProps::default();
        cache.layout.size.width = crate::element::Length::Fill;
        cache.layout.size.height = crate::element::Length::Auto;
        cache.cache_key = 1;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![cx.view_cache(cache, move |cx| {
                    cx.observe_model(&show_more, Invalidation::Layout);
                    let expanded = cx.app.models().get_copied(&show_more).unwrap_or(false);
                    let rows = if expanded { 24 } else { 6 };

                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            (0..rows)
                                .map(|i| cx.text(format!("row {i}")))
                                .collect::<Vec<_>>()
                        },
                    )]
                })]
            },
        )]
    }

    // Frame 0: establish content extent and scroll to the end.
    let _root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-view-cache",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = show_more.update(&mut app, |v, _cx| *v = true);
    app.advance_frame();

    // Frame 1: content grows while we're at the previous max offset.
    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-view-cache",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );

    let scroll_node = ui.children(root1)[0];
    let cache_node = ui.children(scroll_node)[0];
    assert!(
        ui.node_needs_layout(cache_node),
        "expected view-cache scroll content node to be dirty when its children change"
    );
    assert!(
        scroll_handle.offset().y.0 + 0.5 >= scroll_handle.max_offset().y.0,
        "expected scroll handle to remain at the previous max offset before relayout: offset={:?} max={:?}",
        scroll_handle.offset().y,
        scroll_handle.max_offset().y
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let max1 = scroll_handle.max_offset().y;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected scroll extent to grow immediately under view-cache reconciliation: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_probe_cache_shrinks_to_observed_bounds_when_probe_overmeasures() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLargeLayoutSmall {
        measured: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLargeLayoutSmall {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-probe-cache-shrink",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node = ui.children(root)[0];
    let leaf = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(320.0)),
    });
    let child = ui.create_node(MeasureLargeLayoutSmall {
        measured: Size::new(Px(120.0), Px(2000.0)),
        child: leaf,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(320.0)),
        ),
    });
    ui.set_children(child, vec![leaf]);
    ui.set_children(scroll_node, vec![child]);

    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let leaf_bounds = ui.debug_node_bounds(leaf).expect("leaf bounds");
    let content = scroll_handle.content_size();
    let max_offset = scroll_handle.max_offset();

    assert!(
        (leaf_bounds.size.height.0 - 320.0).abs() <= 0.5,
        "expected laid-out leaf height to match the final layout result, got={leaf_bounds:?}"
    );
    assert!(
        (content.height.0 - 320.0).abs() <= 0.5,
        "expected final scroll content height to shrink to the observed laid-out bounds: content={content:?} leaf_bounds={leaf_bounds:?} max_offset={max_offset:?}"
    );
    assert!(
        (max_offset.y.0 - 120.0).abs() <= 0.5,
        "expected max offset to match the shrunken extent: max_offset={max_offset:?} content={content:?} leaf_bounds={leaf_bounds:?}"
    );
}

#[test]
fn scroll_post_layout_growth_relayouts_child_root_same_frame() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureSmallLayoutLarge {
        measured: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureSmallLayoutLarge {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-growth-relayout",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node = ui.children(root)[0];
    let leaf = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(320.0)),
    });
    let child_root = ui.create_node(MeasureSmallLayoutLarge {
        measured: Size::new(Px(120.0), Px(200.0)),
        child: leaf,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(320.0)),
        ),
    });
    ui.set_children(child_root, vec![leaf]);
    ui.set_children(scroll_node, vec![child_root]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let child_root_bounds = ui.debug_node_bounds(child_root).expect("child root bounds");
    let leaf_bounds = ui.debug_node_bounds(leaf).expect("leaf bounds");
    let content = scroll_handle.content_size();

    assert!(
        (content.height.0 - 320.0).abs() <= 0.5,
        "expected post-layout observed overflow to grow content height immediately: content={content:?} child_root={child_root_bounds:?} leaf={leaf_bounds:?}"
    );
    assert!(
        child_root_bounds.size.height.0 + 0.5 >= leaf_bounds.size.height.0,
        "expected child root geometry to relayout against the grown content bounds in the same frame: child_root={child_root_bounds:?} leaf={leaf_bounds:?} content={content:?}"
    );
}

#[test]
fn scroll_post_layout_does_not_grow_from_stale_nonleaf_wrapper_extent() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-nonleaf-overflow-gate",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node = ui.children(root)[0];
    let leaf = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(200.0)),
    });
    let child_root = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(320.0)),
        child: leaf,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(200.0)),
        ),
    });
    ui.set_children(child_root, vec![leaf]);
    ui.set_children(scroll_node, vec![child_root]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let child_root_bounds = ui.debug_node_bounds(child_root).expect("child root bounds");
    let leaf_bounds = ui.debug_node_bounds(leaf).expect("leaf bounds");
    let content = scroll_handle.content_size();
    let max = scroll_handle.max_offset().y;

    assert!(
        (content.height.0 - 200.0).abs() <= 0.5,
        "expected stale non-leaf wrapper bounds to stop at the real leaf extent instead of inflating the scroll range: content={content:?} max={max:?} child_root={child_root_bounds:?} leaf={leaf_bounds:?}"
    );
    assert!(
        max.0 <= 0.5,
        "expected max offset to remain collapsed when only a wrapper node, not real content, exceeds the viewport: content={content:?} max={max:?} child_root={child_root_bounds:?} leaf={leaf_bounds:?}"
    );
}

#[test]
fn scroll_post_layout_edge_revalidation_reuses_previous_extent_for_deep_scan() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-edge-revalidation-deep-scan",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(800.0)),
    });
    let mid0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(400.0)),
        layout_size: Size::new(Px(120.0), Px(400.0)),
        child: leaf0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(800.0)),
        ),
    });
    let child_root0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: mid0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(400.0)),
        ),
    });
    ui.set_children(mid0, vec![leaf0]);
    ui.set_children(child_root0, vec![mid0]);
    ui.set_children(scroll_node0, vec![child_root0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 200.0).abs() <= 0.5,
        "expected stale non-leaf wrappers to stay untrusted until edge revalidation confirms real descendant overflow: content={content0:?} max={max0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let wheel_pos = fret_core::Point::new(Px(8.0), Px(8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-48.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        (content1.height.0 - 800.0).abs() <= 0.5,
        "expected edge revalidation to preserve the previous observed extent long enough to deep-scan the true leaf bounds: before={content0:?} after={content1:?} max_before={max0:?} max_after={max1:?}"
    );
    assert!(
        (max1.0 - 600.0).abs() <= 0.5,
        "expected max offset to match the fully observed content height: content={content1:?} max={max1:?}"
    );
}

#[test]
fn scroll_post_layout_shrink_revalidation_clamps_stale_extent_after_content_contracts() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(800.0)),
    });
    let child_root0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(800.0)),
        ),
    });
    ui.set_children(child_root0, vec![leaf0]);
    ui.set_children(scroll_node0, vec![child_root0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 800.0).abs() <= 0.5,
        "expected initial post-layout observation to capture the tall leaf extent: content={content0:?} max={max0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let child_root1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(child_root1, vec![leaf1]);
    ui.set_children(scroll_node1, vec![child_root1]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 200.0).abs() <= 0.5,
        "expected shrink revalidation to clamp the stale post-layout extent back to the viewport-sized content: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 <= 0.5,
        "expected max offset to collapse once the tall content contracts below the viewport: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp to the shrunken post-layout extent: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_shrink_revalidation_clamps_stale_extent_when_only_descendant_dirty_off_edge()
{
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-descendant-off-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(800.0)),
    });
    let child_root0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(800.0)),
        ),
    });
    ui.set_children(child_root0, vec![leaf0]);
    ui.set_children(scroll_node0, vec![child_root0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 800.0).abs() <= 0.5,
        "expected initial post-layout observation to capture the tall leaf extent: content={content0:?} max={max0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(120.0)));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-descendant-off-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let child_root1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(child_root1, vec![leaf1]);
    ui.set_children(scroll_node1, vec![child_root1]);

    assert!(
        ui.node_needs_layout(leaf1),
        "expected descendant leaf to be layout-invalidated on the contracted frame"
    );
    ui.test_set_layout_invalidation(child_root1, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 200.0).abs() <= 0.5,
        "expected post-layout shrink to clamp stale extent even when only a descendant remains layout-dirty off-edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 <= 0.5,
        "expected max offset to collapse after the contracted descendant is revalidated off-edge: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp after off-edge descendant-only shrink revalidation: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_shrink_revalidation_clamps_stale_extent_for_multi_child_roots() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation-multi-child",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(120.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(700.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 800.0).abs() <= 0.5,
        "expected initial post-layout observation to capture the deepest multi-child descendant extent: content={content0:?} max={max0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation-multi-child",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(120.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 200.0).abs() <= 0.5,
        "expected post-layout shrink revalidation to clamp stale multi-child extent back to the viewport-sized content: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 <= 0.5,
        "expected max offset to collapse once the deepest child root contracts back within the viewport: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp after multi-child post-layout shrink: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_grows_extent_when_only_descendant_dirty_off_edge() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&show_more, Invalidation::Layout);
                                let expanded =
                                    cx.app.models().get_copied(&show_more).unwrap_or(false);
                                let rows = if expanded { 24 } else { 6 };

                                vec![cx.column(
                                    crate::element::ColumnProps {
                                        gap: Px(0.0).into(),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        (0..rows)
                                            .map(|i| cx.text(format!("row {i}")))
                                            .collect::<Vec<_>>()
                                    },
                                )]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(max0.0 > 0.0, "expected initial non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));
    let _ = show_more.update(&mut app, |v, _cx| *v = true);
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(
        ui.node_needs_layout(observed_container),
        "expected descendant container to be layout-invalidated after the model update"
    );
    ui.test_set_layout_invalidation(child_root, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        content1.height.0 > content0.height.0 + 0.5,
        "expected post-layout extent to grow even when only a descendant remains dirty off-edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected max offset to grow when descendant-only invalidation expands content off-edge: before={max0:?} after={max1:?}"
    );
    assert!(
        scroll_handle.offset().y.0 < max0.0 - 0.5,
        "expected test to stay off the previous max edge before relayout: offset={:?} max0={max0:?}",
        scroll_handle.offset().y,
    );

    let _ = root0;
}

#[test]
fn scroll_extent_updates_when_descendant_invalidated_but_child_root_cleared() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&show_more, Invalidation::Layout);
                                let expanded =
                                    cx.app.models().get_copied(&show_more).unwrap_or(false);
                                let rows = if expanded { 24 } else { 6 };

                                vec![cx.column(
                                    crate::element::ColumnProps {
                                        gap: Px(0.0).into(),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        (0..rows)
                                            .map(|i| cx.text(format!("row {i}")))
                                            .collect::<Vec<_>>()
                                    },
                                )]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    // Frame 0: establish content extent and scroll to the end so the next frame is at the
    // previous max offset.
    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let max0 = scroll_handle.max_offset().y;
    assert!(max0.0 > 0.0, "expected a non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = show_more.update(&mut app, |v, _cx| *v = true);
    app.advance_frame();

    // Frame 1: content grows, but simulate a bug where the scroll's direct child root loses its
    // layout invalidation flag while a deeper descendant remains dirty.
    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(
        ui.node_needs_layout(observed_container),
        "expected the observed descendant to be layout-invalidated after the model update"
    );
    ui.test_set_layout_invalidation(child_root, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let max1 = scroll_handle.max_offset().y;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected scroll extent to grow even when a descendant invalidation does not bubble to the scroll child root: before={max0:?} after={max1:?}"
    );

    // Ensure the test's assumptions still hold (i.e. we stayed at the previous extent edge
    // before relayout).
    assert!(
        scroll_handle.offset().y.0 + 0.5 >= max0.0,
        "expected scroll handle to remain at the previous max offset before relayout: offset={:?} max0={max0:?}",
        scroll_handle.offset().y
    );

    // Silence unused warning if the debug build retains the node id.
    let _ = root0;
}

#[test]
fn scroll_post_layout_shrink_revalidation_clamps_stale_extent_for_multi_child_roots_at_edge() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation-multi-child-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(120.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(700.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 800.0).abs() <= 0.5,
        "expected initial multi-child edge test to capture the deepest descendant extent: content={content0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let wheel_pos = fret_core::Point::new(Px(8.0), Px(8.0));

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-revalidation-multi-child-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(120.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-48.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 200.0).abs() <= 0.5,
        "expected direct-invalidated multi-child roots to shrink immediately at edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 <= 0.5,
        "expected max offset to clamp to zero after at-edge multi-child shrink: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp within the shrunken multi-child extent at edge: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_shrink_revalidation_clamps_multi_child_descendant_only_off_edge() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-multi-descendant-off-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(800.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(800.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(600.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(600.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    assert!(
        (content0.height.0 - 800.0).abs() <= 0.5,
        "expected initial multi-child post-layout observation to capture the tallest descendant: content={content0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(120.0)));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-shrink-multi-descendant-off-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1_new = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(100.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_a1_new,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(100.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(600.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(200.0)),
        layout_size: Size::new(Px(120.0), Px(200.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(600.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1_new]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(leaf_a1_new));
    ui.test_set_layout_invalidation(root_a1, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 600.0).abs() <= 0.5,
        "expected multi-child descendant-only off-edge shrink to clamp to the remaining tallest root: before={content0:?} after={content1:?}"
    );
    assert!(
        (max1.0 - 400.0).abs() <= 0.5,
        "expected max offset to reflect the remaining 600px descendant extent: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp within the shrunken multi-child extent: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_grows_multi_child_descendant_only_off_edge() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&show_more, Invalidation::Layout);
                                let expanded =
                                    cx.app.models().get_copied(&show_more).unwrap_or(false);
                                let rows = if expanded { 24 } else { 6 };
                                vec![cx.column(
                                    crate::element::ColumnProps {
                                        gap: Px(0.0).into(),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        (0..rows)
                                            .map(|i| cx.text(format!("row {i}")))
                                            .collect::<Vec<_>>()
                                    },
                                )]
                            }),
                        ]
                    }),
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![cx.column(
                            crate::element::ColumnProps {
                                gap: Px(0.0).into(),
                                ..Default::default()
                            },
                            move |cx| {
                                (0..12)
                                    .map(|i| cx.text(format!("stable {i}")))
                                    .collect::<Vec<_>>()
                            },
                        )]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-multi-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(max0.0 > 0.0, "expected initial non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));
    let _ = show_more.update(&mut app, |v, _cx| *v = true);
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-multi-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );

    let scroll_node1 = ui.children(root1)[0];
    let dynamic_root1 = ui.children(scroll_node1)[0];
    let observed_container = ui.children(dynamic_root1)[0];

    assert!(ui.node_needs_layout(observed_container));
    ui.test_set_layout_invalidation(dynamic_root1, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        content1.height.0 > content0.height.0 + 0.5,
        "expected multi-child descendant-only off-edge growth to expand content extent: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected max offset to grow when one dirty descendant expands inside a multi-child scroll root: before={max0:?} after={max1:?}"
    );
    assert!(
        scroll_handle.offset().y.0 < max0.0 - 0.5,
        "expected test to stay off the previous max edge before relayout: offset={:?} max0={max0:?}",
        scroll_handle.offset().y,
    );

    let _ = root0;
}

#[test]
fn scroll_axis_both_grows_width_when_only_descendant_dirty_off_edge() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&wide, Invalidation::Layout);
                                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                                let mut content = crate::element::ContainerProps::default();
                                content.layout.size.width = crate::element::Length::Px(if wide {
                                    Px(260.0)
                                } else {
                                    Px(140.0)
                                });
                                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                                vec![cx.container(content, |_cx| Vec::new())]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().x;
    assert!(max0.0 > 0.0, "expected initial non-zero x scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(12.0), Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = true);
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(ui.node_needs_layout(observed_container));
    ui.test_set_layout_invalidation(child_root, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    assert!(
        content1.width.0 > content0.width.0 + 0.5,
        "expected x content extent to grow even when only a descendant remains dirty off-edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected x max offset to grow under descendant-only invalidation off-edge: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_axis_both_shrinks_width_when_only_descendant_dirty_off_edge() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(true);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&wide, Invalidation::Layout);
                                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                                let mut content = crate::element::ContainerProps::default();
                                content.layout.size.width = crate::element::Length::Px(if wide {
                                    Px(260.0)
                                } else {
                                    Px(140.0)
                                });
                                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                                vec![cx.container(content, |_cx| Vec::new())]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-shrink-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().x;
    assert!(max0.0 > 0.0, "expected initial non-zero x scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(12.0), Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = false);
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-shrink-off-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(ui.node_needs_layout(observed_container));
    ui.test_set_layout_invalidation(child_root, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    let off1 = scroll_handle.offset().x;
    assert!(
        content1.width.0 + 0.5 < content0.width.0,
        "expected x content extent to shrink when only a descendant remains dirty off-edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 + 0.5 < max0.0,
        "expected x max offset to shrink under descendant-only invalidation off-edge: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected x offset to clamp after descendant-only shrink off-edge: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_axis_both_grows_width_when_only_descendant_dirty_at_edge() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&wide, Invalidation::Layout);
                                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                                let mut content = crate::element::ContainerProps::default();
                                content.layout.size.width = crate::element::Length::Px(if wide {
                                    Px(260.0)
                                } else {
                                    Px(140.0)
                                });
                                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                                vec![cx.container(content, |_cx| Vec::new())]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().x;
    assert!(max0.0 > 0.0, "expected initial non-zero x scroll range");
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = true);
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(ui.node_needs_layout(observed_container));
    ui.test_set_layout_invalidation(child_root, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    assert!(
        content1.width.0 > content0.width.0 + 0.5,
        "expected x content extent to grow even when only a descendant remains dirty at edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected x max offset to grow under descendant-only invalidation at edge: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_axis_both_shrinks_width_when_only_descendant_dirty_at_edge() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(true);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![
                    cx.container(crate::element::ContainerProps::default(), move |cx| {
                        vec![
                            cx.container(crate::element::ContainerProps::default(), move |cx| {
                                cx.observe_model(&wide, Invalidation::Layout);
                                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                                let mut content = crate::element::ContainerProps::default();
                                content.layout.size.width = crate::element::Length::Px(if wide {
                                    Px(260.0)
                                } else {
                                    Px(140.0)
                                });
                                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                                vec![cx.container(content, |_cx| Vec::new())]
                            }),
                        ]
                    }),
                ]
            },
        )]
    }

    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-shrink-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().x;
    assert!(max0.0 > 0.0, "expected initial non-zero x scroll range");
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = false);
    app.advance_frame();

    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-shrink-edge-descendant-invalidation",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let child_root = ui.children(scroll_node)[0];
    let observed_container = ui.children(child_root)[0];

    assert!(ui.node_needs_layout(observed_container));
    ui.test_set_layout_invalidation(child_root, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    let off1 = scroll_handle.offset().x;
    assert!(
        content1.width.0 + 0.5 < content0.width.0,
        "expected x content extent to shrink when only a descendant remains dirty at edge: before={content0:?} after={content1:?}"
    );
    assert!(
        max1.0 + 0.5 < max0.0,
        "expected x max offset to shrink under descendant-only invalidation at edge: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected x offset to clamp after descendant-only shrink at edge: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn outer_y_scroll_does_not_count_nested_both_scroll_descendant_overflow() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "outer-y-scroll-does-not-count-nested-both-scroll-descendant-overflow",
        {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();
            move |cx| {
                let mut outer_layout = crate::element::LayoutStyle::default();
                outer_layout.size.width = crate::element::Length::Fill;
                outer_layout.size.height = crate::element::Length::Fill;
                outer_layout.overflow = crate::element::Overflow::Clip;

                vec![cx.scroll(
                    crate::element::ScrollProps {
                        layout: outer_layout,
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(outer_handle.clone()),
                        probe_unbounded: true,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.column(
                            crate::element::ColumnProps {
                                gap: Px(0.0).into(),
                                ..Default::default()
                            },
                            {
                                let inner_handle = inner_handle.clone();
                                move |cx| {
                                    let top = cx.container(
                                        crate::element::ContainerProps {
                                            layout: crate::element::LayoutStyle {
                                                size: crate::element::SizeStyle {
                                                    width: crate::element::Length::Fill,
                                                    height: crate::element::Length::Px(Px(200.0)),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |_cx| Vec::new(),
                                    );

                                    let mut inner_layout = crate::element::LayoutStyle::default();
                                    inner_layout.size.width = crate::element::Length::Fill;
                                    inner_layout.size.height =
                                        crate::element::Length::Px(Px(400.0));
                                    inner_layout.overflow = crate::element::Overflow::Clip;

                                    let inner = cx.scroll(
                                        crate::element::ScrollProps {
                                            layout: inner_layout,
                                            axis: crate::element::ScrollAxis::Both,
                                            scroll_handle: Some(inner_handle.clone()),
                                            probe_unbounded: true,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.container(
                                                crate::element::ContainerProps {
                                                    layout: crate::element::LayoutStyle {
                                                        size: crate::element::SizeStyle {
                                                            width: crate::element::Length::Px(Px(
                                                                320.0,
                                                            )),
                                                            height: crate::element::Length::Px(Px(
                                                                1100.0,
                                                            )),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                |_cx| Vec::new(),
                                            )]
                                        },
                                    );

                                    let bottom = cx.container(
                                        crate::element::ContainerProps {
                                            layout: crate::element::LayoutStyle {
                                                size: crate::element::SizeStyle {
                                                    width: crate::element::Length::Fill,
                                                    height: crate::element::Length::Px(Px(232.0)),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |_cx| Vec::new(),
                                    );

                                    vec![top, inner, bottom]
                                }
                            },
                        )]
                    },
                )]
            }
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let inner_content = inner_handle.content_size();
    assert!(
        (inner_content.height.0 - 1100.0).abs() <= 0.5,
        "expected inner scroll content height to track its tall child: inner_content={inner_content:?}"
    );

    let outer_content = outer_handle.content_size();
    assert!(
        (outer_content.height.0 - 832.0).abs() <= 0.5,
        "expected outer scroll content height to use the nested viewport height (200 + 400 + 232), not the nested scroll's internal 1100px content: outer_content={outer_content:?} inner_content={inner_content:?}"
    );
    assert!(
        (outer_handle.max_offset().y.0 - 632.0).abs() <= 0.5,
        "expected outer max offset to match the clipped nested viewport contribution: outer_content={outer_content:?} viewport={:?} max_offset={:?}",
        outer_handle.viewport_size(),
        outer_handle.max_offset(),
    );
}

#[test]
fn outer_y_scroll_does_not_count_descendant_overflow_behind_clipped_wrapper() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let outer_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "outer-y-scroll-does-not-count-descendant-overflow-behind-clipped-wrapper",
        {
            let outer_handle = outer_handle.clone();
            move |cx| {
                let mut outer_layout = crate::element::LayoutStyle::default();
                outer_layout.size.width = crate::element::Length::Fill;
                outer_layout.size.height = crate::element::Length::Fill;
                outer_layout.overflow = crate::element::Overflow::Clip;

                vec![cx.scroll(
                    crate::element::ScrollProps {
                        layout: outer_layout,
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(outer_handle.clone()),
                        probe_unbounded: true,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.column(
                            crate::element::ColumnProps {
                                gap: Px(0.0).into(),
                                ..Default::default()
                            },
                            move |cx| {
                                let top = cx.container(
                                    crate::element::ContainerProps {
                                        layout: crate::element::LayoutStyle {
                                            size: crate::element::SizeStyle {
                                                width: crate::element::Length::Fill,
                                                height: crate::element::Length::Px(Px(200.0)),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                );

                                let clipped_wrapper = cx.container(
                                    crate::element::ContainerProps {
                                        layout: crate::element::LayoutStyle {
                                            size: crate::element::SizeStyle {
                                                width: crate::element::Length::Fill,
                                                height: crate::element::Length::Px(Px(400.0)),
                                                ..Default::default()
                                            },
                                            overflow: crate::element::Overflow::Clip,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |cx| {
                                        vec![cx.container(
                                            crate::element::ContainerProps {
                                                layout: crate::element::LayoutStyle {
                                                    size: crate::element::SizeStyle {
                                                        width: crate::element::Length::Px(Px(
                                                            320.0,
                                                        )),
                                                        height: crate::element::Length::Px(Px(
                                                            1100.0,
                                                        )),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::new(),
                                        )]
                                    },
                                );

                                let bottom = cx.container(
                                    crate::element::ContainerProps {
                                        layout: crate::element::LayoutStyle {
                                            size: crate::element::SizeStyle {
                                                width: crate::element::Length::Fill,
                                                height: crate::element::Length::Px(Px(232.0)),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                );

                                vec![top, clipped_wrapper, bottom]
                            },
                        )]
                    },
                )]
            }
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer_content = outer_handle.content_size();
    assert!(
        (outer_content.height.0 - 832.0).abs() <= 0.5,
        "expected outer scroll content height to stop at the clipped wrapper height (200 + 400 + 232), not the clipped descendant's internal 1100px content: outer_content={outer_content:?}"
    );
    assert!(
        (outer_handle.max_offset().y.0 - 632.0).abs() <= 0.5,
        "expected outer max offset to match the clipped wrapper contribution: outer_content={outer_content:?} viewport={:?} max_offset={:?}",
        outer_handle.viewport_size(),
        outer_handle.max_offset(),
    );
}

#[test]
fn scroll_axis_both_mixed_child_invalidation_keeps_descendant_only_growth_authoritative_at_edge() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-mixed-child-invalidation-growth-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Both,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(150.0), Px(40.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(150.0), Px(40.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(140.0), Px(40.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(140.0), Px(40.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let max0 = scroll_handle.max_offset().x;
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-mixed-child-invalidation-growth-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Both,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(160.0), Px(40.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(40.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(260.0), Px(40.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(40.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    assert!(
        (content1.width.0 - 260.0).abs() <= 0.5,
        "expected descendant-only x-growth root to remain authoritative at edge even when a sibling root is directly invalidated: after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected x max offset to reflect the 260px descendant-only growth root at edge: content={content1:?} max={max1:?}"
    );
}

#[test]
fn scroll_axis_both_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(48.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-mixed-child-invalidation-shrink-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Both,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(240.0), Px(40.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(40.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(280.0), Px(40.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(280.0), Px(40.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let max0 = scroll_handle.max_offset().x;
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-mixed-child-invalidation-shrink-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Both,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(260.0), Px(40.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(40.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(140.0), Px(40.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(48.0)),
        layout_size: Size::new(Px(120.0), Px(48.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(140.0), Px(40.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().x;
    let off1 = scroll_handle.offset().x;
    assert!(
        (content1.width.0 - 260.0).abs() <= 0.5,
        "expected descendant-only x-shrink root to stop dominating at edge once a sibling root remains directly invalidated: after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected x max offset to reflect the remaining 260px directly-invalidated root at edge: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected x offset to clamp within the mixed-child shrink extent at edge: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame() {
    const WRAPPER_CHAIN: usize = 8;
    const FANOUT_CHILDREN: usize = 2500;

    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.size
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct PassThroughMeasureLayoutNode {
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for PassThroughMeasureLayoutNode {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            cx.measure_in(self.child, cx.constraints)
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    struct FanoutMeasureLayoutNode {
        layout_size: Size,
        children: Vec<NodeId>,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for FanoutMeasureLayoutNode {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            let mut size = Size::new(Px(0.0), Px(0.0));
            for &child in &self.children {
                let child_size = cx.measure_in(child, cx.constraints);
                size.width = Px(size.width.0.max(child_size.width.0));
                size.height = Px(size.height.0.max(child_size.height.0));
            }
            size
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in &self.children {
                let _ = cx.layout_in(child, self.child_rect);
            }
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in &self.children {
                cx.paint(child, self.child_rect);
            }
        }
    }

    fn build_small_tree(ui: &mut UiTree<TestHost>, leaf_height: Px) -> NodeId {
        ui.create_node(FixedLeaf {
            size: Size::new(Px(120.0), leaf_height),
        })
    }

    fn build_budget_tree(ui: &mut UiTree<TestHost>, overflowing_leaf_height: Px) -> NodeId {
        let viewport_size = Size::new(Px(120.0), Px(120.0));
        let same_bounds_rect = Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), viewport_size);

        let mut wrappers: Vec<NodeId> = Vec::with_capacity(FANOUT_CHILDREN);
        for index in 0..FANOUT_CHILDREN {
            let leaf_height = if index + 1 == FANOUT_CHILDREN {
                overflowing_leaf_height
            } else {
                Px(120.0)
            };
            let leaf_size = Size::new(Px(120.0), leaf_height);
            let leaf = ui.create_node(FixedLeaf { size: leaf_size });
            let wrapper = ui.create_node(PassThroughMeasureLayoutNode {
                layout_size: viewport_size,
                child: leaf,
                child_rect: Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), leaf_size),
            });
            ui.set_children(wrapper, vec![leaf]);
            wrappers.push(wrapper);
        }

        let fanout = ui.create_node(FanoutMeasureLayoutNode {
            layout_size: viewport_size,
            children: wrappers.clone(),
            child_rect: same_bounds_rect,
        });
        ui.set_children(fanout, wrappers);

        let mut child = fanout;
        for _ in 0..WRAPPER_CHAIN {
            let parent = ui.create_node(PassThroughMeasureLayoutNode {
                layout_size: viewport_size,
                child,
                child_rect: same_bounds_rect,
            });
            ui.set_children(parent, vec![child]);
            child = parent;
        }

        child
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-budget-hit-growth-converges-next-frame",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let tree0 = build_small_tree(&mut ui, Px(140.0));
    ui.set_children(scroll_node0, vec![tree0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 140.0).abs() <= 0.5,
        "expected initial probe layout to establish the 140px content height: content={content0:?}"
    );
    assert!(max0.0 > 0.0, "expected initial non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));

    let scroll_element = ui
        .node_element(scroll_node0)
        .expect("expected scroll host node to carry an element id");
    let pending_probe0 = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        !pending_probe0,
        "expected small initial tree to avoid pre-seeding a pending extent probe"
    );

    let tree1 = build_budget_tree(&mut ui, Px(260.0));
    ui.set_children(scroll_node0, vec![tree1]);
    ui.test_set_layout_invalidation(tree1, false);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        (content1.height.0 - 140.0).abs() <= 0.5,
        "expected first post-layout frame to remain stale when wrapper/deep-scan budgets hit before the leaf frontier: after={content1:?}"
    );
    assert!(
        max1.0 <= max0.0 + 0.5,
        "expected first post-layout frame to avoid falsely growing range before pending probe runs: before={max0:?} after={max1:?}"
    );

    let pending_probe = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        pending_probe,
        "expected observation budget hit to schedule a pending extent probe for the next frame"
    );

    let tree2 = build_budget_tree(&mut ui, Px(260.0));
    ui.set_children(scroll_node0, vec![tree2]);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content2 = scroll_handle.content_size();
    let max2 = scroll_handle.max_offset().y;
    assert!(
        (content2.height.0 - 260.0).abs() <= 0.5,
        "expected pending probe to converge to the 260px descendant frontier on the next frame: content={content2:?}"
    );
    assert!(
        (max2.0 - 140.0).abs() <= 0.5,
        "expected max offset to converge after the pending probe frame: content={content2:?} max={max2:?}"
    );

    let pending_probe_after = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        !pending_probe_after,
        "expected pending extent probe to clear after the converged follow-up frame"
    );
}

#[test]
fn scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame() {
    const WRAPPER_CHAIN: usize = 8;
    const FANOUT_CHILDREN: usize = 2500;

    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.size
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct PassThroughMeasureLayoutNode {
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for PassThroughMeasureLayoutNode {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            cx.measure_in(self.child, cx.constraints)
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    struct FanoutMeasureLayoutNode {
        layout_size: Size,
        children: Vec<NodeId>,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for FanoutMeasureLayoutNode {
        fn measure(&mut self, cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            let mut size = Size::new(Px(0.0), Px(0.0));
            for &child in &self.children {
                let child_size = cx.measure_in(child, cx.constraints);
                size.width = Px(size.width.0.max(child_size.width.0));
                size.height = Px(size.height.0.max(child_size.height.0));
            }
            size
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in &self.children {
                let _ = cx.layout_in(child, self.child_rect);
            }
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in &self.children {
                cx.paint(child, self.child_rect);
            }
        }
    }

    fn build_small_tree(ui: &mut UiTree<TestHost>, leaf_height: Px) -> NodeId {
        ui.create_node(FixedLeaf {
            size: Size::new(Px(120.0), leaf_height),
        })
    }

    fn build_budget_tree(ui: &mut UiTree<TestHost>, overflowing_leaf_height: Px) -> NodeId {
        let viewport_size = Size::new(Px(120.0), Px(120.0));
        let same_bounds_rect = Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), viewport_size);

        let mut wrappers: Vec<NodeId> = Vec::with_capacity(FANOUT_CHILDREN);
        for index in 0..FANOUT_CHILDREN {
            let leaf_height = if index + 1 == FANOUT_CHILDREN {
                overflowing_leaf_height
            } else {
                Px(120.0)
            };
            let leaf_size = Size::new(Px(120.0), leaf_height);
            let leaf = ui.create_node(FixedLeaf { size: leaf_size });
            let wrapper = ui.create_node(PassThroughMeasureLayoutNode {
                layout_size: viewport_size,
                child: leaf,
                child_rect: Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), leaf_size),
            });
            ui.set_children(wrapper, vec![leaf]);
            wrappers.push(wrapper);
        }

        let fanout = ui.create_node(FanoutMeasureLayoutNode {
            layout_size: viewport_size,
            children: wrappers.clone(),
            child_rect: same_bounds_rect,
        });
        ui.set_children(fanout, wrappers);

        let mut child = fanout;
        for _ in 0..WRAPPER_CHAIN {
            let parent = ui.create_node(PassThroughMeasureLayoutNode {
                layout_size: viewport_size,
                child,
                child_rect: same_bounds_rect,
            });
            ui.set_children(parent, vec![child]);
            child = parent;
        }

        child
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-budget-hit-shrink-converges-next-frame",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let tree0 = build_small_tree(&mut ui, Px(260.0));
    ui.set_children(scroll_node0, vec![tree0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    let max0 = scroll_handle.max_offset().y;
    assert!(
        (content0.height.0 - 260.0).abs() <= 0.5,
        "expected initial probe layout to establish the 260px content height: content={content0:?}"
    );
    assert!(max0.0 > 0.0, "expected initial non-zero scroll range");
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));

    let scroll_element = ui
        .node_element(scroll_node0)
        .expect("expected scroll host node to carry an element id");
    let pending_probe0 = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        !pending_probe0,
        "expected small initial tree to avoid pre-seeding a pending extent probe"
    );

    let tree1 = build_budget_tree(&mut ui, Px(140.0));
    ui.set_children(scroll_node0, vec![tree1]);
    ui.test_set_layout_invalidation(tree1, false);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 260.0).abs() <= 0.5,
        "expected first post-layout shrink frame to remain stale when wrapper/deep-scan budgets hit before shrink recovery: after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected first post-layout shrink frame to preserve the previous max offset before pending probe runs: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to stay clamped within the stale range before shrink recovery: offset={off1:?} max={max1:?} content={content1:?}"
    );

    let pending_probe = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        pending_probe,
        "expected shrink-side observation budget hit to schedule a pending extent probe for the next frame"
    );

    let tree2 = build_budget_tree(&mut ui, Px(140.0));
    ui.set_children(scroll_node0, vec![tree2]);

    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content2 = scroll_handle.content_size();
    let max2 = scroll_handle.max_offset().y;
    let off2 = scroll_handle.offset().y;
    assert!(
        (content2.height.0 - 140.0).abs() <= 0.5,
        "expected pending probe to converge to the 140px descendant frontier on the next frame: content={content2:?}"
    );
    assert!(
        (max2.0 - 20.0).abs() <= 0.5,
        "expected max offset to shrink after the pending probe frame: content={content2:?} max={max2:?}"
    );
    assert!(
        off2.0 <= max2.0 + 0.5,
        "expected offset to clamp after shrink recovery: offset={off2:?} max={max2:?} content={content2:?}"
    );

    let pending_probe_after = crate::elements::with_element_state(
        &mut app,
        window,
        scroll_element,
        crate::element::ScrollState::default,
        |state| state.pending_extent_probe,
    );
    assert!(
        !pending_probe_after,
        "expected pending extent probe to clear after the shrink recovery frame"
    );
}

#[test]
fn scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_growth_authoritative() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-growth",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(150.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(150.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(140.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(140.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    assert!(
        (content0.height.0 - 150.0).abs() <= 0.5,
        "expected initial extent to follow the tallest child root: content={content0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-growth",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(160.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(160.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(260.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(260.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        (content1.height.0 - 260.0).abs() <= 0.5,
        "expected descendant-only growth child root to remain authoritative even when another child root is directly invalidated: before={content0:?} after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected max offset to reflect the 260px descendant-only growth root: content={content1:?} max={max1:?}"
    );
}

#[test]
fn scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative() {
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-shrink",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(240.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(240.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(280.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(280.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content0 = scroll_handle.content_size();
    assert!(
        (content0.height.0 - 280.0).abs() <= 0.5,
        "expected initial extent to follow the tallest child root: content={content0:?}"
    );
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(12.0)));
    app.advance_frame();

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-shrink",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(260.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(260.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(140.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(140.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 260.0).abs() <= 0.5,
        "expected descendant-only shrink child root to stop dominating once another child root remains directly invalidated: before={content0:?} after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected max offset to reflect the remaining 260px directly-invalidated root: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp within the mixed-child shrink extent: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_growth_authoritative_at_edge()
{
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-growth-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(150.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(150.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(140.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(140.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let wheel_pos = fret_core::Point::new(Px(8.0), Px(8.0));

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-growth-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(160.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(160.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(260.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(260.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-48.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    assert!(
        (content1.height.0 - 260.0).abs() <= 0.5,
        "expected descendant-only growth root to remain authoritative at edge even when a sibling root is directly invalidated: after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected max offset to reflect the 260px descendant-only growth root at edge: content={content1:?} max={max1:?}"
    );
}

#[test]
fn scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge()
{
    struct FixedLeaf {
        size: Size,
    }

    impl<H: UiHost> Widget<H> for FixedLeaf {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            self.size
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    struct MeasureLayoutNode {
        measured: Size,
        layout_size: Size,
        child: NodeId,
        child_rect: Rect,
    }

    impl<H: UiHost> Widget<H> for MeasureLayoutNode {
        fn measure(&mut self, _cx: &mut crate::widget::MeasureCx<'_, H>) -> Size {
            self.measured
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_in(self.child, self.child_rect);
            self.layout_size
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.paint(self.child, self.child_rect);
        }
    }

    let cfg = crate::runtime_config::ui_runtime_config().clone();
    let _cfg_guard = crate::runtime_config::scoped_ui_runtime_config_test_override(cfg);

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-shrink-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node0 = ui.children(root0)[0];
    let leaf_a0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(240.0)),
    });
    let root_a0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(240.0)),
        ),
    });
    let leaf_b0 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(280.0)),
    });
    let root_b0 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b0,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(280.0)),
        ),
    });
    ui.set_children(root_a0, vec![leaf_a0]);
    ui.set_children(root_b0, vec![leaf_b0]);
    ui.set_children(scroll_node0, vec![root_a0, root_b0]);

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let wheel_pos = fret_core::Point::new(Px(8.0), Px(8.0));

    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-post-layout-mixed-child-invalidation-shrink-edge",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(scroll_handle.clone()),
                    probe_unbounded: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    let scroll_node1 = ui.children(root1)[0];
    let leaf_a1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(260.0)),
    });
    let root_a1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_a1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(260.0)),
        ),
    });
    let leaf_b1 = ui.create_node(FixedLeaf {
        size: Size::new(Px(120.0), Px(140.0)),
    });
    let root_b1 = ui.create_node(MeasureLayoutNode {
        measured: Size::new(Px(120.0), Px(120.0)),
        layout_size: Size::new(Px(120.0), Px(120.0)),
        child: leaf_b1,
        child_rect: Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(140.0)),
        ),
    });
    ui.set_children(root_a1, vec![leaf_a1]);
    ui.set_children(root_b1, vec![leaf_b1]);
    ui.set_children(scroll_node1, vec![root_a1, root_b1]);

    assert!(ui.node_needs_layout(root_a1));
    assert!(ui.node_needs_layout(leaf_b1));
    ui.test_set_layout_invalidation(root_b1, false);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-48.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    app.advance_frame();
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let content1 = scroll_handle.content_size();
    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        (content1.height.0 - 260.0).abs() <= 0.5,
        "expected descendant-only shrink root to stop dominating at edge once a sibling root remains directly invalidated: after={content1:?}"
    );
    assert!(
        (max1.0 - 140.0).abs() <= 0.5,
        "expected max offset to reflect the remaining 260px directly-invalidated root at edge: content={content1:?} max={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp within the mixed-child shrink extent at edge: offset={off1:?} max={max1:?} content={content1:?}"
    );
}

#[test]
fn scroll_thumb_drag_updates_offset_horizontal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar track has enough room for Radix-aligned padding + min thumb.
            stack_layout.size.height = crate::element::Length::Px(Px(30.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            axis: crate::element::ScrollAxis::X,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            let mut content_layout = crate::element::LayoutStyle::default();
                            content_layout.size.width = crate::element::Length::Px(Px(300.0));
                            content_layout.size.height = crate::element::Length::Fill;

                            vec![cx.container(
                                crate::element::ContainerProps {
                                    layout: content_layout,
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("abc")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: None.into(),
                            right: Some(Px(0.0)).into(),
                            bottom: Some(Px(0.0)).into(),
                            left: Some(Px(0.0)).into(),
                        },
                        size: crate::element::SizeStyle {
                            height: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let content_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds");

    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let thumb = crate::declarative::paint_helpers::scrollbar_thumb_rect_horizontal(
        scrollbar_bounds,
        scroll_handle.viewport_size().width,
        scroll_handle.content_size().width,
        scroll_handle.offset().x,
        crate::element::ScrollbarStyle::default().track_padding,
    )
    .expect("horizontal thumb rect");
    let down_pos = fret_core::Point::new(Px(thumb.origin.x.0 + 1.0), Px(thumb.origin.y.0 + 1.0));
    let move_pos = fret_core::Point::new(Px(down_pos.x.0 + 12.0), down_pos.y);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured(),
        Some(scrollbar_node),
        "expected horizontal thumb down to capture the pointer on the scrollbar node"
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().x
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds after drag");

    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected content to move left after thumb drag: before={:?} after={:?}",
        before.origin.x,
        after.origin.x
    );
}

#[test]
fn scroll_rounds_scrollable_extent_up_to_next_pixel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_root = handle.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-rounding",
        move |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;

            let mut child_layout = crate::element::LayoutStyle::default();
            child_layout.size.width = Length::Fill;
            child_layout.size.height = Length::Px(Px(100.2));

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(handle_for_root.clone()),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: child_layout,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("content")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max = handle.max_offset();
    assert!((max.y.0 - 51.0).abs() < 0.01, "max_offset.y={:?}", max.y);

    handle.scroll_to_offset(Point::new(Px(0.0), Px(60.0)));
    assert!(
        (handle.offset().y.0 - 51.0).abs() < 0.01,
        "offset.y={:?}",
        handle.offset().y
    );
}
