use fret_render::WgpuContext;

use super::window_redraw_present_target::WindowRedrawPresentTarget;

pub(super) struct WindowRedrawPresentSubmitInput<'a> {
    pub(super) context: &'a WgpuContext,
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    pub(super) present_target: WindowRedrawPresentTarget,
}

pub(super) fn submit_window_redraw_present_frame(input: WindowRedrawPresentSubmitInput<'_>) {
    input.context.queue.submit(input.command_buffers);
    if let Some((frame, _view)) = input.present_target.into_frame_view() {
        frame.present();
    }
}
