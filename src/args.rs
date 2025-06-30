use std::{error::Error, path::PathBuf, str::FromStr};

use chrono::{Duration, NaiveDateTime};
use clap::Parser;
use log::{info, trace, LevelFilter};

use crate::generator::{
    commit::{GitWriter, MarkovCommitGenerator},
    markov::MarkovProcess,
    repository::MarkovRepositoryGenerator,
    utils::TimestampGenerator,
};
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, help = "Markov model description YAML file.")]
    markov: PathBuf,

    #[arg(short, long, help = "Destination repository path")]
    repository: PathBuf,

    #[arg(
        short,
        long,
        default_value_t = 10,
        help = "Number of commits to generate"
    )]
    commits: u32,

    #[arg(
        long,
        default_value = "2023-01-01T12:00:00",
        help = "Generate commits starting at given date (YYYY-MM-DD)"
    )]
    start: String,

    #[arg(
        long,
        default_value_t = 10,
        help = "Average interval time between commits (hours)"
    )]
    hours: i64,

    #[arg(
        long,
        default_value = "INFO",
        help = "Logging level (trace, error, debug, warn, info)"
    )]
    log: String,
}

impl TryFrom<Args> for MarkovRepositoryGenerator {
    type Error = Box<dyn Error>;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        env_logger::builder()
            .filter_level(LevelFilter::from_str(&args.log).unwrap_or(LevelFilter::Info))
            .init();
        info!(
            "Reading markov model from {}",
            args.markov.to_str().unwrap()
        );
        let markov = MarkovProcess::from_yaml(args.markov)?;
        info!(
            "Markov model contains {} states and {} transactions",
            markov.num_states(),
            markov.num_transitions()
        );
        trace!("{}", markov);

        info!(
            "Opening repository at {}",
            args.repository.to_str().unwrap()
        );
        let writer = GitWriter::new(args.repository)?;
        let start =
            NaiveDateTime::from_str(&args.start).with_context(|| "Cannot parse start time")?;
        let ts = TimestampGenerator::new(start, Duration::hours(args.hours));
        info!("Generating {} commits...", args.commits);
        let markov = MarkovCommitGenerator::new(args.commits, markov, ts);
        let markov = MarkovRepositoryGenerator::new(writer, markov);
        Ok(markov)
    }
}
