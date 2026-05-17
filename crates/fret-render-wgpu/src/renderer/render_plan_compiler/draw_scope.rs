use crate::renderer::PlanTarget;

#[derive(Clone, Copy, Debug)]
pub(super) struct DrawScope {
    pub(super) target: PlanTarget,
    pub(super) origin: (u32, u32),
    pub(super) size: (u32, u32),
    pub(super) needs_clear: bool,
    pub(super) clear_color: wgpu::Color,
}

pub(super) fn take_scope_load_for_write(
    draw_scopes: &mut Vec<DrawScope>,
    dst: PlanTarget,
) -> wgpu::LoadOp<wgpu::Color> {
    let Some(index) = draw_scopes.iter().rposition(|s| s.target == dst) else {
        return wgpu::LoadOp::Load;
    };
    if draw_scopes[index].needs_clear {
        draw_scopes[index].needs_clear = false;
        wgpu::LoadOp::Clear(draw_scopes[index].clear_color)
    } else {
        wgpu::LoadOp::Load
    }
}
