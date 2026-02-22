use super::prelude::*;

#[derive(Default, Clone)]
struct ComboboxModelsState {
    clear_value: Option<Model<Option<Arc<str>>>>,
    clear_open: Option<Model<bool>>,
    clear_query: Option<Model<String>>,
    custom_value: Option<Model<Option<Arc<str>>>>,
    custom_open: Option<Model<bool>>,
    custom_query: Option<Model<String>>,
    long_list_value: Option<Model<Option<Arc<str>>>>,
    long_list_open: Option<Model<bool>>,
    long_list_query: Option<Model<String>>,
    groups_value: Option<Model<Option<Arc<str>>>>,
    groups_open: Option<Model<bool>>,
    groups_query: Option<Model<String>>,
    invalid_value: Option<Model<Option<Arc<str>>>>,
    invalid_open: Option<Model<bool>>,
    invalid_query: Option<Model<String>>,
    disabled_value: Option<Model<Option<Arc<str>>>>,
    disabled_open: Option<Model<bool>>,
    disabled_query: Option<Model<String>>,
    input_group_value: Option<Model<Option<Arc<str>>>>,
    input_group_open: Option<Model<bool>>,
    input_group_query: Option<Model<String>>,
    rtl_value: Option<Model<Option<Arc<str>>>>,
    rtl_open: Option<Model<bool>>,
    rtl_query: Option<Model<String>>,
}

#[derive(Clone)]
pub(super) struct ComboboxModels {
    pub(super) clear_value: Model<Option<Arc<str>>>,
    pub(super) clear_open: Model<bool>,
    pub(super) clear_query: Model<String>,
    pub(super) custom_value: Model<Option<Arc<str>>>,
    pub(super) custom_open: Model<bool>,
    pub(super) custom_query: Model<String>,
    pub(super) long_list_value: Model<Option<Arc<str>>>,
    pub(super) long_list_open: Model<bool>,
    pub(super) long_list_query: Model<String>,
    pub(super) groups_value: Model<Option<Arc<str>>>,
    pub(super) groups_open: Model<bool>,
    pub(super) groups_query: Model<String>,
    pub(super) invalid_value: Model<Option<Arc<str>>>,
    pub(super) invalid_open: Model<bool>,
    pub(super) invalid_query: Model<String>,
    pub(super) disabled_value: Model<Option<Arc<str>>>,
    pub(super) disabled_open: Model<bool>,
    pub(super) disabled_query: Model<String>,
    pub(super) input_group_value: Model<Option<Arc<str>>>,
    pub(super) input_group_open: Model<bool>,
    pub(super) input_group_query: Model<String>,
    pub(super) rtl_value: Model<Option<Arc<str>>>,
    pub(super) rtl_open: Model<bool>,
    pub(super) rtl_query: Model<String>,
}

impl ComboboxModels {
    fn try_from_state(state: ComboboxModelsState) -> Option<Self> {
        Some(ComboboxModels {
            clear_value: state.clear_value?,
            clear_open: state.clear_open?,
            clear_query: state.clear_query?,
            custom_value: state.custom_value?,
            custom_open: state.custom_open?,
            custom_query: state.custom_query?,
            long_list_value: state.long_list_value?,
            long_list_open: state.long_list_open?,
            long_list_query: state.long_list_query?,
            groups_value: state.groups_value?,
            groups_open: state.groups_open?,
            groups_query: state.groups_query?,
            invalid_value: state.invalid_value?,
            invalid_open: state.invalid_open?,
            invalid_query: state.invalid_query?,
            disabled_value: state.disabled_value?,
            disabled_open: state.disabled_open?,
            disabled_query: state.disabled_query?,
            input_group_value: state.input_group_value?,
            input_group_open: state.input_group_open?,
            input_group_query: state.input_group_query?,
            rtl_value: state.rtl_value?,
            rtl_open: state.rtl_open?,
            rtl_query: state.rtl_query?,
        })
    }
}

pub(super) fn get_or_init(cx: &mut ElementContext<'_, App>) -> ComboboxModels {
    let state = cx.with_state(ComboboxModelsState::default, |st| st.clone());
    if let Some(models) = ComboboxModels::try_from_state(state) {
        return models;
    }

    let clear_value = cx.app.models_mut().insert(Some(Arc::<str>::from("next")));
    let clear_open = cx.app.models_mut().insert(false);
    let clear_query = cx.app.models_mut().insert(String::new());

    let custom_value = cx.app.models_mut().insert(None);
    let custom_open = cx.app.models_mut().insert(false);
    let custom_query = cx.app.models_mut().insert(String::new());

    let long_list_value = cx.app.models_mut().insert(None);
    let long_list_open = cx.app.models_mut().insert(false);
    let long_list_query = cx.app.models_mut().insert(String::new());

    let groups_value = cx.app.models_mut().insert(None);
    let groups_open = cx.app.models_mut().insert(false);
    let groups_query = cx.app.models_mut().insert(String::new());

    let invalid_value = cx.app.models_mut().insert(None);
    let invalid_open = cx.app.models_mut().insert(false);
    let invalid_query = cx.app.models_mut().insert(String::new());

    let disabled_value = cx.app.models_mut().insert(Some(Arc::<str>::from("banana")));
    let disabled_open = cx.app.models_mut().insert(false);
    let disabled_query = cx.app.models_mut().insert(String::new());

    let input_group_value = cx.app.models_mut().insert(None);
    let input_group_open = cx.app.models_mut().insert(false);
    let input_group_query = cx.app.models_mut().insert(String::new());

    let rtl_value = cx.app.models_mut().insert(None);
    let rtl_open = cx.app.models_mut().insert(false);
    let rtl_query = cx.app.models_mut().insert(String::new());

    cx.with_state(ComboboxModelsState::default, |st| {
        st.clear_value = Some(clear_value.clone());
        st.clear_open = Some(clear_open.clone());
        st.clear_query = Some(clear_query.clone());

        st.custom_value = Some(custom_value.clone());
        st.custom_open = Some(custom_open.clone());
        st.custom_query = Some(custom_query.clone());

        st.long_list_value = Some(long_list_value.clone());
        st.long_list_open = Some(long_list_open.clone());
        st.long_list_query = Some(long_list_query.clone());

        st.groups_value = Some(groups_value.clone());
        st.groups_open = Some(groups_open.clone());
        st.groups_query = Some(groups_query.clone());

        st.invalid_value = Some(invalid_value.clone());
        st.invalid_open = Some(invalid_open.clone());
        st.invalid_query = Some(invalid_query.clone());

        st.disabled_value = Some(disabled_value.clone());
        st.disabled_open = Some(disabled_open.clone());
        st.disabled_query = Some(disabled_query.clone());

        st.input_group_value = Some(input_group_value.clone());
        st.input_group_open = Some(input_group_open.clone());
        st.input_group_query = Some(input_group_query.clone());

        st.rtl_value = Some(rtl_value.clone());
        st.rtl_open = Some(rtl_open.clone());
        st.rtl_query = Some(rtl_query.clone());
    });

    ComboboxModels {
        clear_value,
        clear_open,
        clear_query,
        custom_value,
        custom_open,
        custom_query,
        long_list_value,
        long_list_open,
        long_list_query,
        groups_value,
        groups_open,
        groups_query,
        invalid_value,
        invalid_open,
        invalid_query,
        disabled_value,
        disabled_open,
        disabled_query,
        input_group_value,
        input_group_open,
        input_group_query,
        rtl_value,
        rtl_open,
        rtl_query,
    }
}
