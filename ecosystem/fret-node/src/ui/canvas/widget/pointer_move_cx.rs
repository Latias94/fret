use fret_ui::UiHost;

use super::{
    hover_move_cx::HoverMoveCx, primary_pointer_move_cx::PrimaryPointerMoveCx,
    secondary_pointer_move_cx::SecondaryPointerMoveCx,
};

pub(super) trait PointerMoveCx<H: UiHost>:
    PrimaryPointerMoveCx<H> + SecondaryPointerMoveCx<H> + HoverMoveCx<H>
{
}

impl<H, T> PointerMoveCx<H> for T
where
    H: UiHost,
    T: PrimaryPointerMoveCx<H> + SecondaryPointerMoveCx<H> + HoverMoveCx<H>,
{
}
