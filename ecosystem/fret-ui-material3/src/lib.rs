mod button;
mod checkbox;
mod radio_group;
mod select;
mod switch;
mod text_field;

pub use button::{Button, ButtonStyle, ButtonVariant};
pub use checkbox::{Checkbox, CheckboxStyle};
pub use radio_group::{RadioGroup, RadioGroupItem, RadioGroupStyle};
pub use select::{Select, SelectItem, SelectStyle};
pub use switch::{Switch, SwitchStyle};
pub use text_field::{TextField, TextFieldStyle};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_surfaces_are_constructible() {
        let _ = Button::new("Test").variant(ButtonVariant::Filled);
        let _ = SelectItem::new("value", "Label").disabled(true);
        let _ = RadioGroupItem::new("value", "Label").disabled(true);
    }
}
