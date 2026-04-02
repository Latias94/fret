use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
use fret_ui::UiTree;
use fret_ui::declarative::{render_dismissible_root_with_hooks, render_root};
use fret_ui::element::{LayoutStyle, Length, SemanticsProps, SizeStyle, TextInputProps};

use super::{NullServices, TestUiHostImpl};

#[test]
fn focused_portal_text_input_prevents_key_events_from_reaching_underlay_root() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::new();

    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let underlay_keydowns = Arc::new(AtomicUsize::new(0));
    let underlay_keydowns_hook = underlay_keydowns.clone();
    let underlay = render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "test.underlay",
        |ecx| {
            let root = ecx.root_id();
            ecx.key_on_key_down_for(
                root,
                Arc::new(move |_host, _cx, _down| {
                    underlay_keydowns_hook.fetch_add(1, Ordering::Relaxed);
                    true
                }),
            );

            let mut props = SemanticsProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.semantics(props, |_ecx| Vec::new())]
        },
    );

    ui.set_root(underlay);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    ui.set_focus(Some(underlay));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Delete,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(underlay_keydowns.load(Ordering::Relaxed), 1);

    let input_model = host.models.insert(String::new());
    let portal_root = render_dismissible_root_with_hooks(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "test.portal.text_input",
        |ecx| {
            let mut props = TextInputProps::new(input_model.clone());
            props.layout = LayoutStyle {
                position: fret_ui::element::PositionStyle::Absolute,
                inset: fret_ui::element::InsetStyle {
                    left: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(Px(240.0)),
                    height: Length::Px(Px(32.0)),
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.text_input(props)]
        },
    );

    let _portal_layer = ui.push_overlay_root(portal_root, false);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let input_pos = Point::new(Px(10.0), Px(10.0));
    let hit = ui
        .debug_hit_test(input_pos)
        .hit
        .expect("expected to hit the portal text input");

    ui.set_focus(Some(hit));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Delete,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        underlay_keydowns.load(Ordering::Relaxed),
        1,
        "underlay key hook should not receive key events while portal text input is focused"
    );
}
