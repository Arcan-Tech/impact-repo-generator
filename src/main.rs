use std::{error::Error, str::FromStr};

use args::Args;
use clap::Parser;
use log::{error, LevelFilter};
use stat::git::MarkovRepositoryGenerator;

pub mod args;
pub mod git;
pub mod input;
pub mod stat;

//fn main() {
//    let cli: Cli = Args::parse().into();
//    cli.init_logger();
//    let mut gw = cli.get_git_writer().unwrap();
//    match gw.generate() {
//        Ok(_) => {}
//        Err(e) => {
//            error!("Failed execution: {}", e)
//        }
//    };
//}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            error!("Execution failed: {}", e);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut mrg = MarkovRepositoryGenerator::try_from(args)?;
    mrg.generate()?;
    Ok(())
}
