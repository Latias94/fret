use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Toggled, Tree, TreeUpdate};
use accesskit_winit::Adapter;
use fret_core::{SemanticsNode, SemanticsRole, SemanticsSnapshot};
use slotmap::{Key, KeyData};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

const ROOT_ID: NodeId = NodeId(0);

fn to_accesskit_id(node: fret_core::NodeId) -> NodeId {
    NodeId(node.data().as_ffi().wrapping_add(1))
}

fn from_accesskit_id(node: NodeId) -> Option<fret_core::NodeId> {
    if node.0 == 0 {
        return None;
    }
    Some(fret_core::NodeId::from(KeyData::from_ffi(
        node.0.wrapping_sub(1),
    )))
}

pub struct WinitAccessibility {
    adapter: Adapter,
    actions_rx: mpsc::Receiver<ActionRequest>,
    pending_actions: Arc<Mutex<Vec<ActionRequest>>>,
    activation_requested: Arc<AtomicBool>,
    is_active: Arc<AtomicBool>,
}

impl WinitAccessibility {
    pub fn new(event_loop: &ActiveEventLoop, window: &Window) -> Self {
        let (actions_tx, actions_rx) = mpsc::channel::<ActionRequest>();
        let pending_actions: Arc<Mutex<Vec<ActionRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let activation_requested: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let is_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let pending_actions_for_handler = pending_actions.clone();
        let actions_tx_for_handler = actions_tx.clone();
        let activation_requested_for_handler = activation_requested.clone();
        let is_active_for_handler = is_active.clone();

        struct ActivationHandlerImpl {
            requested: Arc<AtomicBool>,
            active: Arc<AtomicBool>,
        }

        impl accesskit::ActivationHandler for ActivationHandlerImpl {
            fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
                self.requested.store(true, Ordering::SeqCst);
                self.active.store(true, Ordering::SeqCst);
                None
            }
        }

        struct ActionHandlerImpl {
            pending: Arc<Mutex<Vec<ActionRequest>>>,
            tx: mpsc::Sender<ActionRequest>,
        }

        impl accesskit::ActionHandler for ActionHandlerImpl {
            fn do_action(&mut self, request: ActionRequest) {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.push(request.clone());
                }
                let _ = self.tx.send(request);
            }
        }

        struct DeactivationHandlerImpl {
            active: Arc<AtomicBool>,
        }

        impl accesskit::DeactivationHandler for DeactivationHandlerImpl {
            fn deactivate_accessibility(&mut self) {
                self.active.store(false, Ordering::SeqCst);
            }
        }

        let adapter = Adapter::with_direct_handlers(
            event_loop,
            window,
            ActivationHandlerImpl {
                requested: activation_requested_for_handler,
                active: is_active_for_handler.clone(),
            },
            ActionHandlerImpl {
                pending: pending_actions_for_handler,
                tx: actions_tx_for_handler,
            },
            DeactivationHandlerImpl {
                active: is_active_for_handler,
            },
        );

        Self {
            adapter,
            actions_rx,
            pending_actions,
            activation_requested,
            is_active,
        }
    }

    pub fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn take_activation_request(&self) -> bool {
        self.activation_requested.swap(false, Ordering::SeqCst)
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub fn drain_actions(&mut self, out: &mut Vec<ActionRequest>) {
        while let Ok(req) = self.actions_rx.try_recv() {
            out.push(req);
        }
    }

    pub fn drain_actions_fallback(&mut self, out: &mut Vec<ActionRequest>) {
        if let Ok(mut pending) = self.pending_actions.lock() {
            out.append(&mut *pending);
        }
    }
}

fn map_role(role: SemanticsRole) -> Role {
    match role {
        SemanticsRole::Generic => Role::GenericContainer,
        SemanticsRole::Window => Role::Pane,
        SemanticsRole::Panel => Role::Pane,
        SemanticsRole::Dialog => Role::Dialog,
        SemanticsRole::Alert => Role::Alert,
        SemanticsRole::Button => Role::Button,
        SemanticsRole::Checkbox => Role::CheckBox,
        SemanticsRole::Switch => Role::Switch,
        SemanticsRole::Slider => Role::Slider,
        SemanticsRole::ComboBox => Role::ComboBox,
        SemanticsRole::Tab => Role::Tab,
        SemanticsRole::MenuBar => Role::MenuBar,
        SemanticsRole::Menu => Role::Menu,
        SemanticsRole::MenuItem => Role::MenuItem,
        SemanticsRole::Text => Role::Label,
        SemanticsRole::TextField => Role::TextInput,
        SemanticsRole::List => Role::List,
        SemanticsRole::ListItem => Role::ListItem,
        SemanticsRole::TreeItem => Role::TreeItem,
        SemanticsRole::Viewport => Role::ScrollView,
        _ => Role::GenericContainer,
    }
}

fn px_rect_to_accesskit(bounds: fret_core::Rect, scale_factor: f64) -> Rect {
    let x0 = bounds.origin.x.0 as f64 * scale_factor;
    let y0 = bounds.origin.y.0 as f64 * scale_factor;
    let x1 = (bounds.origin.x.0 + bounds.size.width.0) as f64 * scale_factor;
    let y1 = (bounds.origin.y.0 + bounds.size.height.0) as f64 * scale_factor;
    Rect { x0, y0, x1, y1 }
}

