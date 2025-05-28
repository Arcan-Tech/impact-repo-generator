use args::{Args, Cli};
use clap::Parser;
use log::error;

pub mod args;
pub mod git;
pub mod input;

fn main() {
    let cli: Cli = Args::parse().into();
    cli.init_logger();
    let mut gw = cli.get_git_writer().unwrap();
    match gw.generate() {
        Ok(_) => {}
        Err(e) => {
            error!("Failed execution: {}", e)
        }
    };
}
