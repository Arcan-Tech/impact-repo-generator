use std::{path::PathBuf, str::FromStr};

use chrono::{Duration, NaiveDateTime};
use clap::Parser;
use log::LevelFilter;

use crate::{
    git::{CCPairGenerator, CommitGenerator, GitWriter, IssueGenerator, TimeStampGenerator},
    input::CCModel,
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
        let start_ts = NaiveDateTime::from_str(&self.args.start)?
            .and_utc()
            .timestamp();
        Ok(TimeStampGenerator::new(
            start_ts,
            Duration::hours(self.args.hours),
        ))
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
        let cm_gen = CommitGenerator::new(self.args.commits, ts_gen, is_gen);
        let cc_gen = CCPairGenerator::new(self.get_model()?);
        self.force_create_output_dir()?;
        let gw = GitWriter::new(self.args.repository, cc_gen, cm_gen)?;
        Ok(gw)
    }
}
