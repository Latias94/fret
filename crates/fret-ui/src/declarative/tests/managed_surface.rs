use super::*;
use crate::widget::CommandAvailability;
use fret_core::{Edges, Event};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
struct ManagedSurfaceProofFrame {
    left: Rect,
    right: Rect,
}

fn proof_rects() -> ManagedSurfaceProofFrame {
    ManagedSurfaceProofFrame {
        left: Rect::new(
            fret_core::Point::new(Px(8.0), Px(6.0)),
            Size::new(Px(90.0), Px(40.0)),
        ),
        right: Rect::new(
            fret_core::Point::new(Px(120.0), Px(10.0)),
            Size::new(Px(70.0), Px(32.0)),
        ),
    }
}

#[test]
fn managed_surface_places_declarative_child_roots_from_host_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();
    let proof = proof_rects();
    let proof_for_render = proof.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-layout-proof",
        move |cx| {
            let proof_for_layout = proof_for_render.clone();
            vec![cx.managed_surface(
                crate::element::ManagedSurfaceProps::default(),
                move |cx| {
                    let children = cx.children().to_vec();
                    if let Some(&left) = children.first() {
                        let _ = cx.layout_child_root(left, proof_for_layout.left);
                    }
                    if let Some(&right) = children.get(1) {
                        let _ = cx.layout_child_root(right, proof_for_layout.right);
                    }
                },
                move |cx| {
                    let children = cx.children().to_vec();
                    if let Some(frame) = cx.output::<ManagedSurfaceProofFrame>().cloned() {
                        if let Some(&right) = children.get(1) {
                            cx.paint_child(right, frame.right);
                        }
                        if let Some(&left) = children.first() {
                            cx.paint_child(left, frame.left);
                        }
                    }
                },
                |cx| {
                    vec![
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let managed = ui.children(root)[0];
    let children = ui.children(managed);
    let left_node = children[0];
    let right_node = children[1];

    assert_eq!(ui.debug_node_bounds(left_node), Some(proof.left));
    assert_eq!(ui.debug_node_bounds(right_node), Some(proof.right));
}

#[test]
fn managed_surface_paints_child_roots_in_host_selected_order_and_rects() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();
    let proof = proof_rects();
    let proof_for_render = proof.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-paint-proof",
        move |cx| {
            let proof_for_layout = proof_for_render.clone();
            vec![cx.managed_surface(
                crate::element::ManagedSurfaceProps::default(),
                move |cx| {
                    cx.set_output(proof_for_layout.clone());
                    let children = cx.children().to_vec();
                    if let Some(&left) = children.first() {
                        let _ = cx.layout_child_root(left, proof_for_layout.left);
                    }
                    if let Some(&right) = children.get(1) {
                        let _ = cx.layout_child_root(right, proof_for_layout.right);
                    }
                },
                move |cx| {
                    let children = cx.children().to_vec();
                    let frame = cx
                        .output::<ManagedSurfaceProofFrame>()
                        .cloned()
                        .expect("managed surface frame output");
                    if let Some(&right) = children.get(1) {
                        cx.paint_child(right, frame.right);
                    }
                    if let Some(&left) = children.first() {
                        cx.paint_child(left, frame.left);
                    }
                },
                |cx| {
                    vec![
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let quads: Vec<(Rect, Color)> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad {
                rect, background, ..
            } => match background.paint {
                fret_core::Paint::Solid(color) => Some((*rect, color)),
                _ => None,
            },
            _ => None,
        })
        .collect();

    assert_eq!(quads.len(), 2);
    assert_eq!(quads[0].0, proof.right);
    assert_eq!(quads[0].1.g, 1.0);
    assert_eq!(quads[1].0, proof.left);
    assert_eq!(quads[1].1.r, 1.0);
}

#[test]
fn managed_surface_prepaint_hook_runs_when_enabled() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();
    let prepaint_count = Arc::new(AtomicUsize::new(0));
    let prepaint_count_for_hook = Arc::clone(&prepaint_count);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-prepaint-proof",
        move |cx| {
            vec![cx.managed_surface_with_prepaint(
                crate::element::ManagedSurfaceProps::default(),
                move |cx| {
                    cx.layout_unplaced_children(cx.bounds());
                },
                {
                    let prepaint_count_for_hook = Arc::clone(&prepaint_count_for_hook);
                    move |cx| {
                        prepaint_count_for_hook.fetch_add(1, Ordering::Relaxed);
                        cx.set_output(ManagedSurfaceProofFrame {
                            left: cx.bounds(),
                            right: cx.bounds(),
                        });
                    }
                },
                move |cx| {
                    let children = cx.children().to_vec();
                    if let Some(frame) = cx.output::<ManagedSurfaceProofFrame>().cloned() {
                        for child in children {
                            cx.paint_child(child, frame.left);
                        }
                    }
                },
                |cx| vec![cx.text("child")],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(prepaint_count.load(Ordering::Relaxed), 1);
}

#[test]
fn managed_surface_dispatches_event_command_and_availability_hooks() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();
    let event_count = Arc::new(AtomicUsize::new(0));
    let command_count = Arc::new(AtomicUsize::new(0));
    let availability_count = Arc::new(AtomicUsize::new(0));
    let event_count_for_hook = Arc::clone(&event_count);
    let command_count_for_hook = Arc::clone(&command_count);
    let availability_count_for_hook = Arc::clone(&availability_count);
    let command = CommandId::from("managed_surface.test");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-input-hooks-proof",
        {
            let command = command.clone();
            move |cx| {
                let element = cx.managed_surface(
                    crate::element::ManagedSurfaceProps::default(),
                    |cx| {
                        cx.layout_unplaced_children(cx.bounds());
                    },
                    |cx| {
                        let children = cx.children().to_vec();
                        for child in children {
                            cx.paint_child(child, cx.bounds());
                        }
                    },
                    |cx| vec![cx.text("child")],
                );
                cx.managed_surface_on_event_for(element.id, {
                    let event_count_for_hook = Arc::clone(&event_count_for_hook);
                    move |cx, event| {
                        if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
                            event_count_for_hook.fetch_add(1, Ordering::Relaxed);
                            cx.request_focus(cx.node());
                            cx.stop_propagation();
                        }
                    }
                });
                cx.managed_surface_on_command_for(element.id, {
                    let command_count_for_hook = Arc::clone(&command_count_for_hook);
                    let command = command.clone();
                    move |cx, received| {
                        if received.as_str() != command.as_str() {
                            return false;
                        }
                        command_count_for_hook.fetch_add(1, Ordering::Relaxed);
                        cx.request_focus(cx.node());
                        cx.request_redraw();
                        true
                    }
                });
                cx.managed_surface_on_command_availability_for(element.id, {
                    let availability_count_for_hook = Arc::clone(&availability_count_for_hook);
                    let command = command.clone();
                    move |_cx, received| {
                        if received.as_str() != command.as_str() {
                            return CommandAvailability::NotHandled;
                        }
                        availability_count_for_hook.fetch_add(1, Ordering::Relaxed);
                        CommandAvailability::Available
                    }
                });
                vec![element]
            }
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let managed = ui.children(root)[0];
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: fret_core::Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(event_count.load(Ordering::Relaxed), 1);
    assert_eq!(ui.focus(), Some(managed));

    assert_eq!(
        ui.command_availability(&mut app, &command),
        CommandAvailability::Available
    );
    assert_eq!(availability_count.load(Ordering::Relaxed), 1);
    assert!(ui.dispatch_command(&mut app, &mut services, &command));
    assert_eq!(command_count.load(Ordering::Relaxed), 1);
    assert_eq!(ui.focus(), Some(managed));
}

#[test]
fn managed_surface_paint_exposes_services_scale_factor_and_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(90.0)),
    );
    let mut services = FakeTextService::default();
    let proof = proof_rects();
    let proof_for_render = proof.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-paint-context-proof",
        move |cx| {
            let proof_for_layout = proof_for_render.clone();
            vec![cx.managed_surface(
                crate::element::ManagedSurfaceProps::default(),
                move |cx| {
                    let children = cx.children().to_vec();
                    if let Some(&left) = children.first() {
                        let _ = cx.layout_child_root(left, proof_for_layout.left);
                    }
                    if let Some(&right) = children.get(1) {
                        let _ = cx.layout_child_root(right, proof_for_layout.right);
                    }
                },
                move |cx| {
                    assert_eq!(cx.scale_factor(), 2.0);
                    let text_blob = {
                        let services = cx.services();
                        let (blob, _metrics) = services.text().prepare(
                            &fret_core::TextInput::plain(
                                "paint services",
                                fret_core::TextStyle::default(),
                            ),
                            fret_core::TextConstraints::default(),
                        );
                        blob
                    };

                    let children = cx.children().to_vec();
                    if let Some(&left) = children.first() {
                        let child_bounds = cx.child_bounds(left).expect("left child bounds");
                        cx.scene().push(SceneOp::Text {
                            order: fret_core::DrawOrder(0),
                            origin: child_bounds.origin,
                            text: text_blob,
                            paint: fret_core::Paint::Solid(Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            })
                            .into(),
                            outline: None,
                            shadow: None,
                        });
                        cx.paint_child(left, child_bounds);
                        cx.release_text_blob_on_next_paint(text_blob);
                    }
                    if let Some(&right) = children.get(1) {
                        let child_bounds = cx.child_bounds(right).expect("right child bounds");
                        cx.paint_child(right, child_bounds);
                    }
                },
                |cx| {
                    vec![
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                        cx.canvas(crate::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 1.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 2.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 2.0);

    assert_eq!(services.prepare_calls, 1);
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Text { .. })),
        "expected managed-surface paint hook to write a service-prepared text op"
    );
}

#[test]
fn managed_surface_defers_paint_time_text_release_until_next_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(90.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "managed-surface-deferred-text-release-proof",
        move |cx| {
            vec![cx.managed_surface(
                crate::element::ManagedSurfaceProps::default(),
                move |_cx| {},
                move |cx| {
                    let text_blob = {
                        let services = cx.services();
                        let (blob, _metrics) = services.text().prepare(
                            &fret_core::TextInput::plain(
                                "transient paint text",
                                fret_core::TextStyle::default(),
                            ),
                            fret_core::TextConstraints::default(),
                        );
                        blob
                    };
                    let origin = cx.bounds().origin;
                    cx.scene().push(SceneOp::Text {
                        order: fret_core::DrawOrder(0),
                        origin,
                        text: text_blob,
                        paint: fret_core::Paint::Solid(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })
                        .into(),
                        outline: None,
                        shadow: None,
                    });
                    cx.release_text_blob_on_next_paint(text_blob);
                },
                |_cx| Vec::new(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(services.prepare_calls, 1);
    assert_eq!(
        services.release_calls, 0,
        "the scene still references this paint-time text blob during the same paint"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(services.prepare_calls, 2);
    assert_eq!(
        services.release_calls, 1,
        "the previous paint-time text blob should be released when the surface repaints"
    );
}
