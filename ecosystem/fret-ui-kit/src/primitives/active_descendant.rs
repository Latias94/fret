//! Active-descendant helpers (ADR 0073 outcomes).

pub use crate::declarative::active_descendant::active_descendant_for_index;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
    use crate::declarative::model_watch::ModelWatchExt as _;
    use fret_app::App;
    use fret_core::{
        AppWindowId, NodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, SemanticsRole, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService, TextStyle,
    };
    use fret_ui::element::{
        ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, TextInputProps,
    };
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{Theme, ThemeConfig, UiTree};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
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
                let active_index = cx.watch_model(&active_index).cloned().unwrap_or_default();

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
}
