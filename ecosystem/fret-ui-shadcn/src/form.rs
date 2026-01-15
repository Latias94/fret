//! shadcn/ui `Form` facade (taxonomy + recipes).
//!
//! Upstream shadcn's `Form` is tightly integrated with `react-hook-form`. In Fret, we expose a
//! small, framework-agnostic surface focused on composition and consistent spacing.
//!
//! - `Form` maps to a vertical `FieldSet` container.
//! - `FormItem` maps to `Field` (label + control + description + message).
//! - `FormMessage` maps to `FieldError` (destructive text).

#[path = "form_field.rs"]
mod form_field;

pub use crate::field::Field as FormItem;
pub use crate::field::FieldContent as FormControl;
pub use crate::field::FieldDescription as FormDescription;
pub use crate::field::FieldError as FormMessage;
pub use crate::field::FieldLabel as FormLabel;
pub use crate::field::FieldSet as Form;
pub use crate::field::field_set as form;
pub use form_field::{FormErrorVisibility, FormField};
