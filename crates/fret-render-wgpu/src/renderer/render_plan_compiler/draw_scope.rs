use crate::renderer::PlanTarget;

#[derive(Clone, Copy, Debug)]
pub(super) struct DrawScope {
    pub(super) target: PlanTarget,
    pub(super) origin: (u32, u32),
    pub(super) size: (u32, u32),
    pub(super) needs_clear: bool,
    pub(super) clear_color: wgpu::Color,
}

pub(super) struct DrawScopeStack {
    scopes: Vec<DrawScope>,
}

impl DrawScopeStack {
    pub(super) fn new(root: DrawScope) -> Self {
        Self { scopes: vec![root] }
    }

    pub(super) fn current(&self) -> &DrawScope {
        self.scopes.last().expect("draw scope")
    }

    pub(super) fn current_mut(&mut self) -> &mut DrawScope {
        self.scopes.last_mut().expect("draw scope")
    }

    pub(super) fn push(&mut self, scope: DrawScope) {
        self.scopes.push(scope);
    }

    pub(super) fn pop(&mut self) -> Option<DrawScope> {
        self.scopes.pop()
    }

    pub(super) fn iter(&self) -> std::slice::Iter<'_, DrawScope> {
        self.scopes.iter()
    }

    pub(super) fn contains_target(&self, target: PlanTarget) -> bool {
        self.scopes.iter().any(|scope| scope.target == target)
    }

    pub(super) fn take_load_for_write(&mut self, dst: PlanTarget) -> wgpu::LoadOp<wgpu::Color> {
        let Some(index) = self.scopes.iter().rposition(|s| s.target == dst) else {
            return wgpu::LoadOp::Load;
        };
        if self.scopes[index].needs_clear {
            self.scopes[index].needs_clear = false;
            wgpu::LoadOp::Clear(self.scopes[index].clear_color)
        } else {
            wgpu::LoadOp::Load
        }
    }
}
