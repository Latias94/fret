use crate::property::{PropertyPath, PropertyValue};
use crate::world::DemoWorld;

#[derive(Debug, Default, Clone)]
pub struct UndoStack {
    undo: Vec<EditCommand>,
    redo: Vec<EditCommand>,
}

impl UndoStack {
    pub fn push(&mut self, command: EditCommand) {
        self.redo.clear();
        self.undo.push(command);
    }

    pub fn pop_undo(&mut self) -> Option<EditCommand> {
        let cmd = self.undo.pop()?;
        self.redo.push(cmd.clone());
        Some(cmd)
    }

    pub fn pop_redo(&mut self) -> Option<EditCommand> {
        let cmd = self.redo.pop()?;
        self.undo.push(cmd.clone());
        Some(cmd)
    }
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
}

impl EditCommand {
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
        }
    }
}
