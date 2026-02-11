use super::super::super::*;

pub(in crate::ui) fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_field(cx)
}

pub(in crate::ui) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_forms(cx, text_input, text_area, checkbox, switch)
}

pub(in crate::ui) fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .trigger_test_id("ui-gallery-select-trigger")
        .placeholder("Pick a fruit")
        .items(
            [
                shadcn::SelectItem::new("apple", "Apple").test_id("ui-gallery-select-item-apple"),
                shadcn::SelectItem::new("banana", "Banana")
                    .test_id("ui-gallery-select-item-banana"),
                shadcn::SelectItem::new("orange", "Orange")
                    .test_id("ui-gallery-select-item-orange"),
            ]
            .into_iter()
            .chain((1..=40).map(|i| {
                let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                shadcn::SelectItem::new(value, label)
                    .test_id(test_id)
                    .disabled(i == 15)
            })),
        )
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx);

    let selected_label = cx
        .scope(|cx| {
            let selected: Arc<str> = cx
                .get_model_cloned(&value, fret_ui::Invalidation::Paint)
                .unwrap_or_default()
                .unwrap_or_else(|| Arc::<str>::from("<none>"));

            fret_ui::element::AnyElement::new(
                cx.root_id(),
                fret_ui::element::ElementKind::Text(fret_ui::element::TextProps::new(format!(
                    "Selected: {selected}"
                ))),
                Vec::new(),
            )
        })
        .attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .test_id("ui-gallery-select-selected-label"),
        );

    vec![select, selected_label]
}

pub(in crate::ui) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_combobox(cx, value, open, query)
}

pub(in crate::ui) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    pages::preview_date_picker(cx, open, month, selected)
}

pub(in crate::ui) fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct ResizableModels {
        vertical_fractions: Option<Model<Vec<f32>>>,
        handle_fractions: Option<Model<Vec<f32>>>,
        rtl_h_fractions: Option<Model<Vec<f32>>>,
        rtl_v_fractions: Option<Model<Vec<f32>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let box_group =
        |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default().border_1().rounded(Radius::Lg),
                    layout,
                ),
                move |_cx| [body],
            )
        };

    let panel = |cx: &mut ElementContext<'_, App>, label: &'static str, height: Option<Px>| {
        let layout = match height {
            Some(h) => LayoutRefinement::default().w_full().h_px(h),
            None => LayoutRefinement::default().w_full().h_full(),
        };

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_center()
                .justify_center(),
            move |cx| vec![cx.text(label)],
        );

        cx.container(
            decl_style::container_props(theme, ChromeRefinement::default().p(Space::N6), layout),
            move |_cx| [body],
        )
    };

    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(448.0));

    let state = cx.with_state(ResizableModels::default, |st| st.clone());
    let vertical_fractions = match state.vertical_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.vertical_fractions = Some(model.clone())
            });
            model
        }
    };
    let handle_fractions = match state.handle_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.handle_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_h_fractions = match state.rtl_h_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.5, 0.5]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_h_fractions = Some(model.clone())
            });
            model
        }
    };
    let rtl_v_fractions = match state.rtl_v_fractions {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![0.25, 0.75]);
            cx.with_state(ResizableModels::default, |st| {
                st.rtl_v_fractions = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Two", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Three", None)]).into(),
            ])
            .into_element(cx);

        let group = shadcn::ResizablePanelGroup::new(h_fractions)
            .axis(fret_core::Axis::Horizontal)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "One", Some(Px(200.0)))]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([nested_vertical]).into(),
            ])
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .label("Debug:ui-gallery:resizable-panels")
                    .test_id("ui-gallery-resizable-panels"),
            );

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(320.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Demo", body)
    };

    let vertical = {
        let group = shadcn::ResizablePanelGroup::new(vertical_fractions)
            .axis(fret_core::Axis::Vertical)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Header", None)]).into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Vertical", body)
    };

    let handle = {
        let group = shadcn::ResizablePanelGroup::new(handle_fractions)
            .axis(fret_core::Axis::Horizontal)
            .entries([
                shadcn::ResizablePanel::new([panel(cx, "Sidebar", None)]).into(),
                shadcn::ResizableHandle::new().with_handle(true).into(),
                shadcn::ResizablePanel::new([panel(cx, "Content", None)]).into(),
            ])
            .into_element(cx);

        let group = box_group(
            cx,
            max_w_md
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(200.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "Handle", body)
    };

    let rtl = {
        let group = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let nested_vertical = shadcn::ResizablePanelGroup::new(rtl_v_fractions.clone())
                    .axis(fret_core::Axis::Vertical)
                    .entries([
                        shadcn::ResizablePanel::new([panel(cx, "اثنان", None)]).into(),
                        shadcn::ResizableHandle::new().with_handle(true).into(),
                        shadcn::ResizablePanel::new([panel(cx, "ثلاثة", None)]).into(),
                    ])
                    .into_element(cx);

                shadcn::ResizablePanelGroup::new(rtl_h_fractions.clone())
                    .axis(fret_core::Axis::Horizontal)
                    .entries([
                        shadcn::ResizablePanel::new([panel(cx, "واحد", Some(Px(200.0)))]).into(),
                        shadcn::ResizableHandle::new().with_handle(true).into(),
                        shadcn::ResizablePanel::new([nested_vertical]).into(),
                    ])
                    .into_element(cx)
            },
        );

        let group = box_group(
            cx,
            max_w_sm
                .clone()
                .merge(LayoutRefinement::default().h_px(Px(320.0))),
            group,
        );

        let body = centered(cx, group);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Drag the handles to resize panels."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, vertical, handle, rtl]
        }),
    ]
}
