use args::Args;
use chrono::Utc;
use clap::Parser;
use generator::repository::MarkovRepositoryGenerator;
use log::{error, info};
use std::convert::TryFrom;
use std::error::Error;

pub mod args;
pub mod generator;

fn main() {
    let start = Utc::now();
    match run() {
        Ok(_) => {}
        Err(e) => {
            error!("Execution failed: {}", e);
        }
    }
    let end = Utc::now();
    let duration = end - start;
    let duration = duration.num_seconds();
    info!("Execution time {}s", duration)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut mrg = MarkovRepositoryGenerator::try_from(args)?;
    mrg.generate()?;
    Ok(())
}
