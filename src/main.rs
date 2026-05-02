use anyhow::Result;
use clap::Parser;
use ndlocr_lite_rs::app::run_cli;
use ndlocr_lite_rs::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    run_cli(cli)
}
