use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use log::error;

use super::commit::{GitWriter, MarkovCommitGenerator};

pub struct MarkovRepositoryGenerator {
    writer: GitWriter,
    markov: MarkovCommitGenerator,
    progress: ProgressBar,
}

impl MarkovRepositoryGenerator {
    pub fn new(writer: GitWriter, markov: MarkovCommitGenerator) -> Self {
        let progress = ProgressBar::new(markov.n as u64).with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap(),
        );
        Self {
            writer,
            markov,
            progress,
        }
    }

    pub fn generate(&mut self) -> anyhow::Result<()> {
        self.progress.set_message("Simulating commits");
        while let Some(commit) = self.markov.next() {
            self.progress.inc(1);
            match commit {
                Ok(commit) => match self.writer.write_and_commit(&commit) {
                    Err(error) => {
                        debug!("{}", commit);
                        error!("Could not write commit: {}", error);
                        continue;
                    }
                    _ => {}
                },
                Err(error) => {
                    error!("Could not generate synthetic commit commit: {}", error);
                    continue;
                }
            }
        }
        self.progress
            .finish_with_message("All commits were generated.");
        Ok(())
    }
}
