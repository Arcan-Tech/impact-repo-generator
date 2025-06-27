use core::f64;
use std::io::Write;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::input::{CCModel, CCPair};
use anyhow::{bail, Result};
use chrono::{Duration, NaiveDateTime};
use git2::{IndexAddOption, Repository, Signature, Time};
use log::info;
use rand::rng;
use rand_distr::{Distribution, Exp, Poisson};

pub struct TimeStampGenerator {
    start: i64,
    d: Exp<f64>, // How much time between an event
}

impl TimeStampGenerator {
    pub fn new(start: NaiveDateTime, average_interval: Duration) -> Self {
        let lambda = 1.0 / average_interval.num_seconds() as f64;
        Self {
            start: start.and_utc().timestamp(),
            d: Exp::new(lambda).unwrap(),
        }
    }
}

impl Iterator for TimeStampGenerator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let interval = self.d.sample(&mut rand::rng()).round();
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

pub struct AuthorGenerator {
    n: u32,
    i: u32,
    d: Poisson<f64>,
}

pub struct AuthorInput {
    name: String,
    email: String,
}

impl AuthorGenerator {
    pub fn new(authors: u32, average_consecutive_commits: f64) -> Self {
        let d = Poisson::new(1.0 / average_consecutive_commits).unwrap();
        Self {
            n: authors,
            i: 1,
            d,
        }
    }
}

impl Iterator for AuthorGenerator {
    type Item = AuthorInput;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.d.sample(&mut rand::rng());
        if x >= 1.0 {
            self.i = rand::random::<u32>() % self.n;
        }
        Some(AuthorInput {
            name: format!("Author_{}", self.i),
            email: format!("Author_{}@email.com", self.i),
        })
    }
}

pub struct CommitInput {
    message: String,
    committer: Signature<'static>,
}

pub struct CommitGenerator {
    n: u32,
    i: u32,
    timestamp_generator: TimeStampGenerator,
    issue_generator: IssueGenerator,
    author_generator: AuthorGenerator,
}

impl CommitGenerator {
    pub fn new(
        n_commits: u32,
        timestamp_generator: TimeStampGenerator,
        issue_generator: IssueGenerator,
        author_generator: AuthorGenerator,
    ) -> Self {
        Self {
            n: n_commits,
            i: 0,
            timestamp_generator,
            issue_generator,
            author_generator,
        }
    }

    pub fn commit_number(&self) -> u32 {
        self.i
    }
}

impl Iterator for CommitGenerator {
    type Item = CommitInput;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            return None;
        }

        self.i = self.i + 1;
        let issue = self.issue_generator.next().unwrap();
        let message = format!("{} - commit number {}", issue, self.i);
        let ts = Time::new(self.timestamp_generator.next().unwrap(), 0);
        let author = self.author_generator.next().unwrap();
        let committer = Signature::new(&author.name, &author.email, &ts).unwrap();
        Some(CommitInput { message, committer })
    }
}

pub struct CCPairGenerator {
    model: CCModel,
}

impl CCPairGenerator {
    pub fn new(model: CCModel) -> Self {
        Self { model }
    }
}

impl Iterator for CCPairGenerator {
    type Item = Vec<CCPair>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.model.roll_cochanges())
    }
}

pub struct GitWriter {
    repo: Repository,
    fm: FileManager,
    commit_generator: CommitGenerator,
    ccpair_generator: CCPairGenerator,
}

impl GitWriter {
    pub fn new<P>(
        path: P,
        ccpair_generator: CCPairGenerator,
        commit_generator: CommitGenerator,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let repo = Repository::init(path.as_ref())?;
        let fm = FileManager::new(path.as_ref());
        Ok(Self {
            repo,
            fm,
            commit_generator,
            ccpair_generator,
        })
    }

    pub fn generate(&mut self) -> Result<()> {
        while let Some(commit) = self.commit_generator.next() {
            info!("Prepared commit {}", self.commit_generator.commit_number());
            let pairs = self.ccpair_generator.next().unwrap();
            info!("Generated {} co-changes", pairs.len());
            for p in pairs {
                match self.fm.write_pair(&p) {
                    Ok(_) => {}
                    Err(e) => {
                        bail!(
                            "Could not write co-change of file {} and {}: {}. Ensure directories exists.",
                            p.f1,
                            p.f2,
                            e
                        )
                    }
                };
            }
            self.commit(&commit)?;
            info!("Committed changes.");
        }
        Ok(())
    }

    fn commit(&mut self, commit: &CommitInput) -> Result<()> {
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
    use super::{AuthorGenerator, IssueGenerator};

    #[test]
    pub fn test_author_generator() {
        let mut au_gen = AuthorGenerator::new(3, 3.0);
        for _ in 0..100 {
            println!("{}", au_gen.next().unwrap().name);
        }
    }

    #[test]
    pub fn test_issue_generator() {
        let mut is_gen = IssueGenerator::new("#", 3);
        for _ in 0..100 {
            println!("{}", is_gen.next().unwrap());
        }
    }
}
