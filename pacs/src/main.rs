use clap::{CommandFactory, Parser};
use clap_complete::env::CompleteEnv;
use pacs_cli::Cli;

fn main() -> anyhow::Result<()> {
    CompleteEnv::with_factory(Cli::command).complete();
    pacs_cli::run(Cli::parse())
}
