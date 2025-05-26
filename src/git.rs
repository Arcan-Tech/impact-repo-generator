use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::model::CCPair;
use anyhow::Result;
use git2::{IndexAddOption, Repository, Signature};
use log::error;

pub struct GitWriter {
    repo: Repository,
    fm: FileManager,
    i: usize,
}

impl GitWriter {
    pub fn new(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)?;
        let fm = FileManager::new(path);
        Ok(Self { repo, fm, i: 0 })
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
        self.i = self.i + 1;
        self.commit(&format!("Commit number {}", self.i))
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

        let committer = Signature::now("Git Generator", "some@email.com")?;
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
    pub fn new(path: &Path) -> Self {
        Self {
            working_dir: path.into(),
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
