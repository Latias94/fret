mod mode;
mod parse;
mod text;

pub(in crate::controls::color_edit) use mode::{ColorNumericInputMode, color_numeric_input_modes};
pub(in crate::controls::color_edit) use parse::parse_color_numeric_input;
pub(in crate::controls::color_edit) use text::{
    color_numeric_text, hsv_numeric_text, rgb_numeric_text,
};
