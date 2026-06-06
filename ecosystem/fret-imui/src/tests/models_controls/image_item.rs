use super::*;
use fret_core::{ImageId, Px, Size};

mod clicked;
mod context_menu;

fn image_size() -> Size {
    Size::new(Px(48.0), Px(32.0))
}
