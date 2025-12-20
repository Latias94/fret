use crate::editor_shell::DemoSelection;
use crate::hierarchy::{DemoHierarchy, HierarchyMoveOp};
use crate::property::{PropertyPath, PropertyValue};
use crate::world::DemoWorld;
use fret_app::{App, Model};

#[derive(Debug, Default, Clone)]
pub struct UndoStack {
    undo: Vec<EditCommand>,
    redo: Vec<EditCommand>,
    active: Option<EditTransaction>,
}

impl UndoStack {
    pub fn push(&mut self, command: EditCommand) {
        self.active = None;
        self.redo.clear();
        self.undo.push(command);
    }

    pub fn pop_undo(&mut self) -> Option<EditCommand> {
        self.active = None;
        let cmd = self.undo.pop()?;
        self.redo.push(cmd.clone());
        Some(cmd)
    }

    pub fn pop_redo(&mut self) -> Option<EditCommand> {
        self.active = None;
        let cmd = self.redo.pop()?;
        self.undo.push(cmd.clone());
        Some(cmd)
    }

    pub fn cancel_active(&mut self) {
        self.active = None;
    }

    pub fn begin_viewport_translate(&mut self, targets: Vec<u64>, before: Vec<[f32; 3]>) {
        self.active = Some(EditTransaction {
            key: TransactionKey::ViewportTranslate {
                targets: targets.clone(),
            },
            command: EditCommand::SetPositions {
                targets,
                before: before.clone(),
                after: before,
            },
        });
    }

    pub fn update_viewport_translate(&mut self, targets: Vec<u64>, after: Vec<[f32; 3]>) {
        let Some(tx) = self.active.as_mut() else {
            return;
        };
        if tx.key != (TransactionKey::ViewportTranslate { targets }) {
            return;
        }
        let EditCommand::SetPositions {
            after: cmd_after, ..
        } = &mut tx.command
        else {
            return;
        };
        if cmd_after.len() != after.len() {
            return;
        }
        *cmd_after = after;
    }

    pub fn commit_active(&mut self) {
        let Some(tx) = self.active.take() else {
            return;
        };
        if tx.command.is_noop() {
            return;
        }
        self.redo.clear();
        self.undo.push(tx.command);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransactionKey {
    ViewportTranslate { targets: Vec<u64> },
}

#[derive(Debug, Clone)]
struct EditTransaction {
    key: TransactionKey,
    command: EditCommand,
}

#[derive(Debug, Clone)]
pub enum EditCommand {
    SetProperties {
        targets: Vec<u64>,
        path: PropertyPath,
        before: Vec<Option<PropertyValue>>,
        after: PropertyValue,
    },
    SetPositions {
        targets: Vec<u64>,
        before: Vec<[f32; 3]>,
        after: Vec<[f32; 3]>,
    },
    HierarchyMove {
        op: HierarchyMoveOp,
        from_parent: Option<u64>,
        from_index: usize,
        selection: SelectionSnapshot,
    },
}

impl EditCommand {
    pub fn is_noop(&self) -> bool {
        match self {
            EditCommand::SetProperties { before, after, .. } => {
                before.iter().all(|b| b.as_ref() == Some(after))
            }
            EditCommand::SetPositions { before, after, .. } => before == after,
            EditCommand::HierarchyMove { .. } => false,
        }
    }

    pub fn apply(&self, world: &mut DemoWorld) {
        match self {
            EditCommand::SetProperties {
                targets,
                path,
                after,
                ..
            } => {
                world.apply_property_value(targets, path, after.clone());
            }
            EditCommand::SetPositions { targets, after, .. } => {
                for (id, pos) in targets.iter().copied().zip(after.iter().copied()) {
                    world.entity_mut(id).transform.position = pos;
                }
            }
            EditCommand::HierarchyMove { .. } => {}
        }
    }

    pub fn undo(&self, world: &mut DemoWorld) {
        match self {
            EditCommand::SetProperties {
                targets,
                path,
                before,
                ..
            } => {
                for (id, before) in targets.iter().copied().zip(before.iter()) {
                    let Some(value) = before.clone() else {
                        continue;
                    };
                    let _ = world.set_property(id, path, value);
                }
            }
            EditCommand::SetPositions {
                targets, before, ..
            } => {
                for (id, pos) in targets.iter().copied().zip(before.iter().copied()) {
                    world.entity_mut(id).transform.position = pos;
                }
            }
            EditCommand::HierarchyMove { .. } => {}
        }
    }

    pub fn apply_in_app(
        &self,
        app: &mut App,
        hierarchy: Model<DemoHierarchy>,
        selection: Model<DemoSelection>,
    ) {
        match self {
            EditCommand::HierarchyMove {
                op,
                selection: selection_snapshot,
                ..
            } => {
                let _ = hierarchy.update(app, |h, _cx| {
                    let _ = h.apply_move(*op);
                });
                let _ = selection.update(app, |s, _cx| {
                    selection_snapshot.apply_to(s);
                });
            }
            _ => {}
        }
    }

    pub fn undo_in_app(
        &self,
        app: &mut App,
        hierarchy: Model<DemoHierarchy>,
        selection: Model<DemoSelection>,
    ) {
        match self {
            EditCommand::HierarchyMove {
                op,
                from_parent,
                from_index,
                selection: selection_snapshot,
            } => {
                let inverse = HierarchyMoveOp {
                    node: op.node,
                    new_parent: *from_parent,
                    new_index: *from_index,
                };
                let _ = hierarchy.update(app, |h, _cx| {
                    let _ = h.apply_move(inverse);
                });
                let _ = selection.update(app, |s, _cx| {
                    selection_snapshot.apply_to(s);
                });
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSnapshot {
    pub lead_entity: Option<u64>,
    pub selected_entities: Vec<u64>,
}

impl SelectionSnapshot {
    pub fn from_selection(selection: &DemoSelection) -> Self {
        Self {
            lead_entity: selection.lead_entity,
            selected_entities: selection.selected_entities.clone(),
        }
    }

    pub fn apply_to(&self, selection: &mut DemoSelection) {
        selection.lead_entity = self.lead_entity;
        selection.selected_entities = self.selected_entities.clone();
    }
}
