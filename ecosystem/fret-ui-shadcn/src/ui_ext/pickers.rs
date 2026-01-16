use crate::calendar::Calendar;
use crate::calendar_range::CalendarRange;
use crate::date_picker::DatePicker;
use crate::date_range_picker::DateRangePicker;
use crate::radio_group::RadioGroup;

impl_ui_patch_chrome_layout!(Calendar);
impl_ui_patch_chrome_layout!(CalendarRange);
impl_ui_patch_passthrough!(DatePicker);
impl_ui_patch_passthrough!(DateRangePicker);
impl_ui_patch_chrome_layout!(RadioGroup);
