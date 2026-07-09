use super::super::ImUiDebugDrawList;
use super::super::path_builder::ImUiDebugDrawPath;

impl ImUiDebugDrawList {
    pub fn path<F>(&mut self, build: F)
    where
        F: FnOnce(&mut ImUiDebugDrawPath<'_>),
    {
        let mut path = ImUiDebugDrawPath::new(self);
        build(&mut path);
    }

    pub fn command_count(&self) -> usize {
        let split_count = self
            .channel_split
            .as_ref()
            .map(|split| split.channels.iter().map(Vec::len).sum())
            .unwrap_or(0);
        self.commands.len() + split_count
    }

    pub fn is_empty(&self) -> bool {
        self.command_count() == 0
    }
}
