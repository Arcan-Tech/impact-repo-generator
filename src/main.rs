use args::Args;
use clap::Parser;
use generator::repository::MarkovRepositoryGenerator;
use log::error;
use std::convert::TryFrom;
use std::error::Error;

pub mod args;
pub mod generator;

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
