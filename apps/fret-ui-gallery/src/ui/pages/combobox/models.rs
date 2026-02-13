use super::prelude::*;

#[derive(Default, Clone)]
struct ComboboxModelsState {
    custom_value: Option<Model<Option<Arc<str>>>>,
    custom_open: Option<Model<bool>>,
    custom_query: Option<Model<String>>,
    basic_value: Option<Model<Option<Arc<str>>>>,
    basic_open: Option<Model<bool>>,
    basic_query: Option<Model<String>>,
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
    pub(super) custom_value: Model<Option<Arc<str>>>,
    pub(super) custom_open: Model<bool>,
    pub(super) custom_query: Model<String>,
    pub(super) basic_value: Model<Option<Arc<str>>>,
    pub(super) basic_open: Model<bool>,
    pub(super) basic_query: Model<String>,
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
            custom_value: state.custom_value?,
            custom_open: state.custom_open?,
            custom_query: state.custom_query?,
            basic_value: state.basic_value?,
            basic_open: state.basic_open?,
            basic_query: state.basic_query?,
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

    let custom_value = cx.app.models_mut().insert(None);
    let custom_open = cx.app.models_mut().insert(false);
    let custom_query = cx.app.models_mut().insert(String::new());

    let basic_value = cx.app.models_mut().insert(None);
    let basic_open = cx.app.models_mut().insert(false);
    let basic_query = cx.app.models_mut().insert(String::new());

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
        st.custom_value = Some(custom_value.clone());
        st.custom_open = Some(custom_open.clone());
        st.custom_query = Some(custom_query.clone());

        st.basic_value = Some(basic_value.clone());
        st.basic_open = Some(basic_open.clone());
        st.basic_query = Some(basic_query.clone());

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
        custom_value,
        custom_open,
        custom_query,
        basic_value,
        basic_open,
        basic_query,
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
