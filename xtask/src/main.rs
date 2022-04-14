use khonsu_tools::{
    publish,
    universal::{anyhow, audit, clap::Parser, DefaultConfig},
};

fn main() -> anyhow::Result<()> {
    let command = khonsu_tools::Commands::parse();
    command.execute::<Config>()
}

enum Config {}

impl khonsu_tools::Config for Config {
    type Publish = Self;

    type Universal = Self;
}

impl khonsu_tools::universal::Config for Config {
    type Audit = Self;

    type CodeCoverage = DefaultConfig;
}

impl audit::Config for Config {
    fn args() -> Vec<String> {
        vec![
            String::from("--all-features"),
            String::from("--exclude=xtask"),
            String::from("--exclude=benchmarks"),
        ]
    }
}

impl publish::Config for Config {
    fn paths() -> Vec<String> {
        vec![String::from(".")]
    }
}
