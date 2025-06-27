use super::commit::{GitWriter, MarkovCommitGenerator};

pub struct MarkovRepositoryGenerator {
    writer: GitWriter,
    markov: MarkovCommitGenerator,
}

impl MarkovRepositoryGenerator {
    pub fn new(writer: GitWriter, markov: MarkovCommitGenerator) -> Self {
        Self { writer, markov }
    }

    pub fn generate(&mut self) -> anyhow::Result<()> {
        while let Some(commit) = self.markov.next() {
            match commit {
                Ok(commit) => match self.writer.write_and_commit(&commit) {
                    Err(error) => {
                        log::error!("Could not generate synthetic commit commit: {}", error);
                        continue;
                    }
                    _ => {
                        log::info!("Written commit {} of {}", self.markov.i, self.markov.n);
                    }
                },
                Err(error) => {
                    log::error!("Could not generate synthetic commit commit: {}", error);
                    continue;
                }
            }
        }
        Ok(())
    }
}
