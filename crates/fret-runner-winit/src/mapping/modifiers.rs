use fret_core::Modifiers;
use winit::keyboard::ModifiersState;

pub fn map_modifiers(state: ModifiersState, alt_gr_down: bool) -> Modifiers {
    let mut mods = Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        alt_gr: alt_gr_down,
        meta: state.meta_key(),
    };

    if mods.alt_gr {
        mods.ctrl = false;
        mods.alt = false;
    }

    mods
}
