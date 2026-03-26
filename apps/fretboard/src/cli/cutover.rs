use super::contracts::{
    DevCommandArgs, DevTargetContract, FretboardCliContract, FretboardCommandContract,
    ListCommandArgs, ListTargetContract,
};

pub(crate) fn dispatch(contract: FretboardCliContract) -> Result<(), String> {
    match contract.command {
        FretboardCommandContract::Assets(args) => crate::assets::run_assets_contract(args),
        FretboardCommandContract::Config(args) => crate::config::config_cmd(args.args),
        FretboardCommandContract::Dev(args) => dispatch_dev(args),
        FretboardCommandContract::Diag(args) => crate::diag::diag_cmd(args.args),
        FretboardCommandContract::Hotpatch(args) => crate::hotpatch::run_hotpatch_contract(args),
        FretboardCommandContract::Init(args) => crate::scaffold::init_cmd(args.args),
        FretboardCommandContract::List(args) => dispatch_list(args),
        FretboardCommandContract::New(args) => crate::scaffold::new_cmd(args.args),
        FretboardCommandContract::Theme(args) => crate::theme::theme_cmd(args.args),
    }
}

fn dispatch_dev(args: DevCommandArgs) -> Result<(), String> {
    match args.target {
        DevTargetContract::Native(args) => crate::dev::run_native_contract(args),
        DevTargetContract::Web(args) => crate::dev::run_web_contract(args),
    }
}

fn dispatch_list(args: ListCommandArgs) -> Result<(), String> {
    match args.target {
        ListTargetContract::NativeDemos(args) => {
            crate::demos::list_native_demos(list_all_args(args.all))
        }
        ListTargetContract::WebDemos(_) => crate::demos::list_web_demos(Vec::new()),
        ListTargetContract::CookbookExamples(args) => {
            crate::demos::list_cookbook_examples(list_all_args(args.all))
        }
    }
}

fn list_all_args(all: bool) -> Vec<String> {
    if all {
        vec!["--all".to_string()]
    } else {
        Vec::new()
    }
}
