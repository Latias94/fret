use accesskit::Role;
use fret_core::{
    AppWindowId, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsNodeExtra,
    SemanticsOrientation, SemanticsRole, SemanticsRoot, SemanticsSnapshot,
};
use slotmap::KeyData;

use crate::ids::{text_run_id_for, to_accesskit_id};
use crate::roles::map_role;
use crate::{
    SetValueData, StepperAction, replace_selected_text_from_action, scroll_by_from_action,
    set_text_selection_from_action, set_value_from_action, stepper_target_from_action,
    tree_update_from_snapshot,
};

fn node(id: u64) -> fret_core::NodeId {
    fret_core::NodeId::from(KeyData::from_ffi(id))
}

#[test]
fn maps_extended_semantics_roles_to_accesskit_roles() {
    assert_eq!(map_role(SemanticsRole::AlertDialog), Role::AlertDialog);
    assert_eq!(map_role(SemanticsRole::Status), Role::Status);
    assert_eq!(
        map_role(SemanticsRole::ProgressBar),
        Role::ProgressIndicator
    );
    assert_eq!(map_role(SemanticsRole::ScrollBar), Role::ScrollBar);
    assert_eq!(map_role(SemanticsRole::SpinButton), Role::SpinButton);
    assert_eq!(map_role(SemanticsRole::Meter), Role::Meter);
    assert_eq!(map_role(SemanticsRole::Splitter), Role::Splitter);
    assert_eq!(map_role(SemanticsRole::Heading), Role::Heading);
    assert_eq!(map_role(SemanticsRole::RadioGroup), Role::RadioGroup);
    assert_eq!(map_role(SemanticsRole::RadioButton), Role::RadioButton);
    assert_eq!(
        map_role(SemanticsRole::MenuItemCheckbox),
        Role::MenuItemCheckBox
    );
    assert_eq!(map_role(SemanticsRole::MenuItemRadio), Role::MenuItemRadio);
    assert_eq!(map_role(SemanticsRole::Link), Role::Link);
    assert_eq!(map_role(SemanticsRole::Image), Role::Image);
    assert_eq!(map_role(SemanticsRole::Tooltip), Role::Tooltip);
    assert_eq!(map_role(SemanticsRole::Toolbar), Role::Toolbar);
}

#[test]
fn numeric_and_extra_properties_are_emitted_when_present() {
    let window = AppWindowId::default();
    let root = node(1);
    let slider = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let mut extra = SemanticsNodeExtra::default();
    extra.numeric.value = Some(50.0);
    extra.numeric.min = Some(0.0);
    extra.numeric.max = Some(100.0);
    extra.numeric.step = Some(1.0);
    extra.numeric.jump = Some(10.0);
    extra.scroll.x = Some(5.0);
    extra.scroll.x_min = Some(0.0);
    extra.scroll.x_max = Some(10.0);
    extra.level = Some(3);
    extra.orientation = Some(SemanticsOrientation::Vertical);
    extra.url = Some("https://example.com".to_string());

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: slider,
                parent: Some(root),
                role: SemanticsRole::Slider,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Slider".to_string()),
                value: Some("50".to_string()),
                extra,
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let slider_id = to_accesskit_id(slider);
    let slider_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == slider_id).then_some(n))
        .expect("slider node present");

    assert_eq!(slider_node.numeric_value(), Some(50.0));
    assert_eq!(slider_node.min_numeric_value(), Some(0.0));
    assert_eq!(slider_node.max_numeric_value(), Some(100.0));
    assert_eq!(slider_node.numeric_value_step(), Some(1.0));
    assert_eq!(slider_node.numeric_value_jump(), Some(10.0));
    assert_eq!(slider_node.scroll_x(), Some(5.0));
    assert_eq!(slider_node.scroll_x_min(), Some(0.0));
    assert_eq!(slider_node.scroll_x_max(), Some(10.0));
    assert_eq!(slider_node.level(), Some(3));
    assert_eq!(
        slider_node.orientation(),
        Some(accesskit::Orientation::Vertical)
    );
    assert_eq!(slider_node.url(), Some("https://example.com"));
}

