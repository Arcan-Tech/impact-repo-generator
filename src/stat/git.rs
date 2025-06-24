use anyhow::anyhow;
use git2::{Repository, Signature, Time};
use std::collections::HashSet;
use std::fmt::Debug;

use crate::git::{FileManager, TimeStampGenerator};

use super::markov::{MarkovPath, MarkovProcess, State};

pub struct CommitInput {
    pub files: HashSet<String>,
    pub message: String,
    pub committer: Signature<'static>,
}

impl CommitInput {
    fn try_from(path: MarkovPath, i: u32, p: u32, time: Time) -> anyhow::Result<Self> {
        let files = path
            .iter()
            .filter(|s| s.is_file())
            .map(|s| s.name().to_string())
            .collect();
        let issue = path
            .iter()
            .find(|s| s.is_issue())
            // TODO: generating issues this way ignores that commits
            // belonging to the same issue are close to each other
            // We need to simulate the selection of issues using a methodology unrelated to
            // the markov process.
            // We could have 3-4 mother-issues defined in the markov process, and then use
            // a poisson to select the number of commits this will be featured in
            .map(|s| format!("{}_{}", s.name(), i % p))
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

pub struct MarkovCommitGenerator {
    n: u32,
    i: u32,
    markov: MarkovProcess,
    timestamp_generator: TimeStampGenerator,
}

impl MarkovCommitGenerator {
    pub fn new(
        n_commits: u32,
        markov: MarkovProcess,
        timestamp_generator: TimeStampGenerator,
    ) -> Self {
        Self {
            n: n_commits,
            i: 0,
            markov,
            timestamp_generator,
        }
    }
}

// TODO: test this
impl Iterator for MarkovCommitGenerator {
    type Item = anyhow::Result<CommitInput>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i <= self.n {
            self.i = self.i + 1;
            self.markov.reset();
            let path = self.markov.path_until(&State::Commit);
            let ts = Time::new(self.timestamp_generator.next().unwrap(), 0);
            let commit: anyhow::Result<CommitInput> = CommitInput::try_from(path, self.i, 5, ts);
            return Some(commit);
        }
        None
    }
}

pub struct MarkovGitWriter {
    repo: Repository,
    fm: FileManager,
}

impl Debug for CommitInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CommitInput({:?}, {:?}, {:?}, {:?})",
            self.committer.name(),
            self.committer.when(),
            self.message,
            self.files,
        )
    }
}

#[cfg(test)]
mod tests {

    use chrono::Duration;

    use crate::{git::TimeStampGenerator, stat::markov::MarkovProcess};

    use super::MarkovCommitGenerator;

    #[test]
    fn test_generation() {
        let ts = TimeStampGenerator::new(10000000, Duration::days(10));
        let mk = MarkovProcess::from_yaml("./test_data/markov.yaml").unwrap();
        let mut mcg = MarkovCommitGenerator::new(10, mk, ts);
        while let Some(commit) = mcg.next() {
            println!("{:?}", commit);
        }
    }
}
