use core::f64;
use std::io::Write;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::input::CCPair;
use anyhow::Result;
use chrono::Duration;
use git2::{IndexAddOption, Repository, Signature, Time};
use log::error;
use rand::rng;
use rand_distr::{Distribution, Exp, Poisson};

pub struct TimeStampGenerator {
    start: i64,
    d: Exp<f64>, // How much time between an event
}

impl TimeStampGenerator {
    pub fn new(start: i64, average_interval: Duration) -> Self {
        let lambda = 1.0 / average_interval.num_seconds() as f64;
        Self {
            start,
            d: Exp::new(lambda).unwrap(),
        }
    }
}

impl Iterator for TimeStampGenerator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let interval = self.d.sample(&mut rand::rng()).round();
        dbg!(interval);
        self.start = self.start + interval as i64;
        Some(self.start)
    }
}

pub struct IssueGenerator {
    prefix: String,
    last_issue: Option<String>,
    d: Poisson<f64>,
}

impl IssueGenerator {
    pub fn new(prefix: &str, average_linked_commits: u32) -> Self {
        let lambda = 1.0 / average_linked_commits as f64;
        Self {
            prefix: prefix.to_string(),
            last_issue: None,
            d: Poisson::new(lambda).unwrap(),
        }
    }
}

impl Iterator for IssueGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let roll = self.d.sample(&mut rng());
        if self.last_issue.is_none() || roll >= 1.0 {
            let id: u16 = rand::random();
            self.last_issue.replace(format!("{}{}", self.prefix, id));
        }
        self.last_issue.clone()
    }
}

pub struct GitWriter {
    repo: Repository,
    fm: FileManager,
    i: usize,
    timestamp_generator: TimeStampGenerator,
    issue_generator: IssueGenerator,
}

impl GitWriter {
    pub fn new<P>(
        path: P,
        timestamp_generator: TimeStampGenerator,
        issue_generator: IssueGenerator,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let repo = Repository::init(path.as_ref())?;
        let fm = FileManager::new(path.as_ref());
        Ok(Self {
            repo,
            fm,
            i: 0,
            timestamp_generator,
            issue_generator,
        })
    }

    pub fn modify_and_commit(&mut self, pairs: &[CCPair]) -> Result<()> {
        for p in pairs {
            match self.fm.write_pair(p) {
                Ok(_) => {}
                Err(e) => {
                    error!("Could not write pair {}: {}", p, e)
                }
            }
        }
        let issue = self.issue_generator.next().unwrap();
        self.i = self.i + 1;
        self.commit(&format!("{} commit number {}", issue, self.i))
    }

    fn commit(&mut self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let oid = index.write_tree()?;
        let tree = self.repo.find_tree(oid)?;
        let head = self.repo.head().ok();
        let parent_commit = head
            .as_ref()
            .and_then(|h| h.target())
            .and_then(|t| self.repo.find_commit(t).ok());

        let ts = Time::new(self.timestamp_generator.next().unwrap(), 0);
        let committer = Signature::new("Git Generator", "some@email.com", &ts)?;
        match parent_commit {
            Some(parent) => {
                self.repo.commit(
                    Some("HEAD"),
                    &committer,
                    &committer,
                    message,
                    &tree,
                    &[&parent],
                )?;
            }
            None => {
                self.repo
                    .commit(Some("HEAD"), &committer, &committer, message, &tree, &[])?;
            }
        }
        Ok(())
    }
}

pub struct FileManager {
    working_dir: PathBuf,
}

impl FileManager {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            working_dir: path.as_ref().to_path_buf(),
        }
    }

    pub fn write_pair(&self, pair: &CCPair) -> Result<()> {
        let path_f1 = self.working_dir.join(&pair.f1);
        let path_f2 = self.working_dir.join(&pair.f2);

        if !path_f1.exists() {
            File::create(&path_f1)?;
        }

        if !path_f2.exists() {
            File::create(&path_f2)?;
        }

        let mut file_f2 = OpenOptions::new().append(true).open(&path_f2)?;

        writeln!(file_f2, "File changed because {} changed", pair.f1)?;
        Ok(())
    }
}