#[test]
fn scroll_by_actions_are_exposed_and_decoded() {
    let window = AppWindowId::default();
    let root = node(1);
    let scroll = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(320.0), Px(180.0)),
    );

    let mut extra = SemanticsNodeExtra::default();
    extra.scroll.y = Some(5.0);
    extra.scroll.y_min = Some(0.0);
    extra.scroll.y_max = Some(100.0);

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: scroll,
                parent: Some(root),
                role: SemanticsRole::Generic,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra,
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions {
                    scroll_by: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let scroll_id = to_accesskit_id(scroll);
    let scroll_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == scroll_id).then_some(n))
        .expect("scroll node present");
    assert!(scroll_node.supports_action(accesskit::Action::SetScrollOffset));
    assert!(scroll_node.supports_action(accesskit::Action::ScrollDown));
    assert!(scroll_node.supports_action(accesskit::Action::ScrollUp));

    let req = accesskit::ActionRequest {
        action: accesskit::Action::ScrollDown,
        target_tree: accesskit::TreeId::ROOT,
        target_node: scroll_id,
        data: Some(accesskit::ActionData::ScrollUnit(
            accesskit::ScrollUnit::Item,
        )),
    };
    let (target, data) = scroll_by_from_action(&req, &snapshot).expect("decoded scroll down");
    assert_eq!(target, scroll);
    assert_eq!(data.dx, 0.0);
    assert!(data.dy > 0.0);

    let req = accesskit::ActionRequest {
        action: accesskit::Action::SetScrollOffset,
        target_tree: accesskit::TreeId::ROOT,
        target_node: scroll_id,
        data: Some(accesskit::ActionData::SetScrollOffset(accesskit::Point {
            x: 0.0,
            y: 7.0,
        })),
    };
    let (target, data) = scroll_by_from_action(&req, &snapshot).expect("decoded set scroll offset");
    assert_eq!(target, scroll);
    assert_eq!(data.dx, 0.0);
    assert_eq!(data.dy, 2.0);
}

#[test]
fn mixed_checked_state_maps_to_accesskit_toggled_mixed() {
    let window = AppWindowId::default();
    let root = node(1);
    let checkbox = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: checkbox,
                parent: Some(root),
                role: SemanticsRole::Checkbox,
                bounds,
                flags: SemanticsFlags {
                    checked_state: Some(fret_core::SemanticsCheckedState::Mixed),
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Checkbox".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let checkbox_id = to_accesskit_id(checkbox);
    let checkbox_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == checkbox_id).then_some(n))
        .expect("checkbox node present");

    assert_eq!(checkbox_node.toggled(), Some(accesskit::Toggled::Mixed));
}

#[test]
fn pressed_state_maps_to_accesskit_toggled() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let root = node(1);
    let toggle = node(2);

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: toggle,
                parent: Some(root),
                role: SemanticsRole::Button,
                bounds,
                flags: SemanticsFlags {
                    pressed_state: Some(fret_core::SemanticsPressedState::Mixed),
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Toggle".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let toggle_id = to_accesskit_id(toggle);
    let toggle_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == toggle_id).then_some(n))
        .expect("toggle node present");

    assert_eq!(toggle_node.toggled(), Some(accesskit::Toggled::Mixed));
}

#[test]
fn checked_state_takes_precedence_over_pressed_state() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let root = node(1);
    let node = node(2);

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: node,
                parent: Some(root),
                role: SemanticsRole::Checkbox,
                bounds,
                flags: SemanticsFlags {
                    checked_state: Some(fret_core::SemanticsCheckedState::True),
                    pressed_state: Some(fret_core::SemanticsPressedState::False),
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Checkbox".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let id = to_accesskit_id(node);
    let mapped = update
        .nodes
        .iter()
        .find_map(|(nid, n)| (*nid == id).then_some(n))
        .expect("node present");

    assert_eq!(mapped.toggled(), Some(accesskit::Toggled::True));
}

#[test]
fn required_and_invalid_flags_are_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    required: true,
                    invalid: Some(fret_core::SemanticsInvalid::True),
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Input".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let input_id = to_accesskit_id(input);
    let input_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == input_id).then_some(n))
        .expect("input node present");

    assert!(input_node.is_required());
    assert_eq!(input_node.invalid(), Some(accesskit::Invalid::True));
}

#[test]
fn busy_flag_is_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let region = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: region,
                parent: Some(root),
                role: SemanticsRole::Panel,
                bounds,
                flags: SemanticsFlags {
                    busy: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Results".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let region_id = to_accesskit_id(region);
    let region_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == region_id).then_some(n))
        .expect("region node present");

    assert!(region_node.is_busy());
}

