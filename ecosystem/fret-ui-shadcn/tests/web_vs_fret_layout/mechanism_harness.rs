use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use serde::Deserialize;
use slotmap::Key as _;

const RECIPE_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/mechanism_layout_recipe_cases_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecipeScenario {
    ButtonGroupTextAddonsCenterWithInputControl,
    ResponsiveDrawerBottomSheetCapsVisibleLane,
    PopoverCommandShellWrapsHoverRegionMaxHeight,
}

#[test]
fn mechanism_harness_recipe_layout_cases_match_oracles() {
    let suite: MechanismSuite<RecipeScenario> =
        MechanismSuite::from_json_str(RECIPE_CASES).expect("recipe mechanism fixture suite");

    let mut observer: fn(
        &MechanismCase<RecipeScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RecipeScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    if matches!(
        case.scenario,
        RecipeScenario::PopoverCommandShellWrapsHoverRegionMaxHeight
    ) {
        return observe_popover_command_shell_wraps_hover_region_max_height();
    }
    if matches!(
        case.scenario,
        RecipeScenario::ResponsiveDrawerBottomSheetCapsVisibleLane
    ) {
        return observe_responsive_drawer_bottom_sheet_caps_visible_lane();
    }

    let bounds = match case.scenario {
        RecipeScenario::ButtonGroupTextAddonsCenterWithInputControl => Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(360.0), Px(80.0)),
        ),
        RecipeScenario::ResponsiveDrawerBottomSheetCapsVisibleLane => unreachable!(),
        RecipeScenario::PopoverCommandShellWrapsHoverRegionMaxHeight => unreachable!(),
    };
    let (ui, snapshot, _root) = run_fret_root_with_ui(bounds, |cx| match case.scenario {
        RecipeScenario::ButtonGroupTextAddonsCenterWithInputControl => {
            let model: Model<String> = cx.app.models_mut().insert(String::new());
            let control_id = "mechanism-button-group-url";

            vec![
                shadcn::ButtonGroup::new([
                    shadcn::ButtonGroupText::new("https://")
                        .test_id("mechanism-button-group-text-prefix")
                        .into(),
                    shadcn::Input::new(model)
                        .control_id(control_id)
                        .a11y_label("URL")
                        .placeholder("my-app")
                        .test_id("mechanism-button-group-text-control")
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_px(MetricRef::Px(Px(220.0)))
                                .min_w_0(),
                        )
                        .into(),
                    shadcn::ButtonGroupText::new(".com")
                        .test_id("mechanism-button-group-text-suffix")
                        .into(),
                ])
                .into_element(cx)
                .test_id("mechanism-button-group-text"),
            ]
        }
        RecipeScenario::ResponsiveDrawerBottomSheetCapsVisibleLane => unreachable!(),
        RecipeScenario::PopoverCommandShellWrapsHoverRegionMaxHeight => unreachable!(),
    });

    Ok(observed_tree_from_ui(&ui, &snapshot, bounds))
}

fn observe_responsive_drawer_bottom_sheet_caps_visible_lane()
-> Result<ObservedTree, ScenarioObserveError> {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(375.0), Px(240.0)),
    );
    let window = AppWindowId::default();
    let mut app = App::new();

    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();
    let open = app.models_mut().insert(true);

    app.set_frame_id(FrameId(1));
    OverlayController::begin_frame(&mut app, window);
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-responsive-drawer-bottom-sheet-sizing",
        |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let drawer = shadcn::Drawer::new(open.clone())
                .drag_to_dismiss(false)
                .into_element(
                    cx,
                    |_cx| trigger,
                    |cx| {
                        let tall = cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Px(Px(2000.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        );

                        shadcn::DrawerContent::new([tall])
                            .into_element(cx)
                            .test_id("mechanism-responsive-drawer-shell")
                    },
                );

            vec![drawer]
        },
    );
    ui.set_root(root);
    OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("expected semantics snapshot"))?;
    Ok(observed_tree_from_ui(&ui, &snapshot, bounds))
}

fn observe_popover_command_shell_wraps_hover_region_max_height()
-> Result<ObservedTree, ScenarioObserveError> {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );
    let window = AppWindowId::default();
    let mut app = App::new();

    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();
    let open = app.models_mut().insert(false);
    let query = app.models_mut().insert(String::new());

    render_popover_command_shell_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        open.clone(),
        query.clone(),
    );

    let _ = app.models_mut().update(&open, |value| *value = true);
    render_popover_command_shell_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        open.clone(),
        query.clone(),
    );

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("expected semantics snapshot"))?;
    Ok(observed_tree_from_ui(&ui, &snapshot, bounds))
}

fn render_popover_command_shell_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    open: Model<bool>,
    query: Model<String>,
) {
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-popover-command-shell-sizing",
        |cx| {
            let trigger = cx
                .pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(150.0));
                            layout.size.height = Length::Px(Px(36.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                )
                .test_id("mechanism-popover-trigger");

            let query_for_content = query.clone();
            let popover = shadcn::Popover::from_open(open.clone())
                .align(shadcn::PopoverAlign::Start)
                .side(shadcn::PopoverSide::Bottom)
                .side_offset(Px(4.0))
                .motion_durations(std::time::Duration::ZERO, std::time::Duration::ZERO)
                .into_element_with(
                    cx,
                    |_cx| trigger,
                    move |cx| {
                        let input = shadcn::CommandInput::new(query_for_content.clone())
                            .input_test_id("mechanism-popover-input")
                            .into_element(cx);
                        let list = shadcn::CommandList::new([
                            shadcn::CommandItem::new("Backlog"),
                            shadcn::CommandItem::new("Todo"),
                            shadcn::CommandItem::new("In Progress"),
                            shadcn::CommandItem::new("Done"),
                            shadcn::CommandItem::new("Canceled"),
                        ])
                        .refine_scroll_layout(LayoutRefinement::default().max_h(Px(168.0)))
                        .into_element(cx);
                        let command = shadcn::Command::new([input, list])
                            .into_element(cx)
                            .test_id("mechanism-popover-command");

                        shadcn::PopoverContent::new([command])
                            .refine_style(
                                ChromeRefinement::default()
                                    .p(Space::N0)
                                    .border_width(Px(0.0)),
                            )
                            .refine_layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0())
                            .into_element(cx)
                            .test_id("mechanism-popover-shell")
                    },
                );

            vec![popover]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn observed_tree_from_ui(
    ui: &UiTree<App>,
    snapshot: &fret_core::SemanticsSnapshot,
    bounds: Rect,
) -> ObservedTree {
    let mut observed = ObservedTree::from_semantics_snapshot(snapshot, bounds);
    for node in &snapshot.nodes {
        if let Some(layout) = ui.debug_node_bounds(node.id) {
            observed.set_layout_bounds_for_node_id(node.id.data().as_ffi(), layout);
        }
    }
    observed
}
