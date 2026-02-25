use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
    PathMetrics, PathService, PathStyle, Point, PointerEvent, PointerType, Px, Rect, SemanticsRole,
    Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    UiServices,
};
use fret_runtime::{Effect, Model};
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};
use fret_ui_shadcn::dropdown_menu::DropdownMenuSide;
use fret_ui_shadcn::shadcn_themes;
use std::sync::Arc;

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
                size: CoreSize::new(Px(0.0), Px(0.0)),
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
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn deliver_all_timers_from_effects(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn UiServices,
) {
    let effects = app.flush_effects();
    let mut timer_tokens = Vec::new();
    for effect in effects {
        match effect {
            Effect::SetTimer { token, .. } => timer_tokens.push(token),
            other => app.push_effect(other),
        }
    }
    for token in timer_tokens {
        ui.dispatch_event(app, services, &Event::Timer { token });
    }
}

fn right_click_at(ui: &mut UiTree<App>, app: &mut App, services: &mut dyn UiServices, pos: Point) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: pos,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: pos,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    trigger_layout: LayoutStyle,
    request_semantics: bool,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "context-menu",
        |cx| {
            let open = open.clone();
            let entries = (0..64).map(|idx| {
                ContextMenuEntry::Item(
                    ContextMenuItem::new(Arc::<str>::from(format!("Item {idx}")))
                        .test_id(Arc::<str>::from(format!("context-menu-item-{idx}"))),
                )
            });

            let trigger = cx.pressable(
                PressableProps {
                    layout: trigger_layout,
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("Context menu trigger")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let menu = ContextMenu::new(open)
                .side(DropdownMenuSide::Top)
                .side_offset(Px(2.0))
                .window_margin(Px(0.0))
                .content_test_id("context-menu-content")
                .into_element(cx, |_cx| trigger, |_cx| entries);

            vec![menu]
        },
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn context_menu_content_height_clamps_to_available_height() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let open: Model<bool> = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(160.0)),
    );

    let mut trigger_layout = LayoutStyle::default();
    trigger_layout.position = fret_ui::element::PositionStyle::Absolute;
    trigger_layout.inset.left = Some(Px(16.0)).into();
    trigger_layout.inset.top = Some(Px(42.0)).into();
    trigger_layout.size.width = Length::Px(Px(200.0));
    trigger_layout.size.height = Length::Px(Px(40.0));

    // Frame 1: mount closed so hit-test regions exist.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        trigger_layout,
        true,
    );

    let anchor = Point::new(Px(32.0), Px(62.0));
    right_click_at(&mut ui, &mut app, &mut services, anchor);
    deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);

    // Frame 2+: open and render twice so popper vars settle.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        trigger_layout,
        false,
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
        trigger_layout,
        true,
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let content = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("context-menu-content"))
        .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Menu))
        .expect("context menu content semantics node");

    let side_offset = Px(2.0);
    let top_available_height = Px((anchor.y.0 - side_offset.0).max(0.0));
    let bottom_available_height = Px((bounds.size.height.0 - anchor.y.0 - side_offset.0).max(0.0));
    let available_height = if top_available_height.0 >= bottom_available_height.0 {
        top_available_height
    } else {
        bottom_available_height
    };

    let theme = Theme::global(&app).clone();
    let expected_max_height = theme
        .metric_by_key("component.context_menu.max_height")
        .map(|h| Px(h.0.min(available_height.0)))
        .unwrap_or(available_height);

    assert!(
        content.bounds.size.height.0 <= expected_max_height.0 + 2.0,
        "expected context menu height to clamp to available height; got {:?}, expected <= {:?} (available {:?})",
        content.bounds.size.height,
        expected_max_height,
        available_height
    );
    assert!(
        content.bounds.size.height.0 + 2.0 >= expected_max_height.0,
        "expected context menu to reach height cap with large content; got {:?}, expected ~ {:?} (available {:?})",
        content.bounds.size.height,
        expected_max_height,
        available_height
    );
}