#[test]
fn hidden_flag_is_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let decoration = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: decoration,
                parent: Some(root),
                role: SemanticsRole::Button,
                bounds,
                flags: SemanticsFlags {
                    hidden: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Decorative".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let id = to_accesskit_id(decoration);
    let mapped = update
        .nodes
        .iter()
        .find_map(|(nid, n)| (*nid == id).then_some(n))
        .expect("node present");

    assert!(mapped.is_hidden());
}

#[test]
fn visited_flag_is_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let link = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: link,
                parent: Some(root),
                role: SemanticsRole::Link,
                bounds,
                flags: SemanticsFlags {
                    visited: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Docs".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let id = to_accesskit_id(link);
    let mapped = update
        .nodes
        .iter()
        .find_map(|(nid, n)| (*nid == id).then_some(n))
        .expect("node present");

    assert!(mapped.is_visited());
}

#[test]
fn multiselectable_flag_is_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let listbox = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: listbox,
                parent: Some(root),
                role: SemanticsRole::ListBox,
                bounds,
                flags: SemanticsFlags {
                    multiselectable: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("List".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let id = to_accesskit_id(listbox);
    let mapped = update
        .nodes
        .iter()
        .find_map(|(nid, n)| (*nid == id).then_some(n))
        .expect("node present");

    assert!(mapped.is_multiselectable());
}

#[test]
fn live_region_flags_are_mapped() {
    let window = AppWindowId::default();
    let root = node(1);
    let region = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: region,
                parent: Some(root),
                role: SemanticsRole::Panel,
                bounds,
                flags: SemanticsFlags {
                    live: Some(fret_core::SemanticsLive::Polite),
                    live_atomic: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Notifications".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let region_id = to_accesskit_id(region);
    let region_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == region_id).then_some(n))
        .expect("region node present");

    assert_eq!(region_node.live(), Some(accesskit::Live::Polite));
    assert!(region_node.is_live_atomic());
}

#[test]
fn increment_and_decrement_actions_are_exposed_and_decoded() {
    let window = AppWindowId::default();
    let root = node(1);
    let slider = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: slider,
                parent: Some(root),
                role: SemanticsRole::Slider,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Slider".to_string()),
                value: Some("50".to_string()),
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions {
                    set_value: true,
                    increment: true,
                    decrement: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let slider_id = to_accesskit_id(slider);
    let slider_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == slider_id).then_some(n))
        .expect("slider node present");
    assert!(slider_node.supports_action(accesskit::Action::Increment));
    assert!(slider_node.supports_action(accesskit::Action::Decrement));
    assert!(slider_node.supports_action(accesskit::Action::SetValue));

    let increment_req = accesskit::ActionRequest {
        action: accesskit::Action::Increment,
        target_tree: accesskit::TreeId::ROOT,
        target_node: slider_id,
        data: None,
    };
    let (target, action) =
        stepper_target_from_action(&increment_req).expect("decoded increment target");
    assert_eq!(target, slider);
    assert_eq!(action, StepperAction::Increment);

    let decrement_req = accesskit::ActionRequest {
        action: accesskit::Action::Decrement,
        target_tree: accesskit::TreeId::ROOT,
        target_node: slider_id,
        data: None,
    };
    let (target, action) =
        stepper_target_from_action(&decrement_req).expect("decoded decrement target");
    assert_eq!(target, slider);
    assert_eq!(action, StepperAction::Decrement);

    let req = accesskit::ActionRequest {
        action: accesskit::Action::SetValue,
        target_tree: accesskit::TreeId::ROOT,
        target_node: slider_id,
        data: Some(accesskit::ActionData::NumericValue(42.0)),
    };
    let (target, data) = set_value_from_action(&req).expect("decoded set value");
    assert_eq!(target, slider);
    assert_eq!(data, SetValueData::Numeric(42.0));
}

#[test]
fn active_descendant_is_emitted_for_reachable_descendant() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);
    let list = node(3);
    let item = node(4);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: Some(item),
                pos_in_set: None,
                set_size: None,
                label: Some("Command input".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions {
                    focus: true,
                    set_value: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: list,
                parent: Some(root),
                role: SemanticsRole::List,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: item,
                parent: Some(list),
                role: SemanticsRole::ListItem,
                bounds,
                flags: SemanticsFlags {
                    selected: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Item 1".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let input_id = to_accesskit_id(input);
    let item_id = to_accesskit_id(item);

    let input_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == input_id).then_some(n))
        .expect("input node present");

    assert_eq!(
        input_node.active_descendant(),
        Some(item_id),
        "focused text field should reference the active descendant"
    );
}

