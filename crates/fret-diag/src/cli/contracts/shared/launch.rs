use clap::{ArgAction, Args};

#[derive(Debug, Args)]
pub(crate) struct LaunchArgs {
    #[arg(long = "env", value_name = "KEY=VALUE", action = ArgAction::Append)]
    pub env: Vec<String>,

    #[arg(long = "launch-high-priority", requires = "launch")]
    pub launch_high_priority: bool,

    #[arg(long = "launch-write-bundle-json", requires = "launch")]
    pub launch_write_bundle_json: bool,

    #[arg(long = "keep-open", requires = "launch")]
    pub keep_open: bool,

    #[arg(
        long = "launch",
        value_name = "CMD",
        num_args = 1..,
        allow_hyphen_values = true
    )]
    pub launch: Option<Vec<String>>,
}

impl LaunchArgs {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn normalized_launch_argv(&self) -> Option<Vec<String>> {
        self.launch.as_ref().map(|argv| {
            let mut values = argv.clone();
            if values.first().is_some_and(|value| value == "--") {
                values.remove(0);
            }
            values
        })
    }
}
