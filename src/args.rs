use std::{error::Error, path::PathBuf, str::FromStr};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use clap::Parser;
use log::{debug, info, trace, LevelFilter};

use crate::{
    git::{
        AuthorGenerator, CCPairGenerator, CommitGenerator, GitWriter, IssueGenerator,
        TimeStampGenerator,
    },
    input::CCModel,
    stat::{
        git::{MarkovCommitGenerator, MarkovRepositoryGenerator},
        markov::MarkovProcess,
    },
};
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    probabilites: PathBuf,

    #[arg(short, long)]
    repository: PathBuf,

    #[arg(short, long, default_value_t = 10)]
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

    #[arg(long, help = "Ovewrite existing repository")]
    force: bool,

    #[arg(
        short = 'C',
        long,
        default_value_t = 3,
        help = "Average commits per issue"
    )]
    commits_per_issue: u32,

    #[arg(short = 'a', default_value_t = 3)]
    n_authors: u32,

    #[arg(short = 'A', default_value_t = 3)]
    commits_per_author: u32,

    #[arg(
        long,
        default_value = "#",
        help = "Prefix to mark issues in commit messages"
    )]
    issue_prefix: String,

    #[arg(long, default_value = "INFO")]
    log: String,
}

pub struct Cli {
    args: Args,
}

impl From<Args> for Cli {
    fn from(args: Args) -> Self {
        Cli { args }
    }
}

impl Cli {
    pub fn init_logger(&self) {
        env_logger::builder()
            .filter_level(LevelFilter::from_str(&self.args.log).unwrap_or(LevelFilter::Info))
            .init();
    }
    pub fn get_model(&self) -> Result<CCModel> {
        CCModel::from_yaml(&self.args.probabilites)
    }

    pub fn get_timestamp_generator(&self) -> Result<TimeStampGenerator> {
        let start_ts = NaiveDateTime::from_str(&self.args.start)?;
        Ok(TimeStampGenerator::new(
            start_ts,
            Duration::hours(self.args.hours),
        ))
    }

    fn get_author_generator(&self) -> AuthorGenerator {
        AuthorGenerator::new(self.args.n_authors, self.args.commits_per_author as f64)
    }

    pub fn get_issue_generator(&self) -> IssueGenerator {
        IssueGenerator::new(&self.args.issue_prefix, self.args.commits_per_issue)
    }

    pub fn force_create_output_dir(&self) -> Result<bool> {
        if self.args.force && std::fs::exists(&self.args.repository).unwrap() {
            std::fs::remove_dir_all(&self.args.repository)
                .with_context(|| "Cannot remove previous repository")?;
            return Ok(true);
        }
        return Ok(false);
    }

    pub fn get_git_writer(self) -> Result<GitWriter> {
        let ts_gen = self
            .get_timestamp_generator()
            .with_context(|| "Failed getting timestamp generator")?;
        let is_gen = self.get_issue_generator();
        let au_gen = self.get_author_generator();
        let cm_gen = CommitGenerator::new(self.args.commits, ts_gen, is_gen, au_gen);
        let cc_gen = CCPairGenerator::new(self.get_model()?);
        self.force_create_output_dir()?;
        let gw = GitWriter::new(self.args.repository, cc_gen, cm_gen)?;
        Ok(gw)
    }
}

impl TryFrom<Args> for MarkovRepositoryGenerator {
    type Error = Box<dyn Error>;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        env_logger::builder()
            .filter_level(LevelFilter::from_str(&args.log).unwrap_or(LevelFilter::Info))
            .init();
        info!(
            "Reading markov model from {}",
            args.probabilites.to_str().unwrap()
        );
        let markov = MarkovProcess::from_yaml(args.probabilites)?;
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
        let writer = crate::stat::git::GitWriter::new(args.repository)?;
        let start =
            NaiveDateTime::from_str(&args.start).with_context(|| "Cannot parse start time")?;
        let ts = TimeStampGenerator::new(start, Duration::hours(args.hours));
        info!("Generating {} commits...", args.commits);
        let markov = MarkovCommitGenerator::new(args.commits, markov, ts);
        let markov = MarkovRepositoryGenerator::new(writer, markov);
        Ok(markov)
    }
}