fn choose_visible_roots(snapshot: &SemanticsSnapshot) -> Vec<fret_core::NodeId> {
    let mut roots: Vec<_> = snapshot.roots.iter().filter(|r| r.visible).collect();
    roots.sort_by_key(|r| r.z_index);

    if let Some(barrier_root) = snapshot.barrier_root
        && let Some(barrier) = roots.iter().find(|r| r.root == barrier_root)
    {
        let barrier_z = barrier.z_index;
        return roots
            .into_iter()
            .filter(|r| r.z_index >= barrier_z)
            .map(|r| r.root)
            .collect();
    }

    roots.into_iter().map(|r| r.root).collect()
}

fn build_children_index(
    nodes: &[SemanticsNode],
) -> HashMap<fret_core::NodeId, Vec<fret_core::NodeId>> {
    let mut by_parent: HashMap<fret_core::NodeId, Vec<fret_core::NodeId>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent {
            by_parent.entry(parent).or_default().push(node.id);
        }
    }
    by_parent
}

fn collect_reachable(
    roots: &[fret_core::NodeId],
    children: &HashMap<fret_core::NodeId, Vec<fret_core::NodeId>>,
) -> HashSet<fret_core::NodeId> {
    let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
    let mut stack: Vec<fret_core::NodeId> = roots.to_vec();
    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }
        if let Some(kids) = children.get(&node) {
            for &child in kids.iter().rev() {
                stack.push(child);
            }
        }
    }
    seen
}

pub fn tree_update_from_snapshot(snapshot: &SemanticsSnapshot, scale_factor: f64) -> TreeUpdate {
    let visible_roots = choose_visible_roots(snapshot);
    let children = build_children_index(&snapshot.nodes);
    let reachable = collect_reachable(&visible_roots, &children);

    let mut nodes_out: Vec<(NodeId, Node)> = Vec::new();

    let mut root = Node::new(Role::Window);
    root.set_children(
        visible_roots
            .iter()
            .copied()
            .map(to_accesskit_id)
            .collect::<Vec<_>>(),
    );
    nodes_out.push((ROOT_ID, root));

    for node in &snapshot.nodes {
        if !reachable.contains(&node.id) {
            continue;
        }

        let mut out = Node::new(map_role(node.role));
        out.set_bounds(px_rect_to_accesskit(node.bounds, scale_factor));

        if let Some(children) = children.get(&node.id) {
            out.set_children(
                children
                    .iter()
                    .copied()
                    .map(to_accesskit_id)
                    .collect::<Vec<_>>(),
            );
        }

        if node.flags.disabled {
            out.set_disabled();
        }
        if node.flags.selected {
            out.set_selected(true);
        }
        if node.flags.expanded {
            out.set_expanded(true);
        }
        if let Some(checked) = node.flags.checked {
            out.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        }

        if node.actions.focus {
            out.add_action(Action::Focus);
        }

        if node.actions.invoke {
            out.add_action(Action::Click);
        }
        if node.actions.set_value {
            out.add_action(Action::SetValue);
        }

        if let Some(label) = node.label.as_ref() {
            match node.role {
                SemanticsRole::Text => out.set_value(label.clone()),
                _ => out.set_label(label.clone()),
            }
        }
        if let Some(value) = node.value.as_ref() {
            out.set_value(value.clone());
        }

        if let Some(active) = node.active_descendant
            && reachable.contains(&active)
        {
            out.set_active_descendant(to_accesskit_id(active));
        }

        nodes_out.push((to_accesskit_id(node.id), out));
    }

    let focus = snapshot
        .focus
        .filter(|id| reachable.contains(id))
        .map(to_accesskit_id)
        .unwrap_or(ROOT_ID);

    TreeUpdate {
        nodes: nodes_out,
        tree: Some(Tree {
            root: ROOT_ID,
            toolkit_name: Some("fret".to_string()),
            toolkit_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        focus,
    }
}

pub fn focus_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Focus {
        return None;
    }
    from_accesskit_id(req.target)
}

pub fn invoke_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Click {
        return None;
    }
    from_accesskit_id(req.target)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetValueData {
    Text(String),
    Numeric(f64),
}

pub fn set_value_from_action(req: &ActionRequest) -> Option<(fret_core::NodeId, SetValueData)> {
    if !matches!(req.action, Action::SetValue | Action::ReplaceSelectedText) {
        return None;
    }

    let target = from_accesskit_id(req.target)?;
    let data = req.data.as_ref()?;
    match data {
        accesskit::ActionData::Value(v) => Some((target, SetValueData::Text(v.to_string()))),
        accesskit::ActionData::NumericValue(v) => Some((target, SetValueData::Numeric(*v))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{to_accesskit_id, tree_update_from_snapshot};
    use fret_core::{
        AppWindowId, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRole,
        SemanticsRoot, SemanticsSnapshot,
    };
    use slotmap::KeyData;

    fn node(id: u64) -> fret_core::NodeId {
        fret_core::NodeId::from(KeyData::from_ffi(id))
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
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    active_descendant: None,
                    label: None,
                    value: None,
                    actions: SemanticsActions::default(),
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
                    active_descendant: Some(item),
                    label: Some("Command input".to_string()),
                    value: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                },
                SemanticsNode {
                    id: list,
                    parent: Some(root),
                    role: SemanticsRole::List,
                    bounds,
                    flags: SemanticsFlags::default(),
                    active_descendant: None,
                    label: None,
                    value: None,
                    actions: SemanticsActions::default(),
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
                    active_descendant: None,
                    label: Some("Item 1".to_string()),
                    value: None,
                    actions: SemanticsActions::default(),
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
}
