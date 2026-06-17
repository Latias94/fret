use super::{minimum_auto_stack_width, resolve_vec_edit_variant};
use crate::controls::VecEditLayoutVariant;
use fret_core::Px;

#[test]
fn vec_edit_minimum_auto_stack_width_accounts_for_axis_count_and_gaps() {
    assert_eq!(minimum_auto_stack_width(Px(48.0), Px(6.0), 4), Px(210.0));
}

#[test]
fn vec_edit_auto_variant_stacks_only_below_threshold() {
    assert_eq!(
        resolve_vec_edit_variant(VecEditLayoutVariant::Auto, Some(Px(119.0)), Px(120.0)),
        VecEditLayoutVariant::Column
    );
    assert_eq!(
        resolve_vec_edit_variant(VecEditLayoutVariant::Auto, Some(Px(120.0)), Px(120.0)),
        VecEditLayoutVariant::Row
    );
    assert_eq!(
        resolve_vec_edit_variant(VecEditLayoutVariant::Auto, None, Px(120.0)),
        VecEditLayoutVariant::Row
    );
}

#[test]
fn vec_edit_non_auto_variants_do_not_depend_on_bounds() {
    assert_eq!(
        resolve_vec_edit_variant(VecEditLayoutVariant::Row, Some(Px(1.0)), Px(120.0)),
        VecEditLayoutVariant::Row
    );
    assert_eq!(
        resolve_vec_edit_variant(VecEditLayoutVariant::Column, None, Px(120.0)),
        VecEditLayoutVariant::Column
    );
}
