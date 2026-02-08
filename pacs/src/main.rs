use clap::{CommandFactory, Parser};
use clap_complete::env::CompleteEnv;
use pacs_cli::Cli;

fn main() -> anyhow::Result<()> {
    CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    if cli.ui {
        return pacs_tui::run();
    }

    pacs_cli::run(cli)
}