#[test]
fn active_descendant_is_not_emitted_when_not_reachable_under_modal_barrier() {
    let window = AppWindowId::default();
    let underlay_root = node(1);
    let underlay_list = node(2);
    let underlay_item = node(3);

    let modal_root = node(10);
    let input = node(11);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    // The focused input lives in the modal barrier layer, but it (incorrectly) points its
    // active descendant at an underlay list item. The bridge must not emit that association.
    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![
            SemanticsRoot {
                root: underlay_root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            },
            SemanticsRoot {
                root: modal_root,
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 1,
            },
        ],
        barrier_root: Some(modal_root),
        focus_barrier_root: Some(modal_root),
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: underlay_root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: underlay_list,
                parent: Some(underlay_root),
                role: SemanticsRole::ListBox,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Underlay list".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: underlay_item,
                parent: Some(underlay_list),
                role: SemanticsRole::ListBoxOption,
                bounds,
                flags: SemanticsFlags {
                    selected: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: Some(1),
                set_size: Some(1),
                label: Some("Underlay item".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: modal_root,
                parent: None,
                role: SemanticsRole::Dialog,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Modal".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(modal_root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: Some(underlay_item),
                pos_in_set: None,
                set_size: None,
                label: Some("Command input".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions {
                    focus: true,
                    set_value: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let input_id = to_accesskit_id(input);

    let input_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == input_id).then_some(n))
        .expect("input node present");

    assert_eq!(
        input_node.active_descendant(),
        None,
        "active_descendant must be suppressed when it points under the modal barrier"
    );
}

#[test]
fn list_item_pos_in_set_and_set_size_are_emitted() {
    let window = AppWindowId::default();
    let root = node(1);
    let list = node(2);
    let item = node(3);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: list,
                parent: Some(root),
                role: SemanticsRole::List,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: item,
                parent: Some(list),
                role: SemanticsRole::ListItem,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: Some(57),
                set_size: Some(1200),
                label: Some("Item 57".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let item_id = to_accesskit_id(item);

    let item_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == item_id).then_some(n))
        .expect("item node present");

    assert_eq!(item_node.position_in_set(), Some(57));
    assert_eq!(item_node.size_of_set(), Some(1200));
}

#[test]
fn described_by_is_emitted_for_reachable_descendant() {
    let window = AppWindowId::default();
    let root = node(1);
    let dialog = node(2);
    let title = node(3);
    let description = node(4);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: dialog,
                parent: Some(root),
                role: SemanticsRole::Dialog,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: vec![title],
                described_by: vec![description],
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: title,
                parent: Some(dialog),
                role: SemanticsRole::Text,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Title".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: description,
                parent: Some(dialog),
                role: SemanticsRole::Text,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Description".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let dialog_id = to_accesskit_id(dialog);

    let dialog_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == dialog_id).then_some(n))
        .expect("dialog node present");

    let expected_labelled_by = vec![to_accesskit_id(title)];
    assert_eq!(dialog_node.labelled_by(), expected_labelled_by.as_slice());

    let expected_described_by = vec![to_accesskit_id(description)];
    assert_eq!(dialog_node.described_by(), expected_described_by.as_slice());
}

#[test]
fn modal_dialogs_are_marked_modal_under_barrier_root() {
    let window = AppWindowId::default();
    let underlay_root = node(1);
    let modal_root = node(10);
    let dialog = node(11);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![
            SemanticsRoot {
                root: underlay_root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            },
            SemanticsRoot {
                root: modal_root,
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 1,
            },
        ],
        barrier_root: Some(modal_root),
        focus_barrier_root: Some(modal_root),
        focus: None,
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: underlay_root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: modal_root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: dialog,
                parent: Some(modal_root),
                role: SemanticsRole::Dialog,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Modal dialog".to_string()),
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let dialog_id = to_accesskit_id(dialog);
    let dialog_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == dialog_id).then_some(n))
        .expect("dialog node present");

    assert!(
        dialog_node.is_modal(),
        "dialogs in the barrier layer must be marked modal"
    );
}

