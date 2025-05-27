use std::str::FromStr;

use args::Args;
use chrono::{Duration, NaiveDateTime};
use clap::Parser;
use git::{GitWriter, IssueGenerator, TimeStampGenerator};
use input::CCModel;

pub mod args;
pub mod git;
pub mod input;

fn main() {
    let args = Args::parse();
    let m = CCModel::from_yaml(args.probabilites).unwrap();
    let nv = NaiveDateTime::from_str(&args.start).unwrap();

    let ts_gen = TimeStampGenerator::new(nv.and_utc().timestamp(), Duration::hours(args.hours));
    if args.force {
        std::fs::remove_dir_all(&args.repository).unwrap();
    }
    let is_gen = IssueGenerator::new(&args.issue_prefix, args.commits_per_issue);
    let mut gw = GitWriter::new(args.repository, ts_gen, is_gen).unwrap();
    for _ in 0..args.commits {
        let pairs = m.roll_cochanges();
        gw.modify_and_commit(&pairs).unwrap()
    }
}
