//! Active-descendant helpers (ADR 0073 outcomes).

pub use crate::declarative::active_descendant::{
    ActiveOption, active_descendant_for_index, active_option_for_index,
    scroll_active_element_align_top_y, scroll_active_element_into_view_y,
    scroll_handle_align_top_y, scroll_handle_into_view_y,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
    use crate::declarative::model_watch::ModelWatchExt as _;
    use fret_app::App;
    use fret_core::{
        AppWindowId, NodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, SemanticsRole, SvgId, SvgService, TextBlobId, TextConstraints,
        TextInput, TextMetrics, TextService,
    };
    use fret_ui::element::{
        ContainerProps, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y,
        PressableProps, ScrollProps, TextInputProps,
    };
    use fret_ui::elements::GlobalElementId;
    use fret_ui::scroll::ScrollHandle;
    use fret_ui::{Theme, ThemeConfig, UiTree};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        )
    }

    #[test]
    fn active_descendant_is_set_when_active_item_is_present_and_cleared_when_missing() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let show_second = app.models_mut().insert(true);
        let active_index = app.models_mut().insert(Some(1usize));
        let input_model = app.models_mut().insert(String::new());
        let input_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = bounds();

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            show_second: fret_runtime::Model<bool>,
            active_index: fret_runtime::Model<Option<usize>>,
            input_model: fret_runtime::Model<String>,
            input_id_out: Rc<Cell<Option<GlobalElementId>>>,
        ) -> NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let show_second = cx.watch_model(&show_second).copied().unwrap_or(true);
                let active_index = cx.watch_model(&active_index).cloned_or_default();

                let count = if show_second { 2 } else { 1 };

                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: LayoutStyle::default(),
                        direction: fret_core::Axis::Vertical,
                        gap: Px(0.0),
                        padding: fret_core::Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |cx| {
                        let mut option_elements = Vec::new();
                        let alpha = cx.pressable_with_id(
                            PressableProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                                focusable: false,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::ListBoxOption),
                                    label: Some(Arc::from("Alpha")),
                                    ..Default::default()
                                }
                                .with_collection_position(0, count),
                                ..Default::default()
                            },
                            |cx, _st, _id| {
                                vec![cx.container(ContainerProps::default(), |_| Vec::new())]
                            },
                        );
                        option_elements.push(alpha.id);

                        let beta = if show_second {
                            let beta = cx.pressable_with_id(
                                PressableProps {
                                    layout: LayoutStyle::default(),
                                    enabled: true,
                                    focusable: false,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::ListBoxOption),
                                        label: Some(Arc::from("Beta")),
                                        ..Default::default()
                                    }
                                    .with_collection_position(1, count),
                                    ..Default::default()
                                },
                                |cx, _st, _id| {
                                    vec![cx.container(ContainerProps::default(), |_| Vec::new())]
                                },
                            );
                            option_elements.push(beta.id);
                            Some(beta)
                        } else {
                            None
                        };

                        let active_descendant =
                            active_descendant_for_index(cx, &option_elements, active_index);

                        let mut input_props = TextInputProps::new(input_model);
                        input_props.active_descendant = active_descendant;
                        input_props.layout.size.width = Length::Fill;
                        let input = cx.text_input(input_props);
                        input_id_out.set(Some(input.id));

                        let mut out = vec![input, alpha];
                        if let Some(beta) = beta {
                            out.push(beta);
                        }
                        out
                    },
                )]
            })
        }

        // Frame 1: build and focus the input.
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            show_second.clone(),
            active_index.clone(),
            input_model.clone(),
            input_id.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let input_element = input_id.get().expect("input element id");
        let input_node = fret_ui::elements::node_for_element(&mut app, window, input_element)
            .expect("input node");
        ui.set_focus(Some(input_node));

        // Frame 2: active option is present -> active_descendant is set.
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            show_second.clone(),
            active_index.clone(),
            input_model.clone(),
            input_id.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let input_element = input_id.get().expect("input element id");
        let input_node = fret_ui::elements::node_for_element(&mut app, window, input_element)
            .expect("input node");
        ui.set_focus(Some(input_node));
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused = snap.focus.expect("focus");
        assert_eq!(focused, input_node, "focus should remain on the input node");

        let input = snap
            .nodes
            .iter()
            .find(|n| n.id == focused)
            .expect("focused node present");
        let active = input
            .active_descendant
            .expect("active_descendant should be set while active item is visible");

        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");
        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));

        // Frame 3: remove the second item while keeping the same active index -> active_descendant clears.
        let _ = app.models_mut().update(&show_second, |v| *v = false);
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            show_second.clone(),
            active_index.clone(),
            input_model,
            input_id.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let input_element = input_id.get().expect("input element id");
        let input_node = fret_ui::elements::node_for_element(&mut app, window, input_element)
            .expect("input node");
        ui.set_focus(Some(input_node));
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused = snap.focus.expect("focus");
        assert_eq!(focused, input_node);
        let input = snap
            .nodes
            .iter()
            .find(|n| n.id == focused)
            .expect("focused node present");
        assert!(
            input.active_descendant.is_none(),
            "active_descendant should clear when the active option is not present"
        );
    }

    #[test]
    fn scrolls_active_option_into_view_while_focus_stays_in_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let active_index = app.models_mut().insert(Some(2usize));
        let input_model = app.models_mut().insert(String::new());

        let scroll_handle = ScrollHandle::default();
        let scroll_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let active_item_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let input_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = bounds();

        fn rects_intersect(a: Rect, b: Rect) -> bool {
            let ax1 = a.origin.x.0;
            let ay1 = a.origin.y.0;
            let ax2 = ax1 + a.size.width.0;
            let ay2 = ay1 + a.size.height.0;

            let bx1 = b.origin.x.0;
            let by1 = b.origin.y.0;
            let bx2 = bx1 + b.size.width.0;
            let by2 = by1 + b.size.height.0;

            ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            active_index: fret_runtime::Model<Option<usize>>,
            input_model: fret_runtime::Model<String>,
            scroll_handle: ScrollHandle,
            scroll_id_out: Rc<Cell<Option<GlobalElementId>>>,
            active_item_id_out: Rc<Cell<Option<GlobalElementId>>>,
            input_id_out: Rc<Cell<Option<GlobalElementId>>>,
        ) -> NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let active_index = cx.watch_model(&active_index).cloned_or_default();

                let mut option_elements: Vec<GlobalElementId> = Vec::new();
                let option = |cx: &mut fret_ui::ElementContext<'_, App>,
                              label: &'static str,
                              idx: usize,
                              count: usize| {
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Px(Px(20.0));
                                layout
                            },
                            enabled: true,
                            focusable: false,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::ListBoxOption),
                                label: Some(Arc::from(label)),
                                ..Default::default()
                            }
                            .with_collection_position(idx, count),
                            ..Default::default()
                        },
                        |cx, _st, _id| {
                            vec![cx.container(ContainerProps::default(), |_| Vec::new())]
                        },
                    )
                };

                let alpha = option(cx, "Alpha", 0, 3);
                option_elements.push(alpha.id);
                let beta = option(cx, "Beta", 1, 3);
                option_elements.push(beta.id);
                let gamma = option(cx, "Gamma", 2, 3);
                option_elements.push(gamma.id);

                if let Some(idx) = active_index
                    && let Some(el) = option_elements.get(idx).copied()
                {
                    active_item_id_out.set(Some(el));
                }

                let scroll = cx.scroll(
                    ScrollProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(40.0));
                            layout.overflow = Overflow::Clip;
                            layout
                        },
                        scroll_handle: Some(scroll_handle.clone()),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle::default(),
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: fret_core::Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Stretch,
                                wrap: false,
                            },
                            move |_cx| vec![alpha, beta, gamma],
                        )]
                    },
                );
                scroll_id_out.set(Some(scroll.id));

                // Scroll the active option into view (best effort) using last-frame bounds.
                if let (Some(viewport), Some(active)) =
                    (scroll_id_out.get(), active_item_id_out.get())
                {
                    scroll_active_element_into_view_y(cx, &scroll_handle, viewport, active);
                }

                let active_descendant =
                    active_descendant_for_index(cx, &option_elements, active_index);
                let mut input_props = TextInputProps::new(input_model);
                input_props.active_descendant = active_descendant;
                input_props.layout.size.width = Length::Fill;
                let input = cx.text_input(input_props);
                input_id_out.set(Some(input.id));

                vec![input, scroll]
            })
        }

        // Frame 1: establish last-frame bounds with scroll offset 0.
        app.set_frame_id(fret_runtime::FrameId(1));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            active_index.clone(),
            input_model.clone(),
            scroll_handle.clone(),
            scroll_id.clone(),
            active_item_id.clone(),
            input_id.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let input_element = input_id.get().expect("input element id");
        let input_node =
            fret_ui::elements::node_for_element(&mut app, window, input_element).expect("input");
        ui.set_focus(Some(input_node));

        // Frame 2: active option is off-screen; helper should scroll it into view without moving focus.
        app.set_frame_id(fret_runtime::FrameId(2));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            active_index,
            input_model,
            scroll_handle.clone(),
            scroll_id.clone(),
            active_item_id.clone(),
            input_id.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            scroll_handle.offset().y.0 > 0.0,
            "expected scroll handle to move the active option into view"
        );

        let scroll_element = scroll_id.get().expect("scroll element id");
        let scroll_node =
            fret_ui::elements::node_for_element(&mut app, window, scroll_element).expect("scroll");
        let active_element = active_item_id.get().expect("active option element id");
        let active_node =
            fret_ui::elements::node_for_element(&mut app, window, active_element).expect("active");

        let scroll_bounds = ui
            .debug_node_visual_bounds(scroll_node)
            .expect("scroll visual bounds");
        let active_bounds = ui
            .debug_node_visual_bounds(active_node)
            .expect("active visual bounds");
        assert!(
            rects_intersect(scroll_bounds, active_bounds),
            "expected active option to intersect the scroll viewport after scroll-into-view"
        );
        assert_eq!(
            ui.focus(),
            Some(input_node),
            "expected focus to remain on the input node"
        );
    }
}