#[test]
fn text_field_emits_synthetic_text_run_and_text_selection() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some("Search".to_string()),
                value: Some("hello".to_string()),
                extra: SemanticsNodeExtra::default(),
                text_selection: Some((1, 4)),
                text_composition: None,
                actions: SemanticsActions {
                    focus: true,
                    set_value: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let input_id = to_accesskit_id(input);
    let run_id = text_run_id_for(input);

    let input_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == input_id).then_some(n))
        .expect("input node present");
    let run_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == run_id).then_some(n))
        .expect("text run node present");

    assert!(
        input_node.children().contains(&run_id),
        "text field should include synthetic text run child"
    );
    assert_eq!(run_node.value(), Some("hello"));
    assert!(
        !run_node.character_lengths().is_empty(),
        "text run should include character lengths for selection"
    );

    let selection = input_node.text_selection().expect("selection present");
    assert_eq!(selection.anchor.node, run_id);
    assert_eq!(selection.anchor.character_index, 1);
    assert_eq!(selection.focus.node, run_id);
    assert_eq!(selection.focus.character_index, 4);
}

#[test]
fn set_text_selection_action_converts_character_indices_to_utf8_bytes() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: Some("a🦾b".to_string()),
                extra: SemanticsNodeExtra::default(),
                text_selection: Some((0, 0)),
                text_composition: None,
                actions: SemanticsActions {
                    focus: true,
                    set_text_selection: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let req = accesskit::ActionRequest {
        action: accesskit::Action::SetTextSelection,
        target_tree: accesskit::TreeId::ROOT,
        target_node: to_accesskit_id(input),
        data: Some(accesskit::ActionData::SetTextSelection(
            accesskit::TextSelection {
                anchor: accesskit::TextPosition {
                    node: text_run_id_for(input),
                    character_index: 1,
                },
                focus: accesskit::TextPosition {
                    node: text_run_id_for(input),
                    character_index: 2,
                },
            },
        )),
    };

    let (target, data) =
        set_text_selection_from_action(&req, &snapshot).expect("decoded selection");
    assert_eq!(target, input);
    assert_eq!(data.anchor, 1);
    assert_eq!(data.focus, 5);
}

#[test]
fn replace_selected_text_action_is_decoded() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: Some("hello".to_string()),
                extra: SemanticsNodeExtra::default(),
                text_selection: Some((0, 5)),
                text_composition: None,
                actions: SemanticsActions {
                    focus: true,
                    set_value: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let update = tree_update_from_snapshot(&snapshot, 1.0);
    let input_id = to_accesskit_id(input);
    let input_node = update
        .nodes
        .iter()
        .find_map(|(id, n)| (*id == input_id).then_some(n))
        .expect("input node present");
    assert!(
        input_node.supports_action(accesskit::Action::ReplaceSelectedText),
        "text field should expose ReplaceSelectedText when editable"
    );

    let req = accesskit::ActionRequest {
        action: accesskit::Action::ReplaceSelectedText,
        target_tree: accesskit::TreeId::ROOT,
        target_node: to_accesskit_id(input),
        data: Some(accesskit::ActionData::Value("x".into())),
    };
    let (target, value) =
        replace_selected_text_from_action(&req, &snapshot).expect("decoded replace selected text");
    assert_eq!(target, input);
    assert_eq!(value, "x");
}

#[test]
fn replace_selected_text_is_rejected_during_composition() {
    let window = AppWindowId::default();
    let root = node(1);
    let input = node(2);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(10.0), Px(10.0)),
    );

    let snapshot = SemanticsSnapshot {
        window,
        roots: vec![SemanticsRoot {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }],
        barrier_root: None,
        focus_barrier_root: None,
        focus: Some(input),
        captured: None,
        nodes: vec![
            SemanticsNode {
                id: root,
                parent: None,
                role: SemanticsRole::Window,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: None,
                extra: SemanticsNodeExtra::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
            SemanticsNode {
                id: input,
                parent: Some(root),
                role: SemanticsRole::TextField,
                bounds,
                flags: SemanticsFlags {
                    focused: true,
                    ..SemanticsFlags::default()
                },
                test_id: None,
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: None,
                value: Some("he|llo".to_string()),
                extra: SemanticsNodeExtra::default(),
                text_selection: Some((2, 2)),
                text_composition: Some((2, 3)),
                actions: SemanticsActions {
                    focus: true,
                    set_value: true,
                    ..SemanticsActions::default()
                },
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            },
        ],
    };

    let req = accesskit::ActionRequest {
        action: accesskit::Action::ReplaceSelectedText,
        target_tree: accesskit::TreeId::ROOT,
        target_node: to_accesskit_id(input),
        data: Some(accesskit::ActionData::Value("x".into())),
    };
    assert!(
        replace_selected_text_from_action(&req, &snapshot).is_none(),
        "should not mutate text while composing"
    );
}
