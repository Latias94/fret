use fret_ui::UiHost;

use super::super::{low_level_adapter, marquee_cx::MarqueeCx, wire_drag::WireCommitCx};

pub(in crate::ui::canvas::widget) trait LeftClickCx<H: UiHost>:
    MarqueeCx<H> + WireCommitCx<H>
{
    fn left_click_host(&mut self) -> &mut H;
}

impl<H, T> LeftClickCx<H> for T
where
    H: UiHost,
    T: MarqueeCx<H> + WireCommitCx<H>,
{
    fn left_click_host(&mut self) -> &mut H {
        <T as WireCommitCx<H>>::host(self)
    }
}

pub(super) fn capture_pointer_and_invalidate_paint<H: UiHost>(cx: &mut impl LeftClickCx<H>) {
    cx.capture_self_pointer();
    low_level_adapter::invalidate_canvas_paint(cx);
}
