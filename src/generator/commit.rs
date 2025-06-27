use anyhow::{anyhow, bail, Context};
use chrono::DateTime;
use git2::{IndexAddOption, Repository, Signature, Time};
use log::{debug, info};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashSet, path::Path};

use super::markov::{MarkovPath, MarkovProcess};
use super::state::State;
use super::utils::TimestampGenerator;

pub struct CommitInput {
    pub files: HashSet<String>,
    pub message: String,
    pub committer: Signature<'static>,
}

impl CommitInput {
    fn try_from(path: MarkovPath, i: u32, time: Time) -> anyhow::Result<Self> {
        let files = path
            .iter()
            .filter(|s| s.is_file())
            .map(|s| s.name().to_string())
            .collect();
        let issue = path
            .iter()
            .find(|s| s.is_issue())
            .map(|s| format!("{}_{}", s.name(), i))
            .ok_or(anyhow!("No issue found in Markov path"))?;
        let message = format!("{} - commit number {}", issue, i);
        let committer = path
            .iter()
            .find(|s| s.is_author())
            .map(|s| Signature::new(s.name(), s.name(), &time))
            .ok_or(anyhow!("No author found in Markov path"))??;
        Ok(CommitInput {
            files,
            message,
            committer,
        })
    }
}

impl Display for CommitInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = self.committer.when();
        let time = DateTime::from_timestamp(time.seconds(), 0).unwrap();
        write!(
            f,
            "Commit by {} on {} with {} files changed, message: {}",
            self.committer.name().unwrap_or("unknown"),
            time,
            self.files.len(),
            self.message
        )
    }
}

pub struct MarkovCommitGenerator {
    pub n: u32,
    pub i: u32,
    markov: MarkovProcess,
    timestamp_generator: TimestampGenerator,
}

impl MarkovCommitGenerator {
    pub fn new(
        n_commits: u32,
        markov: MarkovProcess,
        timestamp_generator: TimestampGenerator,
    ) -> Self {
        Self {
            n: n_commits,
            i: 0,
            markov,
            timestamp_generator,
        }
    }
}

impl Iterator for MarkovCommitGenerator {
    type Item = anyhow::Result<CommitInput>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.n {
            self.i = self.i + 1;
            self.markov.reset();
            let path = self.markov.path_until(&State::Commit);
            let ts = Time::new(self.timestamp_generator.next().unwrap(), 0);
            let commit: anyhow::Result<CommitInput> = CommitInput::try_from(path, self.i, ts);
            return Some(commit);
        }
        None
    }
}

#[allow(dead_code)]
pub struct GitWriter {
    repo: Repository,
    fm: FileWriter,
}

impl GitWriter {
    pub fn new<P: AsRef<Path>>(repository: P) -> anyhow::Result<Self> {
        let repo = Repository::init(repository.as_ref())
            .with_context(|| "Failed initializing repository")?;
        let fm = FileWriter::new(repository.as_ref());
        Ok(Self { repo, fm })
    }
    pub fn write_and_commit(&mut self, commit: &CommitInput) -> anyhow::Result<()> {
        info!("Writing files...");
        for file in commit.files.iter() {
            match self.fm.write(file, None) {
                Ok(_) => {}
                Err(e) => {
                    bail!(
                        "Could not write changes to file {}. Ensure directories exists. Error: {}",
                        file,
                        e
                    )
                }
            };
        }
        debug!("{}", commit);
        self.commit(&commit)?;
        Ok(())
    }

    fn commit(&mut self, commit: &CommitInput) -> anyhow::Result<()> {
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

        match parent_commit {
            Some(parent) => {
                self.repo.commit(
                    Some("HEAD"),
                    &commit.committer,
                    &commit.committer,
                    &commit.message,
                    &tree,
                    &[&parent],
                )?;
            }
            None => {
                self.repo.commit(
                    Some("HEAD"),
                    &commit.committer,
                    &commit.committer,
                    &commit.message,
                    &tree,
                    &[],
                )?;
            }
        }
        Ok(())
    }
}

struct FileWriter {
    working_dir: PathBuf,
}

impl FileWriter {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            working_dir: path.as_ref().to_path_buf(),
        }
    }

    pub fn write(&self, file: &String, contents: Option<String>) -> anyhow::Result<()> {
        let file = self.working_dir.join(file);
        if !file.exists() {
            File::create(&file)?;
        }
        let mut file = OpenOptions::new().append(true).open(file)?;
        writeln!(file, "File changed {}", contents.unwrap_or("".to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MarkovCommitGenerator;
    use crate::{generator::markov::MarkovProcess, generator::utils::TimestampGenerator};
    use chrono::{Duration, NaiveDate, NaiveTime};

    #[test]
    fn test_generation() {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        let ts = TimestampGenerator::new(start, Duration::days(10));
        let mk = MarkovProcess::from_yaml("./test_data/markov.yaml").unwrap();
        let mut mcg = MarkovCommitGenerator::new(100, mk, ts);
        while let Some(commit) = mcg.next() {
            assert!(commit.is_ok());
            println!("{}", commit.unwrap());
        }
    }
}
