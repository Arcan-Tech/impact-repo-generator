use args::{Args, Command};
use chrono::{Duration, NaiveDateTime, Utc};
use clap::Parser;
use generator::{
    commit::{GitWriter, MarkovCommitGenerator},
    markov::MarkovProcess,
    repository::MarkovRepositoryGenerator,
    utils::TimestampGenerator,
};
use log::{error, info, trace, LevelFilter};
use std::error::Error;
use std::str::FromStr;

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

    match args.command {
        Command::Generate {
            markov,
            repository,
            commits,
            start,
            hours,
            log,
        } => {
            env_logger::builder()
                .filter_level(LevelFilter::from_str(&log).unwrap_or(LevelFilter::Info))
                .init();

            info!("Reading markov model from {}", markov.display());
            let markov_model = MarkovProcess::from_yaml(markov)?;

            info!(
                "Model loaded with {} states and {} transitions",
                markov_model.num_states(),
                markov_model.num_transitions()
            );
            trace!("{:#?}", markov_model);

            info!("Opening repository at {}", repository.display());
            let writer = GitWriter::new(repository)?;
            let start_dt = NaiveDateTime::from_str(&start)?;
            let ts_gen = TimestampGenerator::new(start_dt, Duration::hours(hours));
            let commit_gen = MarkovCommitGenerator::new(commits, markov_model, ts_gen);
            let mut repo_gen = MarkovRepositoryGenerator::new(writer, commit_gen);
            repo_gen.generate()?;
        }

        Command::Dot { markov, output } => {
            let model = MarkovProcess::from_yaml(markov)?;
            let dot = model.to_dot();
            std::fs::write(output, dot)?;
        }
    }

    Ok(())
}
